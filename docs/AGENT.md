# Módulo Agent - Captura de Teclas Avançada

## Visão Geral

O módulo **Agent** é responsável pela captura de eventos de teclado em tempo real, detecção de janelas ativas, filtragem inteligente e processamento assíncrono de dados. É o núcleo do sistema de keylogging do KeyAI Desktop.

## Arquitetura

### Componentes Principais

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   rdev Thread   │───▶│   Event Buffer   │───▶│  Process Thread │
│  (Key Capture)  │    │  (MPSC Channel)  │    │   (Filtering)   │
└─────────────────┘    └──────────────────┘    └─────────────────┘
                                                         │
┌─────────────────┐    ┌──────────────────┐            ▼
│  Window Thread  │───▶│  Window Buffer   │    ┌─────────────────┐
│ (Active Window) │    │  (MPSC Channel)  │    │   Masker +      │
└─────────────────┘    └──────────────────┘    │   Database      │
                                               └─────────────────┘
```

### Threads e Responsabilidades

1. **Thread Principal**: Controle do agente, configuração, métricas
2. **Thread rdev**: Captura de eventos de teclado (alta prioridade)
3. **Thread Processamento**: Filtragem, mascaramento e armazenamento
4. **Thread Janela**: Detecção de janela ativa (multiplataforma)
5. **Thread Métricas**: Coleta e atualização de estatísticas

## Estruturas de Dados

### AgentConfig

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub buffer_size: usize,                    // Tamanho do buffer de eventos
    pub flush_interval_secs: u64,              // Intervalo de flush para banco
    pub capture_modifiers: bool,               // Capturar teclas modificadoras
    pub capture_function_keys: bool,           // Capturar teclas de função
    pub capture_special_keys: bool,            // Capturar teclas especiais
    pub window_update_interval_ms: u64,        // Intervalo de detecção de janela
    pub metrics_update_interval_secs: u64,     // Intervalo de atualização de métricas
    pub ignored_applications: Vec<String>,     // Apps a ignorar
    pub ignored_window_patterns: Vec<String>,  // Padrões de janela a ignorar
    pub max_events_per_flush: usize,          // Máximo de eventos por flush
}
```

### KeyEvent

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyEvent {
    pub timestamp: u64,                        // Timestamp Unix em milissegundos
    pub key: String,                          // Representação da tecla
    pub event_type: String,                   // "press" ou "release"
    pub window_info: Option<WindowInfo>,      // Informações da janela ativa
    pub is_modifier: bool,                    // Se é tecla modificadora
    pub is_function_key: bool,                // Se é tecla de função
}
```

### WindowInfo

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowInfo {
    pub title: String,                        // Título da janela
    pub application: String,                  // Nome da aplicação
    pub process_id: Option<u32>,              // PID do processo
    pub timestamp: u64,                       // Quando foi detectada
}
```

## Funcionalidades

### 1. Captura de Teclas Multiplataforma

#### Windows
```rust
// Usa winapi para detecção de janela ativa
use winapi::um::{winuser, processthreadsapi, psapi};

fn get_active_window_windows() -> Result<WindowInfo> {
    let hwnd = winuser::GetForegroundWindow();
    // ... implementação específica do Windows
}
```

#### macOS
```rust
// Usa Core Foundation e Core Graphics
use core_foundation::*;
use core_graphics::*;

fn get_active_window_macos() -> Result<WindowInfo> {
    // ... implementação específica do macOS
}
```

#### Linux (X11)
```rust
// Usa X11 para detecção de janela
use x11::xlib::*;

fn get_active_window_linux() -> Result<WindowInfo> {
    // ... implementação específica do Linux
}
```

### 2. Sistema de Filtros

#### Filtros por Tipo de Tecla
```rust
impl Agent {
    fn should_filter_event(event: &KeyEvent, config: &AgentConfig) -> bool {
        // Filtrar modificadores
        if !config.capture_modifiers && event.is_modifier {
            return true;
        }
        
        // Filtrar teclas de função
        if !config.capture_function_keys && event.is_function_key {
            return true;
        }
        
        // Aplicar outros filtros...
        false
    }
}
```

#### Filtros por Aplicação
```rust
// Ignorar aplicações específicas
if let Some(window) = &event.window_info {
    if config.ignored_applications.iter()
        .any(|app| window.application.contains(app)) {
        return true;
    }
}
```

#### Filtros por Padrão de Janela
```rust
// Usar regex para filtrar por título de janela
for pattern in &config.ignored_window_patterns {
    if let Ok(regex) = Regex::new(pattern) {
        if regex.is_match(&window.title) {
            return true;
        }
    }
}
```

### 3. Métricas e Monitoramento

```rust
pub struct Metrics {
    events_captured: AtomicU64,     // Total de eventos capturados
    events_processed: AtomicU64,    // Total de eventos processados
    events_stored: AtomicU64,       // Total de eventos armazenados
    events_filtered: AtomicU64,     // Total de eventos filtrados
    window_updates: AtomicU64,      // Total de mudanças de janela
    uptime_seconds: AtomicU64,      // Tempo de execução
    last_event_timestamp: AtomicU64, // Timestamp do último evento
}
```

### 4. Gestão de Estado

```rust
#[derive(Debug)]
enum AgentState {
    Stopped,                        // Agente parado
    Starting,                       // Iniciando threads
    Running {                       // Executando
        capture_handle: JoinHandle<()>,
        process_handle: JoinHandle<()>,
        window_handle: JoinHandle<()>,
        metrics_handle: JoinHandle<()>,
    },
    Stopping,                       // Parando threads
}
```

## API Pública

### Métodos Principais

```rust
impl Agent {
    // Construção
    pub async fn new(masker: Masker, database: Arc<Database>) -> Result<Self>;
    pub async fn with_config(masker: Masker, database: Arc<Database>, config: AgentConfig) -> Result<Self>;
    
    // Controle de ciclo de vida
    pub async fn start(&mut self) -> Result<()>;
    pub async fn stop(&mut self) -> Result<()>;
    pub fn is_running(&self) -> bool;
    
    // Configuração
    pub async fn get_config(&self) -> AgentConfig;
    pub async fn update_config(&self, config: AgentConfig) -> Result<()>;
    
    // Informações de estado
    pub fn get_metrics(&self) -> HashMap<String, u64>;
    pub async fn get_current_window(&self) -> Option<WindowInfo>;
    
    // Métodos utilitários (estáticos)
    pub fn is_modifier_key(key: Key) -> bool;
    pub fn is_function_key(key: Key) -> bool;
    pub fn key_to_string(key: Key) -> String;
    pub fn should_filter_event(event: &KeyEvent, config: &AgentConfig) -> bool;
}
```

### Comandos Tauri Expostos

```rust
// Controle do agente
toggle_agent(enable: bool) -> Result<AgentStatus>
get_agent_status() -> Result<AgentStatus>

// Configuração
update_agent_config(config: AgentConfig) -> Result<AgentStatus>
get_agent_config() -> Result<AgentConfig>

// Informações
get_current_window() -> Result<Option<WindowInfo>>
get_agent_metrics() -> Result<HashMap<String, u64>>
```

## Configuração Padrão

```rust
impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            buffer_size: 1000,
            flush_interval_secs: 5,
            capture_modifiers: false,
            capture_function_keys: true,
            capture_special_keys: true,
            window_update_interval_ms: 500,
            metrics_update_interval_secs: 1,
            ignored_applications: vec![
                "keychain".to_string(),
                "1password".to_string(),
                "bitwarden".to_string(),
                "lastpass".to_string(),
            ],
            ignored_window_patterns: vec![
                r".*[Pp]assword.*".to_string(),
                r".*[Ll]ogin.*".to_string(),
                r".*[Ss]ecure.*".to_string(),
            ],
            max_events_per_flush: 500,
        }
    }
}
```

## Otimizações de Performance

### 1. Buffer Circular
- Uso de buffer circular para eventos
- Evita alocações desnecessárias
- Flush assíncrono em lotes

### 2. Detecção de Janela Eficiente
- Cache de informações de janela
- Update apenas quando necessário
- Throttling de consultas ao sistema

### 3. Processamento Assíncrono
- Threads separadas para captura e processamento
- Canais MPSC para comunicação eficiente
- Backpressure handling

### 4. Métricas Atômicas
- Contadores atômicos para thread safety
- Update lock-free
- Coleta periódica para reduzir overhead

## Tratamento de Erros

### Estratégias de Recuperação

```rust
// Reconexão automática em caso de falha
async fn handle_capture_error(&mut self, error: CaptureError) -> Result<()> {
    match error {
        CaptureError::DeviceDisconnected => {
            warn!("Dispositivo desconectado, tentando reconectar...");
            self.restart_capture().await?;
        },
        CaptureError::PermissionDenied => {
            error!("Permissão negada para captura de teclas");
            return Err(error.into());
        },
        CaptureError::SystemSuspend => {
            info!("Sistema suspenso, pausando captura");
            self.pause_capture().await?;
        },
    }
    Ok(())
}
```

### Logs Estruturados

```rust
// Logging detalhado para debug
trace!("Evento capturado: key={}, type={}", event.key, event.event_type);
debug!("Janela ativa mudou: {} -> {}", old_window, new_window);
info!("Agente iniciado com sucesso, PID: {}", process::id());
warn!("Buffer próximo do limite: {}/{}", current_size, max_size);
error!("Falha ao armazenar eventos: {}", error);
```

## Segurança e Privacidade

### 1. Filtragem de PII
- Integração com módulo Masker
- Filtragem antes do armazenamento
- Padrões configuráveis

### 2. Aplicações Sensíveis
- Lista de aplicações a ignorar
- Padrões de janela sensíveis
- Configuração por usuário

### 3. Criptografia
- Dados sempre criptografados no banco
- Chaves de criptografia seguras
- Sem dados em texto plano

## Testes

### Testes Unitários
```bash
cargo test agent::tests
```

### Testes de Integração
```bash
cargo test --test agent_integration_test
```

### Simulação de Eventos
```rust
// Usar rdev::simulate para testes
use rdev::{simulate, EventType, Key};

#[tokio::test]
async fn test_key_simulation() {
    let event = EventType::KeyPress(Key::KeyA);
    simulate(&event).unwrap();
    // ... verificar captura
}
```

## Benchmarks

### Performance de Captura
```bash
cargo bench capture_performance
```

### Throughput de Processamento
```bash
cargo bench processing_throughput
```

### Latência de Detecção de Janela
```bash
cargo bench window_detection_latency
```

## Limitações Conhecidas

### 1. Wayland (Linux)
- Não suportado na v1.0
- Requer privilégios especiais
- Limitações de segurança do protocolo

### 2. Sandboxing (macOS)
- Apps sandboxed podem ter limitações
- Requer permissões de acessibilidade
- Pode ser bloqueado por SIP

### 3. Antivírus (Windows)
- Pode ser detectado como keylogger
- Requer whitelisting
- Assinatura de código recomendada

## Roadmap

### v1.1
- [ ] Suporte para Wayland (experimental)
- [ ] Detecção de idle/away
- [ ] Compressão de eventos
- [ ] Cache de janelas melhorado

### v1.2
- [ ] Captura de mouse (opcional)
- [ ] Análise de padrões de digitação
- [ ] Exportação de heatmaps
- [ ] Integração com OCR

### v2.0
- [ ] Modo distribuído
- [ ] Sincronização em nuvem
- [ ] Machine learning para filtros
- [ ] API externa para integrações

## Contribuição

Para contribuir com o módulo Agent:

1. Leia o [CONTRIBUTING.md](../CONTRIBUTING.md)
2. Foque em testes e documentação
3. Mantenha compatibilidade multiplataforma
4. Priorize performance e segurança
5. Adicione benchmarks para mudanças críticas

## Referências

- [rdev Documentation](https://docs.rs/rdev/)
- [Tauri Event System](https://tauri.app/v1/guides/features/events)
- [Windows API Reference](https://docs.microsoft.com/en-us/windows/win32/api/)
- [macOS Accessibility Framework](https://developer.apple.com/accessibility/)
- [X11 Protocol Reference](https://www.x.org/releases/current/doc/) 