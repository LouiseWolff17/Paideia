//! `/api` routes for subjects, concepts and materials.
//!
//! ```text
//! GET    /subjects                    list subjects
//! POST   /subjects                    create a subject
//! GET    /subjects/{id}               one subject
//! PUT    /subjects/{id}               edit a subject
//! DELETE /subjects/{id}               delete a subject, its concepts and their material
//! GET    /subjects/{id}/concepts      concepts in a subject
//! POST   /subjects/{id}/concepts      add a concept
//! GET    /concepts/{id}               one concept
//! PUT    /concepts/{id}               edit a concept
//! DELETE /concepts/{id}               delete a concept and its material
//! GET    /concepts/{id}/materials     material attached to a concept
//! POST   /concepts/{id}/materials     add text material
//! GET    /materials/{id}              one piece of material
//! PUT    /materials/{id}              edit material
//! DELETE /materials/{id}              delete material
//! ```

use axum::{
    extract::{rejection::JsonRejection, rejection::PathRejection, Path, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use paideia_core::{
    Concept, ConceptId, ConceptInput, Material, MaterialId, MaterialInput, Subject, SubjectId,
    SubjectInput,
};

use crate::{
    error::{AppError, AppResult},
    store, AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/subjects", get(list_subjects).post(create_subject))
        .route("/subjects/{id}", get(get_subject).put(update_subject).delete(delete_subject))
        .route("/subjects/{id}/concepts", get(list_concepts).post(create_concept))
        .route("/concepts/{id}", get(get_concept).put(update_concept).delete(delete_concept))
        .route("/concepts/{id}/materials", get(list_materials).post(create_material))
        .route("/materials/{id}", get(get_material).put(update_material).delete(delete_material))
}

const SUBJECT: &str = "subject";
const CONCEPT: &str = "concept";
const MATERIAL: &str = "piece of material";

/// A malformed ID (e.g. `/subjects/abc`) can never match anything, so treat it as "not found".
fn id<T>(path: Result<Path<T>, PathRejection>, what: &'static str) -> AppResult<T> {
    path.map(|Path(id)| id).map_err(|_| AppError::NotFound(what))
}

fn body<T>(json: Result<Json<T>, JsonRejection>) -> AppResult<T> {
    Ok(json?.0)
}

fn found<T>(value: Option<T>, what: &'static str) -> AppResult<T> {
    value.ok_or(AppError::NotFound(what))
}

fn deleted(existed: bool, what: &'static str) -> AppResult<StatusCode> {
    if existed {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound(what))
    }
}

// ---------------------------------------------------------------------------
// Subjects
// ---------------------------------------------------------------------------

async fn list_subjects(State(s): State<AppState>) -> AppResult<Json<Vec<Subject>>> {
    Ok(Json(store::list_subjects(&s.db).await?))
}

async fn get_subject(
    State(s): State<AppState>,
    path: Result<Path<SubjectId>, PathRejection>,
) -> AppResult<Json<Subject>> {
    let id = id(path, SUBJECT)?;
    Ok(Json(found(store::get_subject(&s.db, id).await?, SUBJECT)?))
}

async fn create_subject(
    State(s): State<AppState>,
    json: Result<Json<SubjectInput>, JsonRejection>,
) -> AppResult<(StatusCode, Json<Subject>)> {
    let input = body(json)?.validated()?;
    let subject = store::create_subject(&s.db, &input).await?;
    Ok((StatusCode::CREATED, Json(subject)))
}

async fn update_subject(
    State(s): State<AppState>,
    path: Result<Path<SubjectId>, PathRejection>,
    json: Result<Json<SubjectInput>, JsonRejection>,
) -> AppResult<Json<Subject>> {
    let id = id(path, SUBJECT)?;
    let input = body(json)?.validated()?;
    Ok(Json(found(store::update_subject(&s.db, id, &input).await?, SUBJECT)?))
}

async fn delete_subject(
    State(s): State<AppState>,
    path: Result<Path<SubjectId>, PathRejection>,
) -> AppResult<StatusCode> {
    let id = id(path, SUBJECT)?;
    deleted(store::delete_subject(&s.db, id).await?, SUBJECT)
}

// ---------------------------------------------------------------------------
// Concepts
// ---------------------------------------------------------------------------

async fn list_concepts(
    State(s): State<AppState>,
    path: Result<Path<SubjectId>, PathRejection>,
) -> AppResult<Json<Vec<Concept>>> {
    let subject = id(path, SUBJECT)?;
    // Distinguish "no concepts yet" from "no such subject".
    found(store::get_subject(&s.db, subject).await?, SUBJECT)?;
    Ok(Json(store::list_concepts(&s.db, subject).await?))
}

async fn create_concept(
    State(s): State<AppState>,
    path: Result<Path<SubjectId>, PathRejection>,
    json: Result<Json<ConceptInput>, JsonRejection>,
) -> AppResult<(StatusCode, Json<Concept>)> {
    let subject = id(path, SUBJECT)?;
    let input = body(json)?.validated()?;
    let concept = found(store::create_concept(&s.db, subject, &input).await?, SUBJECT)?;
    Ok((StatusCode::CREATED, Json(concept)))
}

async fn get_concept(
    State(s): State<AppState>,
    path: Result<Path<ConceptId>, PathRejection>,
) -> AppResult<Json<Concept>> {
    let id = id(path, CONCEPT)?;
    Ok(Json(found(store::get_concept(&s.db, id).await?, CONCEPT)?))
}

async fn update_concept(
    State(s): State<AppState>,
    path: Result<Path<ConceptId>, PathRejection>,
    json: Result<Json<ConceptInput>, JsonRejection>,
) -> AppResult<Json<Concept>> {
    let id = id(path, CONCEPT)?;
    let input = body(json)?.validated()?;
    Ok(Json(found(store::update_concept(&s.db, id, &input).await?, CONCEPT)?))
}

async fn delete_concept(
    State(s): State<AppState>,
    path: Result<Path<ConceptId>, PathRejection>,
) -> AppResult<StatusCode> {
    let id = id(path, CONCEPT)?;
    deleted(store::delete_concept(&s.db, id).await?, CONCEPT)
}

// ---------------------------------------------------------------------------
// Materials
// ---------------------------------------------------------------------------

async fn list_materials(
    State(s): State<AppState>,
    path: Result<Path<ConceptId>, PathRejection>,
) -> AppResult<Json<Vec<Material>>> {
    let concept = id(path, CONCEPT)?;
    found(store::get_concept(&s.db, concept).await?, CONCEPT)?;
    Ok(Json(store::list_materials(&s.db, concept).await?))
}

async fn create_material(
    State(s): State<AppState>,
    path: Result<Path<ConceptId>, PathRejection>,
    json: Result<Json<MaterialInput>, JsonRejection>,
) -> AppResult<(StatusCode, Json<Material>)> {
    let concept = id(path, CONCEPT)?;
    let input = body(json)?.validated()?;
    let material = found(store::create_material(&s.db, concept, &input).await?, CONCEPT)?;
    Ok((StatusCode::CREATED, Json(material)))
}

async fn get_material(
    State(s): State<AppState>,
    path: Result<Path<MaterialId>, PathRejection>,
) -> AppResult<Json<Material>> {
    let id = id(path, MATERIAL)?;
    Ok(Json(found(store::get_material(&s.db, id).await?, MATERIAL)?))
}

async fn update_material(
    State(s): State<AppState>,
    path: Result<Path<MaterialId>, PathRejection>,
    json: Result<Json<MaterialInput>, JsonRejection>,
) -> AppResult<Json<Material>> {
    let id = id(path, MATERIAL)?;
    let input = body(json)?.validated()?;
    Ok(Json(found(store::update_material(&s.db, id, &input).await?, MATERIAL)?))
}

async fn delete_material(
    State(s): State<AppState>,
    path: Result<Path<MaterialId>, PathRejection>,
) -> AppResult<StatusCode> {
    let id = id(path, MATERIAL)?;
    deleted(store::delete_material(&s.db, id).await?, MATERIAL)
}
