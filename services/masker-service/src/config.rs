use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(default = "default_server_address")]
    pub server_address: String,
    
    #[serde(default = "default_rabbitmq_url")]
    pub rabbitmq_url: String,
    
    #[serde(default = "default_redis_url")]
    pub redis_url: String,
    
    #[serde(default = "default_jaeger_endpoint")]
    pub jaeger_endpoint: String,
    
    #[serde(default = "default_service_name")]
    pub service_name: String,
    
    #[serde(default = "default_log_level")]
    pub log_level: String,
    
    #[serde(default = "default_buffer_size")]
    pub buffer_size: usize,
    
    #[serde(default = "default_flush_interval")]
    pub flush_interval_ms: u64,
    
    #[serde(default = "default_patterns_cache_ttl")]
    pub patterns_cache_ttl_seconds: u64,
    
    #[serde(default = "default_max_text_length")]
    pub max_text_length: usize,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenv::dotenv().ok();
        
        let mut config = config::Config::builder()
            .add_source(config::Environment::with_prefix("MASKER"))
            .build()?;
        
        Ok(config.try_deserialize()?)
    }
}

fn default_server_address() -> String {
    "0.0.0.0:3002".to_string()
}

fn default_rabbitmq_url() -> String {
    "amqp://guest:guest@localhost:5672".to_string()
}

fn default_redis_url() -> String {
    "redis://localhost:6379".to_string()
}

fn default_jaeger_endpoint() -> String {
    "http://localhost:4317".to_string()
}

fn default_service_name() -> String {
    "masker-service".to_string()
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_buffer_size() -> usize {
    1000
}

fn default_flush_interval() -> u64 {
    1000
}

fn default_patterns_cache_ttl() -> u64 {
    3600 // 1 hour
}

fn default_max_text_length() -> usize {
    10000
} 