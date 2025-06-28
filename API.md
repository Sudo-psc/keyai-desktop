# 📡 Documentação da API - KeyAI Desktop

## 📋 Índice

- [Visão Geral](#visão-geral)
- [Comandos Tauri](#comandos-tauri)
- [API do Backend](#api-do-backend)
- [Tipos de Dados](#tipos-de-dados)
- [Códigos de Erro](#códigos-de-erro)
- [Exemplos de Uso](#exemplos-de-uso)
- [Webhooks e Eventos](#webhooks-e-eventos)

## 🎯 Visão Geral

O KeyAI Desktop expõe sua funcionalidade através de comandos Tauri que fazem a ponte entre o frontend React e o backend Rust. Esta API é projetada para ser type-safe, eficiente e fácil de usar.

### Princípios da API

1. **Type Safety**: Tipos TypeScript/Rust rigorosamente definidos
2. **Async/Await**: Todas as operações são assíncronas
3. **Error Handling**: Tratamento consistente de erros
4. **Validation**: Validação de entrada em todas as operações
5. **Performance**: Otimizada para baixa latência

## 🔧 Comandos Tauri

### Busca

#### `search_text`

Realiza busca textual usando FTS5.

```typescript
interface SearchTextParams {
  query: string;
  limit?: number;
  offset?: number;
  filters?: SearchFilters;
}

interface SearchResult {
  id: string;
  content: string;
  timestamp: number;
  application?: string;
  window_title?: string;
  relevance_score: number;
  snippet: string;
}

// Uso
const results = await invoke<SearchResult[]>('search_text', {
  query: 'login password',
  limit: 50,
  offset: 0
});
```

**Rust Implementation:**
```rust
#[tauri::command]
pub async fn search_text(
    query: String,
    limit: Option<u32>,
    offset: Option<u32>,
    filters: Option<SearchFilters>,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<SearchResult>, String> {
    let limit = limit.unwrap_or(50).min(1000);
    let offset = offset.unwrap_or(0);
    
    if query.trim().is_empty() {
        return Err("Query cannot be empty".to_string());
    }
    
    let search_engine = state.search_engine.lock().await;
    search_engine
        .search_text(&query, limit, offset, filters)
        .await
        .map_err(|e| e.to_string())
}
```

#### `search_semantic`

Realiza busca semântica usando embeddings.

```typescript
interface SearchSemanticParams {
  query: string;
  limit?: number;
  threshold?: number;
  filters?: SearchFilters;
}

// Uso
const results = await invoke<SearchResult[]>('search_semantic', {
  query: 'authentication credentials',
  limit: 30,
  threshold: 0.7
});
```

**Rust Implementation:**
```rust
#[tauri::command]
pub async fn search_semantic(
    query: String,
    limit: Option<u32>,
    threshold: Option<f32>,
    filters: Option<SearchFilters>,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<SearchResult>, String> {
    let limit = limit.unwrap_or(50).min(1000);
    let threshold = threshold.unwrap_or(0.5).clamp(0.0, 1.0);
    
    if query.trim().is_empty() {
        return Err("Query cannot be empty".to_string());
    }
    
    let search_engine = state.search_engine.lock().await;
    search_engine
        .search_semantic(&query, limit, threshold, filters)
        .await
        .map_err(|e| e.to_string())
}
```

#### `search_hybrid`

Combina busca textual e semântica usando RRF.

```typescript
interface SearchHybridParams {
  query: string;
  limit?: number;
  text_weight?: number;
  semantic_weight?: number;
  filters?: SearchFilters;
}

// Uso
const results = await invoke<SearchResult[]>('search_hybrid', {
  query: 'email login',
  limit: 50,
  text_weight: 0.7,
  semantic_weight: 0.3
});
```

**Rust Implementation:**
```rust
#[tauri::command]
pub async fn search_hybrid(
    query: String,
    limit: Option<u32>,
    text_weight: Option<f32>,
    semantic_weight: Option<f32>,
    filters: Option<SearchFilters>,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<SearchResult>, String> {
    let limit = limit.unwrap_or(50).min(1000);
    let text_weight = text_weight.unwrap_or(0.7).clamp(0.0, 1.0);
    let semantic_weight = semantic_weight.unwrap_or(0.3).clamp(0.0, 1.0);
    
    // Normalizar pesos
    let total_weight = text_weight + semantic_weight;
    if total_weight == 0.0 {
        return Err("At least one weight must be greater than 0".to_string());
    }
    
    let normalized_text = text_weight / total_weight;
    let normalized_semantic = semantic_weight / total_weight;
    
    let search_engine = state.search_engine.lock().await;
    search_engine
        .search_hybrid(&query, limit, normalized_text, normalized_semantic, filters)
        .await
        .map_err(|e| e.to_string())
}
```

### Configurações

#### `get_settings`

Obtém as configurações atuais da aplicação.

```typescript
interface AppSettings {
  agent: AgentSettings;
  masker: MaskerSettings;
  search: SearchSettings;
  ui: UiSettings;
}

interface AgentSettings {
  enabled: boolean;
  capture_passwords: boolean;
  capture_applications: string[];
  exclude_applications: string[];
  capture_window_titles: boolean;
}

interface MaskerSettings {
  enabled: boolean;
  patterns: string[];
  custom_patterns: CustomPattern[];
  mask_character: string;
}

interface SearchSettings {
  default_limit: number;
  max_limit: number;
  enable_semantic_search: boolean;
  embedding_model: string;
  cache_embeddings: boolean;
}

interface UiSettings {
  theme: 'light' | 'dark' | 'system';
  language: string;
  show_timestamps: boolean;
  show_applications: boolean;
  results_per_page: number;
}

// Uso
const settings = await invoke<AppSettings>('get_settings');
```

#### `update_settings`

Atualiza as configurações da aplicação.

```typescript
// Uso
await invoke('update_settings', {
  settings: {
    agent: {
      enabled: true,
      capture_passwords: false,
      capture_applications: ['*'],
      exclude_applications: ['1Password', 'Bitwarden']
    }
  }
});
```

**Rust Implementation:**
```rust
#[tauri::command]
pub async fn update_settings(
    settings: AppSettings,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    // Validar configurações
    if settings.search.default_limit > settings.search.max_limit {
        return Err("Default limit cannot exceed max limit".to_string());
    }
    
    let mut config = state.config.lock().await;
    config.update(settings).await.map_err(|e| e.to_string())?;
    
    // Notificar componentes sobre mudanças
    state.event_bus.emit("settings_updated", &config.clone()).await;
    
    Ok(())
}
```

### Estatísticas

#### `get_stats`

Obtém estatísticas do banco de dados e performance.

```typescript
interface DatabaseStats {
  total_events: number;
  total_size_mb: number;
  oldest_event: number;
  newest_event: number;
  events_by_application: Record<string, number>;
  events_by_day: Array<{ date: string; count: number }>;
}

interface PerformanceStats {
  capture_latency_ms: number;
  search_latency_ms: number;
  memory_usage_mb: number;
  cpu_usage_percent: number;
  uptime_seconds: number;
}

interface AppStats {
  database: DatabaseStats;
  performance: PerformanceStats;
  agent_status: AgentStatus;
}

// Uso
const stats = await invoke<AppStats>('get_stats');
```

#### `get_health`

Verifica o status de saúde da aplicação.

```typescript
interface HealthStatus {
  status: 'healthy' | 'degraded' | 'unhealthy';
  checks: HealthCheck[];
  timestamp: number;
}

interface HealthCheck {
  name: string;
  status: 'pass' | 'fail' | 'warn';
  message?: string;
  duration_ms?: number;
}

// Uso
const health = await invoke<HealthStatus>('get_health');
```

### Controle do Agente

#### `start_agent`

Inicia o agente de captura de teclas.

```typescript
// Uso
await invoke('start_agent');
```

#### `stop_agent`

Para o agente de captura de teclas.

```typescript
// Uso
await invoke('stop_agent');
```

#### `get_agent_status`

Obtém o status atual do agente.

```typescript
interface AgentStatus {
  running: boolean;
  events_captured: number;
  events_processed: number;
  last_event_time?: number;
  error_count: number;
  last_error?: string;
}

// Uso
const status = await invoke<AgentStatus>('get_agent_status');
```

### Backup e Exportação

#### `export_data`

Exporta dados para arquivo.

```typescript
interface ExportOptions {
  format: 'json' | 'csv' | 'sqlite';
  date_range?: {
    start: number;
    end: number;
  };
  applications?: string[];
  include_masked: boolean;
}

interface ExportResult {
  file_path: string;
  total_records: number;
  file_size_mb: number;
}

// Uso
const result = await invoke<ExportResult>('export_data', {
  options: {
    format: 'json',
    date_range: {
      start: Date.now() - 7 * 24 * 60 * 60 * 1000, // 7 dias atrás
      end: Date.now()
    },
    include_masked: false
  }
});
```

#### `backup_database`

Cria backup do banco de dados.

```typescript
interface BackupOptions {
  destination?: string;
  compress: boolean;
  include_embeddings: boolean;
}

interface BackupResult {
  backup_path: string;
  original_size_mb: number;
  backup_size_mb: number;
  compression_ratio?: number;
}

// Uso
const backup = await invoke<BackupResult>('backup_database', {
  options: {
    compress: true,
    include_embeddings: true
  }
});
```

## 🏗️ API do Backend

### Search Engine

```rust
pub struct SearchEngine {
    db: Arc<Database>,
    embedding_model: Arc<EmbeddingModel>,
    config: SearchConfig,
}

impl SearchEngine {
    pub async fn search_text(
        &self,
        query: &str,
        limit: u32,
        offset: u32,
        filters: Option<SearchFilters>,
    ) -> Result<Vec<SearchResult>, SearchError> {
        // Implementação da busca FTS5
    }
    
    pub async fn search_semantic(
        &self,
        query: &str,
        limit: u32,
        threshold: f32,
        filters: Option<SearchFilters>,
    ) -> Result<Vec<SearchResult>, SearchError> {
        // Implementação da busca vetorial
    }
    
    pub async fn search_hybrid(
        &self,
        query: &str,
        limit: u32,
        text_weight: f32,
        semantic_weight: f32,
        filters: Option<SearchFilters>,
    ) -> Result<Vec<SearchResult>, SearchError> {
        // Implementação da busca híbrida com RRF
    }
}
```

### Database Layer

```rust
pub struct Database {
    connection: Arc<Mutex<Connection>>,
    config: DatabaseConfig,
}

impl Database {
    pub async fn insert_event(&self, event: &KeyEvent) -> Result<u64, DatabaseError> {
        // Inserir evento no banco
    }
    
    pub async fn get_stats(&self) -> Result<DatabaseStats, DatabaseError> {
        // Obter estatísticas do banco
    }
    
    pub async fn vacuum(&self) -> Result<(), DatabaseError> {
        // Otimizar banco de dados
    }
    
    pub async fn backup(&self, path: &Path) -> Result<(), DatabaseError> {
        // Criar backup
    }
}
```

### Agent Controller

```rust
pub struct Agent {
    event_sender: mpsc::Sender<KeyEvent>,
    is_running: Arc<AtomicBool>,
    config: AgentConfig,
    stats: Arc<Mutex<AgentStats>>,
}

impl Agent {
    pub async fn start(&self) -> Result<(), AgentError> {
        // Iniciar captura de teclas
    }
    
    pub async fn stop(&self) -> Result<(), AgentError> {
        // Parar captura de teclas
    }
    
    pub fn is_running(&self) -> bool {
        // Verificar se está rodando
    }
    
    pub async fn get_stats(&self) -> AgentStats {
        // Obter estatísticas do agente
    }
}
```

## 📊 Tipos de Dados

### Core Types

```typescript
// Resultado de busca
interface SearchResult {
  id: string;
  content: string;
  timestamp: number;
  application?: string;
  window_title?: string;
  relevance_score: number;
  snippet: string;
  highlights?: TextHighlight[];
}

// Destaque de texto
interface TextHighlight {
  start: number;
  end: number;
  type: 'match' | 'context';
}

// Filtros de busca
interface SearchFilters {
  date_range?: {
    start: number;
    end: number;
  };
  applications?: string[];
  exclude_applications?: string[];
  min_relevance?: number;
  content_type?: 'text' | 'url' | 'email' | 'all';
}

// Padrão personalizado de mascaramento
interface CustomPattern {
  name: string;
  pattern: string;
  replacement: string;
  enabled: boolean;
  description?: string;
}

// Evento de teclado
interface KeyEvent {
  id: string;
  content: string;
  timestamp: number;
  application?: string;
  window_title?: string;
  masked: boolean;
  raw_content?: string; // Apenas para debug, nunca persistido
}
```

### Rust Types

```rust
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub content: String,
    pub timestamp: i64,
    pub application: Option<String>,
    pub window_title: Option<String>,
    pub relevance_score: f32,
    pub snippet: String,
    pub highlights: Option<Vec<TextHighlight>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchFilters {
    pub date_range: Option<DateRange>,
    pub applications: Option<Vec<String>>,
    pub exclude_applications: Option<Vec<String>>,
    pub min_relevance: Option<f32>,
    pub content_type: Option<ContentType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateRange {
    pub start: i64,
    pub end: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContentType {
    Text,
    Url,
    Email,
    All,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyEvent {
    pub id: String,
    pub content: String,
    pub timestamp: i64,
    pub application: Option<String>,
    pub window_title: Option<String>,
    pub masked: bool,
}
```

## ⚠️ Códigos de Erro

### Error Types

```typescript
// Códigos de erro padronizados
enum ErrorCode {
  // Erros de validação (400x)
  INVALID_QUERY = 4001,
  INVALID_LIMIT = 4002,
  INVALID_FILTERS = 4003,
  INVALID_SETTINGS = 4004,
  
  // Erros de banco de dados (500x)
  DATABASE_CONNECTION = 5001,
  DATABASE_QUERY = 5002,
  DATABASE_CORRUPTION = 5003,
  
  // Erros do agente (600x)
  AGENT_NOT_RUNNING = 6001,
  AGENT_PERMISSION_DENIED = 6002,
  AGENT_PLATFORM_UNSUPPORTED = 6003,
  
  // Erros de busca (700x)
  SEARCH_ENGINE_ERROR = 7001,
  EMBEDDING_MODEL_ERROR = 7002,
  INDEX_CORRUPTION = 7003,
  
  // Erros de sistema (800x)
  INSUFFICIENT_MEMORY = 8001,
  DISK_SPACE_LOW = 8002,
  PERMISSION_DENIED = 8003,
}

interface ApiError {
  code: ErrorCode;
  message: string;
  details?: Record<string, any>;
  timestamp: number;
}
```

### Rust Error Handling

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Invalid query: {message}")]
    InvalidQuery { message: String },
    
    #[error("Database error: {source}")]
    Database {
        #[from]
        source: DatabaseError,
    },
    
    #[error("Agent error: {source}")]
    Agent {
        #[from]
        source: AgentError,
    },
    
    #[error("Search error: {source}")]
    Search {
        #[from]
        source: SearchError,
    },
    
    #[error("Internal server error")]
    Internal,
}

impl From<ApiError> for String {
    fn from(error: ApiError) -> Self {
        error.to_string()
    }
}
```

## 💡 Exemplos de Uso

### Frontend React

```typescript
import { invoke } from '@tauri-apps/api/tauri';
import { useState, useEffect } from 'react';

// Hook para busca
function useSearch() {
  const [results, setResults] = useState<SearchResult[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const search = async (query: string, type: 'text' | 'semantic' | 'hybrid' = 'hybrid') => {
    setLoading(true);
    setError(null);
    
    try {
      const results = await invoke<SearchResult[]>(`search_${type}`, {
        query,
        limit: 50
      });
      setResults(results);
    } catch (err) {
      setError(err as string);
    } finally {
      setLoading(false);
    }
  };

  return { results, loading, error, search };
}

// Componente de busca
function SearchComponent() {
  const { results, loading, error, search } = useSearch();
  const [query, setQuery] = useState('');

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (query.trim()) {
      search(query);
    }
  };

  return (
    <div>
      <form onSubmit={handleSubmit}>
        <input
          type="text"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder="Digite sua busca..."
        />
        <button type="submit" disabled={loading}>
          {loading ? 'Buscando...' : 'Buscar'}
        </button>
      </form>

      {error && <div className="error">{error}</div>}

      <div className="results">
        {results.map((result) => (
          <div key={result.id} className="result-item">
            <div className="content">{result.snippet}</div>
            <div className="metadata">
              <span>{new Date(result.timestamp * 1000).toLocaleString()}</span>
              {result.application && <span>{result.application}</span>}
              <span>Score: {result.relevance_score.toFixed(2)}</span>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
```

### Configurações

```typescript
import { invoke } from '@tauri-apps/api/tauri';

// Hook para configurações
function useSettings() {
  const [settings, setSettings] = useState<AppSettings | null>(null);

  useEffect(() => {
    loadSettings();
  }, []);

  const loadSettings = async () => {
    try {
      const settings = await invoke<AppSettings>('get_settings');
      setSettings(settings);
    } catch (err) {
      console.error('Failed to load settings:', err);
    }
  };

  const updateSettings = async (newSettings: Partial<AppSettings>) => {
    try {
      await invoke('update_settings', {
        settings: { ...settings, ...newSettings }
      });
      await loadSettings(); // Recarregar
    } catch (err) {
      console.error('Failed to update settings:', err);
      throw err;
    }
  };

  return { settings, updateSettings };
}
```

### Monitoramento

```typescript
import { invoke } from '@tauri-apps/api/tauri';

// Hook para estatísticas
function useStats() {
  const [stats, setStats] = useState<AppStats | null>(null);

  useEffect(() => {
    const interval = setInterval(async () => {
      try {
        const stats = await invoke<AppStats>('get_stats');
        setStats(stats);
      } catch (err) {
        console.error('Failed to fetch stats:', err);
      }
    }, 5000); // Atualizar a cada 5 segundos

    return () => clearInterval(interval);
  }, []);

  return stats;
}

// Componente de dashboard
function StatsComponent() {
  const stats = useStats();

  if (!stats) return <div>Carregando...</div>;

  return (
    <div className="stats-dashboard">
      <div className="stat-card">
        <h3>Eventos Capturados</h3>
        <p>{stats.database.total_events.toLocaleString()}</p>
      </div>
      
      <div className="stat-card">
        <h3>Tamanho do Banco</h3>
        <p>{stats.database.total_size_mb.toFixed(1)} MB</p>
      </div>
      
      <div className="stat-card">
        <h3>Uso de Memória</h3>
        <p>{stats.performance.memory_usage_mb.toFixed(1)} MB</p>
      </div>
      
      <div className="stat-card">
        <h3>Latência de Busca</h3>
        <p>{stats.performance.search_latency_ms.toFixed(1)} ms</p>
      </div>
    </div>
  );
}
```

## 🔔 Webhooks e Eventos

### Event System

```typescript
import { listen } from '@tauri-apps/api/event';

// Escutar eventos do sistema
listen('agent_status_changed', (event) => {
  console.log('Agent status:', event.payload);
});

listen('search_completed', (event) => {
  console.log('Search results:', event.payload);
});

listen('settings_updated', (event) => {
  console.log('Settings updated:', event.payload);
});

listen('error_occurred', (event) => {
  console.error('System error:', event.payload);
});
```

### Rust Event Emission

```rust
use tauri::Manager;

// Emitir eventos para o frontend
impl AppState {
    pub async fn emit_agent_status(&self, status: AgentStatus) {
        self.app_handle
            .emit_all("agent_status_changed", &status)
            .unwrap_or_else(|e| eprintln!("Failed to emit event: {}", e));
    }
    
    pub async fn emit_search_completed(&self, results: &[SearchResult]) {
        self.app_handle
            .emit_all("search_completed", results)
            .unwrap_or_else(|e| eprintln!("Failed to emit event: {}", e));
    }
    
    pub async fn emit_error(&self, error: &ApiError) {
        self.app_handle
            .emit_all("error_occurred", error)
            .unwrap_or_else(|e| eprintln!("Failed to emit event: {}", e));
    }
}
```

---

Esta documentação da API fornece uma referência completa para desenvolvedores que trabalham com o KeyAI Desktop. Para exemplos mais específicos ou dúvidas sobre implementação, consulte o código-fonte ou abra uma issue no repositório. 