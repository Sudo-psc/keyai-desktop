use anyhow::Result;
use prometheus::{Encoder, TextEncoder, Registry};
use std::convert::Infallible;
use warp::{Filter, Rejection, Reply};
use tracing::info;

pub async fn run_metrics_server(port: u16, registry: Registry) -> Result<()> {
    let metrics_route = warp::path("metrics")
        .and(with_registry(registry))
        .and_then(metrics_handler);
    
    let health_route = warp::path("health")
        .map(|| warp::reply::with_status("OK", warp::http::StatusCode::OK));
    
    let routes = metrics_route.or(health_route);
    
    info!("Starting metrics server on port {}", port);
    
    warp::serve(routes)
        .run(([0, 0, 0, 0], port))
        .await;
    
    Ok(())
}

fn with_registry(
    registry: Registry,
) -> impl Filter<Extract = (Registry,), Error = Infallible> + Clone {
    warp::any().map(move || registry.clone())
}

async fn metrics_handler(registry: Registry) -> Result<impl Reply, Rejection> {
    let encoder = TextEncoder::new();
    let metric_families = registry.gather();
    
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer)
        .map_err(|e| {
            eprintln!("Failed to encode metrics: {}", e);
            warp::reject::reject()
        })?;
    
    Ok(warp::reply::with_header(
        buffer,
        "Content-Type",
        encoder.format_type(),
    ))
}

// Custom metrics collection
use lazy_static::lazy_static;
use prometheus::{Counter, CounterVec, Histogram, HistogramVec, Opts};

lazy_static! {
    pub static ref EVENTS_CAPTURED: Counter = Counter::new(
        "keyai_capture_events_total",
        "Total number of keyboard events captured"
    ).expect("metric creation failed");
    
    pub static ref EVENTS_PUBLISHED: Counter = Counter::new(
        "keyai_capture_events_published_total",
        "Total number of events successfully published"
    ).expect("metric creation failed");
    
    pub static ref PUBLISH_ERRORS: CounterVec = CounterVec::new(
        Opts::new("keyai_capture_publish_errors_total", "Publish errors by type"),
        &["error_type"]
    ).expect("metric creation failed");
    
    pub static ref CAPTURE_LATENCY: Histogram = Histogram::with_opts(
        prometheus::HistogramOpts::new(
            "keyai_capture_latency_seconds",
            "Time to capture and process an event"
        ).buckets(vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0]),
    ).expect("metric creation failed");
    
    pub static ref BUFFER_SIZE: prometheus::Gauge = prometheus::Gauge::new(
        "keyai_capture_buffer_size",
        "Current size of the event buffer"
    ).expect("metric creation failed");
}

pub fn register_metrics(registry: &Registry) -> Result<()> {
    registry.register(Box::new(EVENTS_CAPTURED.clone()))?;
    registry.register(Box::new(EVENTS_PUBLISHED.clone()))?;
    registry.register(Box::new(PUBLISH_ERRORS.clone()))?;
    registry.register(Box::new(CAPTURE_LATENCY.clone()))?;
    registry.register(Box::new(BUFFER_SIZE.clone()))?;
    
    Ok(())
} 