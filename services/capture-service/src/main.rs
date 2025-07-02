use anyhow::Result;
use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use axum_prometheus::PrometheusMetricLayer;
use lapin::{
    options::*, types::FieldTable, BasicProperties, Channel, Connection, ConnectionProperties,
};
use opentelemetry::global;
use opentelemetry::sdk::propagation::TraceContextPropagator;
use rdev::{listen, Event, EventType};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::{mpsc, RwLock};
use tower::ServiceBuilder;
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    timeout::TimeoutLayer,
    trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
};
use tracing::{error, info, warn, Level};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::{OpenApi, ToSchema};
use utoipa_swagger_ui::SwaggerUi;
use uuid::Uuid;

mod config;
mod error;
mod health;
mod metrics;
mod publisher;

use config::Config;
use error::ServiceError;
use metrics::Metrics;

#[derive(Clone)]
struct AppState {
    config: Arc<Config>,
    channel: Arc<RwLock<Channel>>,
    metrics: Arc<Metrics>,
    event_tx: mpsc::UnboundedSender<KeystrokeEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
struct KeystrokeEvent {
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    id: Uuid,
    #[schema(example = "keypress")]
    event_type: String,
    #[schema(example = "a")]
    key: Option<String>,
    #[schema(example = "65")]
    key_code: Option<u32>,
    #[schema(example = 1704067200000)]
    timestamp: u64,
    #[schema(example = "user123")]
    user_id: String,
    #[schema(example = "session456")]
    session_id: String,
    #[schema(example = "VS Code")]
    application: Option<String>,
    #[schema(example = {"shift": false, "ctrl": false, "alt": false, "meta": false})]
    modifiers: KeyModifiers,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
struct KeyModifiers {
    shift: bool,
    ctrl: bool,
    alt: bool,
    meta: bool,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
struct CaptureStatus {
    #[schema(example = true)]
    active: bool,
    #[schema(example = 12345)]
    events_captured: u64,
    #[schema(example = 123)]
    events_published: u64,
    #[schema(example = 2)]
    errors: u64,
    #[schema(example = "2024-01-01T00:00:00Z")]
    started_at: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
struct ApiError {
    #[schema(example = "Bad Request")]
    error: String,
    #[schema(example = "Invalid input")]
    message: String,
    #[schema(example = "REQ123")]
    request_id: String,
}

#[derive(OpenApi)]
#[openapi(
    paths(
        health_check,
        get_status,
        start_capture,
        stop_capture,
        toggle_capture
    ),
    components(
        schemas(CaptureStatus, ApiError, KeystrokeEvent, KeyModifiers)
    ),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "capture", description = "Capture control endpoints")
    ),
    info(
        title = "KeyAI Capture Service",
        version = "1.0.0",
        description = "Keyboard capture microservice for KeyAI",
        contact(
            name = "KeyAI Team",
            email = "api@keyai.com"
        ),
        license(
            name = "MIT"
        )
    ),
    servers(
        (url = "http://localhost:3001", description = "Local development"),
        (url = "http://capture-service:3001", description = "Docker environment")
    )
)]
struct ApiDoc;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    init_tracing();

    // Load configuration
    let config = Arc::new(Config::from_env()?);
    info!("Starting Capture Service on {}", config.server_address);

    // Initialize RabbitMQ connection
    let conn = Connection::connect(&config.rabbitmq_url, ConnectionProperties::default()).await?;
    let channel = Arc::new(RwLock::new(conn.create_channel().await?));

    // Declare exchange and queue
    {
        let chan = channel.read().await;
        chan.exchange_declare(
            "keystrokes",
            lapin::ExchangeKind::Topic,
            ExchangeDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;

        chan.queue_declare(
            "keystrokes.raw",
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;

        chan.queue_bind(
            "keystrokes.raw",
            "keystrokes",
            "raw.*",
            QueueBindOptions::default(),
            FieldTable::default(),
        )
        .await?;
    }

    // Create metrics
    let metrics = Arc::new(Metrics::new());

    // Create event channel
    let (event_tx, mut event_rx) = mpsc::unbounded_channel::<KeystrokeEvent>();

    // Create app state
    let state = AppState {
        config: config.clone(),
        channel: channel.clone(),
        metrics: metrics.clone(),
        event_tx,
    };

    // Start event publisher task
    let publisher_state = state.clone();
    tokio::spawn(async move {
        while let Some(event) = event_rx.recv().await {
            if let Err(e) = publish_event(&publisher_state, event).await {
                error!("Failed to publish event: {}", e);
                publisher_state.metrics.increment_errors();
            }
        }
    });

    // Start keyboard capture in separate thread
    let capture_tx = state.event_tx.clone();
    let capture_metrics = state.metrics.clone();
    thread::spawn(move || {
        info!("Starting keyboard capture thread");
        
        if let Err(e) = listen(move |event| {
            if let Some(keystroke) = process_keyboard_event(event) {
                capture_metrics.increment_captured();
                if let Err(e) = capture_tx.send(keystroke) {
                    error!("Failed to send keystroke event: {}", e);
                }
            }
        }) {
            error!("Keyboard capture error: {:?}", e);
        }
    });

    // Setup metrics layer
    let (prometheus_layer, metric_handle) = PrometheusMetricLayer::pair();

    // Build router
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/status", get(get_status))
        .route("/capture/start", post(start_capture))
        .route("/capture/stop", post(stop_capture))
        .route("/capture/toggle", post(toggle_capture))
        .route("/metrics", get(|| async move { metric_handle.render() }))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .layer(
            ServiceBuilder::new()
                .layer(prometheus_layer)
                .layer(
                    TraceLayer::new_for_http()
                        .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                        .on_request(DefaultOnRequest::new().level(Level::INFO))
                        .on_response(DefaultOnResponse::new().level(Level::INFO)),
                )
                .layer(CompressionLayer::new())
                .layer(TimeoutLayer::new(Duration::from_secs(30)))
                .layer(
                    CorsLayer::new()
                        .allow_origin(Any)
                        .allow_methods(Any)
                        .allow_headers(Any),
                ),
        )
        .with_state(state);

    // Start server
    let addr = config.server_address.parse::<SocketAddr>()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!("Capture Service listening on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}

fn init_tracing() {
    global::set_text_map_propagator(TraceContextPropagator::new());

    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(opentelemetry_otlp::new_exporter().tonic())
        .with_trace_config(
            opentelemetry::sdk::trace::config()
                .with_resource(opentelemetry::sdk::Resource::new(vec![
                    opentelemetry::KeyValue::new("service.name", "capture-service"),
                ])),
        )
        .install_simple()
        .unwrap();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer().json())
        .with(tracing_opentelemetry::layer().with_tracer(tracer))
        .init();
}

fn process_keyboard_event(event: Event) -> Option<KeystrokeEvent> {
    match event.event_type {
        EventType::KeyPress(key) => {
            let key_str = format!("{:?}", key);
            Some(KeystrokeEvent {
                id: Uuid::new_v4(),
                event_type: "keypress".to_string(),
                key: Some(key_str),
                key_code: None,
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64,
                user_id: "default".to_string(), // TODO: Get from auth context
                session_id: "default".to_string(), // TODO: Generate session ID
                application: get_active_application(),
                modifiers: KeyModifiers {
                    shift: false, // TODO: Detect modifiers
                    ctrl: false,
                    alt: false,
                    meta: false,
                },
            })
        }
        _ => None,
    }
}

async fn publish_event(state: &AppState, event: KeystrokeEvent) -> Result<()> {
    let payload = serde_json::to_vec(&event)?;
    let channel = state.channel.read().await;
    
    channel
        .basic_publish(
            "keystrokes",
            "raw.keystroke",
            BasicPublishOptions::default(),
            &payload,
            BasicProperties::default()
                .with_content_type("application/json".into())
                .with_delivery_mode(2), // Persistent
        )
        .await?;

    state.metrics.increment_published();
    Ok(())
}

fn get_active_application() -> Option<String> {
    // Platform-specific implementation
    #[cfg(target_os = "windows")]
    {
        // TODO: Implement Windows active window detection
        Some("Unknown".to_string())
    }
    #[cfg(target_os = "macos")]
    {
        // TODO: Implement macOS active window detection
        Some("Unknown".to_string())
    }
    #[cfg(target_os = "linux")]
    {
        // TODO: Implement Linux active window detection
        Some("Unknown".to_string())
    }
}

// API Handlers

#[utoipa::path(
    get,
    path = "/health",
    tag = "health",
    responses(
        (status = 200, description = "Service is healthy", body = String),
        (status = 503, description = "Service is unhealthy", body = ApiError)
    )
)]
async fn health_check(State(state): State<AppState>) -> impl IntoResponse {
    // Check RabbitMQ connection
    let channel = state.channel.read().await;
    match channel.status().state() {
        lapin::ChannelState::Connected => (StatusCode::OK, "healthy"),
        _ => (StatusCode::SERVICE_UNAVAILABLE, "unhealthy"),
    }
}

#[utoipa::path(
    get,
    path = "/status",
    tag = "capture",
    responses(
        (status = 200, description = "Capture status", body = CaptureStatus)
    )
)]
async fn get_status(State(state): State<AppState>) -> impl IntoResponse {
    let status = CaptureStatus {
        active: true, // TODO: Implement capture control
        events_captured: state.metrics.get_captured(),
        events_published: state.metrics.get_published(),
        errors: state.metrics.get_errors(),
        started_at: chrono::Utc::now().to_rfc3339(),
    };
    
    Json(status)
}

#[utoipa::path(
    post,
    path = "/capture/start",
    tag = "capture",
    responses(
        (status = 200, description = "Capture started"),
        (status = 400, description = "Capture already active", body = ApiError)
    )
)]
async fn start_capture(State(_state): State<AppState>) -> impl IntoResponse {
    // TODO: Implement capture control
    StatusCode::OK
}

#[utoipa::path(
    post,
    path = "/capture/stop",
    tag = "capture",
    responses(
        (status = 200, description = "Capture stopped"),
        (status = 400, description = "Capture not active", body = ApiError)
    )
)]
async fn stop_capture(State(_state): State<AppState>) -> impl IntoResponse {
    // TODO: Implement capture control
    StatusCode::OK
}

#[utoipa::path(
    post,
    path = "/capture/toggle",
    tag = "capture",
    responses(
        (status = 200, description = "Capture toggled", body = CaptureStatus)
    )
)]
async fn toggle_capture(State(state): State<AppState>) -> impl IntoResponse {
    // TODO: Implement capture control
    get_status(State(state)).await
} 