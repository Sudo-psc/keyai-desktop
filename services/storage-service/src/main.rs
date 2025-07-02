use anyhow::Result;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use axum_prometheus::PrometheusMetricLayer;
use chrono::{DateTime, Utc};
use lapin::{
    options::*, types::FieldTable, Connection, ConnectionProperties,
};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::mpsc;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing::{error, info};
use utoipa::{OpenApi, ToSchema};
use utoipa_swagger_ui::SwaggerUi;
use uuid::Uuid;
use validator::Validate;

mod config;
mod error;
mod models;
mod repository;

use config::Config;
use error::ApiError;
use models::{CreateEventRequest, Event, EventQuery, PaginatedEvents};
use repository::EventRepository;

#[derive(Clone)]
pub struct AppState {
    pub repo: Arc<EventRepository>,
    pub rabbitmq: Arc<Connection>,
}

#[derive(OpenApi)]
#[openapi(
    paths(
        create_event,
        get_event,
        list_events,
        delete_event,
        health_check
    ),
    components(
        schemas(Event, CreateEventRequest, EventQuery, PaginatedEvents, ApiError)
    ),
    tags(
        (name = "storage", description = "Event storage operations"),
        (name = "health", description = "Health check endpoints")
    ),
    info(
        title = "KeyAI Storage Service API",
        version = "1.0.0",
        description = "Service for storing and managing captured events"
    )
)]
struct ApiDoc;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize services
    let config = Config::from_env()?;
    
    // Database connection
    let db_pool = PgPool::connect(&config.database_url).await?;
    sqlx::migrate!("./migrations").run(&db_pool).await?;
    
    // RabbitMQ connection
    let rabbitmq_conn = Connection::connect(&config.rabbitmq_url, ConnectionProperties::default()).await?;
    
    // App state
    let state = AppState {
        repo: Arc::new(EventRepository::new(db_pool)),
        rabbitmq: Arc::new(rabbitmq_conn),
    };

    // Start RabbitMQ consumer
    let (tx, mut rx) = mpsc::unbounded_channel::<Event>();
    let consumer_state = state.clone();
    tokio::spawn(async move {
        consume_events(consumer_state, tx).await;
    });

    let repo_clone = state.repo.clone();
    tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            if let Err(e) = repo_clone.create_event(event).await {
                error!("Failed to store event: {}", e);
            }
        }
    });

    // API router
    let (prometheus_layer, metric_handle) = PrometheusMetricLayer::pair();
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/events", post(create_event).get(list_events))
        .route("/events/:id", get(get_event).delete(delete_event))
        .route("/metrics", get(|| async move { metric_handle.render() }))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()))
        .with_state(state);

    // Start server
    let addr = config.server_address.parse::<SocketAddr>()?;
    info!("Storage Service listening on {}", addr);
    axum::serve(tokio::net::TcpListener::bind(addr).await?, app).await?;
    
    Ok(())
}

async fn consume_events(state: AppState, tx: mpsc::UnboundedSender<Event>) {
    let channel = state.rabbitmq.create_channel().await.unwrap();
    
    channel
        .queue_declare("masked_events", QueueDeclareOptions::default(), FieldTable::default())
        .await
        .unwrap();
        
    let mut consumer = channel
        .basic_consume(
            "masked_events",
            "storage_consumer",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await
        .unwrap();

    info!("Event consumer started, waiting for messages...");

    while let Some(delivery) = consumer.next().await {
        if let Ok(delivery) = delivery {
            if let Ok(event) = serde_json::from_slice::<Event>(&delivery.data) {
                if tx.send(event).is_err() {
                    error!("Receiver dropped, cannot process event");
                }
            }
            delivery.ack(BasicAckOptions::default()).await.unwrap();
        }
    }
}

// API Handlers

#[utoipa::path(
    post,
    path = "/events",
    request_body = CreateEventRequest,
    responses(
        (status = 201, description = "Event created", body = Event),
        (status = 400, description = "Invalid request", body = ApiError),
    ),
    tag = "storage"
)]
async fn create_event(
    State(state): State<AppState>,
    Json(payload): Json<CreateEventRequest>,
) -> Result<impl IntoResponse, ApiError> {
    payload.validate()?;
    let event = state.repo.create_event(payload.into()).await?;
    Ok((StatusCode::CREATED, Json(event)))
}

#[utoipa::path(
    get,
    path = "/events/{id}",
    params(("id" = Uuid, Path, description = "Event ID")),
    responses(
        (status = 200, description = "Event found", body = Event),
        (status = 404, description = "Event not found", body = ApiError),
    ),
    tag = "storage"
)]
async fn get_event(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Event>, ApiError> {
    let event = state.repo.get_event(id).await?.ok_or(ApiError::NotFound)?;
    Ok(Json(event))
}

#[utoipa::path(
    get,
    path = "/events",
    params(EventQuery),
    responses(
        (status = 200, description = "Events listed", body = PaginatedEvents),
    ),
    tag = "storage"
)]
async fn list_events(
    State(state): State<AppState>,
    Query(query): Query<EventQuery>,
) -> Result<Json<PaginatedEvents>, ApiError> {
    let events = state.repo.list_events(query).await?;
    Ok(Json(events))
}

#[utoipa::path(
    delete,
    path = "/events/{id}",
    params(("id" = Uuid, Path, description = "Event ID")),
    responses(
        (status = 204, description = "Event deleted"),
        (status = 404, description = "Event not found", body = ApiError),
    ),
    tag = "storage"
)]
async fn delete_event(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    state.repo.delete_event(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/health",
    responses((status = 200, description = "Service is healthy")),
    tag = "health"
)]
async fn health_check() -> StatusCode {
    StatusCode::OK
}
