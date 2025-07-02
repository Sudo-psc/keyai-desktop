// Health check is integrated into main.rs via the /health endpoint

use anyhow::Result;
use tonic::{transport::Server, Request, Response, Status};
use tonic_health::server::{HealthReporter, HealthService, health_reporter};
use tracing::{info, error};

pub async fn run_health_server(port: u16) -> Result<()> {
    let (mut reporter, service) = health_reporter();
    
    // Set initial health status
    reporter
        .set_serving::<HealthService<HealthReporter>>()
        .await;
    
    let addr = format!("0.0.0.0:{}", port).parse()?;
    
    info!("Starting health check server on {}", addr);
    
    Server::builder()
        .add_service(service)
        .serve(addr)
        .await
        .map_err(|e| {
            error!("Health server error: {}", e);
            anyhow::anyhow!("Health server failed: {}", e)
        })
}

// Custom health check implementation with dependencies
pub struct HealthChecker {
    nats_healthy: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl HealthChecker {
    pub fn new() -> Self {
        Self {
            nats_healthy: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true)),
        }
    }
    
    pub fn set_nats_health(&self, healthy: bool) {
        self.nats_healthy.store(healthy, std::sync::atomic::Ordering::Relaxed);
    }
    
    pub fn is_healthy(&self) -> bool {
        self.nats_healthy.load(std::sync::atomic::Ordering::Relaxed)
    }
} 