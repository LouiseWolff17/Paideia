//! One error type for every `/api` handler, turned into a calm JSON message.

use axum::{
    extract::rejection::JsonRejection,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use paideia_core::{api::ApiError, ValidationError};

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
        };
        (status, Json(ApiError { message, field })).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;
