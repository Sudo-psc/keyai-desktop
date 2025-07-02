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
pub enum DatabaseError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    
    #[error("Query failed: {0}")]
    QueryFailed(String),
    
    #[error("Migration failed: {0}")]
    MigrationFailed(String),
    
    #[error("Transaction failed: {0}")]
    TransactionFailed(String),
    
    #[error("Constraint violation: {0}")]
    ConstraintViolation(String),
    
    #[error("Record not found")]
    NotFound,
    
    #[error("Timeout: {0}")]
    Timeout(String),
}

#[derive(Debug)]
pub enum ServiceError {
    Internal(String),
    BadRequest(String),
    NotFound(String),
    Unauthorized(String),
    ServiceUnavailable(String),
    Config(String),
    Database(DatabaseError),
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
            ServiceError::Database(err) => write!(f, "Database error: {}", err),
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

impl From<DatabaseError> for ServiceError {
    fn from(err: DatabaseError) -> Self {
        ServiceError::Database(err)
    }
}

impl From<sqlx::Error> for ServiceError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => ServiceError::NotFound("Record not found".to_string()),
            sqlx::Error::Database(db_err) => {
                ServiceError::Database(DatabaseError::QueryFailed(db_err.to_string()))
            }
            sqlx::Error::PoolTimedOut => {
                ServiceError::Database(DatabaseError::Timeout("Connection pool timeout".to_string()))
            }
            sqlx::Error::Io(io_err) => {
                ServiceError::Database(DatabaseError::ConnectionFailed(io_err.to_string()))
            }
            _ => ServiceError::Database(DatabaseError::QueryFailed(err.to_string())),
        }
    }
}

impl From<config::ConfigError> for ServiceError {
    fn from(err: config::ConfigError) -> Self {
        ServiceError::Config(err.to_string())
    }
} 