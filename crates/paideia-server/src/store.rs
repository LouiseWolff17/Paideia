//! SQL for subjects, concepts and materials.
//!
//! Every function takes already-validated input. `Ok(None)` means "not found".

use paideia_core::{
    catalog::Timestamp, Concept, ConceptId, ConceptInput, Material, MaterialId, MaterialInput,
    MaterialKind, Subject, SubjectId, SubjectInput,
};
use sqlx::{FromRow, SqlitePool};

pub fn now() -> Timestamp {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as Timestamp)
        .unwrap_or(0)
}

// ---------------------------------------------------------------------------
// Subjects
// ---------------------------------------------------------------------------

#[derive(FromRow)]
struct SubjectRow {
    id: i64,
    name: String,
    description: String,
    concept_count: i64,
    created_at: i64,
    updated_at: i64,
}

impl From<SubjectRow> for Subject {
    fn from(r: SubjectRow) -> Self {
        Subject {
            id: SubjectId(r.id),
            name: r.name,
            description: r.description,
            concept_count: r.concept_count as u32,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

const SUBJECT_SELECT: &str = "
    SELECT s.id, s.name, s.description, s.created_at, s.updated_at,
           (SELECT COUNT(*) FROM concepts c WHERE c.subject_id = s.id) AS concept_count
    FROM subjects s";

pub async fn list_subjects(db: &SqlitePool) -> sqlx::Result<Vec<Subject>> {
    let sql = format!("{SUBJECT_SELECT} ORDER BY s.name COLLATE NOCASE, s.id");
    let rows: Vec<SubjectRow> = sqlx::query_as(&sql).fetch_all(db).await?;
    Ok(rows.into_iter().map(Into::into).collect())
}

pub async fn get_subject(db: &SqlitePool, id: SubjectId) -> sqlx::Result<Option<Subject>> {
    let sql = format!("{SUBJECT_SELECT} WHERE s.id = ?");
    let row: Option<SubjectRow> = sqlx::query_as(&sql).bind(id.0).fetch_optional(db).await?;
    Ok(row.map(Into::into))
}

pub async fn create_subject(db: &SqlitePool, input: &SubjectInput) -> sqlx::Result<Subject> {
    let t = now();
    let id: i64 = sqlx::query_scalar(
        "INSERT INTO subjects (name, description, created_at, updated_at)
         VALUES (?, ?, ?, ?) RETURNING id",
    )
    .bind(&input.name)
    .bind(&input.description)
    .bind(t)
    .bind(t)
    .fetch_one(db)
    .await?;
    Ok(Subject {
        id: SubjectId(id),
        name: input.name.clone(),
        description: input.description.clone(),
        concept_count: 0,
        created_at: t,
        updated_at: t,
    })
}

pub async fn update_subject(
    db: &SqlitePool,
    id: SubjectId,
    input: &SubjectInput,
) -> sqlx::Result<Option<Subject>> {
    let changed = sqlx::query(
        "UPDATE subjects SET name = ?, description = ?, updated_at = ? WHERE id = ?",
    )
    .bind(&input.name)
    .bind(&input.description)
    .bind(now())
    .bind(id.0)
    .execute(db)
    .await?
    .rows_affected();
    if changed == 0 {
        return Ok(None);
    }
    get_subject(db, id).await
}

/// Deletes the subject together with its concepts and their material.
pub async fn delete_subject(db: &SqlitePool, id: SubjectId) -> sqlx::Result<bool> {
    let r = sqlx::query("DELETE FROM subjects WHERE id = ?").bind(id.0).execute(db).await?;
    Ok(r.rows_affected() > 0)
}

// ---------------------------------------------------------------------------
// Concepts
// ---------------------------------------------------------------------------

#[derive(FromRow)]
struct ConceptRow {
    id: i64,
    subject_id: i64,
    name: String,
    summary: String,
    material_count: i64,
    created_at: i64,
    updated_at: i64,
}

impl From<ConceptRow> for Concept {
    fn from(r: ConceptRow) -> Self {
        Concept {
            id: ConceptId(r.id),
            subject_id: SubjectId(r.subject_id),
            name: r.name,
            summary: r.summary,
            material_count: r.material_count as u32,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

const CONCEPT_SELECT: &str = "
    SELECT c.id, c.subject_id, c.name, c.summary, c.created_at, c.updated_at,
           (SELECT COUNT(*) FROM materials m WHERE m.concept_id = c.id) AS material_count
    FROM concepts c";

/// Concepts in the order they were added, which usually follows the course.
pub async fn list_concepts(db: &SqlitePool, subject: SubjectId) -> sqlx::Result<Vec<Concept>> {
    let sql = format!("{CONCEPT_SELECT} WHERE c.subject_id = ? ORDER BY c.id");
    let rows: Vec<ConceptRow> = sqlx::query_as(&sql).bind(subject.0).fetch_all(db).await?;
    Ok(rows.into_iter().map(Into::into).collect())
}

pub async fn get_concept(db: &SqlitePool, id: ConceptId) -> sqlx::Result<Option<Concept>> {
    let sql = format!("{CONCEPT_SELECT} WHERE c.id = ?");
    let row: Option<ConceptRow> = sqlx::query_as(&sql).bind(id.0).fetch_optional(db).await?;
    Ok(row.map(Into::into))
}

/// Returns `None` if the subject does not exist.
pub async fn create_concept(
    db: &SqlitePool,
    subject: SubjectId,
    input: &ConceptInput,
) -> sqlx::Result<Option<Concept>> {
    let t = now();
    // INSERT … SELECT inserts nothing when the subject is missing, which avoids
    // a separate existence check and a race with a concurrent delete.
    let id: Option<i64> = sqlx::query_scalar(
        "INSERT INTO concepts (subject_id, name, summary, created_at, updated_at)
         SELECT id, ?, ?, ?, ? FROM subjects WHERE id = ?
         RETURNING id",
    )
    .bind(&input.name)
    .bind(&input.summary)
    .bind(t)
    .bind(t)
    .bind(subject.0)
    .fetch_optional(db)
    .await?;
    Ok(id.map(|id| Concept {
        id: ConceptId(id),
        subject_id: subject,
        name: input.name.clone(),
        summary: input.summary.clone(),
        material_count: 0,
        created_at: t,
        updated_at: t,
    }))
}

pub async fn update_concept(
    db: &SqlitePool,
    id: ConceptId,
    input: &ConceptInput,
) -> sqlx::Result<Option<Concept>> {
    let changed = sqlx::query("UPDATE concepts SET name = ?, summary = ?, updated_at = ? WHERE id = ?")
        .bind(&input.name)
        .bind(&input.summary)
        .bind(now())
        .bind(id.0)
        .execute(db)
        .await?
        .rows_affected();
    if changed == 0 {
        return Ok(None);
    }
    get_concept(db, id).await
}

/// Deletes the concept together with its material.
pub async fn delete_concept(db: &SqlitePool, id: ConceptId) -> sqlx::Result<bool> {
    let r = sqlx::query("DELETE FROM concepts WHERE id = ?").bind(id.0).execute(db).await?;
    Ok(r.rows_affected() > 0)
}

// ---------------------------------------------------------------------------
// Materials
// ---------------------------------------------------------------------------

#[derive(FromRow)]
struct MaterialRow {
    id: i64,
    concept_id: i64,
    kind: String,
    title: String,
    source: String,
    body: String,
    created_at: i64,
    updated_at: i64,
}

impl TryFrom<MaterialRow> for Material {
    type Error = sqlx::Error;
    fn try_from(r: MaterialRow) -> Result<Self, Self::Error> {
        let kind = MaterialKind::parse(&r.kind).ok_or_else(|| {
            sqlx::Error::Decode(format!("unknown material kind {:?}", r.kind).into())
        })?;
        Ok(Material {
            id: MaterialId(r.id),
            concept_id: ConceptId(r.concept_id),
            kind,
            title: r.title,
            source: r.source,
            body: r.body,
            created_at: r.created_at,
            updated_at: r.updated_at,
        })
    }
}

const MATERIAL_SELECT: &str = "
    SELECT id, concept_id, kind, title, source, body, created_at, updated_at
    FROM materials";

pub async fn list_materials(db: &SqlitePool, concept: ConceptId) -> sqlx::Result<Vec<Material>> {
    let sql = format!("{MATERIAL_SELECT} WHERE concept_id = ? ORDER BY id");
    let rows: Vec<MaterialRow> = sqlx::query_as(&sql).bind(concept.0).fetch_all(db).await?;
    rows.into_iter().map(TryInto::try_into).collect()
}

pub async fn get_material(db: &SqlitePool, id: MaterialId) -> sqlx::Result<Option<Material>> {
    let sql = format!("{MATERIAL_SELECT} WHERE id = ?");
    let row: Option<MaterialRow> = sqlx::query_as(&sql).bind(id.0).fetch_optional(db).await?;
    row.map(TryInto::try_into).transpose()
}

/// Adds manually entered text. Returns `None` if the concept does not exist.
pub async fn create_material(
    db: &SqlitePool,
    concept: ConceptId,
    input: &MaterialInput,
) -> sqlx::Result<Option<Material>> {
    let t = now();
    let kind = MaterialKind::ManualText;
    let id: Option<i64> = sqlx::query_scalar(
        "INSERT INTO materials (concept_id, kind, title, source, body, created_at, updated_at)
         SELECT id, ?, ?, ?, ?, ?, ? FROM concepts WHERE id = ?
         RETURNING id",
    )
    .bind(kind.as_str())
    .bind(&input.title)
    .bind(&input.source)
    .bind(&input.body)
    .bind(t)
    .bind(t)
    .bind(concept.0)
    .fetch_optional(db)
    .await?;
    Ok(id.map(|id| Material {
        id: MaterialId(id),
        concept_id: concept,
        kind,
        title: input.title.clone(),
        source: input.source.clone(),
        body: input.body.clone(),
        created_at: t,
        updated_at: t,
    }))
}

pub async fn update_material(
    db: &SqlitePool,
    id: MaterialId,
    input: &MaterialInput,
) -> sqlx::Result<Option<Material>> {
    let changed = sqlx::query(
        "UPDATE materials SET title = ?, source = ?, body = ?, updated_at = ? WHERE id = ?",
    )
    .bind(&input.title)
    .bind(&input.source)
    .bind(&input.body)
    .bind(now())
    .bind(id.0)
    .execute(db)
    .await?
    .rows_affected();
    if changed == 0 {
        return Ok(None);
    }
    get_material(db, id).await
}

pub async fn delete_material(db: &SqlitePool, id: MaterialId) -> sqlx::Result<bool> {
    let r = sqlx::query("DELETE FROM materials WHERE id = ?").bind(id.0).execute(db).await?;
    Ok(r.rows_affected() > 0)
}
