use thiserror::Error;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use std::fmt;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum MaskingError {
    #[error("Invalid pattern: {0}")]
    InvalidPattern(String),
    
    #[error("Text too large: {size} characters (max: {max})")]
    TextTooLarge { size: usize, max: usize },
    
    #[error("Processing timeout")]
    ProcessingTimeout,
    
    #[error("Pattern compilation failed: {0}")]
    PatternCompilation(String),
    
    #[error("Cache error: {0}")]
    CacheError(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

#[derive(Debug)]
pub enum ServiceError {
    Internal(String),
    BadRequest(String),
    NotFound(String),
    Unauthorized(String),
    ServiceUnavailable(String),
    Config(String),
    Database(String),
    MessageQueue(String),
    Masking(MaskingError),
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
    message: String,
    request_id: String,
}

impl fmt::Display for ServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServiceError::Internal(msg) => write!(f, "Internal error: {}", msg),
            ServiceError::BadRequest(msg) => write!(f, "Bad request: {}", msg),
            ServiceError::NotFound(msg) => write!(f, "Not found: {}", msg),
            ServiceError::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
            ServiceError::ServiceUnavailable(msg) => write!(f, "Service unavailable: {}", msg),
            ServiceError::Config(msg) => write!(f, "Configuration error: {}", msg),
            ServiceError::Database(msg) => write!(f, "Database error: {}", msg),
            ServiceError::MessageQueue(msg) => write!(f, "Message queue error: {}", msg),
            ServiceError::Masking(err) => write!(f, "Masking error: {}", err),
        }
    }
}

impl std::error::Error for ServiceError {}

impl IntoResponse for ServiceError {
    fn into_response(self) -> Response {
        let (status, error_type) = match &self {
            ServiceError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error"),
            ServiceError::BadRequest(_) => (StatusCode::BAD_REQUEST, "Bad Request"),
            ServiceError::NotFound(_) => (StatusCode::NOT_FOUND, "Not Found"),
            ServiceError::Unauthorized(_) => (StatusCode::UNAUTHORIZED, "Unauthorized"),
            ServiceError::ServiceUnavailable(_) => (StatusCode::SERVICE_UNAVAILABLE, "Service Unavailable"),
            ServiceError::Config(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Configuration Error"),
            ServiceError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Database Error"),
            ServiceError::MessageQueue(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Message Queue Error"),
            ServiceError::Masking(_) => (StatusCode::UNPROCESSABLE_ENTITY, "Masking Error"),
        };

        let body = Json(ErrorResponse {
            error: error_type.to_string(),
            message: self.to_string(),
            request_id: Uuid::new_v4().to_string(),
        });

        (status, body).into_response()
    }
}

// Conversion implementations
impl From<anyhow::Error> for ServiceError {
    fn from(err: anyhow::Error) -> Self {
        ServiceError::Internal(err.to_string())
    }
}

impl From<MaskingError> for ServiceError {
    fn from(err: MaskingError) -> Self {
        ServiceError::Masking(err)
    }
}

impl From<config::ConfigError> for ServiceError {
    fn from(err: config::ConfigError) -> Self {
        ServiceError::Config(err.to_string())
    }
}

impl From<redis::RedisError> for ServiceError {
    fn from(err: redis::RedisError) -> Self {
        ServiceError::Database(err.to_string())
    }
} 