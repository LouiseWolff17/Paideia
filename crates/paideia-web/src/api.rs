//! Calls to the local Paideia server. The browser never talks to Anthropic directly.
//!
//! Every function returns a plain-language error message on failure, ready to show.

use gloo_net::http::{Request, RequestBuilder, Response};
use paideia_core::{
    api::{ApiError, HealthResponse},
    Concept, ConceptId, ConceptInput, Material, MaterialId, MaterialInput, Subject, SubjectId,
    SubjectInput,
};
use serde::{de::DeserializeOwned, Serialize};

const UNREACHABLE: &str = "Can't reach the local server. Is it still running?";

async fn send(req: Result<Request, gloo_net::Error>) -> Result<Response, String> {
    let resp = req
        .map_err(|_| "The request couldn't be prepared.".to_string())?
        .send()
        .await
        .map_err(|_| UNREACHABLE.to_string())?;
    if resp.ok() {
        return Ok(resp);
    }
    // Server errors carry a learner-friendly message; use it when present.
    match resp.json::<ApiError>().await {
        Ok(e) => Err(e.message),
        Err(_) => Err(format!("The local server answered with an error ({}).", resp.status())),
    }
}

async fn read<T: DeserializeOwned>(resp: Response) -> Result<T, String> {
    resp.json::<T>()
        .await
        .map_err(|_| "The local server sent something unexpected.".to_string())
}

async fn get<T: DeserializeOwned>(url: &str) -> Result<T, String> {
    read(send(Request::get(url).build()).await?).await
}

async fn with_body<B: Serialize, T: DeserializeOwned>(
    builder: RequestBuilder,
    body: &B,
) -> Result<T, String> {
    read(send(builder.json(body)).await?).await
}

async fn delete(url: &str) -> Result<(), String> {
    send(Request::delete(url).build()).await.map(|_| ())
}

pub async fn health() -> Result<HealthResponse, String> {
    get("/api/health").await
}

// Subjects

pub async fn list_subjects() -> Result<Vec<Subject>, String> {
    get("/api/subjects").await
}

pub async fn get_subject(id: SubjectId) -> Result<Subject, String> {
    get(&format!("/api/subjects/{id}")).await
}

pub async fn create_subject(input: SubjectInput) -> Result<Subject, String> {
    with_body(Request::post("/api/subjects"), &input).await
}

pub async fn update_subject(id: SubjectId, input: SubjectInput) -> Result<Subject, String> {
    with_body(Request::put(&format!("/api/subjects/{id}")), &input).await
}

pub async fn delete_subject(id: SubjectId) -> Result<(), String> {
    delete(&format!("/api/subjects/{id}")).await
}

// Concepts

pub async fn list_concepts(subject: SubjectId) -> Result<Vec<Concept>, String> {
    get(&format!("/api/subjects/{subject}/concepts")).await
}

pub async fn get_concept(id: ConceptId) -> Result<Concept, String> {
    get(&format!("/api/concepts/{id}")).await
}

pub async fn create_concept(subject: SubjectId, input: ConceptInput) -> Result<Concept, String> {
    with_body(Request::post(&format!("/api/subjects/{subject}/concepts")), &input).await
}

pub async fn update_concept(id: ConceptId, input: ConceptInput) -> Result<Concept, String> {
    with_body(Request::put(&format!("/api/concepts/{id}")), &input).await
}

pub async fn delete_concept(id: ConceptId) -> Result<(), String> {
    delete(&format!("/api/concepts/{id}")).await
}

// Materials

pub async fn list_materials(concept: ConceptId) -> Result<Vec<Material>, String> {
    get(&format!("/api/concepts/{concept}/materials")).await
}

pub async fn create_material(concept: ConceptId, input: MaterialInput) -> Result<Material, String> {
    with_body(Request::post(&format!("/api/concepts/{concept}/materials")), &input).await
}

pub async fn update_material(id: MaterialId, input: MaterialInput) -> Result<Material, String> {
    with_body(Request::put(&format!("/api/materials/{id}")), &input).await
}

pub async fn delete_material(id: MaterialId) -> Result<(), String> {
    delete(&format!("/api/materials/{id}")).await
}
