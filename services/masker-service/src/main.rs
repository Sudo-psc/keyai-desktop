use axum::{
    extract::{State, Json},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use lapin::{
    options::*, Connection, ConnectionProperties, message::DeliveryResult,
    types::FieldTable,
};
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
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
        mask_text,
        mask_batch,
        get_patterns,
        update_patterns,
        get_statistics
    ),
    components(
        schemas(
            MaskRequest,
            MaskBatchRequest,
            MaskResponse,
            PiiPattern,
            MaskStatistics,
            ApiResponse
        )
    ),
    tags(
        (name = "masker", description = "PII masking operations")
    )
)]
struct ApiDoc;

#[derive(Debug, Clone)]
struct AppState {
    masker: Arc<Mutex<PiiMasker>>,
    queue_connection: Arc<Mutex<Option<Connection>>>,
    stats: Arc<Mutex<MaskStatistics>>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
struct MaskRequest {
    /// Text to be masked
    #[schema(example = "My email is john.doe@example.com and my CPF is 123.456.789-00")]
    text: String,
    
    /// Optional context (application name, window title, etc)
    context: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
struct MaskBatchRequest {
    /// Array of texts to be masked
    texts: Vec<MaskRequest>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
struct MaskResponse {
    /// Original text
    original: String,
    
    /// Masked text
    masked: String,
    
    /// List of PII found and masked
    pii_found: Vec<PiiFound>,
    
    /// Processing time in milliseconds
    processing_time_ms: f64,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
struct PiiFound {
    /// Type of PII detected
    pii_type: String,
    
    /// Position in original text
    start: usize,
    end: usize,
    
    /// Original value (partially masked for security)
    original_value: String,
    
    /// Masked value
    masked_value: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
struct PiiPattern {
    /// Name of the pattern
    name: String,
    
    /// Regex pattern
    pattern: String,
    
    /// Replacement template
    replacement: String,
    
    /// Is pattern enabled
    enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
struct MaskStatistics {
    /// Total texts processed
    total_processed: u64,
    
    /// Total PII items found
    total_pii_found: u64,
    
    /// Processing stats by PII type
    pii_type_stats: std::collections::HashMap<String, u64>,
    
    /// Average processing time
    avg_processing_time_ms: f64,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
}

// PII Masker implementation
struct PiiMasker {
    patterns: Vec<PiiPattern>,
}

impl PiiMasker {
    fn new() -> Self {
        Self {
            patterns: Self::default_patterns(),
        }
    }

    fn default_patterns() -> Vec<PiiPattern> {
        vec![
            PiiPattern {
                name: "email".to_string(),
                pattern: r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}".to_string(),
                replacement: "***@***.***".to_string(),
                enabled: true,
            },
            PiiPattern {
                name: "cpf".to_string(),
                pattern: r"\d{3}\.\d{3}\.\d{3}-\d{2}".to_string(),
                replacement: "***.***.***-**".to_string(),
                enabled: true,
            },
            PiiPattern {
                name: "phone_br".to_string(),
                pattern: r"(\+55\s?)?(\(?\d{2}\)?\s?)?\d{4,5}-?\d{4}".to_string(),
                replacement: "(**) *****-****".to_string(),
                enabled: true,
            },
            PiiPattern {
                name: "credit_card".to_string(),
                pattern: r"\d{4}[\s-]?\d{4}[\s-]?\d{4}[\s-]?\d{4}".to_string(),
                replacement: "****-****-****-****".to_string(),
                enabled: true,
            },
            PiiPattern {
                name: "ipv4".to_string(),
                pattern: r"\b(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\b".to_string(),
                replacement: "***.***.***.***".to_string(),
                enabled: true,
            },
        ]
    }

    fn mask_text(&self, text: &str) -> (String, Vec<PiiFound>) {
        let mut masked = text.to_string();
        let mut pii_found = Vec::new();
        let mut offset = 0;

        for pattern in &self.patterns {
            if !pattern.enabled {
                continue;
            }

            if let Ok(regex) = Regex::new(&pattern.pattern) {
                let mut new_masked = masked.clone();
                let mut local_offset = 0;

                for mat in regex.find_iter(&masked) {
                    let start = mat.start();
                    let end = mat.end();
                    let original_value = mat.as_str().to_string();
                    
                    // Partially mask the original value for logging
                    let partial_mask = if original_value.len() > 4 {
                        format!("{}...{}", 
                            &original_value[..2], 
                            &original_value[original_value.len()-2..])
                    } else {
                        "****".to_string()
                    };

                    pii_found.push(PiiFound {
                        pii_type: pattern.name.clone(),
                        start: start + offset,
                        end: end + offset,
                        original_value: partial_mask,
                        masked_value: pattern.replacement.clone(),
                    });

                    // Replace in the new string
                    new_masked.replace_range(
                        (start + local_offset)..(end + local_offset),
                        &pattern.replacement
                    );
                    
                    // Update offset for next replacements
                    local_offset += pattern.replacement.len() as i32 - (end - start) as i32;
                }

                masked = new_masked;
                offset += local_offset;
            }
        }

        (masked, pii_found)
    }
}

// Queue event structure
#[derive(Debug, Serialize, Deserialize)]
struct KeyEvent {
    id: Uuid,
    timestamp: u64,
    text: String,
    application: Option<String>,
    window_title: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct MaskedEvent {
    id: Uuid,
    timestamp: u64,
    original_text: String,
    masked_text: String,
    pii_found: Vec<PiiFound>,
    application: Option<String>,
    window_title: Option<String>,
}

// Queue consumer
async fn setup_queue_consumer(app_state: Arc<AppState>) -> Result<(), Box<dyn std::error::Error>> {
    let addr = std::env::var("RABBITMQ_URL")
        .unwrap_or_else(|_| "amqp://guest:guest@localhost:5672".to_string());
    
    let conn = Connection::connect(&addr, ConnectionProperties::default()).await?;
    let channel = conn.create_channel().await?;
    
    // Declare queues
    channel.queue_declare(
        "key_events",
        QueueDeclareOptions::default(),
        FieldTable::default(),
    ).await?;
    
    channel.queue_declare(
        "masked_events",
        QueueDeclareOptions::default(),
        FieldTable::default(),
    ).await?;
    
    // Set up consumer
    let consumer = channel
        .basic_consume(
            "key_events",
            "masker_consumer",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;
    
    let app_state_clone = app_state.clone();
    
    info!("Queue consumer started, waiting for messages...");
    
    // Process messages
    tokio::spawn(async move {
        let mut consumer_stream = consumer;
        while let Some(delivery) = consumer_stream.recv().await {
            match delivery {
                Ok(delivery) => {
                    if let Ok(event) = serde_json::from_slice::<KeyEvent>(&delivery.data) {
                        // Mask the text
                        let masker = app_state_clone.masker.lock().await;
                        let (masked_text, pii_found) = masker.mask_text(&event.text);
                        
                        // Create masked event
                        let masked_event = MaskedEvent {
                            id: event.id,
                            timestamp: event.timestamp,
                            original_text: event.text,
                            masked_text,
                            pii_found,
                            application: event.application,
                            window_title: event.window_title,
                        };
                        
                        // Publish to masked_events queue
                        if let Ok(payload) = serde_json::to_vec(&masked_event) {
                            let _ = channel
                                .basic_publish(
                                    "",
                                    "masked_events",
                                    BasicPublishOptions::default(),
                                    &payload,
                                    BasicProperties::default(),
                                )
                                .await;
                        }
                        
                        // Update statistics
                        let mut stats = app_state_clone.stats.lock().await;
                        stats.total_processed += 1;
                        stats.total_pii_found += masked_event.pii_found.len() as u64;
                        
                        for pii in &masked_event.pii_found {
                            *stats.pii_type_stats.entry(pii.pii_type.clone()).or_insert(0) += 1;
                        }
                    }
                    
                    delivery
                        .ack(BasicAckOptions::default())
                        .await
                        .expect("Failed to ack");
                }
                Err(error) => {
                    error!("Error receiving message: {:?}", error);
                }
            }
        }
    });
    
    // Store connection
    let mut conn_lock = app_state.queue_connection.lock().await;
    *conn_lock = Some(conn);
    
    Ok(())
}

// API Endpoints
#[utoipa::path(
    post,
    path = "/api/v1/mask",
    tag = "masker",
    request_body = MaskRequest,
    responses(
        (status = 200, description = "Text masked successfully", body = ApiResponse<MaskResponse>),
        (status = 400, description = "Invalid request", body = ApiResponse<String>)
    )
)]
async fn mask_text(
    State(state): State<AppState>,
    Json(request): Json<MaskRequest>,
) -> impl IntoResponse {
    let start = std::time::Instant::now();
    
    let masker = state.masker.lock().await;
    let (masked, pii_found) = masker.mask_text(&request.text);
    
    let processing_time_ms = start.elapsed().as_secs_f64() * 1000.0;
    
    // Update statistics
    let mut stats = state.stats.lock().await;
    stats.total_processed += 1;
    stats.total_pii_found += pii_found.len() as u64;
    stats.avg_processing_time_ms = 
        (stats.avg_processing_time_ms * (stats.total_processed - 1) as f64 + processing_time_ms) 
        / stats.total_processed as f64;
    
    for pii in &pii_found {
        *stats.pii_type_stats.entry(pii.pii_type.clone()).or_insert(0) += 1;
    }
    
    Json(ApiResponse {
        success: true,
        data: Some(MaskResponse {
            original: request.text,
            masked,
            pii_found,
            processing_time_ms,
        }),
        error: None,
    })
}

#[utoipa::path(
    post,
    path = "/api/v1/mask/batch",
    tag = "masker",
    request_body = MaskBatchRequest,
    responses(
        (status = 200, description = "Batch masked successfully", body = ApiResponse<Vec<MaskResponse>>)
    )
)]
async fn mask_batch(
    State(state): State<AppState>,
    Json(request): Json<MaskBatchRequest>,
) -> impl IntoResponse {
    let masker = state.masker.lock().await;
    let mut responses = Vec::new();
    
    for text_request in request.texts {
        let start = std::time::Instant::now();
        let (masked, pii_found) = masker.mask_text(&text_request.text);
        let processing_time_ms = start.elapsed().as_secs_f64() * 1000.0;
        
        responses.push(MaskResponse {
            original: text_request.text,
            masked,
            pii_found,
            processing_time_ms,
        });
    }
    
    // Update statistics
    let mut stats = state.stats.lock().await;
    for response in &responses {
        stats.total_processed += 1;
        stats.total_pii_found += response.pii_found.len() as u64;
        stats.avg_processing_time_ms = 
            (stats.avg_processing_time_ms * (stats.total_processed - 1) as f64 + response.processing_time_ms) 
            / stats.total_processed as f64;
        
        for pii in &response.pii_found {
            *stats.pii_type_stats.entry(pii.pii_type.clone()).or_insert(0) += 1;
        }
    }
    
    Json(ApiResponse {
        success: true,
        data: Some(responses),
        error: None,
    })
}

#[utoipa::path(
    get,
    path = "/api/v1/patterns",
    tag = "masker",
    responses(
        (status = 200, description = "Patterns retrieved", body = ApiResponse<Vec<PiiPattern>>)
    )
)]
async fn get_patterns(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let masker = state.masker.lock().await;
    
    Json(ApiResponse {
        success: true,
        data: Some(masker.patterns.clone()),
        error: None,
    })
}

#[utoipa::path(
    post,
    path = "/api/v1/patterns",
    tag = "masker",
    request_body = Vec<PiiPattern>,
    responses(
        (status = 200, description = "Patterns updated", body = ApiResponse<String>)
    )
)]
async fn update_patterns(
    State(state): State<AppState>,
    Json(patterns): Json<Vec<PiiPattern>>,
) -> impl IntoResponse {
    let mut masker = state.masker.lock().await;
    masker.patterns = patterns;
    
    Json(ApiResponse {
        success: true,
        data: Some("Patterns updated successfully".to_string()),
        error: None,
    })
}

#[utoipa::path(
    get,
    path = "/api/v1/statistics",
    tag = "masker",
    responses(
        (status = 200, description = "Statistics retrieved", body = ApiResponse<MaskStatistics>)
    )
)]
async fn get_statistics(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let stats = state.stats.lock().await;
    
    Json(ApiResponse {
        success: true,
        data: Some(stats.clone()),
        error: None,
    })
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Create shared state
    let state = Arc::new(AppState {
        masker: Arc::new(Mutex::new(PiiMasker::new())),
        queue_connection: Arc::new(Mutex::new(None)),
        stats: Arc::new(Mutex::new(MaskStatistics {
            total_processed: 0,
            total_pii_found: 0,
            pii_type_stats: std::collections::HashMap::new(),
            avg_processing_time_ms: 0.0,
        })),
    });

    // Setup queue consumer
    let state_clone = state.clone();
    tokio::spawn(async move {
        if let Err(e) = setup_queue_consumer(state_clone).await {
            error!("Failed to setup queue consumer: {}", e);
        }
    });

    // Build the application
    let app = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .route("/api/v1/mask", post(mask_text))
        .route("/api/v1/mask/batch", post(mask_batch))
        .route("/api/v1/patterns", get(get_patterns).post(update_patterns))
        .route("/api/v1/statistics", get(get_statistics))
        .route("/health", get(|| async { "OK" }))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8002")
        .await
        .unwrap();
    
    info!("Masker Service listening on http://0.0.0.0:8002");
    info!("Swagger UI available at http://0.0.0.0:8002/swagger-ui");
    
    axum::serve(listener, app).await.unwrap();
} 