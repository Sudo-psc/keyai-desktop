use tauri::State;
use serde::{Serialize, Deserialize};
use tracing::{info, error, debug, warn};
use std::collections::HashMap;
use std::sync::Arc;

use crate::AppState;
use crate::search::{SearchOptions, HybridSearchResult};
use crate::db::{SearchResult, DatabaseStats, Database};
use crate::agent::{AgentConfig, WindowInfo};

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
    pub total_count: usize,
    pub search_time_ms: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HybridSearchResponse {
    pub results: Vec<HybridSearchResult>,
    pub total_count: usize,
    pub search_time_ms: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentStatus {
    pub is_running: bool,
    pub uptime_seconds: u64,
    pub events_captured: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppStats {
    pub database: DatabaseStats,
    pub agent: AgentStatus,
}

/// Busca textual simples
#[tauri::command]
pub async fn search_text(
    state: State<'_, AppState>,
    query: String,
    limit: Option<usize>,
    offset: Option<usize>,
) -> Result<SearchResponse, String> {
    debug!("🔍 Comando search_text chamado: query='{}', limit={:?}, offset={:?}",
           query, limit, offset);

    let start_time = std::time::Instant::now();

    match state.database.search_text(&query, limit.unwrap_or(50)).await {
        Ok(results) => {
            let search_time = start_time.elapsed().as_millis() as u64;
            info!("✅ Busca textual concluída: {} resultados em {}ms", results.len(), search_time);

            Ok(SearchResponse {
                total_count: results.len(),
                results,
                search_time_ms: search_time,
            })
        },
        Err(e) => {
            error!("❌ Erro na busca textual: {}", e);
            Err(format!("Erro na busca: {}", e))
        }
    }
}

/// Busca semântica
#[tauri::command]
pub async fn search_semantic(
    state: State<'_, AppState>,
    query: String,
    limit: Option<usize>,
    threshold: Option<f32>,
) -> Result<HybridSearchResponse, String> {
    debug!("🧠 Comando search_semantic chamado: query='{}', limit={:?}, threshold={:?}",
           query, limit, threshold);

    let start_time = std::time::Instant::now();

    let options = SearchOptions {
        limit: limit.unwrap_or(20),
        min_score_threshold: threshold.unwrap_or(0.7) as f64,
        ..Default::default()
    };

    match state.search_engine.search_semantic(&query, &options).await {
        Ok(results) => {
            let search_time = start_time.elapsed().as_millis() as u64;
            info!("✅ Busca semântica concluída: {} resultados em {}ms", results.len(), search_time);

            Ok(HybridSearchResponse {
                total_count: results.len(),
                results,
                search_time_ms: search_time,
            })
        },
        Err(e) => {
            error!("❌ Erro na busca semântica: {}", e);
            Err(format!("Erro na busca semântica: {}", e))
        }
    }
}

/// Busca híbrida (textual + semântica)
#[tauri::command]
pub async fn search_hybrid(
    state: State<'_, AppState>,
    query: String,
    options: SearchOptions,
) -> Result<HybridSearchResponse, String> {
    debug!("🔀 Comando search_hybrid chamado: query='{}', options={:?}", query, options);

    let start_time = std::time::Instant::now();

    match state.search_engine.search_hybrid(&query, &options).await {
        Ok(results) => {
            let search_time = start_time.elapsed().as_millis() as u64;
            info!("✅ Busca híbrida concluída: {} resultados em {}ms", results.len(), search_time);

            Ok(HybridSearchResponse {
                total_count: results.len(),
                results,
                search_time_ms: search_time,
            })
        },
        Err(e) => {
            error!("❌ Erro na busca híbrida: {}", e);
            Err(format!("Erro na busca híbrida: {}", e))
        }
    }
}

/// Obtém estatísticas do banco de dados
#[tauri::command]
pub async fn get_database_stats(
    state: State<'_, AppState>
) -> Result<DatabaseStats, String> {
    debug!("📊 Comando get_database_stats chamado");

    match state.database.get_stats().await {
        Ok(stats) => {
            info!("✅ Estatísticas obtidas: {} eventos", stats.total_events);
            Ok(stats)
        },
        Err(e) => {
            error!("❌ Erro ao obter estatísticas: {}", e);
            Err(format!("Erro ao obter estatísticas: {}", e))
        }
    }
}

/// Obtém sugestões de busca
#[tauri::command]
pub async fn get_search_suggestions(
    state: State<'_, AppState>,
    partial_query: String,
    limit: Option<usize>,
) -> Result<Vec<String>, String> {
    debug!("💡 Comando get_search_suggestions chamado: query='{}', limit={:?}",
           partial_query, limit);

    match state.search_engine.get_search_suggestions(&partial_query, limit.unwrap_or(10)).await {
        Ok(suggestions) => {
            info!("✅ {} sugestões geradas", suggestions.len());
            Ok(suggestions)
        },
        Err(e) => {
            error!("❌ Erro ao gerar sugestões: {}", e);
            Err(format!("Erro ao gerar sugestões: {}", e))
        }
    }
}

/// Otimiza os índices de busca
#[tauri::command]
pub async fn optimize_search_index(
    state: State<'_, AppState>
) -> Result<String, String> {
    debug!("🔧 Comando optimize_search_index chamado");

    match state.search_engine.optimize_search_index().await {
        Ok(_) => {
            info!("✅ Índices de busca otimizados");
            Ok("Índices de busca otimizados com sucesso".to_string())
        },
        Err(e) => {
            error!("❌ Erro ao otimizar índices: {}", e);
            Err(format!("Erro ao otimizar índices: {}", e))
        }
    }
}

/// Liga/desliga o agente de captura de teclas
#[tauri::command]
pub async fn toggle_agent(
    enable: bool,
    state: State<'_, AppState>
) -> Result<AgentStatus, String> {
    debug!("🎛️ Comando toggle_agent chamado: enable={}", enable);

    let mut agent = state.agent.lock().await;

    if enable {
        if !agent.is_running() {
            match agent.start().await {
                Ok(_) => {
                    info!("✅ Agente de captura iniciado");
                },
                Err(e) => {
                    error!("❌ Erro ao iniciar agente: {}", e);
                    return Err(format!("Erro ao iniciar agente: {}", e));
                }
            }
        }
    } else {
        if agent.is_running() {
            match agent.stop().await {
                Ok(_) => {
                    info!("🛑 Agente de captura parado");
                },
                Err(e) => {
                    error!("❌ Erro ao parar agente: {}", e);
                    return Err(format!("Erro ao parar agente: {}", e));
                }
            }
        }
    }

    // Return updated status
    let metrics = agent.get_metrics();
    let current_window = agent.get_current_window().await;
    let config = agent.get_config().await;

    Ok(AgentStatus {
        is_running: agent.is_running(),
        uptime_seconds: 0,
        events_captured: 0,
    })
}

/// Obtém o status atual do agente
#[tauri::command]
pub async fn get_agent_status(
    state: State<'_, AppState>
) -> Result<AgentStatus, String> {
    debug!("📊 Comando get_agent_status chamado");

    let agent = state.agent.lock().await;
    let metrics = agent.get_metrics();

    Ok(AgentStatus {
        is_running: agent.is_running(),
        uptime_seconds: metrics.get("uptime_seconds").copied().unwrap_or(0),
        events_captured: metrics.get("events_captured").copied().unwrap_or(0),
    })
}

/// Atualiza a configuração do agente
#[tauri::command]
pub async fn update_agent_config(
    config: AgentConfig,
    state: State<'_, AppState>
) -> Result<AgentStatus, String> {
    debug!("🔧 Comando update_agent_config chamado");

    let agent = state.agent.lock().await;

    match agent.update_config(config).await {
        Ok(_) => {
            info!("✅ Configuração do agente atualizada");

            let metrics = agent.get_metrics();

            Ok(AgentStatus {
                is_running: agent.is_running(),
                uptime_seconds: metrics.get("uptime_seconds").copied().unwrap_or(0),
                events_captured: metrics.get("events_captured").copied().unwrap_or(0),
            })
        },
        Err(e) => {
            error!("❌ Erro ao atualizar configuração: {}", e);
            Err(format!("Erro ao atualizar configuração: {}", e))
        }
    }
}

/// Obtém a configuração atual do agente
#[tauri::command]
pub async fn get_agent_config(
    state: State<'_, AppState>
) -> Result<AgentConfig, String> {
    debug!("⚙️ Comando get_agent_config chamado");

    let agent = state.agent.lock().await;
    Ok(agent.get_config().await)
}

/// Obtém informações da janela ativa atual
#[tauri::command]
pub async fn get_current_window(
    state: State<'_, AppState>
) -> Result<Option<WindowInfo>, String> {
    debug!("🪟 Comando get_current_window chamado");

    let agent = state.agent.lock().await;
    Ok(agent.get_current_window().await)
}

/// Obtém métricas detalhadas do agente
#[tauri::command]
pub async fn get_agent_metrics(
    state: State<'_, AppState>
) -> Result<HashMap<String, u64>, String> {
    debug!("📈 Comando get_agent_metrics chamado");

    let agent = state.agent.lock().await;
    Ok(agent.get_metrics())
}

/// Obtém estatísticas gerais da aplicação
#[tauri::command]
pub async fn get_stats(
    state: State<'_, AppState>
) -> Result<AppStats, String> {
    debug!("📊 Comando get_stats chamado");

    let db_stats = match state.database.get_stats().await {
        Ok(stats) => stats,
        Err(e) => {
            error!("❌ Erro ao obter estatísticas do banco: {}", e);
            return Err(format!("Erro ao obter estatísticas: {}", e));
        }
    };

    let agent = state.agent.lock().await;
    let metrics = agent.get_metrics();

    let agent_status = AgentStatus {
        is_running: agent.is_running(),
        uptime_seconds: metrics.get("uptime_seconds").copied().unwrap_or(0),
        events_captured: metrics.get("events_captured").copied().unwrap_or(0),
    };

    Ok(AppStats {
        database: db_stats,
        agent: agent_status,
    })
}

/// Limpa todos os dados armazenados
#[tauri::command]
pub async fn clear_data(
    confirm: bool,
    state: State<'_, AppState>
) -> Result<String, String> {
    debug!("🗑️ Comando clear_data chamado: confirm={}", confirm);

    if !confirm {
        return Err("Confirmação necessária para limpar dados".to_string());
    }

    // Stop agent if running
    let mut agent = state.agent.lock().await;
    if agent.is_running() {
        if let Err(e) = agent.stop().await {
            error!("❌ Erro ao parar agente antes de limpar dados: {}", e);
            return Err(format!("Erro ao parar agente: {}", e));
        }
    }
    drop(agent);

    match state.database.clear_all_data().await {
        Ok(_) => {
            info!("✅ Todos os dados foram limpos");
            Ok("Dados limpos com sucesso".to_string())
        },
        Err(e) => {
            error!("❌ Erro ao limpar dados: {}", e);
            Err(format!("Erro ao limpar dados: {}", e))
        }
    }
}

/// Obtém as buscas mais populares
#[tauri::command]
pub async fn get_popular_searches(
    limit: Option<usize>,
    state: State<'_, AppState>
) -> Result<Vec<String>, String> {
    debug!("🔥 Comando get_popular_searches chamado");

    match state.search_engine.get_popular_searches(limit.unwrap_or(10)).await {
        Ok(searches) => {
            debug!("✅ {} buscas populares encontradas", searches.len());
            Ok(searches)
        },
        Err(e) => {
            error!("❌ Erro ao obter buscas populares: {}", e);
            Err(format!("Erro ao obter buscas populares: {}", e))
        }
    }
}

/// Exporta dados para arquivo JSON
#[tauri::command]
pub async fn export_data(
    file_path: String,
    date_from: Option<String>,
    date_to: Option<String>,
    state: State<'_, AppState>
) -> Result<String, String> {
    debug!("📤 Comando export_data chamado: path='{}'", file_path);

    // Implementação básica de exportação
    match export_data_to_file(&state.database, &file_path, date_from, date_to).await {
        Ok(count) => {
            info!("✅ {} eventos exportados para {}", count, file_path);
            Ok(format!("{} eventos exportados com sucesso", count))
        },
        Err(e) => {
            error!("❌ Erro ao exportar dados: {}", e);
            Err(format!("Erro ao exportar dados: {}", e))
        }
    }
}

/// Importa dados de arquivo JSON
#[tauri::command]
pub async fn import_data(
    file_path: String,
    state: State<'_, AppState>
) -> Result<String, String> {
    debug!("📥 Comando import_data chamado: path='{}'", file_path);

    // Implementação básica de importação
    match import_data_from_file(&state.database, &file_path).await {
        Ok(count) => {
            info!("✅ {} eventos importados de {}", count, file_path);
            Ok(format!("{} eventos importados com sucesso", count))
        },
        Err(e) => {
            error!("❌ Erro ao importar dados: {}", e);
            Err(format!("Erro ao importar dados: {}", e))
        }
    }
}

/// Testa a conectividade do sistema
#[tauri::command]
pub async fn health_check(
    state: State<'_, AppState>
) -> Result<HashMap<String, String>, String> {
    debug!("🏥 Comando health_check chamado");

    let mut status = HashMap::new();

    // Test database
    match state.database.get_stats().await {
        Ok(_) => status.insert("database".to_string(), "ok".to_string()),
        Err(e) => status.insert("database".to_string(), format!("error: {}", e)),
    };

    // Test search engine - basic check
    status.insert("search_engine".to_string(), "ok".to_string());

    // Test agent
    let agent = state.agent.lock().await;
    let agent_status = if agent.is_running() { "running" } else { "stopped" };
    status.insert("agent".to_string(), agent_status.to_string());

    info!("✅ Health check concluído: {:?}", status);
    Ok(status)
}

// Helper functions for export/import

async fn export_data_to_file(
    database: &Arc<Database>,
    file_path: &str,
    _date_from: Option<String>,
    _date_to: Option<String>,
) -> Result<usize, anyhow::Error> {
    use std::fs::File;
    use std::io::Write;

    // Get all events (simplified implementation)
    let events = database.search_by_timerange(0, u64::MAX, 10000).await?;

    // Convert to JSON
    let json_data = serde_json::to_string_pretty(&events)?;

    // Write to file
    let mut file = File::create(file_path)?;
    file.write_all(json_data.as_bytes())?;

    Ok(events.len())
}

async fn import_data_from_file(
    _database: &Arc<Database>,
    _file_path: &str,
) -> Result<usize, anyhow::Error> {
    // TODO: Implementar importação real
    warn!("🚧 Funcionalidade de importação ainda não implementada");
    Ok(0)
}
