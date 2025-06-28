// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::{Arc, Mutex};
use tauri::Manager;
use tracing::{info, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod agent;
mod masker;
mod db;
mod search;
mod commands;

use agent::AgentStatus;
use db::Database;
use search::SearchEngine;

pub struct AppState {
    pub db: Arc<Mutex<Database>>,
    pub search_engine: Arc<Mutex<SearchEngine>>,
    pub agent_status: Arc<Mutex<AgentStatus>>,
}

fn main() {
    // Configurar logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("🚀 Iniciando KeyAI Desktop v1.0");

    // Inicializar banco de dados
    let db = match Database::new("keyai.db", "chave_secreta_temporaria") {
        Ok(database) => Arc::new(Mutex::new(database)),
        Err(e) => {
            error!("❌ Erro ao inicializar banco de dados: {}", e);
            std::process::exit(1);
        }
    };

    // Inicializar engine de busca
    let search_engine = Arc::new(Mutex::new(SearchEngine::new(Arc::clone(&db))));

    // Inicializar status do agente
    let agent_status = Arc::new(Mutex::new(AgentStatus {
        is_running: false,
        started_at: None,
        uptime_seconds: 0,
        events_captured: 0,
    }));

    // Criar estado da aplicação
    let app_state = AppState {
        db: Arc::clone(&db),
        search_engine: Arc::clone(&search_engine),
        agent_status: Arc::clone(&agent_status),
    };

    // Construir aplicação Tauri
    tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            commands::search_text,
            commands::search_semantic,
            commands::search_hybrid,
            commands::toggle_agent,
            commands::get_stats,
            commands::get_search_suggestions,
            commands::get_popular_searches,
            commands::clear_data,
            commands::optimize_search_index,
        ])
        .setup(|app| {
            info!("✅ Aplicação Tauri inicializada");
            
            // TODO: Iniciar agente de captura automaticamente se configurado
            
            Ok(())
        })
                .run(tauri::generate_context!())
        .expect("Erro ao executar aplicação Tauri");
} 