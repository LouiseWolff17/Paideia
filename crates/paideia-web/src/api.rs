//! Calls to the local Paideia server. The browser never talks to Anthropic directly.

use gloo_net::http::Request;
use paideia_core::api::HealthResponse;

pub async fn health() -> Result<HealthResponse, String> {
    let resp = Request::get("/api/health")
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !resp.ok() {
        return Err(format!("server answered with status {}", resp.status()));
    }
    resp.json::<HealthResponse>().await.map_err(|e| e.to_string())
}
