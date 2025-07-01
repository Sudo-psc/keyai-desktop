# 🏗️ Revisão de Código e Proposta de Arquitetura de Microsserviços - KeyAI Desktop

## 📋 Índice

1. [Resumo Executivo](#resumo-executivo)
2. [Revisão da Arquitetura Atual](#revisão-da-arquitetura-atual)
3. [Análise de Código e Problemas Identificados](#análise-de-código-e-problemas-identificados)
4. [Proposta de Arquitetura de Microsserviços](#proposta-de-arquitetura-de-microsserviços)
5. [Arquitetura Detalhada dos Microsserviços](#arquitetura-detalhada-dos-microsserviços)
6. [Stack Tecnológico Recomendado](#stack-tecnológico-recomendado)
7. [Plano de Migração](#plano-de-migração)
8. [Considerações de Segurança](#considerações-de-segurança)
9. [Monitoramento e Observabilidade](#monitoramento-e-observabilidade)
10. [Custos e Recursos](#custos-e-recursos)
11. [Riscos e Mitigações](#riscos-e-mitigações)
12. [Conclusão e Próximos Passos](#conclusão-e-próximos-passos)

## 📍 Resumo Executivo

### Estado Atual
O KeyAI Desktop é atualmente uma aplicação monolítica baseada em Tauri com backend Rust e frontend React. Embora funcional, a arquitetura atual apresenta limitações de escalabilidade, manutenibilidade e extensibilidade.

### Proposta
Migrar para uma arquitetura de microsserviços distribuídos, mantendo a opção de deployment local (on-premise) para preservar a privacidade, mas permitindo também deployment em nuvem para usuários que desejam funcionalidades avançadas.

### Benefícios Principais
- **Escalabilidade**: Cada serviço pode escalar independentemente
- **Manutenibilidade**: Código mais modular e fácil de manter
- **Extensibilidade**: Facilita adição de novos recursos
- **Flexibilidade**: Suporta tanto deployment local quanto cloud
- **Performance**: Melhor distribuição de carga

## 🔍 Revisão da Arquitetura Atual

### Pontos Fortes
1. **Privacidade**: Todos os dados permanecem locais
2. **Performance**: Baixa latência por ser local
3. **Segurança**: Criptografia end-to-end com SQLCipher
4. **Simplicidade**: Um único binário para distribuir

### Limitações Identificadas
1. **Acoplamento**: Componentes fortemente acoplados
2. **Escalabilidade**: Limitado aos recursos da máquina local
3. **Manutenibilidade**: Mudanças requerem rebuild completo
4. **Testabilidade**: Difícil testar componentes isoladamente
5. **Extensibilidade**: Adicionar novos recursos é complexo

### Arquitetura Atual Simplificada
```
┌─────────────────────────────────────────┐
│          Aplicação Monolítica           │
│  ┌───────────┐  ┌──────────┐  ┌──────┐ │
│  │   Agent   │  │  Masker  │  │  DB  │ │
│  └───────────┘  └──────────┘  └──────┘ │
│  ┌───────────┐  ┌──────────┐           │
│  │  Search   │  │   GUI    │           │
│  └───────────┘  └──────────┘           │
└─────────────────────────────────────────┘
```

## 🐛 Análise de Código e Problemas Identificados

### 1. **Estrutura Monolítica (src/main.rs)**
```rust
// PROBLEMA: Todos os componentes inicializados no mesmo processo
let database = Database::new("keyai.db").await?;
let search_engine = SearchEngine::new(Arc::clone(&database)).await?;
let masker = Masker::new();
let agent = Agent::new(masker, Arc::clone(&database)).await?;
```
**Impacto**: Falha em um componente derruba toda a aplicação.

### 2. **Acoplamento Forte (src/agent/mod.rs)**
```rust
pub struct Agent {
    masker: Masker,           // Acoplamento direto
    database: Arc<Database>,  // Dependência direta
}
```
**Impacto**: Impossível testar Agent sem Masker e Database.

### 3. **Comunicação Síncrona**
```rust
// PROBLEMA: Processamento síncrono pode bloquear captura
let masked_text = self.masker.mask_text(&event.text);
self.database.insert_event(masked_text).await?;
```
**Impacto**: Latência no mascaramento ou DB pode perder eventos.

### 4. **Falta de Abstração de Transporte**
```rust
// PROBLEMA: Comunicação via canais MPSC hardcoded
let (tx, rx) = mpsc::channel::<KeyEvent>(1000);
```
**Impacto**: Difícil migrar para comunicação remota.

### 5. **Gerenciamento de Estado Global**
```rust
pub struct AppState {
    pub database: Arc<Database>,
    pub search_engine: Arc<SearchEngine>,
    pub agent: Arc<Mutex<Agent>>,
}
```
**Impacto**: Estado compartilhado dificulta distribuição.

### 6. **Falta de Circuit Breakers**
```rust
// PROBLEMA: Sem proteção contra falhas em cascata
match state.database.search_text(&query, limit).await {
    Ok(results) => Ok(results),
    Err(e) => Err(format!("Erro: {}", e)) // Falha direta
}
```

### 7. **Logging e Métricas Limitados**
```rust
// PROBLEMA: Apenas logging básico, sem métricas
info!("✅ Busca concluída: {} resultados", results.len());
```
**Impacto**: Difícil monitorar e debugar em produção.

## 🎯 Proposta de Arquitetura de Microsserviços

### Visão Geral

```
┌─────────────────────────────────────────────────────────────────────────┐
│                          API Gateway (Kong/Traefik)                     │
└─────────────────────────────────────────────────────────────────────────┘
                |              |              |              |
     ┌──────────▼───┐   ┌─────▼──────┐   ┌──▼───────┐   ┌─▼──────────┐
     │ Auth Service │   │ Web Client │   │ Desktop  │   │ API Service│
     │   (Rust)     │   │  (React)   │   │  Client  │   │  (GraphQL) │
     └──────────────┘   └────────────┘   └──────────┘   └────────────┘
                                                               |
     ┌──────────────────────────────────────────────────────────┐
     │                    Message Bus (NATS/RabbitMQ)           │
     └──────────────────────────────────────────────────────────┘
        |           |            |            |            |
   ┌────▼────┐  ┌──▼──────┐  ┌─▼──────┐  ┌─▼──────┐  ┌─▼──────────┐
   │ Capture │  │ Masking │  │ Search │  │Storage │  │ Analytics  │
   │ Service │  │ Service │  │Service │  │Service │  │  Service   │
   │ (Rust)  │  │ (Rust)  │  │ (Rust) │  │ (Rust) │  │   (Rust)   │
   └─────────┘  └─────────┘  └────────┘  └────────┘  └────────────┘
        |           |            |            |            |
   ┌────▼────────────────────────────────────▼────────────▼──────┐
   │                     Data Layer                              │
   │  ┌────────────┐  ┌──────────────┐  ┌──────────────┐        │
   │  │ TimeSeries │  │  PostgreSQL  │  │   Vector DB  │        │
   │  │   (QuestDB)│  │  (Main Data) │  │ (Qdrant/Weaviate)    │
   │  └────────────┘  └──────────────┘  └──────────────┘        │
   └─────────────────────────────────────────────────────────────┘
```

### Princípios de Design

1. **Domain-Driven Design (DDD)**: Cada serviço representa um bounded context
2. **Event-Driven Architecture**: Comunicação assíncrona via eventos
3. **CQRS**: Separação de comandos e queries
4. **API-First**: Contratos bem definidos entre serviços
5. **Cloud-Native**: Preparado para Kubernetes mas rodável localmente

## 🔧 Arquitetura Detalhada dos Microsserviços

### 1. **Capture Service** 🎯
**Responsabilidade**: Captura de eventos de teclado

```rust
// capture-service/src/main.rs
#[derive(Debug, Serialize, Deserialize)]
pub struct CaptureEvent {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub raw_text: String,
    pub application: Option<String>,
    pub window_title: Option<String>,
}

pub trait EventPublisher {
    async fn publish(&self, event: CaptureEvent) -> Result<(), Error>;
}

pub struct CaptureService<P: EventPublisher> {
    publisher: P,
    config: CaptureConfig,
}

impl<P: EventPublisher> CaptureService<P> {
    pub async fn run(&self) -> Result<(), Error> {
        let (tx, mut rx) = mpsc::channel(1000);
        
        // Thread de captura isolada
        thread::spawn(move || {
            rdev::listen(move |event| {
                if let Event::KeyPress(key) = event {
                    let _ = tx.send(KeyEvent::from(key));
                }
            }).expect("Failed to listen");
        });
        
        // Processamento assíncrono
        while let Some(key_event) = rx.recv().await {
            let capture_event = self.process_event(key_event)?;
            
            // Publicar evento sem bloquear
            tokio::spawn(async move {
                if let Err(e) = self.publisher.publish(capture_event).await {
                    error!("Failed to publish event: {}", e);
                }
            });
        }
        
        Ok(())
    }
}
```

**Características**:
- Stateless e horizontalmente escalável
- Tolerante a falhas de downstream
- Suporta múltiplas instâncias por usuário
- Circuit breaker para publisher

### 2. **Masking Service** 🔒
**Responsabilidade**: Mascaramento de PII

```rust
// masking-service/src/main.rs
#[derive(Debug)]
pub struct MaskingService {
    rules: Vec<MaskRule>,
    cache: LruCache<String, String>,
}

#[tonic::async_trait]
impl MaskingGrpc for MaskingService {
    async fn mask_text(
        &self,
        request: Request<MaskTextRequest>,
    ) -> Result<Response<MaskTextResponse>, Status> {
        let text = request.into_inner().text;
        
        // Check cache first
        if let Some(masked) = self.cache.get(&text) {
            return Ok(Response::new(MaskTextResponse {
                masked_text: masked.clone(),
                patterns_found: vec![],
            }));
        }
        
        // Apply masking rules
        let (masked_text, patterns) = self.apply_rules(&text)?;
        
        // Update cache
        self.cache.put(text, masked_text.clone());
        
        Ok(Response::new(MaskTextResponse {
            masked_text,
            patterns_found: patterns,
        }))
    }
}

// gRPC service definition
service MaskingService {
    rpc MaskText(MaskTextRequest) returns (MaskTextResponse);
    rpc MaskBatch(MaskBatchRequest) returns (MaskBatchResponse);
    rpc UpdateRules(UpdateRulesRequest) returns (UpdateRulesResponse);
}
```

**Características**:
- Caching inteligente com LRU
- Regras customizáveis por usuário
- Suporta processamento em batch
- Métricas de patterns detectados

### 3. **Search Service** 🔍
**Responsabilidade**: Busca híbrida (textual + semântica)

```rust
// search-service/src/main.rs
pub struct SearchService {
    text_engine: Box<dyn TextSearchEngine>,
    vector_engine: Box<dyn VectorSearchEngine>,
    fusion_strategy: Box<dyn FusionStrategy>,
}

#[tonic::async_trait]
impl SearchGrpc for SearchService {
    async fn search(
        &self,
        request: Request<SearchRequest>,
    ) -> Result<Response<SearchResponse>, Status> {
        let query = request.into_inner();
        
        // Parallel search execution
        let (text_results, vector_results) = tokio::join!(
            self.text_engine.search(&query),
            self.vector_engine.search(&query)
        );
        
        // Fusion with configurable strategy
        let combined_results = self.fusion_strategy.combine(
            text_results?,
            vector_results?,
            query.weights,
        )?;
        
        Ok(Response::new(SearchResponse {
            results: combined_results,
            total_count: results.len() as u32,
            search_time_ms: start.elapsed().as_millis() as u32,
        }))
    }
}

// Strategy pattern for fusion
pub trait FusionStrategy: Send + Sync {
    fn combine(
        &self,
        text_results: Vec<SearchResult>,
        vector_results: Vec<SearchResult>,
        weights: SearchWeights,
    ) -> Result<Vec<SearchResult>, Error>;
}

pub struct ReciprocRankFusion {
    k: f32,
}

impl FusionStrategy for ReciprocRankFusion {
    fn combine(&self, text_results: Vec<SearchResult>, vector_results: Vec<SearchResult>, weights: SearchWeights) -> Result<Vec<SearchResult>, Error> {
        // RRF implementation
    }
}
```

**Características**:
- Busca paralela em múltiplos backends
- Estratégias de fusão plugáveis
- Cache distribuído de resultados
- Suporte para faceted search

### 4. **Storage Service** 💾
**Responsabilidade**: Persistência e gestão de dados

```rust
// storage-service/src/main.rs
pub struct StorageService {
    write_pool: PgPool,
    read_pool: PgPool,
    event_store: Box<dyn EventStore>,
}

#[tonic::async_trait]
impl StorageGrpc for StorageService {
    async fn store_event(
        &self,
        request: Request<StoreEventRequest>,
    ) -> Result<Response<StoreEventResponse>, Status> {
        let event = request.into_inner();
        
        // Event sourcing pattern
        let stored_event = self.event_store.append(event).await?;
        
        // Update read models asynchronously
        tokio::spawn(async move {
            if let Err(e) = self.update_read_models(stored_event).await {
                error!("Failed to update read models: {}", e);
            }
        });
        
        Ok(Response::new(StoreEventResponse {
            event_id: stored_event.id.to_string(),
            timestamp: stored_event.timestamp,
        }))
    }
    
    async fn query_events(
        &self,
        request: Request<QueryEventsRequest>,
    ) -> Result<Response<stream::ReceiverStream<Result<Event, Status>>>, Status> {
        let query = request.into_inner();
        
        // Stream results for large queries
        let (tx, rx) = mpsc::channel(100);
        
        tokio::spawn(async move {
            let mut stream = self.read_pool.query_stream(&query).await?;
            
            while let Some(event) = stream.next().await {
                if tx.send(Ok(event)).await.is_err() {
                    break;
                }
            }
        });
        
        Ok(Response::new(ReceiverStream::new(rx)))
    }
}
```

**Características**:
- CQRS com event sourcing
- Read/write splitting
- Streaming para queries grandes
- Backup e recovery automáticos

### 5. **Analytics Service** 📊
**Responsabilidade**: Análise e insights

```rust
// analytics-service/src/main.rs
pub struct AnalyticsService {
    clickhouse: ClickHouseClient,
    ml_pipeline: Box<dyn MLPipeline>,
}

impl AnalyticsService {
    pub async fn process_event(&self, event: AnalyticsEvent) -> Result<(), Error> {
        // Real-time analytics
        self.update_counters(&event).await?;
        
        // Batch analytics
        self.clickhouse.insert_event(event.clone()).await?;
        
        // ML pipeline for patterns
        if let Some(pattern) = self.ml_pipeline.detect_pattern(&event).await? {
            self.publish_insight(pattern).await?;
        }
        
        Ok(())
    }
    
    pub async fn generate_report(&self, params: ReportParams) -> Result<Report, Error> {
        let data = self.clickhouse.query_aggregated(params).await?;
        
        Ok(Report {
            typing_speed: self.calculate_typing_speed(&data),
            common_words: self.extract_common_words(&data),
            productivity_score: self.calculate_productivity(&data),
            patterns: self.ml_pipeline.extract_patterns(&data).await?,
        })
    }
}
```

**Características**:
- Analytics em tempo real e batch
- Machine learning para detecção de padrões
- Geração de relatórios customizados
- Integração com ferramentas de BI

### 6. **API Gateway Service** 🌐
**Responsabilidade**: Roteamento, autenticação, rate limiting

```yaml
# kong.yml ou traefik.yml
services:
  - name: capture-service
    url: http://capture-service:50051
    routes:
      - paths: ["/api/v1/capture"]
        methods: ["POST"]
        plugins:
          - name: jwt
          - name: rate-limiting
            config:
              minute: 100
  
  - name: search-service
    url: http://search-service:50052
    routes:
      - paths: ["/api/v1/search"]
        methods: ["GET", "POST"]
        plugins:
          - name: jwt
          - name: response-cache
            config:
              ttl: 300
```

### 7. **Auth Service** 🔐
**Responsabilidade**: Autenticação e autorização

```rust
// auth-service/src/main.rs
pub struct AuthService {
    user_store: Box<dyn UserStore>,
    token_store: Box<dyn TokenStore>,
    permissions: Box<dyn PermissionEngine>,
}

#[tonic::async_trait]
impl AuthGrpc for AuthService {
    async fn authenticate(
        &self,
        request: Request<AuthRequest>,
    ) -> Result<Response<AuthResponse>, Status> {
        let creds = request.into_inner();
        
        // Multi-factor authentication
        let user = self.user_store.verify_credentials(creds).await?;
        
        if user.requires_mfa() {
            return Ok(Response::new(AuthResponse::mfa_required()));
        }
        
        // Generate JWT with permissions
        let permissions = self.permissions.get_user_permissions(&user).await?;
        let token = self.generate_jwt(user, permissions)?;
        
        Ok(Response::new(AuthResponse::success(token)))
    }
}
```

## 🛠️ Stack Tecnológico Recomendado

### Core Services
| Componente | Tecnologia | Justificativa |
|------------|------------|---------------|
| Linguagem | Rust | Performance, segurança, consistência |
| Framework | Tokio + Tonic | Async runtime + gRPC |
| Message Bus | NATS | Simplicidade e performance |
| API Gateway | Kong ou Traefik | Features enterprise-ready |

### Data Layer
| Componente | Tecnologia | Justificativa |
|------------|------------|---------------|
| Primary DB | PostgreSQL | ACID, JSON support |
| Time Series | QuestDB | Performance para eventos |
| Vector DB | Qdrant | Rust-native, performance |
| Cache | Redis | Standard da indústria |
| Search | MeiliSearch | Rust-based, fácil de usar |

### Infrastructure
| Componente | Tecnologia | Justificativa |
|------------|------------|---------------|
| Container | Docker | Padrão de mercado |
| Orchestration | Kubernetes | Escalabilidade |
| Service Mesh | Linkerd | Lightweight, Rust-based |
| Monitoring | Prometheus + Grafana | Padrão CNCF |
| Tracing | Jaeger | Distributed tracing |
| Logs | Loki | Integração com Grafana |

### Development
| Componente | Tecnologia | Justificativa |
|------------|------------|---------------|
| API Spec | OpenAPI 3.0 | Documentação automática |
| Testing | cargo-test + Postman | Unit + Integration |
| CI/CD | GitHub Actions | Já em uso |
| IaC | Terraform | Multi-cloud support |

## 📈 Plano de Migração

### Fase 1: Preparação (2-3 semanas)
1. **Extrair interfaces**
   ```rust
   // Criar traits para todos os componentes
   pub trait KeyboardCapture: Send + Sync {
       async fn capture(&self) -> Result<KeyEvent, Error>;
   }
   
   pub trait PiiMasker: Send + Sync {
       async fn mask(&self, text: &str) -> Result<String, Error>;
   }
   ```

2. **Implementar adaptadores**
   ```rust
   // Adapter para comunicação local ou remota
   pub enum TransportAdapter {
       Local(mpsc::Sender<Event>),
       Remote(NatsClient),
   }
   ```

3. **Adicionar feature flags**
   ```rust
   #[cfg(feature = "microservices")]
   let transport = TransportAdapter::Remote(nats_client);
   
   #[cfg(not(feature = "microservices"))]
   let transport = TransportAdapter::Local(channel);
   ```

### Fase 2: Implementação Core (4-6 semanas)
1. **Capture Service**
   - Extrair lógica de captura
   - Implementar publisher de eventos
   - Adicionar health checks

2. **Masking Service**
   - Criar serviço gRPC
   - Implementar cache distribuído
   - Adicionar métricas

3. **Storage Service**
   - Migrar de SQLite para PostgreSQL
   - Implementar event sourcing
   - Criar read models

### Fase 3: Serviços Avançados (4-6 semanas)
1. **Search Service**
   - Integrar MeiliSearch
   - Implementar vector search com Qdrant
   - Criar fusion strategies

2. **Analytics Service**
   - Setup QuestDB
   - Implementar pipelines de analytics
   - Criar dashboards

3. **API Gateway**
   - Configurar Kong/Traefik
   - Implementar rate limiting
   - Setup caching

### Fase 4: Migração de Dados (2-3 semanas)
1. **ETL Pipeline**
   ```rust
   pub struct MigrationPipeline {
       source: SqliteConnection,
       target: PostgresConnection,
       transformer: Box<dyn DataTransformer>,
   }
   ```

2. **Validação**
   - Comparar checksums
   - Verificar integridade
   - Testes de regressão

### Fase 5: Deployment (2-3 semanas)
1. **Local Development**
   ```yaml
   # docker-compose.yml
   version: '3.8'
   services:
     capture-service:
       build: ./services/capture
       environment:
         - NATS_URL=nats://nats:4222
     
     masking-service:
       build: ./services/masking
       ports:
         - "50051:50051"
   ```

2. **Kubernetes**
   ```yaml
   # k8s/capture-service.yaml
   apiVersion: apps/v1
   kind: Deployment
   metadata:
     name: capture-service
   spec:
     replicas: 3
     selector:
       matchLabels:
         app: capture-service
   ```

### Fase 6: Cutover (1-2 semanas)
1. **Blue-Green Deployment**
2. **Monitoramento intensivo**
3. **Rollback plan**

## 🔒 Considerações de Segurança

### 1. **Zero-Trust Architecture**
```rust
// Todos os serviços validam tokens
#[derive(Debug)]
pub struct SecurityMiddleware {
    jwt_validator: JwtValidator,
    permission_checker: PermissionChecker,
}

impl<S> Layer<S> for SecurityMiddleware {
    type Service = SecureService<S>;
    
    fn layer(&self, service: S) -> Self::Service {
        SecureService {
            inner: service,
            security: self.clone(),
        }
    }
}
```

### 2. **Encryption at Rest and in Transit**
- TLS 1.3 para toda comunicação
- Criptografia de dados sensíveis
- Key rotation automática

### 3. **Compliance**
- GDPR compliance com data residency
- Audit logs completos
- Right to be forgotten implementado

### 4. **Secret Management**
```yaml
# Usando Vault
apiVersion: v1
kind: Secret
metadata:
  name: db-credentials
  annotations:
    vault.hashicorp.com/agent-inject: "true"
    vault.hashicorp.com/role: "keyai-service"
```

## 📊 Monitoramento e Observabilidade

### 1. **Métricas (Prometheus)**
```rust
use prometheus::{Counter, Histogram, Registry};

lazy_static! {
    static ref EVENTS_CAPTURED: Counter = Counter::new(
        "keyai_events_captured_total", 
        "Total events captured"
    ).unwrap();
    
    static ref SEARCH_LATENCY: Histogram = Histogram::with_opts(
        HistogramOpts::new("keyai_search_duration_seconds", "Search latency")
            .buckets(vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0]),
    ).unwrap();
}
```

### 2. **Distributed Tracing (OpenTelemetry)**
```rust
use opentelemetry::{global, trace::Tracer};

#[instrument]
pub async fn process_search_request(query: SearchQuery) -> Result<SearchResults> {
    let tracer = global::tracer("search-service");
    let span = tracer.start("process_search");
    
    // Adicionar contexto
    span.set_attribute("query.length", query.text.len() as i64);
    span.set_attribute("query.type", query.search_type.to_string());
    
    let results = search_internal(query).await?;
    
    span.set_attribute("results.count", results.len() as i64);
    span.end();
    
    Ok(results)
}
```

### 3. **Logging Estruturado**
```rust
use tracing::{info, instrument};
use serde_json::json;

#[instrument(skip(sensitive_data))]
pub fn log_event(event_type: &str, metadata: Value) {
    info!(
        event_type = event_type,
        metadata = %metadata,
        service = "capture-service",
        version = env!("CARGO_PKG_VERSION"),
    );
}
```

### 4. **Dashboards Grafana**
```json
{
  "dashboard": {
    "title": "KeyAI Microservices",
    "panels": [
      {
        "title": "Events per Second",
        "targets": [{
          "expr": "rate(keyai_events_captured_total[5m])"
        }]
      },
      {
        "title": "Search Latency P95",
        "targets": [{
          "expr": "histogram_quantile(0.95, keyai_search_duration_seconds)"
        }]
      }
    ]
  }
}
```

### 5. **Alerting**
```yaml
# prometheus-alerts.yml
groups:
  - name: keyai-alerts
    rules:
      - alert: HighSearchLatency
        expr: histogram_quantile(0.95, keyai_search_duration_seconds) > 0.5
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High search latency detected"
          
      - alert: ServiceDown
        expr: up{job=~"keyai-.*"} == 0
        for: 1m
        labels:
          severity: critical
```

## 💰 Custos e Recursos

### Desenvolvimento (6 meses)
| Recurso | Quantidade | Custo Mensal | Total |
|---------|------------|---------------|-------|
| Rust Developers | 3 FTE | $10,000 | $180,000 |
| DevOps Engineer | 1 FTE | $9,000 | $54,000 |
| Architect | 0.5 FTE | $6,000 | $36,000 |
| **Total Dev** | | | **$270,000** |

### Infraestrutura (Mensal)
| Componente | Especificação | Custo |
|------------|---------------|-------|
| Kubernetes Cluster | 3 nodes (8 vCPU, 32GB) | $500 |
| PostgreSQL | Managed, HA | $300 |
| Redis | Managed cluster | $200 |
| QuestDB | Self-hosted | $150 |
| Qdrant | Managed | $400 |
| Monitoring | Grafana Cloud | $100 |
| **Total Infra** | | **$1,650/mês** |

### Comparação Local vs Cloud

#### Opção 1: Full Local (On-Premise)
```yaml
# docker-compose-local.yml
version: '3.8'
services:
  # Todos os serviços rodando localmente
  # Custo: $0/mês (após hardware)
  # Hardware recomendado: 16GB RAM, 8 cores
```

#### Opção 2: Hybrid
- Capture/Masking local
- Search/Storage/Analytics cloud
- Custo: ~$500/mês

#### Opção 3: Full Cloud
- Todos os serviços na nuvem
- Custo: ~$1,650/mês
- Benefícios: Sem manutenção local

## ⚠️ Riscos e Mitigações

### 1. **Complexidade Aumentada**
- **Risco**: Mais componentes para gerenciar
- **Mitigação**: 
  - Automação completa com IaC
  - Monitoramento robusto
  - Documentação detalhada

### 2. **Latência de Rede**
- **Risco**: Performance degradada vs monolito
- **Mitigação**:
  - Cache agressivo
  - Processamento assíncrono
  - Edge computing para capture

### 3. **Custos Operacionais**
- **Risco**: Custos maiores que monolito
- **Mitigação**:
  - Opção de deployment local
  - Auto-scaling baseado em uso
  - Otimização contínua

### 4. **Segurança Distribuída**
- **Risco**: Maior superfície de ataque
- **Mitigação**:
  - mTLS entre serviços
  - Service mesh (Linkerd)
  - Security scanning automatizado

### 5. **Migração de Dados**
- **Risco**: Perda ou corrupção de dados
- **Mitigação**:
  - Migração incremental
  - Validação em cada etapa
  - Rollback automatizado

## 🎯 Conclusão e Próximos Passos

### Benefícios da Migração

1. **Escalabilidade**: Cada componente escala independentemente
2. **Resiliência**: Falhas isoladas não afetam todo o sistema  
3. **Velocidade de Desenvolvimento**: Times podem trabalhar em paralelo
4. **Flexibilidade**: Suporta tanto cloud quanto on-premise
5. **Observabilidade**: Visibilidade completa do sistema

### Recomendações

1. **Começar com PoC**
   - Implementar Capture e Masking services
   - Validar performance e latência
   - Medir overhead vs benefícios

2. **Migração Gradual**
   - Manter compatibilidade com versão monolítica
   - Feature flags para rollout controlado
   - Métricas A/B entre arquiteturas

3. **Foco em Developer Experience**
   ```bash
   # Desenvolvimento local simplificado
   make dev-up        # Sobe todos os serviços
   make test-e2e      # Roda testes end-to-end
   make deploy-local  # Deploy local completo
   ```

4. **Documentação como Código**
   ```rust
   /// Capture Service API
   /// 
   /// Responsável por capturar eventos de teclado e publicá-los
   /// no message bus para processamento downstream.
   #[derive(OpenApi)]
   #[openapi(
       paths(capture_event, get_status),
       components(schemas(CaptureEvent, ServiceStatus))
   )]
   pub struct ApiDoc;
   ```

### Próximos Passos Imediatos

1. **Semana 1-2**: Revisão e aprovação da arquitetura
2. **Semana 3-4**: Setup do ambiente de desenvolvimento
3. **Semana 5-8**: Implementação do PoC (Capture + Masking)
4. **Semana 9-10**: Avaliação e decisão go/no-go
5. **Semana 11+**: Início da migração completa

### Métricas de Sucesso

- **Performance**: Latência P95 < 100ms para busca
- **Disponibilidade**: 99.9% uptime
- **Escalabilidade**: Suportar 10x crescimento sem refactor
- **Developer Velocity**: Deploy de novos features em < 1 dia
- **Custo**: TCO menor que 2x do monolito

---

## 📚 Apêndices

### A. Exemplo de Configuração Completa

```yaml
# config/services.yaml
services:
  capture:
    replicas: 2
    resources:
      requests:
        memory: "256Mi"
        cpu: "250m"
      limits:
        memory: "512Mi"
        cpu: "500m"
    env:
      - name: RUST_LOG
        value: "info,capture=debug"
      - name: NATS_URL
        value: "nats://nats:4222"
      
  masking:
    replicas: 3
    resources:
      requests:
        memory: "512Mi"
        cpu: "500m"
    autoscaling:
      enabled: true
      minReplicas: 3
      maxReplicas: 10
      targetCPUUtilizationPercentage: 70
```

### B. Estrutura de Diretórios Recomendada

```
keyai-microservices/
├── services/
│   ├── capture/
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   ├── Dockerfile
│   │   └── k8s/
│   ├── masking/
│   ├── search/
│   ├── storage/
│   └── analytics/
├── libraries/
│   ├── common/
│   ├── proto/
│   └── telemetry/
├── infrastructure/
│   ├── terraform/
│   ├── k8s/
│   └── docker/
├── scripts/
├── docs/
└── Makefile
```

### C. Ferramentas de Desenvolvimento

```makefile
# Makefile
.PHONY: dev test deploy

dev:
	docker-compose -f docker-compose.dev.yml up -d
	@echo "Services running at:"
	@echo "  - Capture: http://localhost:50051"
	@echo "  - Search: http://localhost:50052"
	@echo "  - Grafana: http://localhost:3000"

test:
	cargo test --workspace
	cargo clippy --workspace -- -D warnings
	./scripts/integration-tests.sh

deploy-local:
	kubectl apply -f k8s/local/

generate-proto:
	buf generate proto/

migrate-data:
	cargo run --bin migrator -- \
		--source sqlite://keyai.db \
		--target postgres://localhost/keyai
```

---

**Este documento serve como guia completo para a migração de uma arquitetura monolítica para microsserviços, mantendo o foco em performance, segurança e flexibilidade de deployment.** 