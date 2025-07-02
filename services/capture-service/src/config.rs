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
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenv::dotenv().ok();
        
        let mut config = config::Config::builder()
            .add_source(config::Environment::with_prefix("CAPTURE"))
            .build()?;
        
        Ok(config.try_deserialize()?)
    }
}

fn default_server_address() -> String {
    "0.0.0.0:3001".to_string()
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
    "capture-service".to_string()
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_buffer_size() -> usize {
    1000
}

fn default_flush_interval_ms() -> u64 {
    1000
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.service_name, "capture-service");
        assert_eq!(config.nats_url, "nats://localhost:4222");
        assert_eq!(config.buffer_size, 1000);
    }

    #[test]
    fn test_config_validation() {
        let mut config = Config::default();
        assert!(config.validate().is_ok());
        
        config.user_id = String::new();
        assert!(config.validate().is_err());
        
        config.user_id = "test".to_string();
        config.buffer_size = 0;
        assert!(config.validate().is_err());
        
        config.buffer_size = 100;
        config.batch_size = 200;
        assert!(config.validate().is_err());
    }
} 