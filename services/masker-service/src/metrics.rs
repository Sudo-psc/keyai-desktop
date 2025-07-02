use prometheus::{Counter, Histogram, Registry};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

pub struct Metrics {
    texts_processed: AtomicU64,
    patterns_detected: AtomicU64,
    total_processing_time_ms: AtomicU64,
    errors: AtomicU64,
    
    // Prometheus metrics
    processing_counter: Counter,
    patterns_counter: Counter,
    error_counter: Counter,
    processing_latency: Histogram,
}

impl Metrics {
    pub fn new() -> Self {
        let registry = Registry::new();
        
        let processing_counter = Counter::new("masker_texts_processed_total", "Total texts processed")
            .expect("metric creation failed");
        let patterns_counter = Counter::new("masker_patterns_detected_total", "Total PII patterns detected")
            .expect("metric creation failed");
        let error_counter = Counter::new("masker_errors_total", "Total errors")
            .expect("metric creation failed");
        let processing_latency = Histogram::with_opts(
            prometheus::HistogramOpts::new(
                "masker_processing_latency_seconds",
                "Time to process and mask text"
            ).buckets(vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0]),
        ).expect("metric creation failed");
        
        // Register metrics
        registry.register(Box::new(processing_counter.clone())).unwrap();
        registry.register(Box::new(patterns_counter.clone())).unwrap();
        registry.register(Box::new(error_counter.clone())).unwrap();
        registry.register(Box::new(processing_latency.clone())).unwrap();
        
        Self {
            texts_processed: AtomicU64::new(0),
            patterns_detected: AtomicU64::new(0),
            total_processing_time_ms: AtomicU64::new(0),
            errors: AtomicU64::new(0),
            processing_counter,
            patterns_counter,
            error_counter,
            processing_latency,
        }
    }
    
    pub fn increment_processed(&self) {
        self.texts_processed.fetch_add(1, Ordering::Relaxed);
        self.processing_counter.inc();
    }
    
    pub fn add_patterns_detected(&self, count: u64) {
        self.patterns_detected.fetch_add(count, Ordering::Relaxed);
        self.patterns_counter.inc_by(count as f64);
    }
    
    pub fn add_processing_time(&self, time_ms: u64) {
        self.total_processing_time_ms.fetch_add(time_ms, Ordering::Relaxed);
    }
    
    pub fn increment_errors(&self) {
        self.errors.fetch_add(1, Ordering::Relaxed);
        self.error_counter.inc();
    }
    
    pub fn get_processed(&self) -> u64 {
        self.texts_processed.load(Ordering::Relaxed)
    }
    
    pub fn get_patterns_detected(&self) -> u64 {
        self.patterns_detected.load(Ordering::Relaxed)
    }
    
    pub fn get_errors(&self) -> u64 {
        self.errors.load(Ordering::Relaxed)
    }
    
    pub fn get_average_processing_time(&self) -> f64 {
        let total_time = self.total_processing_time_ms.load(Ordering::Relaxed);
        let total_processed = self.texts_processed.load(Ordering::Relaxed);
        
        if total_processed == 0 {
            0.0
        } else {
            total_time as f64 / total_processed as f64
        }
    }
    
    pub fn start_processing_timer(&self) -> prometheus::HistogramTimer {
        self.processing_latency.start_timer()
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
} 