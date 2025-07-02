use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(default = "default_server_address")]
    pub server_address: String,
    
    #[serde(default = "default_database_url")]
    pub database_url: String,
    
    #[serde(default = "default_rabbitmq_url")]
    pub rabbitmq_url: String,
    
    #[serde(default = "default_service_name")]
    pub service_name: String,
    
    #[serde(default = "default_log_level")]
    pub log_level: String,
    
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
    
    #[serde(default = "default_connection_timeout")]
    pub connection_timeout_seconds: u64,
    
    #[serde(default = "default_query_timeout")]
    pub query_timeout_seconds: u64,
    
    #[serde(default = "default_migration_timeout")]
    pub migration_timeout_seconds: u64,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenv::dotenv().ok();
        
        let mut config = config::Config::builder()
            .add_source(config::Environment::with_prefix("STORAGE"))
            .build()?;
        
        Ok(config.try_deserialize()?)
    }
}

fn default_server_address() -> String {
    "0.0.0.0:3003".to_string()
}

fn default_database_url() -> String {
    "postgres://keyai:keyaipass@localhost:5432/keyai".to_string()
}

fn default_rabbitmq_url() -> String {
    "amqp://guest:guest@localhost:5672".to_string()
}

fn default_service_name() -> String {
    "storage-service".to_string()
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_max_connections() -> u32 {
    10
}

fn default_connection_timeout() -> u64 {
    30
}

fn default_query_timeout() -> u64 {
    15
}

fn default_migration_timeout() -> u64 {
    300 // 5 minutes
} 