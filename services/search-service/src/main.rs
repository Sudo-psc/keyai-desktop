use axum::{
    extract::{Query, State, Json},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use chrono::{DateTime, Utc};
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPool, Pool, Postgres};
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;
use tracing::{info, error, warn};
use utoipa::{OpenApi, ToSchema};
use utoipa_swagger_ui::SwaggerUi;
use uuid::Uuid;

#[derive(OpenApi)]
#[openapi(
    paths(
        search_text,
        search_semantic,
        search_hybrid,
        get_suggestions,
        get_trending,
        clear_cache
    ),
    components(
        schemas(
            SearchRequest,
            SearchResponse,
            SearchResult,
            SuggestionsResponse,
            TrendingResponse,
            SearchStats,
            ApiResponse
        )
    ),
    tags(
        (name = "search", description = "Search operations")
    )
)]
struct ApiDoc;

#[derive(Debug, Clone)]
struct AppState {
    db_pool: Arc<PgPool>,
    redis_client: Arc<redis::Client>,
    embedding_model: Arc<Mutex<Option<EmbeddingModel>>>,
    search_stats: Arc<Mutex<SearchStats>>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
struct SearchRequest {
    /// Search query
    query: String,
    
    /// Search type (text, semantic, hybrid)
    #[serde(default = "default_search_type")]
    search_type: String,
    
    /// Maximum results to return
    #[serde(default = "default_limit")]
    limit: i32,
    
    /// Offset for pagination
    #[serde(default)]
    offset: i32,
    
    /// Filter by application
    #[serde(skip_serializing_if = "Option::is_none")]
    application: Option<String>,
    
    /// Filter by date range (ISO 8601)
    #[serde(skip_serializing_if = "Option::is_none")]
    start_date: Option<DateTime<Utc>>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    end_date: Option<DateTime<Utc>>,
    
    /// Text weight for hybrid search (0.0 to 1.0)
    #[serde(default = "default_text_weight")]
    text_weight: f32,
    
    /// Semantic weight for hybrid search (0.0 to 1.0)
    #[serde(default = "default_semantic_weight")]
    semantic_weight: f32,
}

fn default_search_type() -> String {
    "hybrid".to_string()
}

fn default_limit() -> i32 {
    20
}

fn default_text_weight() -> f32 {
    0.7
}

fn default_semantic_weight() -> f32 {
    0.3
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
struct SearchResponse {
    results: Vec<SearchResult>,
    total_results: i64,
    query_time_ms: f64,
    search_type: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
struct SearchResult {
    id: Uuid,
    content: String,
    timestamp: DateTime<Utc>,
    application: Option<String>,
    relevance_score: f64,
    highlight: Option<String>,
    context: Option<SearchContext>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
struct SearchContext {
    before: Option<String>,
    after: Option<String>,
    window_title: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
struct SuggestionsResponse {
    suggestions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
struct TrendingResponse {
    trending_queries: Vec<TrendingQuery>,
    trending_applications: Vec<TrendingApp>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
struct TrendingQuery {
    query: String,
    count: i64,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
struct TrendingApp {
    application: String,
    event_count: i64,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
struct SearchStats {
    total_searches: u64,
    text_searches: u64,
    semantic_searches: u64,
    hybrid_searches: u64,
    cache_hits: u64,
    cache_misses: u64,
    avg_query_time_ms: f64,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
}

// Placeholder for embedding model
struct EmbeddingModel;

impl EmbeddingModel {
    async fn encode(&self, text: &str) -> Result<Vec<f32>, anyhow::Error> {
        // TODO: Implement actual embedding generation
        // For now, return a dummy embedding
        Ok(vec![0.1; 384])
    }
    
    fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
        
        if norm_a == 0.0 || norm_b == 0.0 {
            0.0
        } else {
            dot_product / (norm_a * norm_b)
        }
    }
}

// Cache helpers
async fn get_cached_results(
    redis_conn: &mut redis::aio::Connection,
    cache_key: &str,
) -> Option<Vec<SearchResult>> {
    match redis_conn.get::<_, String>(cache_key).await {
        Ok(cached) => {
            match serde_json::from_str(&cached) {
                Ok(results) => Some(results),
                Err(_) => None,
            }
        }
        Err(_) => None,
    }
}

async fn cache_results(
    redis_conn: &mut redis::aio::Connection,
    cache_key: &str,
    results: &[SearchResult],
    ttl_seconds: usize,
) {
    if let Ok(json) = serde_json::to_string(results) {
        let _ = redis_conn.set_ex::<_, _, ()>(cache_key, json, ttl_seconds).await;
    }
}

// API Endpoints
#[utoipa::path(
    post,
    path = "/api/v1/search/text",
    tag = "search",
    request_body = SearchRequest,
    responses(
        (status = 200, description = "Search completed", body = ApiResponse<SearchResponse>)
    )
)]
async fn search_text(
    State(state): State<AppState>,
    Json(request): Json<SearchRequest>,
) -> impl IntoResponse {
    let start = std::time::Instant::now();
    
    // Check cache
    let cache_key = format!("search:text:{}", serde_json::to_string(&request).unwrap_or_default());
    let mut redis_conn = state.redis_client.get_async_connection().await.ok();
    
    if let Some(ref mut conn) = redis_conn {
        if let Some(cached_results) = get_cached_results(conn, &cache_key).await {
            let mut stats = state.search_stats.lock().await;
            stats.cache_hits += 1;
            
            return Json(ApiResponse {
                success: true,
                data: Some(SearchResponse {
                    results: cached_results,
                    total_results: cached_results.len() as i64,
                    query_time_ms: start.elapsed().as_secs_f64() * 1000.0,
                    search_type: "text".to_string(),
                }),
                error: None,
            });
        }
    }
    
    // Build SQL query
    let mut query = String::from(
        r#"
        SELECT 
            id, 
            masked_text as content, 
            timestamp, 
            application,
            ts_rank(to_tsvector('english', masked_text), plainto_tsquery('english', $1)) as relevance_score,
            ts_headline('english', masked_text, plainto_tsquery('english', $1)) as highlight
        FROM key_events
        WHERE to_tsvector('english', masked_text) @@ plainto_tsquery('english', $1)
        "#
    );
    
    if request.application.is_some() {
        query.push_str(" AND application = $2");
    }
    
    if request.start_date.is_some() {
        query.push_str(" AND timestamp >= $3");
    }
    
    if request.end_date.is_some() {
        query.push_str(" AND timestamp <= $4");
    }
    
    query.push_str(" ORDER BY relevance_score DESC, timestamp DESC");
    query.push_str(&format!(" LIMIT {} OFFSET {}", request.limit, request.offset));
    
    // Execute search
    let results: Vec<(Uuid, String, DateTime<Utc>, Option<String>, f32, String)> = 
        sqlx::query_as(&query)
            .bind(&request.query)
            .fetch_all(&*state.db_pool)
            .await
            .unwrap_or_default();
    
    let search_results: Vec<SearchResult> = results
        .into_iter()
        .map(|(id, content, timestamp, application, score, highlight)| SearchResult {
            id,
            content,
            timestamp,
            application,
            relevance_score: score as f64,
            highlight: Some(highlight),
            context: None,
        })
        .collect();
    
    // Cache results
    if let Some(ref mut conn) = redis_conn {
        cache_results(conn, &cache_key, &search_results, 300).await;
    }
    
    // Update stats
    let mut stats = state.search_stats.lock().await;
    stats.total_searches += 1;
    stats.text_searches += 1;
    stats.cache_misses += 1;
    let query_time = start.elapsed().as_secs_f64() * 1000.0;
    stats.avg_query_time_ms = (stats.avg_query_time_ms * (stats.total_searches - 1) as f64 + query_time) 
        / stats.total_searches as f64;
    
    Json(ApiResponse {
        success: true,
        data: Some(SearchResponse {
            total_results: search_results.len() as i64,
            results: search_results,
            query_time_ms: query_time,
            search_type: "text".to_string(),
        }),
        error: None,
    })
}

#[utoipa::path(
    post,
    path = "/api/v1/search/semantic",
    tag = "search",
    request_body = SearchRequest,
    responses(
        (status = 200, description = "Search completed", body = ApiResponse<SearchResponse>)
    )
)]
async fn search_semantic(
    State(state): State<AppState>,
    Json(request): Json<SearchRequest>,
) -> impl IntoResponse {
    let start = std::time::Instant::now();
    
    // Check if embedding model is available
    let model_lock = state.embedding_model.lock().await;
    if model_lock.is_none() {
        return Json(ApiResponse {
            success: false,
            data: None,
            error: Some("Semantic search not available - embedding model not loaded".to_string()),
        });
    }
    
    // TODO: Implement actual semantic search with embeddings
    // For now, return empty results
    let query_time = start.elapsed().as_secs_f64() * 1000.0;
    
    // Update stats
    let mut stats = state.search_stats.lock().await;
    stats.total_searches += 1;
    stats.semantic_searches += 1;
    
    Json(ApiResponse {
        success: true,
        data: Some(SearchResponse {
            results: vec![],
            total_results: 0,
            query_time_ms: query_time,
            search_type: "semantic".to_string(),
        }),
        error: None,
    })
}

#[utoipa::path(
    post,
    path = "/api/v1/search/hybrid",
    tag = "search",
    request_body = SearchRequest,
    responses(
        (status = 200, description = "Search completed", body = ApiResponse<SearchResponse>)
    )
)]
async fn search_hybrid(
    State(state): State<AppState>,
    Json(mut request): Json<SearchRequest>,
) -> impl IntoResponse {
    let start = std::time::Instant::now();
    
    // Perform text search
    request.search_type = "text".to_string();
    let text_results = search_text(State(state.clone()), Json(request.clone())).await;
    
    // Extract text results
    let text_search_results = match text_results.into_response().into_body() {
        axum::body::Body::Empty => vec![],
        _ => vec![], // TODO: Properly extract results
    };
    
    // Perform semantic search (if available)
    request.search_type = "semantic".to_string();
    let semantic_results = search_semantic(State(state.clone()), Json(request.clone())).await;
    
    // TODO: Implement Reciprocal Rank Fusion (RRF) to combine results
    
    let query_time = start.elapsed().as_secs_f64() * 1000.0;
    
    // Update stats
    let mut stats = state.search_stats.lock().await;
    stats.total_searches += 1;
    stats.hybrid_searches += 1;
    
    Json(ApiResponse {
        success: true,
        data: Some(SearchResponse {
            results: text_search_results,
            total_results: 0,
            query_time_ms: query_time,
            search_type: "hybrid".to_string(),
        }),
        error: None,
    })
}

#[utoipa::path(
    get,
    path = "/api/v1/search/suggestions",
    tag = "search",
    params(
        ("q" = String, Query, description = "Partial query for suggestions")
    ),
    responses(
        (status = 200, description = "Suggestions retrieved", body = ApiResponse<SuggestionsResponse>)
    )
)]
async fn get_suggestions(
    State(state): State<AppState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    let query = params.get("q").cloned().unwrap_or_default();
    
    if query.is_empty() {
        return Json(ApiResponse {
            success: true,
            data: Some(SuggestionsResponse { suggestions: vec![] }),
            error: None,
        });
    }
    
    // Get recent unique queries from database
    let recent_queries: Vec<String> = sqlx::query_scalar(
        r#"
        SELECT DISTINCT masked_text
        FROM key_events
        WHERE masked_text ILIKE $1
        ORDER BY timestamp DESC
        LIMIT 10
        "#
    )
    .bind(format!("%{}%", query))
    .fetch_all(&*state.db_pool)
    .await
    .unwrap_or_default();
    
    // Use fuzzy matcher to rank suggestions
    let matcher = SkimMatcherV2::default();
    let mut suggestions: Vec<(String, i64)> = recent_queries
        .into_iter()
        .filter_map(|text| {
            matcher.fuzzy_match(&text, &query).map(|score| (text, score))
        })
        .collect();
    
    // Sort by score descending
    suggestions.sort_by(|a, b| b.1.cmp(&a.1));
    
    let suggestions: Vec<String> = suggestions
        .into_iter()
        .take(5)
        .map(|(text, _)| text)
        .collect();
    
    Json(ApiResponse {
        success: true,
        data: Some(SuggestionsResponse { suggestions }),
        error: None,
    })
}

#[utoipa::path(
    get,
    path = "/api/v1/search/trending",
    tag = "search",
    responses(
        (status = 200, description = "Trending data retrieved", body = ApiResponse<TrendingResponse>)
    )
)]
async fn get_trending(
    State(state): State<AppState>,
) -> impl IntoResponse {
    // Get trending applications
    let trending_apps: Vec<(String, i64)> = sqlx::query_as(
        r#"
        SELECT application, COUNT(*) as count
        FROM key_events
        WHERE application IS NOT NULL
          AND timestamp > NOW() - INTERVAL '24 hours'
        GROUP BY application
        ORDER BY count DESC
        LIMIT 10
        "#
    )
    .fetch_all(&*state.db_pool)
    .await
    .unwrap_or_default();
    
    let trending_applications: Vec<TrendingApp> = trending_apps
        .into_iter()
        .map(|(app, count)| TrendingApp {
            application: app,
            event_count: count,
        })
        .collect();
    
    // TODO: Implement trending queries based on search history
    let trending_queries = vec![];
    
    Json(ApiResponse {
        success: true,
        data: Some(TrendingResponse {
            trending_queries,
            trending_applications,
        }),
        error: None,
    })
}

#[utoipa::path(
    post,
    path = "/api/v1/search/cache/clear",
    tag = "search",
    responses(
        (status = 200, description = "Cache cleared", body = ApiResponse<String>)
    )
)]
async fn clear_cache(
    State(state): State<AppState>,
) -> impl IntoResponse {
    match state.redis_client.get_async_connection().await {
        Ok(mut conn) => {
            match redis::cmd("FLUSHDB").query_async::<_, ()>(&mut conn).await {
                Ok(_) => Json(ApiResponse {
                    success: true,
                    data: Some("Cache cleared successfully".to_string()),
                    error: None,
                }),
                Err(e) => Json(ApiResponse {
                    success: false,
                    data: None,
                    error: Some(format!("Failed to clear cache: {}", e)),
                }),
            }
        }
        Err(e) => Json(ApiResponse {
            success: false,
            data: None,
            error: Some(format!("Failed to connect to Redis: {}", e)),
        }),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Database connection
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://keyai:keyai@localhost:5432/keyai".to_string());
    
    let db_pool = PgPool::connect(&database_url).await?;
    
    // Redis connection
    let redis_url = std::env::var("REDIS_URL")
        .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    
    let redis_client = redis::Client::open(redis_url)?;
    
    // Create shared state
    let state = AppState {
        db_pool: Arc::new(db_pool),
        redis_client: Arc::new(redis_client),
        embedding_model: Arc::new(Mutex::new(None)), // TODO: Load actual model
        search_stats: Arc::new(Mutex::new(SearchStats {
            total_searches: 0,
            text_searches: 0,
            semantic_searches: 0,
            hybrid_searches: 0,
            cache_hits: 0,
            cache_misses: 0,
            avg_query_time_ms: 0.0,
        })),
    };

    // Build the application
    let app = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/api/v1/search/text", post(search_text))
        .route("/api/v1/search/semantic", post(search_semantic))
        .route("/api/v1/search/hybrid", post(search_hybrid))
        .route("/api/v1/search/suggestions", get(get_suggestions))
        .route("/api/v1/search/trending", get(get_trending))
        .route("/api/v1/search/cache/clear", post(clear_cache))
        .route("/health", get(|| async { "OK" }))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8004")
        .await
        .unwrap();
    
    info!("Search Service listening on http://0.0.0.0:8004");
    info!("Swagger UI available at http://0.0.0.0:8004/swagger-ui");
    
    axum::serve(listener, app).await.unwrap();
    
    Ok(())
}
