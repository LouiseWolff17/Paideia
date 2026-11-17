//! Paideia local application server.
//!
//! Serves the JSON API under `/api` and, in production, the built web interface.
//! Listens on loopback only by default (design doc §8, security requirements).

mod config;

use std::sync::Arc;

use axum::{extract::State, routing::get, Json, Router};
use paideia_core::api::HealthResponse;
use tower_http::{
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};
use tracing_subscriber::EnvFilter;

use crate::config::Config;

#[derive(Clone)]
struct AppState {
    config: Arc<Config>,
}

#[tokio::main]
async fn main() {
    // `.env` is optional; real environment variables take precedence.
    let _ = dotenvy::dotenv();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("paideia_server=info,tower_http=warn")),
        )
        .init();

    let config = match Config::from_env() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Configuration error: {e}");
            std::process::exit(1);
        }
    };

    if !config.bind.ip().is_loopback() {
        tracing::warn!(
            "PAIDEIA_BIND is {} — other devices on your network may be able to reach Paideia.",
            config.bind
        );
    }

    if let Err(e) = std::fs::create_dir_all(&config.data_dir) {
        eprintln!("Could not create data directory {:?}: {e}", config.data_dir);
        std::process::exit(1);
    }

    tracing::info!(?config, "starting Paideia");

    let bind = config.bind;
    let app = router(AppState { config: Arc::new(config) });

    let listener = tokio::net::TcpListener::bind(bind)
        .await
        .unwrap_or_else(|e| panic!("could not listen on {bind}: {e}"));

    tracing::info!("Paideia is running at http://{bind}");

    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await
        .expect("server error");
}

fn router(state: AppState) -> Router {
    let api = Router::new().route("/health", get(health));

    // Serve the built frontend, falling back to index.html so client-side
    // routes like /learn work when the page is reloaded.
    let dist = state.config.web_dist.clone();
    let frontend = ServeDir::new(&dist).fallback(ServeFile::new(dist.join("index.html")));

    Router::new()
        .nest("/api", api)
        .fallback_service(frontend)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn health(State(state): State<AppState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".into(),
        version: env!("CARGO_PKG_VERSION").into(),
        ai_configured: state.config.ai_configured(),
    })
}
