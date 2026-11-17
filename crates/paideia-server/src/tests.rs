//! End-to-end tests of the `/api` routes against an in-memory database.

use std::sync::Arc;

use axum::{
    body::Body,
    http::{Method, Request, StatusCode},
    Router,
};
use http_body_util::BodyExt;
use paideia_core::{api::ApiError, Concept, Material, MaterialKind, Subject};
use serde::de::DeserializeOwned;
use serde_json::{json, Value};
use tower::ServiceExt;

use crate::{
    ai::{fake::FakeTutor, AiError, Tutor},
    config::Config,
    db, router, AppState,
};

struct TestApp {
    router: Router,
}

impl TestApp {
    async fn new() -> Self {
        Self::with_tutor(Tutor::Off).await
    }

    async fn with_tutor(tutor: Tutor) -> Self {
        let state = AppState {
            config: Arc::new(Config::for_tests()),
            db: db::open_in_memory().await,
            tutor: Arc::new(tutor),
        };
        Self { router: router(state) }
    }

    async fn call(&self, method: Method, uri: &str, body: Option<Value>) -> (StatusCode, Vec<u8>) {
        let mut req = Request::builder().method(method).uri(uri);
        let body = match body {
            Some(v) => {
                req = req.header("content-type", "application/json");
                Body::from(v.to_string())
            }
            None => Body::empty(),
        };
        let resp = self.router.clone().oneshot(req.body(body).unwrap()).await.unwrap();
        let status = resp.status();
        let bytes = resp.into_body().collect().await.unwrap().to_bytes().to_vec();
        (status, bytes)
    }

    /// Call and decode a JSON body, asserting the status first.
    async fn json<T: DeserializeOwned>(
        &self,
        method: Method,
        uri: &str,
        body: Option<Value>,
        expected: StatusCode,
    ) -> T {
        let (status, bytes) = self.call(method, uri, body).await;
        assert_eq!(status, expected, "body: {}", String::from_utf8_lossy(&bytes));
        serde_json::from_slice(&bytes).expect("valid JSON")
    }

    async fn status(&self, method: Method, uri: &str, body: Option<Value>) -> StatusCode {
        self.call(method, uri, body).await.0
    }

    async fn subject(&self, name: &str) -> Subject {
        self.json(Method::POST, "/api/subjects", Some(json!({ "name": name })), StatusCode::CREATED)
            .await
    }

    async fn concept(&self, subject: &Subject, name: &str) -> Concept {
        let uri = format!("/api/subjects/{}/concepts", subject.id);
        self.json(Method::POST, &uri, Some(json!({ "name": name })), StatusCode::CREATED).await
    }

    async fn material(&self, concept: &Concept, title: &str, body: &str) -> Material {
        let uri = format!("/api/concepts/{}/materials", concept.id);
        self.json(
            Method::POST,
            &uri,
            Some(json!({ "title": title, "body": body })),
            StatusCode::CREATED,
        )
        .await
    }
}

#[tokio::test]
async fn subjects_can_be_created_listed_edited_and_deleted() {
    let app = TestApp::new().await;

    let empty: Vec<Subject> = app.json(Method::GET, "/api/subjects", None, StatusCode::OK).await;
    assert!(empty.is_empty());

    let created: Subject = app
        .json(
            Method::POST,
            "/api/subjects",
            Some(json!({ "name": "  Statistics ", "description": "Second year" })),
            StatusCode::CREATED,
        )
        .await;
    assert_eq!(created.name, "Statistics", "input is trimmed");
    assert_eq!(created.description, "Second year");
    app.subject("algebra").await;

    let list: Vec<Subject> = app.json(Method::GET, "/api/subjects", None, StatusCode::OK).await;
    let names: Vec<_> = list.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(names, ["algebra", "Statistics"], "sorted by name, ignoring case");

    let uri = format!("/api/subjects/{}", created.id);
    let edited: Subject = app
        .json(Method::PUT, &uri, Some(json!({ "name": "Statistics II" })), StatusCode::OK)
        .await;
    assert_eq!(edited.id, created.id);
    assert_eq!(edited.name, "Statistics II");
    assert_eq!(edited.description, "", "description omitted means cleared");
    assert!(edited.updated_at >= created.updated_at);

    let fetched: Subject = app.json(Method::GET, &uri, None, StatusCode::OK).await;
    assert_eq!(fetched, edited);

    assert_eq!(app.status(Method::DELETE, &uri, None).await, StatusCode::NO_CONTENT);
    assert_eq!(app.status(Method::GET, &uri, None).await, StatusCode::NOT_FOUND);
    assert_eq!(app.status(Method::DELETE, &uri, None).await, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn invalid_input_gets_a_readable_message() {
    let app = TestApp::new().await;

    let err: ApiError = app
        .json(
            Method::POST,
            "/api/subjects",
            Some(json!({ "name": "   " })),
            StatusCode::UNPROCESSABLE_ENTITY,
        )
        .await;
    assert_eq!(err.field.as_deref(), Some("name"));
    assert!(!err.message.is_empty());

    let err: ApiError = app
        .json(Method::POST, "/api/subjects", Some(json!({ "nom": 3 })), StatusCode::BAD_REQUEST)
        .await;
    assert!(err.field.is_none(), "malformed body is not blamed on a field");

    // A body that is not JSON at all.
    let (status, bytes) = app.call(Method::POST, "/api/subjects", None).await;
    assert!(status.is_client_error());
    serde_json::from_slice::<ApiError>(&bytes).expect("errors are always JSON");
}

#[tokio::test]
async fn concepts_belong_to_a_subject() {
    let app = TestApp::new().await;
    let stats = app.subject("Statistics").await;
    let other = app.subject("Other").await;

    let mean = app.concept(&stats, "Mean").await;
    let variance = app.concept(&stats, "Variance").await;
    app.concept(&other, "Unrelated").await;
    assert_eq!(mean.subject_id, stats.id);

    let uri = format!("/api/subjects/{}/concepts", stats.id);
    let list: Vec<Concept> = app.json(Method::GET, &uri, None, StatusCode::OK).await;
    let names: Vec<_> = list.iter().map(|c| c.name.as_str()).collect();
    assert_eq!(names, ["Mean", "Variance"], "kept in the order they were added");

    let subject: Subject =
        app.json(Method::GET, &format!("/api/subjects/{}", stats.id), None, StatusCode::OK).await;
    assert_eq!(subject.concept_count, 2);

    let uri = format!("/api/concepts/{}", variance.id);
    let edited: Concept = app
        .json(
            Method::PUT,
            &uri,
            Some(json!({ "name": "Variance", "summary": "How spread out values are" })),
            StatusCode::OK,
        )
        .await;
    assert_eq!(edited.summary, "How spread out values are");
    assert_eq!(edited.subject_id, stats.id);

    assert_eq!(app.status(Method::DELETE, &uri, None).await, StatusCode::NO_CONTENT);
    let list: Vec<Concept> = app
        .json(Method::GET, &format!("/api/subjects/{}/concepts", stats.id), None, StatusCode::OK)
        .await;
    assert_eq!(list.len(), 1);
}

#[tokio::test]
async fn missing_parents_are_reported_as_not_found() {
    let app = TestApp::new().await;

    let status = app
        .status(Method::POST, "/api/subjects/999/concepts", Some(json!({ "name": "Orphan" })))
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(app.status(Method::GET, "/api/subjects/999/concepts", None).await, StatusCode::NOT_FOUND);

    let status = app
        .status(
            Method::POST,
            "/api/concepts/999/materials",
            Some(json!({ "title": "T", "body": "B" })),
        )
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    let err: ApiError =
        app.json(Method::GET, "/api/subjects/not-a-number", None, StatusCode::NOT_FOUND).await;
    assert!(err.message.contains("subject"));
}

#[tokio::test]
async fn text_material_can_be_added_to_a_concept() {
    let app = TestApp::new().await;
    let stats = app.subject("Statistics").await;
    let mean = app.concept(&stats, "Mean").await;

    let body = "The mean is the sum divided by the count.\n\n    x̄ = Σx / n";
    let notes: Material = app
        .json(
            Method::POST,
            &format!("/api/concepts/{}/materials", mean.id),
            Some(json!({ "title": "Lecture notes", "source": "Week 2 slides", "body": body })),
            StatusCode::CREATED,
        )
        .await;
    assert_eq!(notes.kind, MaterialKind::ManualText);
    assert_eq!(notes.concept_id, mean.id);
    assert_eq!(notes.body, body, "inner formatting and unicode are kept");
    assert_eq!(notes.source, "Week 2 slides");

    let concept: Concept =
        app.json(Method::GET, &format!("/api/concepts/{}", mean.id), None, StatusCode::OK).await;
    assert_eq!(concept.material_count, 1);

    let uri = format!("/api/materials/{}", notes.id);
    let edited: Material = app
        .json(
            Method::PUT,
            &uri,
            Some(json!({ "title": "Notes", "body": "Shorter." })),
            StatusCode::OK,
        )
        .await;
    assert_eq!((edited.title.as_str(), edited.body.as_str(), edited.source.as_str()), ("Notes", "Shorter.", ""));

    let list: Vec<Material> = app
        .json(Method::GET, &format!("/api/concepts/{}/materials", mean.id), None, StatusCode::OK)
        .await;
    assert_eq!(list, vec![edited]);

    assert_eq!(app.status(Method::DELETE, &uri, None).await, StatusCode::NO_CONTENT);
    assert_eq!(app.status(Method::GET, &uri, None).await, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn deleting_a_subject_removes_everything_inside_it() {
    let app = TestApp::new().await;
    let stats = app.subject("Statistics").await;
    let mean = app.concept(&stats, "Mean").await;
    let notes = app.material(&mean, "Notes", "Some text").await;
    let keep = app.subject("Keep me").await;
    let kept_concept = app.concept(&keep, "Still here").await;

    let status = app.status(Method::DELETE, &format!("/api/subjects/{}", stats.id), None).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    for uri in [format!("/api/concepts/{}", mean.id), format!("/api/materials/{}", notes.id)] {
        assert_eq!(app.status(Method::GET, &uri, None).await, StatusCode::NOT_FOUND, "{uri}");
    }
    let status = app.status(Method::GET, &format!("/api/concepts/{}", kept_concept.id), None).await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn ids_are_never_reused_after_deletion() {
    let app = TestApp::new().await;
    let first = app.subject("First").await;
    app.status(Method::DELETE, &format!("/api/subjects/{}", first.id), None).await;
    let second = app.subject("Second").await;
    assert!(second.id > first.id);
}

#[tokio::test]
async fn unknown_api_paths_return_json_not_the_web_page() {
    let app = TestApp::new().await;
    let err: ApiError = app.json(Method::GET, "/api/nope", None, StatusCode::NOT_FOUND).await;
    assert!(!err.message.is_empty());
}

// ---------------------------------------------------------------------------
// Milestone 3: sessions and the tutor
// ---------------------------------------------------------------------------

use paideia_core::{
    Activity, AiStatus, LearningStage, SessionDetail, SessionSummary, TeachingStrategy,
};

fn reply(title: &str) -> Value {
    json!({
        "title": title,
        "paragraphs": ["A first idea.", "  "],
        "steps": [],
        "key_points": ["Remember this"],
        "check_question": "What changes?",
        "caveat": ""
    })
}

/// An app with a scripted tutor, AI consent given, and one concept with two
/// pieces of material.
async fn tutored_app() -> (TestApp, FakeTutor, Concept, Material, Material) {
    let fake = FakeTutor::default();
    let app = TestApp::with_tutor(Tutor::Fake(fake.clone())).await;
    let _: AiStatus =
        app.json(Method::PUT, "/api/ai/consent", Some(json!({ "consented": true })), StatusCode::OK).await;
    let subject = app.subject("Linear algebra").await;
    let concept = app.concept(&subject, "Eigenvalues").await;
    let shared = app.material(&concept, "Lecture notes", "Av = λv").await;
    let private = app.material(&concept, "Private diary", "do not send").await;
    (app, fake, concept, shared, private)
}

async fn start(app: &TestApp, concept: &Concept, materials: &[&Material]) -> SessionSummary {
    let ids: Vec<_> = materials.iter().map(|m| m.id).collect();
    let uri = format!("/api/concepts/{}/sessions", concept.id);
    app.json(Method::POST, &uri, Some(json!({ "material_ids": ids })), StatusCode::CREATED).await
}

#[tokio::test]
async fn ai_status_reports_setup_and_consent() {
    let app = TestApp::new().await;
    let status: AiStatus = app.json(Method::GET, "/api/ai", None, StatusCode::OK).await;
    assert_eq!(status, AiStatus { configured: false, consented: false });

    let status: AiStatus = app
        .json(Method::PUT, "/api/ai/consent", Some(json!({ "consented": true })), StatusCode::OK)
        .await;
    assert!(status.consented);
    let status: AiStatus = app
        .json(Method::PUT, "/api/ai/consent", Some(json!({ "consented": false })), StatusCode::OK)
        .await;
    assert!(!status.consented, "consent can be withdrawn");
}

#[tokio::test]
async fn sessions_start_at_the_introduction_with_only_chosen_material() {
    let (app, _fake, concept, shared, _private) = tutored_app().await;
    let other_subject = app.subject("Other").await;
    let other_concept = app.concept(&other_subject, "Other").await;
    let foreign = app.material(&other_concept, "Foreign", "not this concept").await;

    let s = start(&app, &concept, &[&shared, &foreign]).await;
    assert_eq!(s.session.stage, LearningStage::Introduction);
    assert_eq!(s.session.material_ids, vec![shared.id], "material from other concepts is ignored");
    assert_eq!(s.concept_name, "Eigenvalues");
    assert_eq!(s.subject_name, "Linear algebra");

    let missing = app
        .status(Method::POST, "/api/concepts/999/sessions", Some(json!({ "material_ids": [] })))
        .await;
    assert_eq!(missing, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn tutor_replies_are_validated_saved_and_only_use_shared_material() {
    let (app, fake, concept, shared, _private) = tutored_app().await;
    let s = start(&app, &concept, &[&shared]).await;
    let uri = format!("/api/sessions/{}/activities", s.session.id);

    fake.reply(reply("  What eigenvalues are "));
    let a: Activity = app.json(Method::POST, &uri, Some(json!({})), StatusCode::CREATED).await;
    assert_eq!(a.content.title, "What eigenvalues are", "reply is validated and trimmed");
    assert_eq!(a.content.paragraphs, vec!["A first idea."]);
    assert_eq!(a.stage, LearningStage::Introduction);

    let prompt = fake.prompts.lock().unwrap()[0].clone();
    assert!(prompt.user.contains("Av = λv"));
    assert!(!prompt.user.contains("do not send"), "unshared material never leaves the device");

    // "This doesn't make sense yet": the learner picks another approach.
    fake.reply(reply("Like a stretchy sheet"));
    let b: Activity = app
        .json(Method::POST, &uri, Some(json!({ "strategy": "analogy" })), StatusCode::CREATED)
        .await;
    assert_eq!(b.strategy, Some(TeachingStrategy::Analogy));
    assert!(fake.prompts.lock().unwrap()[1].user.contains("<previous_reply>"));

    // A follow-up question.
    fake.reply(reply("Why only square matrices"));
    let c: Activity = app
        .json(Method::POST, &uri, Some(json!({ "follow_up": " Why square? " })), StatusCode::CREATED)
        .await;
    assert_eq!(c.follow_up.as_deref(), Some("Why square?"));

    // Everything persists and comes back in order.
    let detail: SessionDetail =
        app.json(Method::GET, &format!("/api/sessions/{}", s.session.id), None, StatusCode::OK).await;
    let titles: Vec<_> = detail.activities.iter().map(|a| a.content.title.as_str()).collect();
    assert_eq!(titles, ["What eigenvalues are", "Like a stretchy sheet", "Why only square matrices"]);
    assert_eq!(detail.summary.session.activity_count, 3);
}

#[tokio::test]
async fn unusable_replies_are_not_saved() {
    let (app, fake, concept, shared, _) = tutored_app().await;
    let s = start(&app, &concept, &[&shared]).await;
    let uri = format!("/api/sessions/{}/activities", s.session.id);

    fake.reply(json!({ "title": "", "paragraphs": [] }));
    let err: ApiError = app.json(Method::POST, &uri, Some(json!({})), StatusCode::BAD_GATEWAY).await;
    assert!(err.message.contains("wasn't saved"));

    fake.fail(AiError::Busy);
    assert_eq!(app.status(Method::POST, &uri, Some(json!({}))).await, StatusCode::SERVICE_UNAVAILABLE);

    let detail: SessionDetail =
        app.json(Method::GET, &format!("/api/sessions/{}", s.session.id), None, StatusCode::OK).await;
    assert!(detail.activities.is_empty());
}

#[tokio::test]
async fn nothing_is_sent_without_consent_or_a_key() {
    // Consent withdrawn: the tutor is never called (no reply is queued, so a call would panic).
    let (app, fake, concept, shared, _) = tutored_app().await;
    let s = start(&app, &concept, &[&shared]).await;
    let _: AiStatus =
        app.json(Method::PUT, "/api/ai/consent", Some(json!({ "consented": false })), StatusCode::OK).await;
    let uri = format!("/api/sessions/{}/activities", s.session.id);
    let err: ApiError = app.json(Method::POST, &uri, Some(json!({})), StatusCode::FORBIDDEN).await;
    assert_eq!(err.field.as_deref(), Some("consent"));
    assert!(fake.prompts.lock().unwrap().is_empty());

    // No key: a calm 503, and every non-AI route still works.
    let app = TestApp::new().await;
    let subject = app.subject("S").await;
    let concept = app.concept(&subject, "C").await;
    let s = start(&app, &concept, &[]).await;
    let uri = format!("/api/sessions/{}/activities", s.session.id);
    let err: ApiError = app.json(Method::POST, &uri, Some(json!({})), StatusCode::SERVICE_UNAVAILABLE).await;
    assert!(err.message.contains("isn't set up"));
}

#[tokio::test]
async fn the_learner_can_move_freely_between_stages_and_resume() {
    let (app, fake, concept, shared, _) = tutored_app().await;
    let first = start(&app, &concept, &[&shared]).await;
    let second = start(&app, &concept, &[]).await;
    let stage_uri = format!("/api/sessions/{}/stage", first.session.id);

    // Skipping ahead is the learner's right; so is going back.
    let s: SessionSummary = app
        .json(Method::PUT, &stage_uri, Some(json!({ "stage": "guided_practice" })), StatusCode::OK)
        .await;
    assert_eq!(s.session.stage, LearningStage::GuidedPractice);
    fake.reply(reply("Practice"));
    let a: Activity = app
        .json(Method::POST, &format!("/api/sessions/{}/activities", first.session.id), Some(json!({})), StatusCode::CREATED)
        .await;
    assert_eq!(a.stage, LearningStage::GuidedPractice);
    assert_eq!(a.strategy, Some(TeachingStrategy::StepByStep), "the stage's default approach");

    // The most recently active session comes first: "what was last studied".
    let recent: Vec<SessionSummary> = app.json(Method::GET, "/api/sessions", None, StatusCode::OK).await;
    let ids: Vec<_> = recent.iter().map(|s| s.session.id).collect();
    assert_eq!(ids, [first.session.id, second.session.id]);
    assert_eq!(recent[0].session.stage, LearningStage::GuidedPractice);

    let for_concept: Vec<SessionSummary> = app
        .json(Method::GET, &format!("/api/concepts/{}/sessions", concept.id), None, StatusCode::OK)
        .await;
    assert_eq!(for_concept.len(), 2);

    let bad = app.status(Method::PUT, &stage_uri, Some(json!({ "stage": "nonsense" }))).await;
    assert_eq!(bad, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn deleting_sessions_and_concepts_cleans_up() {
    let (app, fake, concept, shared, _) = tutored_app().await;
    let s = start(&app, &concept, &[&shared]).await;
    fake.reply(reply("Intro"));
    let uri = format!("/api/sessions/{}", s.session.id);
    let _: Activity = app
        .json(Method::POST, &format!("{uri}/activities"), Some(json!({})), StatusCode::CREATED)
        .await;

    // Deleting shared material just removes it from the session.
    app.status(Method::DELETE, &format!("/api/materials/{}", shared.id), None).await;
    let detail: SessionDetail = app.json(Method::GET, &uri, None, StatusCode::OK).await;
    assert!(detail.summary.session.material_ids.is_empty());
    assert_eq!(detail.activities.len(), 1);

    assert_eq!(app.status(Method::DELETE, &uri, None).await, StatusCode::NO_CONTENT);
    assert_eq!(app.status(Method::GET, &uri, None).await, StatusCode::NOT_FOUND);

    let s = start(&app, &concept, &[]).await;
    app.status(Method::DELETE, &format!("/api/concepts/{}", concept.id), None).await;
    let gone = app.status(Method::GET, &format!("/api/sessions/{}", s.session.id), None).await;
    assert_eq!(gone, StatusCode::NOT_FOUND);
}
