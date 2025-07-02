use thiserror::Error;

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