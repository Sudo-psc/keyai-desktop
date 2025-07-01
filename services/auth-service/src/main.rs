use anyhow::Result;
use axum::{
    extract::State,
    http::StatusCode,
    middleware,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use axum_prometheus::PrometheusMetricLayer;
use opentelemetry::global;
use opentelemetry::sdk::propagation::TraceContextPropagator;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tower::ServiceBuilder;
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    timeout::TimeoutLayer,
    trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
};
use tracing::{info, Level};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::{OpenApi, ToSchema};
use utoipa_swagger_ui::SwaggerUi;
use uuid::Uuid;
use validator::Validate;

mod auth;
mod config;
mod error;
mod handlers;
mod middleware as auth_middleware;
mod models;
mod repository;
mod services;

use crate::config::Config;
use crate::error::{ApiError, ApiResult};
use crate::services::AuthService;

#[derive(Clone)]
pub struct AppState {
    pub auth_service: Arc<AuthService>,
}

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::auth::register,
        handlers::auth::login,
        handlers::auth::refresh,
        handlers::auth::logout,
        handlers::auth::verify,
        handlers::health::health_check,
        handlers::health::ready_check,
    ),
    components(
        schemas(
            models::RegisterRequest,
            models::LoginRequest,
            models::RefreshRequest,
            models::AuthResponse,
            models::TokenPair,
            models::User,
            models::ErrorResponse,
            models::HealthResponse,
        )
    ),
    tags(
        (name = "auth", description = "Authentication endpoints"),
        (name = "health", description = "Health check endpoints"),
    ),
    info(
        title = "KeyAI Auth Service API",
        version = "1.0.0",
        description = "Authentication and authorization service for KeyAI platform",
        contact(
            name = "KeyAI Team",
            email = "api@keyai.com",
        ),
        license(
            name = "MIT",
        )
    )
)]
struct ApiDoc;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize configuration
    let config = Config::from_env()?;
    
    // Initialize tracing
    init_tracing(&config)?;
    
    // Initialize OpenTelemetry
    init_telemetry()?;
    
    info!("🚀 Starting KeyAI Auth Service v{}", env!("CARGO_PKG_VERSION"));
    
    // Initialize database connection
    let db_pool = PgPoolOptions::new()
        .max_connections(config.database.max_connections)
        .connect_timeout(Duration::from_secs(config.database.connect_timeout))
        .connect(&config.database.url)
        .await?;
    
    info!("✅ Connected to PostgreSQL database");
    
    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await?;
    
    info!("✅ Database migrations completed");
    
    // Initialize Redis connection
    let redis_client = redis::Client::open(config.redis.url.clone())?;
    let redis_conn = redis_client.get_connection_manager().await?;
    
    info!("✅ Connected to Redis");
    
    // Initialize services
    let auth_service = Arc::new(AuthService::new(
        db_pool.clone(),
        redis_conn,
        config.jwt.clone(),
    ));
    
    let app_state = AppState { auth_service };
    
    // Initialize metrics
    let (prometheus_layer, metric_handle) = PrometheusMetricLayer::pair();
    
    // Build application router
    let app = create_router(app_state)
        .layer(prometheus_layer)
        .layer(
            ServiceBuilder::new()
                .layer(
                    TraceLayer::new_for_http()
                        .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                        .on_request(DefaultOnRequest::new().level(Level::INFO))
                        .on_response(DefaultOnResponse::new().level(Level::INFO)),
                )
                .layer(CorsLayer::new()
                    .allow_origin(Any)
                    .allow_methods(Any)
                    .allow_headers(Any)
                    .expose_headers(Any)
                    .max_age(Duration::from_secs(3600)))
                .layer(CompressionLayer::new())
                .layer(TimeoutLayer::new(Duration::from_secs(30)))
        );
    
    // Add OpenAPI documentation
    let app = app
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/metrics", get(|| async move { metric_handle.render() }));
    
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));
    info!("🎯 Auth Service listening on {}", addr);
    
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;
    
    Ok(())
}

fn create_router(state: AppState) -> Router {
    Router::new()
        // Auth endpoints
        .route("/api/v1/auth/register", post(handlers::auth::register))
        .route("/api/v1/auth/login", post(handlers::auth::login))
        .route("/api/v1/auth/refresh", post(handlers::auth::refresh))
        .route("/api/v1/auth/logout", post(handlers::auth::logout))
        .route("/api/v1/auth/verify", get(handlers::auth::verify))
        
        // Health endpoints
        .route("/health", get(handlers::health::health_check))
        .route("/ready", get(handlers::health::ready_check))
        
        // Protected routes example
        .route(
            "/api/v1/auth/profile",
            get(handlers::auth::get_profile)
                .route_layer(middleware::from_fn_with_state(
                    state.clone(),
                    auth_middleware::jwt::require_auth,
                )),
        )
        
        .with_state(state)
}

fn init_tracing(config: &Config) -> Result<()> {
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(&config.log.level));
    
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(false)
        .with_thread_ids(true)
        .with_thread_names(true)
        .json();
    
    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .init();
    
    Ok(())
}

fn init_telemetry() -> Result<()> {
    global::set_text_map_propagator(TraceContextPropagator::new());
    
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(opentelemetry_otlp::new_exporter().tonic())
        .with_trace_config(
            opentelemetry::sdk::trace::config()
                .with_resource(opentelemetry::sdk::Resource::new(vec![
                    opentelemetry::KeyValue::new("service.name", "auth-service"),
                ])),
        )
        .install_batch(opentelemetry::runtime::Tokio)?;
    
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
    
    tracing_subscriber::registry()
        .with(telemetry)
        .init();
    
    Ok(())
}

// Module definitions
mod handlers {
    pub mod auth {
        use super::super::*;
        use crate::models::{LoginRequest, RegisterRequest, RefreshRequest, AuthResponse};
        
        #[utoipa::path(
            post,
            path = "/api/v1/auth/register",
            request_body = RegisterRequest,
            responses(
                (status = 201, description = "User registered successfully", body = AuthResponse),
                (status = 400, description = "Invalid input", body = ErrorResponse),
                (status = 409, description = "User already exists", body = ErrorResponse),
            ),
            tag = "auth"
        )]
        pub async fn register(
            State(state): State<AppState>,
            Json(req): Json<RegisterRequest>,
        ) -> ApiResult<Json<AuthResponse>> {
            req.validate()?;
            let response = state.auth_service.register(req).await?;
            Ok(Json(response))
        }
        
        #[utoipa::path(
            post,
            path = "/api/v1/auth/login",
            request_body = LoginRequest,
            responses(
                (status = 200, description = "Login successful", body = AuthResponse),
                (status = 401, description = "Invalid credentials", body = ErrorResponse),
            ),
            tag = "auth"
        )]
        pub async fn login(
            State(state): State<AppState>,
            Json(req): Json<LoginRequest>,
        ) -> ApiResult<Json<AuthResponse>> {
            req.validate()?;
            let response = state.auth_service.login(req).await?;
            Ok(Json(response))
        }
        
        #[utoipa::path(
            post,
            path = "/api/v1/auth/refresh",
            request_body = RefreshRequest,
            responses(
                (status = 200, description = "Token refreshed", body = AuthResponse),
                (status = 401, description = "Invalid refresh token", body = ErrorResponse),
            ),
            tag = "auth"
        )]
        pub async fn refresh(
            State(state): State<AppState>,
            Json(req): Json<RefreshRequest>,
        ) -> ApiResult<Json<AuthResponse>> {
            let response = state.auth_service.refresh_token(req).await?;
            Ok(Json(response))
        }
        
        #[utoipa::path(
            post,
            path = "/api/v1/auth/logout",
            security(
                ("bearer_auth" = [])
            ),
            responses(
                (status = 200, description = "Logout successful"),
                (status = 401, description = "Unauthorized", body = ErrorResponse),
            ),
            tag = "auth"
        )]
        pub async fn logout() -> ApiResult<StatusCode> {
            // Implementation depends on token invalidation strategy
            Ok(StatusCode::OK)
        }
        
        #[utoipa::path(
            get,
            path = "/api/v1/auth/verify",
            security(
                ("bearer_auth" = [])
            ),
            responses(
                (status = 200, description = "Token is valid"),
                (status = 401, description = "Invalid token", body = ErrorResponse),
            ),
            tag = "auth"
        )]
        pub async fn verify() -> ApiResult<StatusCode> {
            Ok(StatusCode::OK)
        }
        
        pub async fn get_profile() -> ApiResult<Json<serde_json::Value>> {
            Ok(Json(serde_json::json!({
                "message": "Protected route accessed successfully"
            })))
        }
    }
    
    pub mod health {
        use super::super::*;
        use crate::models::HealthResponse;
        
        #[utoipa::path(
            get,
            path = "/health",
            responses(
                (status = 200, description = "Service is healthy", body = HealthResponse),
            ),
            tag = "health"
        )]
        pub async fn health_check() -> Json<HealthResponse> {
            Json(HealthResponse {
                status: "healthy".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            })
        }
        
        #[utoipa::path(
            get,
            path = "/ready",
            responses(
                (status = 200, description = "Service is ready"),
                (status = 503, description = "Service not ready"),
            ),
            tag = "health"
        )]
        pub async fn ready_check(State(state): State<AppState>) -> StatusCode {
            // Check database and Redis connectivity
            if state.auth_service.is_ready().await {
                StatusCode::OK
            } else {
                StatusCode::SERVICE_UNAVAILABLE
            }
        }
    }
} 