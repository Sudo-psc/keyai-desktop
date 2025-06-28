# 🏗️ Arquitetura do Sistema - KeyAI Desktop

## 📋 Índice

- [Visão Geral](#visão-geral)
- [Arquitetura de Alto Nível](#arquitetura-de-alto-nível)
- [Componentes do Sistema](#componentes-do-sistema)
- [Fluxo de Dados](#fluxo-de-dados)
- [Camada de Persistência](#camada-de-persistência)
- [Busca Híbrida](#busca-híbrida)
- [Segurança e Privacidade](#segurança-e-privacidade)
- [Performance](#performance)
- [Decisões Arquiteturais](#decisões-arquiteturais)

## 🎯 Visão Geral

O KeyAI Desktop é um sistema de captura e busca de teclas focado em privacidade, construído com uma arquitetura desacoplada que prioriza performance, segurança e resiliência. O sistema opera inteiramente offline, garantindo que todos os dados permaneçam no dispositivo do usuário.

### Princípios Arquiteturais

1. **Privacidade por Design**: Todos os dados permanecem locais
2. **Desacoplamento**: Componentes independentes comunicam-se via canais
3. **Performance**: Otimizado para baixa latência e uso eficiente de recursos
4. **Resiliência**: Falhas em um componente não afetam outros
5. **Segurança**: Múltiplas camadas de proteção

## 🏛️ Arquitetura de Alto Nível

```
┌─────────────────────────────────────────────────────────────────┐
│                        KeyAI Desktop                            │
├─────────────────────────────────────────────────────────────────┤
│                    Frontend (React + TypeScript)               │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐             │
│  │   Search    │  │  Settings   │  │   Results   │             │
│  │ Component   │  │ Component   │  │ Component   │             │
│  └─────────────┘  └─────────────┘  └─────────────┘             │
├─────────────────────────────────────────────────────────────────┤
│                    Tauri Bridge Layer                          │
│  ┌─────────────────────────────────────────────────────────────┐ │
│  │                 Tauri Commands                              │ │
│  │  search_text | search_semantic | search_hybrid | ...       │ │
│  └─────────────────────────────────────────────────────────────┘ │
├─────────────────────────────────────────────────────────────────┤
│                    Backend (Rust Core)                         │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐             │
│  │   Agent     │  │   Masker    │  │   Search    │             │
│  │  (Capture)  │  │ (PII Filter)│  │  (Hybrid)   │             │
│  └─────────────┘  └─────────────┘  └─────────────┘             │
│         │                │                │                    │
│         └────────────────┼────────────────┘                    │
│                          │                                     │
│  ┌─────────────────────────────────────────────────────────────┐ │
│  │                Database Layer (SQLite)                     │ │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐         │ │
│  │  │   Events    │  │    FTS5     │  │ sqlite-vec  │         │ │
│  │  │   Table     │  │   Search    │  │  Embeddings │         │ │
│  │  └─────────────┘  └─────────────┘  └─────────────┘         │ │
│  └─────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

## 🔧 Componentes do Sistema

### 1. Frontend (React + TypeScript)

**Responsabilidades:**
- Interface do usuário responsiva
- Comunicação com backend via Tauri
- Gerenciamento de estado da aplicação
- Exibição de resultados de busca

**Estrutura:**
```
frontend/
├── src/
│   ├── components/
│   │   ├── SearchBar.tsx      # Barra de busca principal
│   │   ├── ResultsList.tsx    # Lista de resultados
│   │   ├── SettingsPanel.tsx  # Painel de configurações
│   │   └── Header.tsx         # Cabeçalho da aplicação
│   ├── hooks/
│   │   ├── useSearch.ts       # Hook para busca
│   │   ├── useSettings.ts     # Hook para configurações
│   │   └── useKeyboard.ts     # Hook para atalhos
│   ├── types.ts              # Tipos TypeScript
│   └── App.tsx               # Componente principal
```

### 2. Tauri Bridge Layer

**Responsabilidades:**
- Ponte entre frontend e backend
- Validação de comandos
- Serialização/deserialização de dados
- Gerenciamento de estado da aplicação

**Comandos Expostos:**
```rust
#[tauri::command]
async fn search_text(query: String, limit: Option<u32>) -> Result<Vec<SearchResult>, String>

#[tauri::command]
async fn search_semantic(query: String, limit: Option<u32>) -> Result<Vec<SearchResult>, String>

#[tauri::command]
async fn search_hybrid(query: String, text_weight: f32, semantic_weight: f32) -> Result<Vec<SearchResult>, String>

#[tauri::command]
async fn get_stats() -> Result<DatabaseStats, String>

#[tauri::command]
async fn update_settings(settings: AppSettings) -> Result<(), String>
```

### 3. Agent (Captura de Teclas)

**Responsabilidades:**
- Captura global de eventos de teclado
- Filtragem de eventos relevantes
- Envio de dados para o Masker

**Implementação:**
```rust
pub struct Agent {
    event_sender: mpsc::Sender<KeyEvent>,
    is_running: Arc<AtomicBool>,
    config: AgentConfig,
}

impl Agent {
    pub fn new(sender: mpsc::Sender<KeyEvent>) -> Result<Self, AgentError> {
        // Inicialização do agente
    }
    
    pub async fn start(&self) -> Result<(), AgentError> {
        // Thread de captura de alta prioridade
        rdev::listen(move |event| {
            if let Event::KeyPress(key) = event {
                self.process_key_event(key);
            }
        })?;
    }
}
```

**Características:**
- Thread de alta prioridade para não perder eventos
- Filtros configuráveis (aplicações, tipos de eventos)
- Tratamento de erros robusto
- Suporte multiplataforma (Windows, macOS X11, Linux X11)

### 4. Masker (Filtragem de PII)

**Responsabilidades:**
- Recebimento de eventos do Agent
- Aplicação de padrões de mascaramento
- Envio de dados mascarados para o banco

**Padrões de Mascaramento:**
```rust
pub struct PiiMasker {
    patterns: Vec<MaskPattern>,
    config: MaskConfig,
}

pub enum MaskPattern {
    Cpf(Regex),
    Email(Regex),
    Phone(Regex),
    CreditCard(Regex),
    Password(Regex),
    Custom(Regex, String),
}

impl PiiMasker {
    pub fn mask_text(&self, text: &str) -> String {
        let mut masked = text.to_string();
        
        for pattern in &self.patterns {
            masked = pattern.apply(&masked);
        }
        
        masked
    }
}
```

### 5. Database Layer (SQLite + SQLCipher)

**Responsabilidades:**
- Armazenamento criptografado de dados
- Indexação para busca rápida
- Gestão de transações
- Backup e recuperação

**Schema do Banco:**
```sql
-- Tabela principal de eventos
CREATE TABLE events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    content TEXT NOT NULL,           -- Texto mascarado
    timestamp INTEGER NOT NULL,      -- Unix timestamp
    application TEXT,               -- Aplicação ativa (opcional)
    window_title TEXT,              -- Título da janela (opcional)
    created_at INTEGER DEFAULT (unixepoch())
);

-- Índices para performance
CREATE INDEX idx_events_timestamp ON events(timestamp);
CREATE INDEX idx_events_application ON events(application);

-- Tabela virtual para busca full-text
CREATE VIRTUAL TABLE events_fts USING fts5(
    content,
    application,
    window_title,
    content='events',
    content_rowid='id'
);

-- Triggers para manter FTS sincronizado
CREATE TRIGGER events_fts_insert AFTER INSERT ON events BEGIN
    INSERT INTO events_fts(rowid, content, application, window_title)
    VALUES (new.id, new.content, new.application, new.window_title);
END;

-- Tabela para embeddings vetoriais
CREATE VIRTUAL TABLE embeddings USING vec0(
    id INTEGER PRIMARY KEY,
    embedding FLOAT[384]  -- Dimensão do modelo de embedding
);
```

### 6. Search Engine (Busca Híbrida)

**Responsabilidades:**
- Busca textual via FTS5
- Busca semântica via embeddings
- Combinação de resultados (RRF)
- Ranking e paginação

**Implementação:**
```rust
pub struct SearchEngine {
    db: Database,
    embedding_model: EmbeddingModel,
    config: SearchConfig,
}

impl SearchEngine {
    pub async fn search_hybrid(
        &self,
        query: &str,
        text_weight: f32,
        semantic_weight: f32,
        limit: u32,
    ) -> Result<Vec<SearchResult>, SearchError> {
        // Busca textual
        let text_results = self.search_text(query, limit * 2).await?;
        
        // Busca semântica
        let semantic_results = self.search_semantic(query, limit * 2).await?;
        
        // Combinar usando Reciprocal Rank Fusion
        let combined = self.combine_results(
            text_results,
            semantic_results,
            text_weight,
            semantic_weight,
        );
        
        Ok(combined.into_iter().take(limit as usize).collect())
    }
}
```

## 🔄 Fluxo de Dados

### 1. Captura e Armazenamento

```
[Usuário Digita] 
    ↓
[rdev captura evento]
    ↓
[Agent processa e filtra]
    ↓ (MPSC Channel)
[Masker aplica padrões PII]
    ↓
[Database Writer armazena em lote]
    ↓
[SQLite + FTS5 + Embeddings]
```

### 2. Busca e Recuperação

```
[Usuário faz busca]
    ↓
[Frontend React]
    ↓ (Tauri Command)
[Search Engine]
    ├─ [FTS5 Query] ──┐
    └─ [Vector Query] ─┤
                       ├─ [RRF Fusion]
                       ↓
[Resultados Rankeados]
    ↓ (Tauri Response)
[Frontend exibe resultados]
```

### 3. Comunicação Entre Threads

```rust
// Canais de comunicação
let (key_tx, key_rx) = mpsc::channel::<KeyEvent>(1000);
let (mask_tx, mask_rx) = mpsc::channel::<MaskedEvent>(1000);
let (db_tx, db_rx) = mpsc::channel::<DatabaseEvent>(100);

// Thread do Agent
tokio::spawn(async move {
    agent.run(key_tx).await;
});

// Thread do Masker
tokio::spawn(async move {
    masker.run(key_rx, mask_tx).await;
});

// Thread do Database Writer
tokio::spawn(async move {
    db_writer.run(mask_rx).await;
});
```

## 💾 Camada de Persistência

### Configuração SQLCipher

```rust
pub struct Database {
    connection: Arc<Mutex<Connection>>,
    config: DatabaseConfig,
}

impl Database {
    pub async fn new(db_path: &Path, password: &str) -> Result<Self, DatabaseError> {
        let mut conn = Connection::open(db_path)?;
        
        // Configurar SQLCipher
        conn.execute(&format!("PRAGMA key = '{}';", password), [])?;
        conn.execute("PRAGMA cipher_page_size = 4096;", [])?;
        conn.execute("PRAGMA kdf_iter = 256000;", [])?;
        conn.execute("PRAGMA cipher_hmac_algorithm = HMAC_SHA256;", [])?;
        
        // Configurações de performance
        conn.execute("PRAGMA journal_mode = WAL;", [])?;
        conn.execute("PRAGMA synchronous = NORMAL;", [])?;
        conn.execute("PRAGMA cache_size = 10000;", [])?;
        
        Ok(Database {
            connection: Arc::new(Mutex::new(conn)),
            config: DatabaseConfig::default(),
        })
    }
}
```

### Estratégia de Backup

```rust
impl Database {
    pub async fn backup(&self, backup_path: &Path) -> Result<(), DatabaseError> {
        let conn = self.connection.lock().await;
        
        // Usar backup API do SQLite
        let backup = rusqlite::backup::Backup::new(&*conn, backup_path)?;
        backup.run_to_completion(5, Duration::from_millis(250), None)?;
        
        Ok(())
    }
    
    pub async fn vacuum(&self) -> Result<(), DatabaseError> {
        let conn = self.connection.lock().await;
        conn.execute("VACUUM;", [])?;
        Ok(())
    }
}
```

## 🔍 Busca Híbrida

### Algoritmo de Fusão (RRF)

```rust
pub fn reciprocal_rank_fusion(
    text_results: Vec<SearchResult>,
    semantic_results: Vec<SearchResult>,
    text_weight: f32,
    semantic_weight: f32,
    k: f32,
) -> Vec<SearchResult> {
    let mut scores = HashMap::new();
    
    // Pontuação da busca textual
    for (rank, result) in text_results.iter().enumerate() {
        let score = text_weight / (k + rank as f32 + 1.0);
        scores.entry(result.id).and_modify(|s| *s += score).or_insert(score);
    }
    
    // Pontuação da busca semântica
    for (rank, result) in semantic_results.iter().enumerate() {
        let score = semantic_weight / (k + rank as f32 + 1.0);
        scores.entry(result.id).and_modify(|s| *s += score).or_insert(score);
    }
    
    // Ordenar por pontuação
    let mut combined: Vec<_> = scores.into_iter().collect();
    combined.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    
    combined.into_iter().map(|(id, score)| {
        // Buscar resultado completo por ID
        get_result_by_id(id)
    }).collect()
}
```

### Geração de Embeddings

```rust
pub struct EmbeddingModel {
    model: BertModel,
    tokenizer: BertTokenizer,
    device: Device,
}

impl EmbeddingModel {
    pub fn generate_embedding(&self, text: &str) -> Result<Vec<f32>, EmbeddingError> {
        let tokens = self.tokenizer.encode(text, 512, &TruncationStrategy::LongestFirst, 0);
        let input_ids = Tensor::of_slice(&tokens.token_ids).to(self.device);
        
        let output = self.model.forward_t(&input_ids, None, None, false);
        let embeddings = output.pooled_output;
        
        // Normalizar embeddings
        let normalized = embeddings / embeddings.norm_dim(1, true, Kind::Float);
        
        Ok(normalized.into())
    }
}
```

## 🔒 Segurança e Privacidade

### Arquitetura de Segurança

```
┌─────────────────────────────────────────────────────────┐
│                 Camada de Aplicação                     │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐     │
│  │ Validação   │  │ Sanitização │  │  Controle   │     │
│  │   Input     │  │   Output    │  │   Acesso    │     │
│  └─────────────┘  └─────────────┘  └─────────────┘     │
├─────────────────────────────────────────────────────────┤
│                  Camada de Lógica                       │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐     │
│  │ Mascaramento│  │ Validação   │  │  Gestão     │     │
│  │     PII     │  │   Dados     │  │ Memória     │     │
│  └─────────────┘  └─────────────┘  └─────────────┘     │
├─────────────────────────────────────────────────────────┤
│                Camada de Persistência                   │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐     │
│  │ Criptografia│  │  Controle   │  │ Integridade │     │
│  │  AES-256    │  │   Acesso    │  │   Dados     │     │
│  └─────────────┘  └─────────────┘  └─────────────┘     │
├─────────────────────────────────────────────────────────┤
│                  Camada do Sistema                      │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐     │
│  │ Permissões  │  │ Isolamento  │  │  Auditoria  │     │
│  │  Arquivo    │  │  Processo   │  │   Sistema   │     │
│  └─────────────┘  └─────────────┘  └─────────────┘     │
└─────────────────────────────────────────────────────────┘
```

### Gestão de Memória Segura

```rust
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(ZeroizeOnDrop)]
pub struct SecureString {
    data: String,
}

impl SecureString {
    pub fn new(data: String) -> Self {
        Self { data }
    }
    
    pub fn as_str(&self) -> &str {
        &self.data
    }
}

// Automaticamente zera a memória quando sai de escopo
```

## ⚡ Performance

### Otimizações de Performance

1. **Captura de Teclas**
   - Thread de alta prioridade
   - Buffer circular para eventos
   - Filtragem em tempo real

2. **Banco de Dados**
   - Escritas em lote
   - Índices otimizados
   - WAL mode para concorrência

3. **Busca**
   - Cache de embeddings
   - Paginação eficiente
   - Paralelização de queries

### Métricas de Performance

```rust
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub capture_latency_ms: f64,
    pub masking_latency_ms: f64,
    pub search_latency_ms: f64,
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
}

impl PerformanceMetrics {
    pub fn collect() -> Self {
        // Coleta métricas do sistema
    }
}
```

### Benchmarks

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn search_benchmark(c: &mut Criterion) {
    let db = setup_test_database();
    
    c.bench_function("text_search", |b| {
        b.iter(|| {
            db.search_text(black_box("test query"), black_box(50))
        })
    });
    
    c.bench_function("semantic_search", |b| {
        b.iter(|| {
            db.search_semantic(black_box("test query"), black_box(50))
        })
    });
}

criterion_group!(benches, search_benchmark);
criterion_main!(benches);
```

## 🎯 Decisões Arquiteturais

### 1. Rust como Linguagem Principal

**Motivação:**
- Performance próxima ao C/C++
- Segurança de memória
- Concorrência sem data races
- Ecossistema maduro

**Trade-offs:**
- ✅ Segurança e performance
- ❌ Curva de aprendizado mais íngreme

### 2. Tauri vs Electron

**Motivação:**
- Binários menores (~10MB vs ~100MB)
- Menor uso de memória
- Melhor performance
- Segurança por padrão

**Trade-offs:**
- ✅ Performance e segurança
- ❌ Ecossistema menor que Electron

### 3. SQLite + SQLCipher

**Motivação:**
- Banco embarcado (sem servidor)
- Criptografia nativa
- FTS5 integrado
- Suporte a extensões (sqlite-vec)

**Trade-offs:**
- ✅ Simplicidade e segurança
- ❌ Limitações de concorrência

### 4. Arquitetura Baseada em Canais

**Motivação:**
- Desacoplamento de componentes
- Tolerância a falhas
- Paralelismo eficiente
- Backpressure natural

**Trade-offs:**
- ✅ Resiliência e performance
- ❌ Complexidade de debugging

### 5. Busca Híbrida (FTS5 + Embeddings)

**Motivação:**
- Melhor relevância de resultados
- Busca semântica avançada
- Fallback para busca textual
- Configurabilidade de pesos

**Trade-offs:**
- ✅ Qualidade de busca superior
- ❌ Maior complexidade e uso de recursos

## 🔄 Evolução da Arquitetura

### Versão 1.0 (Atual)

- Arquitetura básica estabelecida
- Funcionalidades core implementadas
- Foco em estabilidade e performance

### Versão 1.1 (Planejada)

- Suporte para Wayland
- Customização de padrões PII
- Melhorias na busca semântica
- API para extensões

### Versão 2.0 (Futuro)

- Arquitetura de plugins
- OCR para captura de texto
- Sincronização opcional (E2E encrypted)
- Machine Learning local avançado

---

Esta documentação de arquitetura serve como referência técnica para desenvolvedores e contribuidores do projeto KeyAI Desktop. Para dúvidas específicas sobre implementação, consulte o código-fonte ou abra uma issue no repositório. 