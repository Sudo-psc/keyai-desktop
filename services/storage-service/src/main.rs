use axum::{
    extract::{Path, Query, State, Json},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post, delete},
    Router,
};
use chrono::{DateTime, Utc};
use lapin::{
    options::*, Connection, ConnectionProperties,
    types::FieldTable,
};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPool, Pool, Postgres};
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;
use tracing::{info, error};
use utoipa::{OpenApi, ToSchema};
use utoipa_swagger_ui::SwaggerUi;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // For now, just a simple health check server
    let app = Router::new()
        .route("/health", get(|| async { "OK" }))
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8003")
        .await
        .unwrap();
    
    info!("Storage Service listening on http://0.0.0.0:8003");
    
    axum::serve(listener, app).await.unwrap();
    
    Ok(())
}
