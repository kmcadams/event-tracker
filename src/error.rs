use actix_web::{HttpResponse, ResponseError};
use log::{error, warn};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Internal server error: {0}")]
    InternalError(String),
    #[error("Bad request: {0}")]
    BadRequest(String),
    #[error("Resource not found: {0}")]
    NotFound(String),
    #[error("Unexpected error: {0}")]
    Unexpected(String),
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        match self {
            AppError::BadRequest(msg) => {
                warn!("Bad request: {}", msg);
                HttpResponse::BadRequest()
                    .json(serde_json::json!({ "error": "Bad request", "message": msg }))
            }
            AppError::NotFound(msg) => {
                warn!("Resource not found: {}", msg);
                HttpResponse::NotFound()
                    .json(serde_json::json!({ "error": "Not found", "message": msg }))
            }
            AppError::Unexpected(msg) => {
                error!("Unexpected error: {}", msg);
                HttpResponse::InternalServerError()
                    .json(serde_json::json!({ "error": "Unexpected error", "message": msg }))
            }
            AppError::InternalError(msg) => {
                error!("Internal server error: {}", msg);
                HttpResponse::InternalServerError()
                    .json(serde_json::json!({ "error": "Internal server error", "message": msg }))
            }
        }
    }
}
