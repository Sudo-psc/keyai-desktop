use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::{sync::Arc, time::Duration};
use tokio::time::timeout;
use tracing::{error, info, warn};
use uuid::Uuid;

mod config;
mod error;
mod masker;
mod metrics;

use config::Config;
use error::ServiceError;
use masker::{MaskingEngine, MaskingResult};
use metrics::Metrics;

#[derive(Debug, Serialize, Deserialize)]
pub struct MaskingRequest {
    pub text: String,
    pub context: Option<String>,
    pub user_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MaskingResponse {
    pub original_text: String,
    pub masked_text: String,
    pub detected_patterns: Vec<String>,
    pub processing_time_ms: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub service: String,
    pub version: String,
    pub uptime_seconds: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MetricsResponse {
    pub texts_processed: u64,
    pub patterns_detected: u64,
    pub errors: u64,
    pub average_processing_time_ms: f64,
}

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub masking_engine: Arc<MaskingEngine>,
    pub metrics: Arc<Metrics>,
    pub start_time: std::time::Instant,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration
    let config = Config::from_env().map_err(|e| {
        eprintln!("Failed to load configuration: {}", e);
        e
    })?;

    // Initialize tracing
    init_tracing(&config.log_level)?;

    info!("Starting masker-service v{}", env!("CARGO_PKG_VERSION"));
    info!("Configuration loaded: {:?}", config);

    // Initialize components
    let masking_engine = Arc::new(MaskingEngine::new());
    let metrics = Arc::new(Metrics::new());
    
    // Create application state
    let state = AppState {
        config: Arc::new(config.clone()),
        masking_engine,
        metrics,
        start_time: std::time::Instant::now(),
    };

    // Create router
    let app = create_router(state);

    // Start server
    let listener = tokio::net::TcpListener::bind(&config.server_address).await?;
    info!("Masker service listening on {}", config.server_address);

    axum::serve(listener, app).await?;

    Ok(())
}

fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/metrics", get(get_metrics))
        .route("/mask", post(mask_text))
        .route("/mask/batch", post(mask_batch))
        .with_state(state)
}

async fn health_check(State(state): State<AppState>) -> Result<Json<HealthResponse>, ServiceError> {
    let uptime = state.start_time.elapsed().as_secs();
    
    Ok(Json(HealthResponse {
        status: "healthy".to_string(),
        service: "masker-service".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: uptime,
    }))
}

async fn get_metrics(State(state): State<AppState>) -> Result<Json<MetricsResponse>, ServiceError> {
    let metrics = state.metrics.as_ref();
    
    Ok(Json(MetricsResponse {
        texts_processed: metrics.get_processed(),
        patterns_detected: metrics.get_patterns_detected(),
        errors: metrics.get_errors(),
        average_processing_time_ms: metrics.get_average_processing_time(),
    }))
}

async fn mask_text(
    State(state): State<AppState>,
    Json(request): Json<MaskingRequest>,
) -> Result<Json<MaskingResponse>, ServiceError> {
    let _timer = state.metrics.start_processing_timer();
    let start_time = std::time::Instant::now();

    // Validate input
    if request.text.trim().is_empty() {
        return Err(ServiceError::BadRequest("Text cannot be empty".to_string()));
    }

    if request.text.len() > 10000 {
        return Err(ServiceError::BadRequest("Text too long (max 10000 characters)".to_string()));
    }

    // Process with timeout
    let result = timeout(
        Duration::from_secs(5),
        state.masking_engine.mask_text(&request.text, request.context.as_deref())
    ).await;

    let masking_result = match result {
        Ok(Ok(result)) => result,
        Ok(Err(e)) => {
            state.metrics.increment_errors();
            error!("Masking failed: {}", e);
            return Err(ServiceError::Internal(format!("Masking failed: {}", e)));
        }
        Err(_) => {
            state.metrics.increment_errors();
            warn!("Masking timeout for text length: {}", request.text.len());
            return Err(ServiceError::ServiceUnavailable("Request timeout".to_string()));
        }
    };

    let processing_time = start_time.elapsed().as_millis() as u64;
    
    // Update metrics
    state.metrics.increment_processed();
    if !masking_result.detected_patterns.is_empty() {
        state.metrics.add_patterns_detected(masking_result.detected_patterns.len() as u64);
    }

    Ok(Json(MaskingResponse {
        original_text: request.text,
        masked_text: masking_result.masked_text,
        detected_patterns: masking_result.detected_patterns,
        processing_time_ms: processing_time,
    }))
}

#[derive(Debug, Deserialize)]
struct BatchRequest {
    texts: Vec<MaskingRequest>,
}

#[derive(Debug, Serialize)]
struct BatchResponse {
    results: Vec<MaskingResponse>,
    total_processing_time_ms: u64,
}

async fn mask_batch(
    State(state): State<AppState>,
    Json(request): Json<BatchRequest>,
) -> Result<Json<BatchResponse>, ServiceError> {
    let start_time = std::time::Instant::now();

    // Validate batch size
    if request.texts.is_empty() {
        return Err(ServiceError::BadRequest("Batch cannot be empty".to_string()));
    }

    if request.texts.len() > 100 {
        return Err(ServiceError::BadRequest("Batch too large (max 100 items)".to_string()));
    }

    let mut results = Vec::new();

    for text_request in request.texts {
        match mask_single_text(&state, text_request).await {
            Ok(response) => results.push(response),
            Err(e) => {
                error!("Failed to process text in batch: {}", e);
                // Continue processing other texts in batch
                results.push(MaskingResponse {
                    original_text: "".to_string(),
                    masked_text: "".to_string(),
                    detected_patterns: vec![],
                    processing_time_ms: 0,
                });
            }
        }
    }

    let total_time = start_time.elapsed().as_millis() as u64;

    Ok(Json(BatchResponse {
        results,
        total_processing_time_ms: total_time,
    }))
}

async fn mask_single_text(
    state: &AppState,
    request: MaskingRequest,
) -> Result<MaskingResponse, ServiceError> {
    let start_time = std::time::Instant::now();

    let result = state.masking_engine.mask_text(&request.text, request.context.as_deref()).await
        .map_err(|e| ServiceError::Internal(format!("Masking failed: {}", e)))?;

    let processing_time = start_time.elapsed().as_millis() as u64;

    Ok(MaskingResponse {
        original_text: request.text,
        masked_text: result.masked_text,
        detected_patterns: result.detected_patterns,
        processing_time_ms: processing_time,
    })
}

fn init_tracing(log_level: &str) -> Result<(), Box<dyn std::error::Error>> {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

    let log_level = log_level.parse().unwrap_or(tracing::Level::INFO);

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("masker_service={},axum=debug", log_level).into()),
        )
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    Ok(())
} 