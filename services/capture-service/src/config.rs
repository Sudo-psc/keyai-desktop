use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    // Service identification
    pub service_name: String,
    pub user_id: String,
    
    // NATS configuration
    pub nats_url: String,
    pub nats_subject: String,
    pub nats_user: Option<String>,
    pub nats_password: Option<String>,
    
    // Service configuration
    pub buffer_size: usize,
    pub batch_size: usize,
    pub flush_interval_ms: u64,
    
    // Health check
    pub health_port: u16,
    
    // Metrics
    pub metrics_port: u16,
    pub metrics_path: String,
    
    // Feature flags
    pub capture_window_info: bool,
    pub capture_modifiers: bool,
    pub filter_passwords: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            service_name: "capture-service".to_string(),
            user_id: "default-user".to_string(),
            nats_url: "nats://localhost:4222".to_string(),
            nats_subject: "keyai.events.capture".to_string(),
            nats_user: None,
            nats_password: None,
            buffer_size: 1000,
            batch_size: 100,
            flush_interval_ms: 1000,
            health_port: 8080,
            metrics_port: 9090,
            metrics_path: "/metrics".to_string(),
            capture_window_info: true,
            capture_modifiers: false,
            filter_passwords: true,
        }
    }
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let mut config = config::Config::builder()
            .add_source(config::Config::try_from(&Config::default())?)
            .add_source(config::Environment::with_prefix("CAPTURE")
                .separator("_")
                .try_parsing(true))
            .build()?;

        // Handle special cases for optional fields
        if let Ok(user) = std::env::var("CAPTURE_NATS_USER") {
            config.set("nats_user", Some(user))?;
        }
        
        if let Ok(password) = std::env::var("CAPTURE_NATS_PASSWORD") {
            config.set("nats_password", Some(password))?;
        }

        Ok(config.try_deserialize()?)
    }

    pub fn validate(&self) -> Result<()> {
        if self.user_id.is_empty() {
            anyhow::bail!("user_id cannot be empty");
        }
        
        if self.buffer_size == 0 {
            anyhow::bail!("buffer_size must be greater than 0");
        }
        
        if self.batch_size > self.buffer_size {
            anyhow::bail!("batch_size cannot be greater than buffer_size");
        }
        
        Ok(())
    }
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