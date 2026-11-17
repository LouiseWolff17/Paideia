//! One error type for every `/api` handler, turned into a calm JSON message.

use axum::{
    extract::rejection::JsonRejection,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use paideia_core::{api::ApiError, ValidationError};

use crate::ai::AiError;

#[derive(Debug)]
pub enum AppError {
    /// The thing asked for does not exist (or no longer exists).
    NotFound(&'static str),
    /// No API route at this path.
    NoRoute,
    /// The learner's input broke a rule.
    Invalid(ValidationError),
    /// The request body could not be read.
    BadRequest(String),
    /// Something went wrong on our side. Details are logged, not shown.
    Internal(String),
    /// No Anthropic API key is configured.
    AiNotConfigured,
    /// The learner has not yet agreed to AI requests.
    ConsentNeeded,
    /// The AI provider could not produce a usable reply.
    Ai(AiError),
}

impl From<AiError> for AppError {
    fn from(e: AiError) -> Self {
        match e {
            AiError::NotConfigured => AppError::AiNotConfigured,
            e => AppError::Ai(e),
        }
    }
}

impl From<ValidationError> for AppError {
    fn from(e: ValidationError) -> Self {
        AppError::Invalid(e)
    }
}

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        // sqlx errors describe the query, never the stored data or our inputs.
        AppError::Internal(e.to_string())
    }
}

impl From<JsonRejection> for AppError {
    fn from(e: JsonRejection) -> Self {
        AppError::BadRequest(e.body_text())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message, field) = match self {
            AppError::NotFound(what) => (
                StatusCode::NOT_FOUND,
                format!("That {what} doesn't exist any more. It may have been deleted."),
                None,
            ),
            AppError::NoRoute => {
                (StatusCode::NOT_FOUND, "There's nothing at this address.".to_owned(), None)
            }
            AppError::Invalid(e) => (StatusCode::UNPROCESSABLE_ENTITY, e.message, Some(e.field.to_owned())),
            AppError::BadRequest(detail) => {
                tracing::debug!(%detail, "rejected request body");
                (
                    StatusCode::BAD_REQUEST,
                    "The request couldn't be read. Please try again.".to_owned(),
                    None,
                )
            }
            AppError::Internal(detail) => {
                tracing::error!(%detail, "internal error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Something went wrong on the local server. Your saved work is safe; please try again."
                        .to_owned(),
                    None,
                )
            }
            AppError::AiNotConfigured => (
                StatusCode::SERVICE_UNAVAILABLE,
                "The AI tutor isn't set up yet. Add an Anthropic API key to .env and restart the \
                 server. Everything else in Paideia works without it."
                    .to_owned(),
                None,
            ),
            AppError::ConsentNeeded => (
                StatusCode::FORBIDDEN,
                "Before the tutor can help, please read how AI requests work and agree to them."
                    .to_owned(),
                Some("consent".to_owned()),
            ),
            AppError::Ai(e) => ai_response(e),
        };
        (status, Json(ApiError { message, field })).into_response()
    }
}

/// Log what went wrong (never any learner content) and pick a calm message.
fn ai_response(e: AiError) -> (StatusCode, String, Option<String>) {
    let (status, message) = match e {
        AiError::NotConfigured => unreachable!("mapped to AiNotConfigured"),
        AiError::KeyRejected => {
            tracing::warn!("Anthropic rejected the API key");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                "Anthropic didn't accept the API key. Check ANTHROPIC_API_KEY in .env, then restart the server.",
            )
        }
        AiError::Busy => {
            tracing::warn!("Anthropic is rate limiting or overloaded");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                "The AI tutor is busy right now. Please wait a minute and try again.",
            )
        }
        AiError::Unavailable(detail) => {
            tracing::warn!(%detail, "Anthropic unavailable");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                "The AI tutor couldn't be reached just now. Nothing was lost; please try again in a moment.",
            )
        }
        AiError::Declined => {
            tracing::info!("tutor declined a request");
            (
                StatusCode::BAD_GATEWAY,
                "The tutor couldn't help with this request. Try asking in a different way, or pick another approach.",
            )
        }
        AiError::BadReply(detail) => {
            tracing::warn!(%detail, "unusable tutor reply");
            (
                StatusCode::BAD_GATEWAY,
                "The tutor's reply didn't come through in a usable form, so it wasn't saved. Please try again.",
            )
        }
        AiError::Request(detail) => {
            tracing::error!(%detail, "Anthropic rejected the request");
            (
                StatusCode::BAD_GATEWAY,
                "Something went wrong while asking the tutor. Your saved work is safe; please try again.",
            )
        }
    };
    (status, message.to_owned(), None)
}

pub type AppResult<T> = Result<T, AppError>;
