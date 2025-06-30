use std::sync::Arc;
use tokio::sync::Mutex;

// Re-export modules for tests and benchmarks
pub mod agent;
pub mod masker;
pub mod db;
pub mod search;
pub mod commands;

// AppState for Tauri commands
#[derive(Clone)]
pub struct AppState {
    pub database: Arc<db::Database>,
    pub search_engine: Arc<search::SearchEngine>,
    pub agent: Arc<Mutex<agent::Agent>>,
}

impl AppState {
    pub async fn new(db_path: &std::path::Path) -> anyhow::Result<Self> {
        let database = Arc::new(db::Database::new(db_path).await?);
        let search_engine = Arc::new(search::SearchEngine::new(database.clone()).await?);
        let masker = masker::Masker::new();
        let agent = Arc::new(Mutex::new(agent::Agent::new(masker, database.clone()).await?));
        
        Ok(Self {
            database,
            search_engine,
            agent,
        })
    }
} 