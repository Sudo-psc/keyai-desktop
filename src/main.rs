// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;
use tokio::sync::Mutex;
use tauri::Manager;
use tracing::{info, error, warn, debug};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod agent;
mod masker;
mod db;
mod search;
mod commands;

use agent::Agent;
use masker::Masker;
use db::Database;
use search::SearchEngine;

pub struct AppState {
    pub database: Arc<Database>,
    pub search_engine: Arc<SearchEngine>,
    pub agent: Arc<Mutex<Agent>>,
}

#[tokio::main]
async fn main() {
    // Configurar logging com nível mais detalhado para debug
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "debug".into()), // Mudado de "info" para "debug"
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("🚀 Iniciando KeyAI Desktop v1.0");
    debug!("🔧 Modo debug ativado - logs detalhados habilitados");

    // Verificar permissões antes de inicializar componentes
    if !Agent::check_permissions() {
        error!("❌ Permissões insuficientes detectadas");
        warn!("⚠️ A aplicação pode não funcionar corretamente sem as permissões adequadas");
    }

    // Inicializar banco de dados com tratamento robusto de erros
    let database = match Database::new("keyai.db").await {
        Ok(db) => {
            info!("✅ Banco de dados inicializado com sucesso");
            Arc::new(db)
        },
        Err(e) => {
            error!("❌ Erro crítico ao inicializar banco de dados: {}", e);
            error!("💡 Verifique se o arquivo keyai.db pode ser criado/acessado");
            std::process::exit(1);
        }
    };

    // Inicializar engine de busca
    let search_engine = match SearchEngine::new(Arc::clone(&database)).await {
        Ok(engine) => {
            info!("✅ Engine de busca inicializada com sucesso");
            Arc::new(engine)
        },
        Err(e) => {
            error!("❌ Erro crítico ao inicializar engine de busca: {}", e);
            error!("💡 Verifique se as dependências de busca estão disponíveis");
            std::process::exit(1);
        }
    };

    // Inicializar masker
    let masker = Masker::new();
    info!("✅ Masker de PII inicializado");

    // Inicializar agente de captura com tratamento robusto
    let agent = match Agent::new(masker, Arc::clone(&database)).await {
        Ok(agent) => {
            info!("✅ Agente de captura inicializado");
            Arc::new(Mutex::new(agent))
        },
        Err(e) => {
            error!("❌ Erro crítico ao inicializar agente: {}", e);
            error!("💡 Verifique se as permissões de acessibilidade estão concedidas");
            std::process::exit(1);
        }
    };

    // Criar estado da aplicação
    let app_state = AppState {
        database: Arc::clone(&database),
        search_engine: Arc::clone(&search_engine),
        agent: Arc::clone(&agent),
    };

    info!("✅ Todos os componentes inicializados com sucesso");
    debug!("🔧 Estado da aplicação criado, iniciando interface Tauri");

    // Construir aplicação Tauri com tratamento de erros
    let app_result = tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            // Comandos de busca
            commands::search_text,
            commands::search_semantic,
            commands::search_hybrid,
            commands::get_search_suggestions,
            commands::get_popular_searches,
            
            // Comandos do agente
            commands::toggle_agent,
            commands::get_agent_status,
            commands::update_agent_config,
            commands::get_agent_config,
            commands::get_current_window,
            commands::get_agent_metrics,
            
            // Comandos de dados
            commands::get_stats,
            commands::clear_data,
            commands::export_data,
            commands::import_data,
            
            // Comandos de sistema
            commands::optimize_search_index,
            commands::health_check,
        ])
        .setup(|app| {
            info!("✅ Aplicação Tauri inicializada");
            debug!("🔧 Configurando handlers de eventos");
            
            // Configurar handlers de eventos de janela
            let app_handle = app.handle();
            
            // Handler para quando a janela é fechada
            app.listen_global("tauri://close-requested", move |_event| {
                info!("🔄 Aplicação sendo fechada graciosamente...");
                // TODO: Implementar graceful shutdown do agente
            });
            
            // Handler para erros não capturados
            app.listen_global("tauri://error", move |event| {
                error!("❌ Erro não capturado na aplicação: {:?}", event);
            });
            
            Ok(())
        })
        .run(tauri::generate_context!());

    // Tratar erros de execução da aplicação Tauri
    if let Err(e) = app_result {
        error!("❌ Erro fatal ao executar aplicação Tauri: {}", e);
        error!("💡 Verifique se todas as dependências do Tauri estão instaladas");
        std::process::exit(1);
    }
}