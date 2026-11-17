//! `/api` routes for AI settings, learning sessions and the tutor.
//!
//! ```text
//! GET    /ai                           is AI set up, and has the learner agreed to it?
//! PUT    /ai/consent                   agree to (or withdraw from) AI requests
//! GET    /sessions?limit=N             most recently active sessions
//! GET    /concepts/{id}/sessions       sessions about one concept
//! POST   /concepts/{id}/sessions       start a session, choosing which material to share
//! GET    /sessions/{id}                a session with all its activities
//! PUT    /sessions/{id}/stage          move to a stage (the learner's choice)
//! DELETE /sessions/{id}                delete a session and its activities
//! POST   /sessions/{id}/activities     ask the tutor for the next piece of teaching
//! ```

use axum::{
    extract::{rejection::JsonRejection, rejection::PathRejection, Path, Query, State},
    http::StatusCode,
    routing::{get, post, put},
    Json, Router,
};
use paideia_core::{
    Activity, ActivityRequest, AiStatus, ConceptId, ConsentChange, NewSession, SessionDetail,
    SessionId, SessionSummary, StageChange, TransitionCause, TutorPlan,
};
use serde::Deserialize;

use crate::{
    ai::prompt::{self, PromptInput},
    error::{AppError, AppResult},
    session_store as ss, store, AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/ai", get(ai_status))
        .route("/ai/consent", put(set_consent))
        .route("/sessions", get(recent))
        .route("/concepts/{id}/sessions", get(concept_sessions).post(start))
        .route("/sessions/{id}", get(detail).delete(delete))
        .route("/sessions/{id}/stage", put(set_stage))
        .route("/sessions/{id}/activities", post(generate))
}

const SESSION: &str = "session";
const CONCEPT: &str = "concept";

fn id<T>(path: Result<Path<T>, PathRejection>, what: &'static str) -> AppResult<T> {
    path.map(|Path(id)| id).map_err(|_| AppError::NotFound(what))
}

fn found<T>(value: Option<T>, what: &'static str) -> AppResult<T> {
    value.ok_or(AppError::NotFound(what))
}

// ---------------------------------------------------------------------------
// AI settings
// ---------------------------------------------------------------------------

async fn status(s: &AppState) -> AppResult<AiStatus> {
    Ok(AiStatus { configured: s.tutor.is_configured(), consented: ss::ai_consented(&s.db).await? })
}

async fn ai_status(State(s): State<AppState>) -> AppResult<Json<AiStatus>> {
    Ok(Json(status(&s).await?))
}

async fn set_consent(
    State(s): State<AppState>,
    json: Result<Json<ConsentChange>, JsonRejection>,
) -> AppResult<Json<AiStatus>> {
    let change = json?.0;
    ss::set_ai_consent(&s.db, change.consented).await?;
    Ok(Json(status(&s).await?))
}

// ---------------------------------------------------------------------------
// Sessions
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct RecentQuery {
    limit: Option<u32>,
}

async fn recent(
    State(s): State<AppState>,
    Query(q): Query<RecentQuery>,
) -> AppResult<Json<Vec<SessionSummary>>> {
    let limit = q.limit.unwrap_or(5).clamp(1, 50);
    Ok(Json(ss::recent_sessions(&s.db, limit).await?))
}

async fn concept_sessions(
    State(s): State<AppState>,
    path: Result<Path<ConceptId>, PathRejection>,
) -> AppResult<Json<Vec<SessionSummary>>> {
    let concept = id(path, CONCEPT)?;
    found(store::get_concept(&s.db, concept).await?, CONCEPT)?;
    Ok(Json(ss::concept_sessions(&s.db, concept).await?))
}

async fn start(
    State(s): State<AppState>,
    path: Result<Path<ConceptId>, PathRejection>,
    json: Result<Json<NewSession>, JsonRejection>,
) -> AppResult<(StatusCode, Json<SessionSummary>)> {
    let concept = id(path, CONCEPT)?;
    let input = json?.0;
    let session = found(ss::create_session(&s.db, concept, &input.material_ids).await?, CONCEPT)?;
    Ok((StatusCode::CREATED, Json(session)))
}

async fn detail(
    State(s): State<AppState>,
    path: Result<Path<SessionId>, PathRejection>,
) -> AppResult<Json<SessionDetail>> {
    let id = id(path, SESSION)?;
    let summary = found(ss::get_session(&s.db, id).await?, SESSION)?;
    let activities = ss::list_activities(&s.db, id).await?;
    Ok(Json(SessionDetail { summary, activities }))
}

async fn set_stage(
    State(s): State<AppState>,
    path: Result<Path<SessionId>, PathRejection>,
    json: Result<Json<StageChange>, JsonRejection>,
) -> AppResult<Json<SessionSummary>> {
    let id = id(path, SESSION)?;
    let change = json?.0;
    let current = found(ss::get_session(&s.db, id).await?, SESSION)?;
    // The state machine is the authority; a learner's own move is always allowed.
    let stage = current
        .session
        .stage
        .transition_to(change.stage, TransitionCause::LearnerChoice)
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(Json(found(ss::set_stage(&s.db, id, stage).await?, SESSION)?))
}

async fn delete(
    State(s): State<AppState>,
    path: Result<Path<SessionId>, PathRejection>,
) -> AppResult<StatusCode> {
    let id = id(path, SESSION)?;
    if ss::delete_session(&s.db, id).await? {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound(SESSION))
    }
}

// ---------------------------------------------------------------------------
// Tutor
// ---------------------------------------------------------------------------

/// Ask the tutor for the next activity at the session's current stage.
///
/// Rules are checked before anything leaves the device: the AI must be set up,
/// the learner must have agreed to AI requests, and only the material chosen
/// for this session is included.
async fn generate(
    State(s): State<AppState>,
    path: Result<Path<SessionId>, PathRejection>,
    json: Result<Json<ActivityRequest>, JsonRejection>,
) -> AppResult<(StatusCode, Json<Activity>)> {
    let id = id(path, SESSION)?;
    let request = json?.0.validated()?;
    let summary = found(ss::get_session(&s.db, id).await?, SESSION)?;

    if !s.tutor.is_configured() {
        return Err(AppError::AiNotConfigured);
    }
    if !ss::ai_consented(&s.db).await? {
        return Err(AppError::ConsentNeeded);
    }

    let session = &summary.session;
    let concept = found(store::get_concept(&s.db, session.concept_id).await?, SESSION)?;
    let materials: Vec<_> = store::list_materials(&s.db, session.concept_id)
        .await?
        .into_iter()
        .filter(|m| session.material_ids.contains(&m.id))
        .collect();
    let history = ss::list_activities(&s.db, id).await?;
    let plan = TutorPlan::choose(session.stage, &request);

    let prompt = prompt::build(&PromptInput {
        subject_name: &summary.subject_name,
        concept: &concept,
        materials: &materials,
        plan: &plan,
        history: &history,
    });
    let generated = s.tutor.generate(&prompt).await?;
    tracing::info!(session = %id, stage = plan.stage.as_str(), model = %generated.model, "tutor reply saved");

    let activity = ss::add_activity(&s.db, id, &plan, &generated.content, &generated.model).await?;
    Ok((StatusCode::CREATED, Json(found(activity, SESSION)?)))
}
