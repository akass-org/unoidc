use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Configuration error: {0}")]
    ConfigError(#[from] anyhow::Error),

    #[error("Authentication failed")]
    AuthenticationFailed,

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("User not found")]
    UserNotFound,

    #[error("Client not found")]
    ClientNotFound,

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Forbidden")]
    Forbidden,

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("OIDC error: {0}")]
    OidcError(String),

    #[error("Token expired")]
    TokenExpired,

    #[error("Invalid token")]
    InvalidToken,

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Internal server error")]
    InternalServerError,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message): (StatusCode, String) = match self {
            AppError::DatabaseError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Database error".into())
            }
            AppError::ConfigError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Configuration error".into())
            }
            AppError::AuthenticationFailed => {
                (StatusCode::UNAUTHORIZED, "Authentication failed".into())
            }
            AppError::InvalidCredentials => {
                (StatusCode::UNAUTHORIZED, "Invalid credentials".into())
            }
            AppError::UserNotFound => {
                (StatusCode::NOT_FOUND, "User not found".into())
            }
            AppError::ClientNotFound => {
                (StatusCode::NOT_FOUND, "Client not found".into())
            }
            AppError::Unauthorized => {
                (StatusCode::UNAUTHORIZED, "Unauthorized".into())
            }
            AppError::Forbidden => {
                (StatusCode::FORBIDDEN, "Forbidden".into())
            }
            AppError::InvalidRequest(msg) => {
                (StatusCode::BAD_REQUEST, msg)
            }
            AppError::OidcError(msg) => {
                (StatusCode::BAD_REQUEST, msg)
            }
            AppError::TokenExpired => {
                (StatusCode::UNAUTHORIZED, "Token expired".into())
            }
            AppError::InvalidToken => {
                (StatusCode::UNAUTHORIZED, "Invalid token".into())
            }
            AppError::RateLimitExceeded => {
                (StatusCode::TOO_MANY_REQUESTS, "Rate limit exceeded".into())
            }
            AppError::InternalServerError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".into())
            }
        };

        let body = Json(json!({
            "error": error_message,
            "status": status.as_u16(),
        }));

        (status, body).into_response()
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
