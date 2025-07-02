use async_nats::Client as NatsClient;
use async_trait::async_trait;
use failsafe::{CircuitBreaker, Config as FailsafeConfig, Error as FailsafeError};
use serde_json;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, instrument};

use crate::error::CaptureError;
use crate::CaptureEvent;

#[async_trait]
pub trait EventPublisher: Send + Sync {
    async fn publish(&self, event: CaptureEvent) -> Result<(), CaptureError>;
    async fn publish_batch(&self, events: Vec<CaptureEvent>) -> Result<(), CaptureError>;
}

pub struct NatsPublisher {
    client: NatsClient,
    subject: String,
    circuit_breaker: Arc<CircuitBreaker>,
}

impl NatsPublisher {
    pub fn new(client: NatsClient, subject: String) -> Self {
        let circuit_breaker_config = FailsafeConfig::new()
            .failure_rate_threshold(0.5)
            .success_rate_threshold(0.9)
            .delay(Duration::from_secs(30))
            .build();

        Self {
            client,
            subject,
            circuit_breaker: Arc::new(CircuitBreaker::from_config(circuit_breaker_config)),
        }
    }

    #[instrument(skip(self, event))]
    async fn publish_internal(&self, event: &CaptureEvent) -> Result<(), CaptureError> {
        let payload = serde_json::to_vec(event)?;
        
        debug!(
            "Publishing event {} to subject {}",
            event.id,
            self.subject
        );

        self.client
            .publish(&self.subject, payload.into())
            .await
            .map_err(|e| CaptureError::PublishError(e.to_string()))?;

        debug!("Successfully published event {}", event.id);
        Ok(())
    }
}

#[async_trait]
impl EventPublisher for NatsPublisher {
    #[instrument(skip(self, event))]
    async fn publish(&self, event: CaptureEvent) -> Result<(), CaptureError> {
        let circuit_breaker = Arc::clone(&self.circuit_breaker);
        
        match circuit_breaker
            .call(|| async { self.publish_internal(&event).await })
            .await
        {
            Ok(result) => result,
            Err(FailsafeError::Inner(e)) => {
                error!("Circuit breaker inner error: {:?}", e);
                Err(e)
            }
            Err(FailsafeError::Rejected) => {
                error!("Circuit breaker open, rejecting publish");
                Err(CaptureError::CircuitBreakerOpen)
            }
        }
    }

    #[instrument(skip(self, events))]
    async fn publish_batch(&self, events: Vec<CaptureEvent>) -> Result<(), CaptureError> {
        let mut errors = Vec::new();
        
        for event in events {
            if let Err(e) = self.publish(event).await {
                errors.push(e);
            }
        }

        if !errors.is_empty() {
            error!("Failed to publish {} events", errors.len());
            return Err(CaptureError::BatchPublishError(errors.len()));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::mock;

    mock! {
        Publisher {}
        
        #[async_trait]
        impl EventPublisher for Publisher {
            async fn publish(&self, event: CaptureEvent) -> Result<(), CaptureError>;
            async fn publish_batch(&self, events: Vec<CaptureEvent>) -> Result<(), CaptureError>;
        }
    }

    #[tokio::test]
    async fn test_mock_publisher() {
        let mut mock = MockPublisher::new();
        
        mock.expect_publish()
            .times(1)
            .returning(|_| Ok(()));

        let event = CaptureEvent {
            id: uuid::Uuid::new_v4(),
            timestamp: chrono::Utc::now(),
            event_type: "test".to_string(),
            key_code: None,
            key_char: None,
            application: None,
            window_title: None,
            user_id: "test_user".to_string(),
        };

        assert!(mock.publish(event).await.is_ok());
    }
} 