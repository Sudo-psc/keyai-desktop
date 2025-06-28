use tauri::State;
use serde::{Serialize, Deserialize};
use tracing::{info, error, debug};

use crate::{
    AppState,
    agent::AgentStatus,
    db::Database,
    search::{SearchEngine, SearchResult, HybridSearchResult},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
    pub search_time_ms: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HybridSearchResponse {
    pub results: Vec<HybridSearchResult>,
    pub search_time_ms: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppStats {
    pub agent: AgentStatus,
    pub database: DatabaseStats,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseStats {
    pub total_events: u64,
    pub total_size_bytes: u64,
    pub oldest_event: Option<i64>,
    pub newest_event: Option<i64>,
}

/// Realiza busca textual usando FTS5
#[tauri::command]
pub fn search_text(
    state: State<AppState>,
    query: String,
    limit: usize,
) -> Result<SearchResponse, String> {
    debug!("🔍 Comando search_text chamado: query='{}', limit={}", query, limit);
    
    let start_time = std::time::Instant::now();
    let search_engine = state.search_engine.lock().unwrap();
    
    let results = search_engine
        .search_text(&query, limit)
        .map_err(|e| e.to_string())?;
    
    let search_time_ms = start_time.elapsed().as_millis() as u64;
    info!("✅ Busca textual concluída: {} resultados em {}ms", results.len(), search_time_ms);
    
    Ok(SearchResponse {
        results,
        search_time_ms,
    })
}

/// Realiza busca semântica usando embeddings
#[tauri::command]
pub fn search_semantic(
    state: State<AppState>,
    query: String,
    limit: usize,
) -> Result<HybridSearchResponse, String> {
    debug!("🧠 Comando search_semantic chamado: query='{}', limit={}", query, limit);
    
    let start_time = std::time::Instant::now();
    let search_engine = state.search_engine.lock().unwrap();
    
    let results = search_engine
        .search_semantic(&query, limit)
        .map_err(|e| e.to_string())?;
    
    let search_time_ms = start_time.elapsed().as_millis() as u64;
    info!("✅ Busca semântica concluída: {} resultados em {}ms", results.len(), search_time_ms);
    
    Ok(HybridSearchResponse {
        results,
        search_time_ms,
    })
}

/// Realiza busca híbrida combinando busca textual e semântica
#[tauri::command]
pub fn search_hybrid(
    state: State<AppState>,
    query: String,
    limit: usize,
    text_weight: f32,
    semantic_weight: f32,
) -> Result<HybridSearchResponse, String> {
    debug!("🔍🧠 Comando search_hybrid chamado: query='{}', limit={}", query, limit);
    
    let start_time = std::time::Instant::now();
    let search_engine = state.search_engine.lock().unwrap();
    
    let results = search_engine
        .search_hybrid(&query, limit, text_weight, semantic_weight)
        .map_err(|e| e.to_string())?;
    
    let search_time_ms = start_time.elapsed().as_millis() as u64;
    info!("✅ Busca híbrida concluída: {} resultados em {}ms", results.len(), search_time_ms);
    
    Ok(HybridSearchResponse {
        results,
        search_time_ms,
    })
}

/// Liga/desliga o agente de captura de teclas
#[tauri::command]
pub fn toggle_agent(
    state: State<AppState>,
    enable: bool,
) -> Result<AgentStatus, String> {
    debug!("🎛️ Comando toggle_agent chamado: enable={}", enable);
    
    let mut agent_status = state.agent_status.lock().unwrap();
    
    if enable && !agent_status.is_running {
        // Iniciar o agente
        agent_status.is_running = true;
        agent_status.started_at = Some(chrono::Utc::now().timestamp());
        info!("✅ Agente de captura iniciado");
        // TODO: Realmente iniciar o agente de captura
    } else if !enable && agent_status.is_running {
        // Parar o agente
        agent_status.is_running = false;
        info!("🛑 Agente de captura parado");
        // TODO: Realmente parar o agente de captura
    }
    
    Ok(agent_status.clone())
}

/// Obtém estatísticas gerais da aplicação
#[tauri::command]
pub fn get_stats(state: State<AppState>) -> Result<AppStats, String> {
    debug!("📊 Comando get_stats chamado");
    
    let db = state.db.lock().unwrap();
    let agent_status = state.agent_status.lock().unwrap();
    
    // Obter estatísticas do banco de dados
    let total_events = db.get_total_events().map_err(|e| e.to_string())?;
    let total_size_bytes = db.get_database_size().map_err(|e| e.to_string())?;
    let (oldest_event, newest_event) = db.get_event_time_range().map_err(|e| e.to_string())?;
    
    Ok(AppStats {
        agent: agent_status.clone(),
        database: DatabaseStats {
            total_events,
            total_size_bytes,
            oldest_event,
            newest_event,
        },
    })
}

/// Obtém sugestões de busca baseadas em texto parcial
#[tauri::command]
pub fn get_search_suggestions(
    _state: State<AppState>,
    partial_query: String,
    limit: usize,
) -> Result<Vec<String>, String> {
    debug!("💡 Comando get_search_suggestions chamado: partial_query='{}'", partial_query);
    
    if partial_query.trim().is_empty() || partial_query.len() < 2 {
        return Ok(Vec::new());
    }
    
    // Por enquanto, retorna sugestões simuladas
    // TODO: Implementar busca real de sugestões baseada em histórico
    Ok(vec![
        format!("{} documento", partial_query),
        format!("{} email", partial_query),
        format!("{} código", partial_query),
    ].into_iter().take(limit).collect())
}

/// Obtém buscas populares
#[tauri::command]
pub fn get_popular_searches(
    _state: State<AppState>,
    limit: usize,
) -> Result<Vec<String>, String> {
    debug!("📊 Comando get_popular_searches chamado");
    
    // Por enquanto, retorna buscas populares simuladas
    // TODO: Implementar tracking real de buscas populares
    Ok(vec![
        "relatório mensal".to_string(),
        "email cliente".to_string(),
        "código python".to_string(),
        "documento projeto".to_string(),
        "senha sistema".to_string(),
    ].into_iter().take(limit).collect())
}

/// Limpa todos os dados armazenados
#[tauri::command]
pub fn clear_data(
    state: State<AppState>,
    confirm: bool,
) -> Result<(), String> {
    debug!("🗑️ Comando clear_data chamado: confirm={}", confirm);
    
    if !confirm {
        return Err("Confirmação necessária".to_string());
    }
    
    let mut db = state.db.lock().unwrap();
    db.clear_all_data().map_err(|e| e.to_string())?;
    
    info!("✅ Todos os dados foram removidos");
    Ok(())
}

/// Otimiza os índices de busca
#[tauri::command]
pub fn optimize_search_index(
    state: State<AppState>,
) -> Result<(), String> {
    debug!("🔧 Comando optimize_search_index chamado");
    
    let db = state.db.lock().unwrap();
    db.optimize_indexes().map_err(|e| e.to_string())?;
    
    info!("✅ Índices de busca otimizados");
    Ok(())
}

// Extensões para o Database
impl Database {
    pub fn get_total_events(&self) -> rusqlite::Result<u64> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT COUNT(*) FROM events",
            [],
            |row| row.get(0),
        )
    }
    
    pub fn get_database_size(&self) -> rusqlite::Result<u64> {
        // Obtém o tamanho do arquivo do banco de dados
        let db_path = std::path::Path::new("keyai.db");
        if db_path.exists() {
            let metadata = std::fs::metadata(db_path).map_err(|e| {
                rusqlite::Error::SqliteFailure(
                    rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_ERROR),
                    Some(e.to_string()),
                )
            })?;
            Ok(metadata.len())
        } else {
            Ok(0)
        }
    }
    
    pub fn get_event_time_range(&self) -> rusqlite::Result<(Option<i64>, Option<i64>)> {
        let conn = self.conn.lock().unwrap();
        let oldest: Option<i64> = conn.query_row(
            "SELECT MIN(timestamp) FROM events",
            [],
            |row| row.get(0),
        ).unwrap_or(None);
        
        let newest: Option<i64> = conn.query_row(
            "SELECT MAX(timestamp) FROM events",
            [],
            |row| row.get(0),
        ).unwrap_or(None);
        
        Ok((oldest, newest))
    }
    
    pub fn clear_all_data(&mut self) -> rusqlite::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM events", [])?;
        conn.execute("DELETE FROM events_fts", [])?;
        conn.execute("DELETE FROM embeddings", [])?;
        Ok(())
    }
    
    pub fn optimize_indexes(&self) -> rusqlite::Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("INSERT INTO events_fts(events_fts) VALUES('optimize')", [])?;
        conn.execute("VACUUM", [])?;
        Ok(())
    }
} 