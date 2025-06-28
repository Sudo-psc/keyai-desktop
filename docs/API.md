# 📡 Documentação da API - KeyAI Desktop

## 📋 Índice

- [Visão Geral](#visão-geral)
- [Comandos Tauri](#comandos-tauri)
- [API do Backend](#api-do-backend)
- [Tipos de Dados](#tipos-de-dados)
- [Códigos de Erro](#códigos-de-erro)
- [Exemplos de Uso](#exemplos-de-uso)

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
      capture_passwords: false
    }
  }
});
```

### Estatísticas

#### `get_stats`

Obtém estatísticas do banco de dados e performance.

```typescript
interface AppStats {
  database: DatabaseStats;
  performance: PerformanceStats;
  agent_status: AgentStatus;
}

// Uso
const stats = await invoke<AppStats>('get_stats');
```

### Controle do Agente

#### `start_agent`

Inicia o agente de captura de teclas.

```typescript
await invoke('start_agent');
```

#### `stop_agent`

Para o agente de captura de teclas.

```typescript
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
```

### Rust Types

```rust
use serde::{Deserialize, Serialize};

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
```

## ⚠️ Códigos de Erro

### Error Types

```typescript
enum ErrorCode {
  // Erros de validação (400x)
  INVALID_QUERY = 4001,
  INVALID_LIMIT = 4002,
  INVALID_FILTERS = 4003,
  
  // Erros de banco de dados (500x)
  DATABASE_CONNECTION = 5001,
  DATABASE_QUERY = 5002,
  DATABASE_CORRUPTION = 5003,
  
  // Erros do agente (600x)
  AGENT_NOT_RUNNING = 6001,
  AGENT_PERMISSION_DENIED = 6002,
  AGENT_PLATFORM_UNSUPPORTED = 6003,
}

interface ApiError {
  code: ErrorCode;
  message: string;
  details?: Record<string, any>;
  timestamp: number;
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

---

Esta documentação da API fornece uma referência completa para desenvolvedores que trabalham com o KeyAI Desktop. Para exemplos mais específicos ou dúvidas sobre implementação, consulte o código-fonte ou abra uma issue no repositório. 