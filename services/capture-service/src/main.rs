use anyhow::Result;
use async_nats::Client as NatsClient;
use chrono::{DateTime, Utc};
use prometheus::{Counter, Histogram, Registry};
use rdev::{listen, Event, EventType};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{error, info, instrument, warn};
use uuid::Uuid;

mod config;
mod error;
mod health;
mod metrics;
mod publisher;

use config::Config;
use error::CaptureError;
use publisher::{EventPublisher, NatsPublisher};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureEvent {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub event_type: String,
    pub key_code: Option<u32>,
    pub key_char: Option<String>,
    pub application: Option<String>,
    pub window_title: Option<String>,
    pub user_id: String,
}

pub struct CaptureService {
    publisher: Arc<dyn EventPublisher>,
    config: Config,
    metrics: ServiceMetrics,
}

struct ServiceMetrics {
    events_captured: Counter,
    events_published: Counter,
    publish_errors: Counter,
    capture_latency: Histogram,
}

impl CaptureService {
    pub fn new(publisher: Arc<dyn EventPublisher>, config: Config) -> Self {
        let registry = Registry::new();
        let metrics = ServiceMetrics {
            events_captured: Counter::new("capture_events_total", "Total events captured")
                .expect("metric creation failed"),
            events_published: Counter::new("capture_events_published", "Events published successfully")
                .expect("metric creation failed"),
            publish_errors: Counter::new("capture_publish_errors", "Failed publish attempts")
                .expect("metric creation failed"),
            capture_latency: Histogram::with_opts(
                prometheus::HistogramOpts::new(
                    "capture_latency_seconds",
                    "Time to capture and process event"
                ).buckets(vec![0.001, 0.005, 0.01, 0.05, 0.1]),
            ).expect("metric creation failed"),
        };

        // Register metrics
        registry.register(Box::new(metrics.events_captured.clone())).unwrap();
        registry.register(Box::new(metrics.events_published.clone())).unwrap();
        registry.register(Box::new(metrics.publish_errors.clone())).unwrap();
        registry.register(Box::new(metrics.capture_latency.clone())).unwrap();

        Self {
            publisher,
            config,
            metrics,
        }
    }

    #[instrument(skip(self))]
    pub async fn run(&self) -> Result<()> {
        info!("Starting capture service");
        
        let (tx, mut rx) = mpsc::channel::<CaptureEvent>(1000);
        
        // Spawn keyboard listener in blocking thread
        let tx_clone = tx.clone();
        let user_id = self.config.user_id.clone();
        
        std::thread::spawn(move || {
            info!("Starting keyboard listener thread");
            
            if let Err(e) = listen(move |event| {
                if let EventType::KeyPress(key) = event.event_type {
                    let capture_event = CaptureEvent {
                        id: Uuid::new_v4(),
                        timestamp: Utc::now(),
                        event_type: "keypress".to_string(),
                        key_code: Some(key as u32),
                        key_char: None, // Will be resolved by downstream service
                        application: get_active_application(),
                        window_title: get_active_window_title(),
                        user_id: user_id.clone(),
                    };
                    
                    // Non-blocking send
                    if let Err(e) = tx_clone.try_send(capture_event) {
                        warn!("Channel full, dropping event: {}", e);
                    }
                }
            }) {
                error!("Keyboard listener error: {}", e);
            }
        });

        // Process events asynchronously
        while let Some(event) = rx.recv().await {
            let publisher = Arc::clone(&self.publisher);
            let metrics = self.metrics.clone();
            
            // Spawn task to publish without blocking receiver
            tokio::spawn(async move {
                let timer = metrics.capture_latency.start_timer();
                
                metrics.events_captured.inc();
                
                match publisher.publish(event).await {
                    Ok(_) => {
                        metrics.events_published.inc();
                    }
                    Err(e) => {
                        error!("Failed to publish event: {}", e);
                        metrics.publish_errors.inc();
                    }
                }
                
                timer.observe_duration();
            });
        }

        Ok(())
    }
}

// Platform-specific implementations
#[cfg(target_os = "windows")]
fn get_active_application() -> Option<String> {
    // Windows implementation using WinAPI
    None
}

#[cfg(target_os = "macos")]
fn get_active_application() -> Option<String> {
    // macOS implementation using Accessibility API
    None
}

#[cfg(target_os = "linux")]
fn get_active_application() -> Option<String> {
    // Linux implementation using X11/Wayland
    None
}

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
fn get_active_application() -> Option<String> {
    None
}

fn get_active_window_title() -> Option<String> {
    // Similar platform-specific implementation
    None
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("capture_service=debug,info")
        .json()
        .init();

    // Load configuration
    let config = Config::from_env()?;
    info!("Loaded configuration: {:?}", config);

    // Connect to NATS
    let nats_client = async_nats::connect(&config.nats_url).await?;
    info!("Connected to NATS at {}", config.nats_url);

    // Create publisher
    let publisher = Arc::new(NatsPublisher::new(
        nats_client,
        config.nats_subject.clone(),
    ));

    // Create and run service
    let service = CaptureService::new(publisher, config.clone());
    
    // Spawn health check server
    let health_handle = tokio::spawn(health::run_health_server(config.health_port));
    
    // Run capture service
    let capture_handle = tokio::spawn(async move {
        if let Err(e) = service.run().await {
            error!("Capture service error: {}", e);
        }
    });

    // Wait for shutdown
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            info!("Received shutdown signal");
        }
        _ = capture_handle => {
            error!("Capture service terminated unexpectedly");
        }
        _ = health_handle => {
            error!("Health server terminated unexpectedly");
        }
    }

    info!("Shutting down capture service");
    Ok(())
} 