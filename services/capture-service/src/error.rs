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
pub enum CaptureError {
    #[error("Failed to publish event: {0}")]
    PublishError(String),
    
    #[error("Circuit breaker is open")]
    CircuitBreakerOpen,
    
    #[error("Failed to publish {0} events in batch")]
    BatchPublishError(usize),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Failed to connect to NATS: {0}")]
    NatsConnectionError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("Channel send error")]
    ChannelError,
    
    #[error("Platform-specific error: {0}")]
    PlatformError(String),
    
    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl From<anyhow::Error> for CaptureError {
    fn from(err: anyhow::Error) -> Self {
        CaptureError::Unknown(err.to_string())
    }
}

// Telemetry helpers
impl CaptureError {
    pub fn error_type(&self) -> &'static str {
        match self {
            CaptureError::PublishError(_) => "publish_error",
            CaptureError::CircuitBreakerOpen => "circuit_breaker_open",
            CaptureError::BatchPublishError(_) => "batch_publish_error",
            CaptureError::ConfigError(_) => "config_error",
            CaptureError::NatsConnectionError(_) => "nats_connection_error",
            CaptureError::SerializationError(_) => "serialization_error",
            CaptureError::ChannelError => "channel_error",
            CaptureError::PlatformError(_) => "platform_error",
            CaptureError::Unknown(_) => "unknown_error",
        }
    }
    
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            CaptureError::PublishError(_) | 
            CaptureError::NatsConnectionError(_) |
            CaptureError::ChannelError
        )
    }
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

impl From<lapin::Error> for ServiceError {
    fn from(err: lapin::Error) -> Self {
        ServiceError::MessageQueue(err.to_string())
    }
}

impl From<config::ConfigError> for ServiceError {
    fn from(err: config::ConfigError) -> Self {
        ServiceError::Config(err.to_string())
    }
} 