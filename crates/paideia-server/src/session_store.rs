//! SQL for learning sessions, the tutor's activities and small settings.
//!
//! `Ok(None)` means "not found", as in `store.rs`.

use paideia_core::{
    Activity, ActivityId, ConceptId, LearningSession, LearningStage, MaterialId, SessionId,
    SessionSummary, SubjectId, TeachingStrategy, TutorContent, TutorPlan,
};
use sqlx::{FromRow, SqlitePool};

use crate::store::now;

fn decode_error(what: &str, value: &str) -> sqlx::Error {
    sqlx::Error::Decode(format!("unknown {what} {value:?}").into())
}

// ---------------------------------------------------------------------------
// Sessions
// ---------------------------------------------------------------------------

#[derive(FromRow)]
struct SessionRow {
    id: i64,
    concept_id: i64,
    stage: String,
    created_at: i64,
    updated_at: i64,
    activity_count: i64,
    material_ids: Option<String>,
    concept_name: String,
    subject_id: i64,
    subject_name: String,
}

impl TryFrom<SessionRow> for SessionSummary {
    type Error = sqlx::Error;
    fn try_from(r: SessionRow) -> Result<Self, Self::Error> {
        let stage = LearningStage::parse(&r.stage).ok_or_else(|| decode_error("stage", &r.stage))?;
        let mut material_ids: Vec<MaterialId> = r
            .material_ids
            .unwrap_or_default()
            .split(',')
            .filter_map(|s| s.parse().ok())
            .collect();
        material_ids.sort();
        Ok(SessionSummary {
            session: LearningSession {
                id: SessionId(r.id),
                concept_id: ConceptId(r.concept_id),
                stage,
                material_ids,
                activity_count: r.activity_count as u32,
                created_at: r.created_at,
                updated_at: r.updated_at,
            },
            concept_name: r.concept_name,
            subject_id: SubjectId(r.subject_id),
            subject_name: r.subject_name,
        })
    }
}

const SESSION_SELECT: &str = "
    SELECT s.id, s.concept_id, s.stage, s.created_at, s.updated_at,
           (SELECT COUNT(*) FROM activities a WHERE a.session_id = s.id) AS activity_count,
           (SELECT GROUP_CONCAT(sm.material_id) FROM session_materials sm
             WHERE sm.session_id = s.id) AS material_ids,
           c.name AS concept_name, c.subject_id, sub.name AS subject_name
    FROM learning_sessions s
    JOIN concepts c ON c.id = s.concept_id
    JOIN subjects sub ON sub.id = c.subject_id";

fn rows(rows: Vec<SessionRow>) -> sqlx::Result<Vec<SessionSummary>> {
    rows.into_iter().map(TryInto::try_into).collect()
}

pub async fn get_session(db: &SqlitePool, id: SessionId) -> sqlx::Result<Option<SessionSummary>> {
    let sql = format!("{SESSION_SELECT} WHERE s.id = ?");
    let row: Option<SessionRow> = sqlx::query_as(&sql).bind(id.0).fetch_optional(db).await?;
    row.map(TryInto::try_into).transpose()
}

/// Most recently active first: "what was last studied".
pub async fn recent_sessions(db: &SqlitePool, limit: u32) -> sqlx::Result<Vec<SessionSummary>> {
    let sql = format!("{SESSION_SELECT} ORDER BY s.updated_at DESC, s.id DESC LIMIT ?");
    rows(sqlx::query_as(&sql).bind(limit).fetch_all(db).await?)
}

pub async fn concept_sessions(db: &SqlitePool, concept: ConceptId) -> sqlx::Result<Vec<SessionSummary>> {
    let sql = format!("{SESSION_SELECT} WHERE s.concept_id = ? ORDER BY s.updated_at DESC, s.id DESC");
    rows(sqlx::query_as(&sql).bind(concept.0).fetch_all(db).await?)
}

/// Start a session at the first stage. Material that does not belong to the
/// concept is silently skipped. Returns `None` if the concept does not exist.
pub async fn create_session(
    db: &SqlitePool,
    concept: ConceptId,
    material_ids: &[MaterialId],
) -> sqlx::Result<Option<SessionSummary>> {
    let t = now();
    let mut tx = db.begin().await?;
    let id: Option<i64> = sqlx::query_scalar(
        "INSERT INTO learning_sessions (concept_id, stage, created_at, updated_at)
         SELECT id, ?, ?, ? FROM concepts WHERE id = ?
         RETURNING id",
    )
    .bind(LearningStage::first().as_str())
    .bind(t)
    .bind(t)
    .bind(concept.0)
    .fetch_optional(&mut *tx)
    .await?;
    let Some(id) = id else {
        return Ok(None);
    };
    for m in material_ids {
        sqlx::query(
            "INSERT OR IGNORE INTO session_materials (session_id, material_id)
             SELECT ?, id FROM materials WHERE id = ? AND concept_id = ?",
        )
        .bind(id)
        .bind(m.0)
        .bind(concept.0)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    get_session(db, SessionId(id)).await
}

pub async fn set_stage(
    db: &SqlitePool,
    id: SessionId,
    stage: LearningStage,
) -> sqlx::Result<Option<SessionSummary>> {
    let changed = sqlx::query("UPDATE learning_sessions SET stage = ?, updated_at = ? WHERE id = ?")
        .bind(stage.as_str())
        .bind(now())
        .bind(id.0)
        .execute(db)
        .await?
        .rows_affected();
    if changed == 0 {
        return Ok(None);
    }
    get_session(db, id).await
}

pub async fn delete_session(db: &SqlitePool, id: SessionId) -> sqlx::Result<bool> {
    let r = sqlx::query("DELETE FROM learning_sessions WHERE id = ?").bind(id.0).execute(db).await?;
    Ok(r.rows_affected() > 0)
}

// ---------------------------------------------------------------------------
// Activities
// ---------------------------------------------------------------------------

#[derive(FromRow)]
struct ActivityRow {
    id: i64,
    session_id: i64,
    stage: String,
    strategy: Option<String>,
    reason: String,
    follow_up: Option<String>,
    content: String,
    created_at: i64,
}

impl TryFrom<ActivityRow> for Activity {
    type Error = sqlx::Error;
    fn try_from(r: ActivityRow) -> Result<Self, Self::Error> {
        let stage = LearningStage::parse(&r.stage).ok_or_else(|| decode_error("stage", &r.stage))?;
        let strategy = r
            .strategy
            .map(|s| TeachingStrategy::parse(&s).ok_or_else(|| decode_error("strategy", &s)))
            .transpose()?;
        let content: TutorContent = serde_json::from_str(&r.content)
            .map_err(|e| sqlx::Error::Decode(format!("stored activity content: {e}").into()))?;
        Ok(Activity {
            id: ActivityId(r.id),
            session_id: SessionId(r.session_id),
            stage,
            strategy,
            reason: r.reason,
            follow_up: r.follow_up,
            content,
            created_at: r.created_at,
        })
    }
}

pub async fn list_activities(db: &SqlitePool, session: SessionId) -> sqlx::Result<Vec<Activity>> {
    let rows: Vec<ActivityRow> = sqlx::query_as(
        "SELECT id, session_id, stage, strategy, reason, follow_up, content, created_at
         FROM activities WHERE session_id = ? ORDER BY id",
    )
    .bind(session.0)
    .fetch_all(db)
    .await?;
    rows.into_iter().map(TryInto::try_into).collect()
}

/// Save a validated tutor reply. Returns `None` if the session was deleted
/// while the reply was being generated.
pub async fn add_activity(
    db: &SqlitePool,
    session: SessionId,
    plan: &TutorPlan,
    content: &TutorContent,
    model: &str,
) -> sqlx::Result<Option<Activity>> {
    let t = now();
    let json = serde_json::to_string(content)
        .map_err(|e| sqlx::Error::Encode(Box::new(e)))?;
    let mut tx = db.begin().await?;
    let id: Option<i64> = sqlx::query_scalar(
        "INSERT INTO activities (session_id, stage, strategy, reason, follow_up, content, model, created_at)
         SELECT id, ?, ?, ?, ?, ?, ?, ? FROM learning_sessions WHERE id = ?
         RETURNING id",
    )
    .bind(plan.stage.as_str())
    .bind(plan.strategy.map(TeachingStrategy::as_str))
    .bind(&plan.reason)
    .bind(&plan.follow_up)
    .bind(&json)
    .bind(model)
    .bind(t)
    .bind(session.0)
    .fetch_optional(&mut *tx)
    .await?;
    let Some(id) = id else {
        return Ok(None);
    };
    sqlx::query("UPDATE learning_sessions SET updated_at = ? WHERE id = ?")
        .bind(t)
        .bind(session.0)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(Some(Activity {
        id: ActivityId(id),
        session_id: session,
        stage: plan.stage,
        strategy: plan.strategy,
        reason: plan.reason.clone(),
        follow_up: plan.follow_up.clone(),
        content: content.clone(),
        created_at: t,
    }))
}

// ---------------------------------------------------------------------------
// Settings
// ---------------------------------------------------------------------------

const AI_CONSENT: &str = "ai_consent_at";

pub async fn ai_consented(db: &SqlitePool) -> sqlx::Result<bool> {
    let v: Option<String> = sqlx::query_scalar("SELECT value FROM settings WHERE key = ?")
        .bind(AI_CONSENT)
        .fetch_optional(db)
        .await?;
    Ok(v.is_some())
}

pub async fn set_ai_consent(db: &SqlitePool, consented: bool) -> sqlx::Result<()> {
    if consented {
        sqlx::query("INSERT INTO settings (key, value) VALUES (?, ?) ON CONFLICT (key) DO NOTHING")
            .bind(AI_CONSENT)
            .bind(now().to_string())
            .execute(db)
            .await?;
    } else {
        sqlx::query("DELETE FROM settings WHERE key = ?").bind(AI_CONSENT).execute(db).await?;
    }
    Ok(())
}
