use std::sync::{Arc, atomic::{AtomicBool, AtomicU64, Ordering}};
use std::time::{SystemTime, UNIX_EPOCH, Duration, Instant};
use std::collections::HashMap;
use tokio::sync::{mpsc, RwLock};
use tokio::time::{sleep, interval};
use tracing::{info, warn, error, debug, trace};
use rdev::{listen, Event, EventType, Key};
use anyhow::{Result, anyhow};
use serde::{Serialize, Deserialize};

use crate::masker::Masker;
use crate::db::Database;

// Platform-specific imports
#[cfg(target_os = "windows")]
use std::ffi::OsString;
#[cfg(target_os = "windows")]
use std::os::windows::ffi::OsStringExt;
#[cfg(target_os = "windows")]
use winapi::um::{winuser, processthreadsapi, psapi, handleapi};
#[cfg(target_os = "windows")]
use winapi::shared::windef::HWND;

// macOS imports temporarily disabled

#[cfg(target_os = "linux")]
use x11::xlib::{Display, XOpenDisplay, XGetWindowProperty, XFree, XDefaultRootWindow};

/// Configurações do agente de captura
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Intervalo de flush do buffer em segundos
    pub flush_interval_secs: u64,
    /// Tamanho máximo do buffer antes do flush forçado
    pub buffer_size: usize,
    /// Capturar apenas teclas de texto (ignorar modificadores)
    pub text_keys_only: bool,
    /// Lista de aplicações a ignorar
    pub ignored_applications: Vec<String>,
    /// Lista de títulos de janela a ignorar (regex)
    pub ignored_window_patterns: Vec<String>,
    /// Ativar captura de modificadores (Ctrl, Alt, etc.)
    pub capture_modifiers: bool,
    /// Ativar captura de teclas de função (F1-F12)
    pub capture_function_keys: bool,
    /// Ativar detecção de janela ativa
    pub enable_window_detection: bool,
    /// Intervalo de atualização de janela ativa em ms
    pub window_update_interval_ms: u64,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            flush_interval_secs: 5,
            buffer_size: 100,
            text_keys_only: false,
            ignored_applications: vec![
                "keyai-desktop".to_string(),
                "password".to_string(),
                "1password".to_string(),
                "bitwarden".to_string(),
                "lastpass".to_string(),
            ],
            ignored_window_patterns: vec![
                r".*[Pp]assword.*".to_string(),
                r".*[Ll]ogin.*".to_string(),
                r".*[Ss]ecure.*".to_string(),
            ],
            capture_modifiers: true,
            capture_function_keys: true,
            enable_window_detection: true,
            window_update_interval_ms: 500,
        }
    }
}

/// Informações sobre a janela ativa
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowInfo {
    pub title: String,
    pub application: String,
    pub process_id: Option<u32>,
    pub timestamp: u64,
}

/// Evento de tecla capturado
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyEvent {
    pub timestamp: u64,
    pub key: String,
    pub event_type: String,
    pub window_info: Option<WindowInfo>,
    pub is_modifier: bool,
    pub is_function_key: bool,
}

/// Métricas do agente
#[derive(Debug, Default)]
pub struct AgentMetrics {
    pub events_captured: AtomicU64,
    pub events_processed: AtomicU64,
    pub events_discarded: AtomicU64,
    pub last_event_timestamp: AtomicU64,
    pub uptime_start: AtomicU64,
}

impl AgentMetrics {
    pub fn new() -> Self {
        Self {
            uptime_start: AtomicU64::new(0),
            ..Default::default()
        }
    }

    pub fn get_summary(&self) -> HashMap<String, u64> {
        let mut summary = HashMap::new();
        summary.insert("events_captured".to_string(), self.events_captured.load(Ordering::Relaxed));
        summary.insert("events_processed".to_string(), self.events_processed.load(Ordering::Relaxed));
        summary.insert("events_discarded".to_string(), self.events_discarded.load(Ordering::Relaxed));
        summary.insert("last_event_timestamp".to_string(), self.last_event_timestamp.load(Ordering::Relaxed));
        summary.insert("uptime_seconds".to_string(), self.uptime_start.load(Ordering::Relaxed) as u64);

        summary
    }
}

/// Agente principal de captura de teclas
pub struct Agent {
    config: Arc<RwLock<AgentConfig>>,
    masker: Masker,
    database: Arc<Database>,
    is_running: Arc<AtomicBool>,
    event_sender: Option<mpsc::UnboundedSender<KeyEvent>>,
    current_window: Arc<RwLock<Option<WindowInfo>>>,
    metrics: Arc<AgentMetrics>,
    shutdown_signal: Arc<AtomicBool>,
}

impl Agent {
    /// Cria uma nova instância do agente
    pub async fn new(masker: Masker, database: Arc<Database>) -> Result<Self> {
        Ok(Self {
            config: Arc::new(RwLock::new(AgentConfig::default())),
            masker,
            database,
            is_running: Arc::new(AtomicBool::new(false)),
            event_sender: None,
            current_window: Arc::new(RwLock::new(None)),
            metrics: Arc::new(AgentMetrics::new()),
            shutdown_signal: Arc::new(AtomicBool::new(false)),
        })
    }

    /// Cria uma nova instância com configuração customizada
    pub async fn with_config(masker: Masker, database: Arc<Database>, config: AgentConfig) -> Result<Self> {
        Ok(Self {
            config: Arc::new(RwLock::new(config)),
            masker,
            database,
            is_running: Arc::new(AtomicBool::new(false)),
            event_sender: None,
            current_window: Arc::new(RwLock::new(None)),
            metrics: Arc::new(AgentMetrics::new()),
            shutdown_signal: Arc::new(AtomicBool::new(false)),
        })
    }

    /// Verifica se as permissões necessárias estão disponíveis
    pub fn check_permissions() -> bool {
        #[cfg(target_os = "macos")]
        {
            info!("🔍 Verificando permissões de acessibilidade no macOS...");

            // Tentativa simples de verificar se temos permissões
            // Vamos tentar capturar um evento de teste
            let test_result = std::panic::catch_unwind(|| {
                // Teste rápido para ver se o rdev funciona
                let (tx, _rx) = std::sync::mpsc::channel();

                // Timeout muito curto para teste
                let start = std::time::Instant::now();
                std::thread::spawn(move || {
                    if let Err(_) = rdev::listen(move |event| {
                        let _ = tx.send(event);
                    }) {
                        // Erro esperado se não houver permissões
                    }
                });

                // Aguarda um pouco para ver se há erro imediato
                std::thread::sleep(std::time::Duration::from_millis(100));
                start.elapsed() < std::time::Duration::from_millis(200)
            });

            match test_result {
                Ok(_) => {
                    info!("✅ Permissões de acessibilidade parecem estar OK");
                    true
                },
                Err(_) => {
                    error!("❌ Erro ao verificar permissões - provavelmente sem acesso de acessibilidade");
                    Self::show_macos_permission_dialog();
                    false
                }
            }
        }

        #[cfg(not(target_os = "macos"))]
        {
            true
        }
    }

    /// Mostra diálogo de permissões para macOS
    #[cfg(target_os = "macos")]
    pub fn show_macos_permission_dialog() {
        error!("🚨 PERMISSÕES NECESSÁRIAS NO MACOS:");
        error!("   1. Vá para Configurações do Sistema > Privacidade e Segurança");
        error!("   2. Clique em 'Acessibilidade' na barra lateral");
        error!("   3. Adicione 'KeyAI Desktop' à lista de apps permitidos");
        error!("   4. Reinicie o aplicativo após conceder as permissões");

        // Tentar abrir as configurações automaticamente
        std::process::Command::new("open")
            .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
            .spawn()
            .unwrap_or_else(|_| {
                warn!("Não foi possível abrir as configurações automaticamente");
                std::process::Command::new("sh")
                    .arg("-c")
                    .arg("osascript -e 'tell app \"System Preferences\" to activate' -e 'tell app \"System Preferences\" to reveal anchor \"Privacy_Accessibility\" of pane id \"com.apple.preference.security\"'")
                    .spawn()
                    .unwrap_or_else(|_| {
                        error!("Abra manualmente: Configurações do Sistema > Privacidade e Segurança > Acessibilidade");
                        std::process::Command::new("echo").spawn().unwrap()
                    })
            });
    }

    #[cfg(not(target_os = "macos"))]
    pub fn show_macos_permission_dialog() {
        // Não faz nada em outras plataformas
    }

    /// Inicia o agente de captura
    pub async fn start(&mut self) -> Result<()> {
        if self.is_running.load(Ordering::Relaxed) {
            warn!("⚠️ Agente já está em execução");
            return Ok(());
        }

        info!("🎯 Iniciando agente de captura de teclas...");

        // Check permissions first
        if !Self::check_permissions() {
            return Err(anyhow!("Permissões insuficientes para captura de teclas"));
        }

        // Reset shutdown signal
        self.shutdown_signal.store(false, Ordering::Relaxed);

        // Set uptime start
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.metrics.uptime_start.store(now, Ordering::Relaxed);

        // Create channel for key events
        let (tx, rx) = mpsc::unbounded_channel::<KeyEvent>();
        self.event_sender = Some(tx.clone());

        // Start event processing task
        self.start_event_processor(rx).await?;

        // Start window detection task if enabled
        let config = self.config.read().await;
        if config.enable_window_detection {
            self.start_window_detector().await?;
        }
        drop(config);

        // Start metrics reporter
        self.start_metrics_reporter().await?;

        // Start key listener (with special handling for macOS)
        match self.start_key_listener(tx).await {
            Ok(()) => {
                info!("✅ Listener de teclas iniciado");
                #[cfg(target_os = "macos")]
                {
                    info!("🍎 Para macOS: Se não houver captura de teclas, verifique as permissões de Acessibilidade");
                }
            }
            Err(e) => {
                error!("❌ Falha ao iniciar listener de teclas: {}", e);
                #[cfg(target_os = "macos")]
                {
                    Self::show_macos_permission_dialog();
                }
                // Continue mesmo sem o listener - outros componentes podem funcionar
                warn!("🔧 Aplicação continuará em modo degradado sem captura de teclas");
            }
        }

        // Mark as running after successful initialization
        self.is_running.store(true, Ordering::Relaxed);

        info!("✅ Agente de captura iniciado com sucesso");
        Ok(())
    }

    /// Para o agente de captura
    pub async fn stop(&mut self) -> Result<()> {
        if !self.is_running.load(Ordering::Relaxed) {
            warn!("⚠️ Agente não está em execução");
            return Ok(());
        }

        info!("🛑 Parando agente de captura...");

        // Signal shutdown to all tasks
        self.shutdown_signal.store(true, Ordering::Relaxed);
        self.is_running.store(false, Ordering::Relaxed);
        self.event_sender = None;

        // Give tasks time to shutdown gracefully
        sleep(Duration::from_millis(100)).await;

        info!("✅ Agente parado com sucesso");
        Ok(())
    }

    /// Verifica se o agente está em execução
    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::Relaxed)
    }

    /// Obtém as métricas atuais
    pub fn get_metrics(&self) -> HashMap<String, u64> {
        self.metrics.get_summary()
    }

    /// Atualiza a configuração do agente
    pub async fn update_config(&self, new_config: AgentConfig) -> Result<()> {
        let mut config = self.config.write().await;
        *config = new_config;
        info!("🔧 Configuração do agente atualizada");
        Ok(())
    }

    /// Obtém a configuração atual
    pub async fn get_config(&self) -> AgentConfig {
        self.config.read().await.clone()
    }

    /// Obtém informações da janela ativa atual
    pub async fn get_current_window(&self) -> Option<WindowInfo> {
        self.current_window.read().await.clone()
    }

    /// Inicia o processador de eventos
    async fn start_event_processor(&self, mut rx: mpsc::UnboundedReceiver<KeyEvent>) -> Result<()> {
        let masker = self.masker.clone();
        let database = self.database.clone();
        let config = self.config.clone();
        let metrics = self.metrics.clone();
        let shutdown_signal = self.shutdown_signal.clone();

        tokio::spawn(async move {
            let mut buffer = Vec::new();
            let mut last_flush = Instant::now();

            while !shutdown_signal.load(Ordering::Relaxed) {
                // Try to receive events with timeout
                match tokio::time::timeout(Duration::from_millis(100), rx.recv()).await {
                    Ok(Some(event)) => {
                        trace!("📝 Evento recebido: {:?}", event);
                        metrics.events_captured.fetch_add(1, Ordering::Relaxed);

                        // Check if event should be filtered
                        let config_guard = config.read().await;
                        if Self::should_filter_event(&event, &config_guard) {
                            metrics.events_discarded.fetch_add(1, Ordering::Relaxed);
                            continue;
                        }
                        drop(config_guard);

                        // Apply PII masking
                        let masked_event = masker.mask_event(event);
                        buffer.push(masked_event);
                        metrics.events_processed.fetch_add(1, Ordering::Relaxed);

                        // Check if we need to flush
                        let config_guard = config.read().await;
                        let should_flush = buffer.len() >= config_guard.buffer_size ||
                                         last_flush.elapsed() >= Duration::from_secs(config_guard.flush_interval_secs);
                        drop(config_guard);

                        if should_flush {
                            Self::flush_events(&database, &mut buffer, &metrics).await;
                            last_flush = Instant::now();
                        }
                    }
                    Ok(None) => break, // Channel closed
                    Err(_) => continue, // Timeout, check shutdown signal
                }
            }

            // Flush remaining events on shutdown
            if !buffer.is_empty() {
                Self::flush_events(&database, &mut buffer, &metrics).await;
            }

            info!("🔄 Processador de eventos finalizado");
        });

        Ok(())
    }

    /// Inicia o detector de janelas
    async fn start_window_detector(&self) -> Result<()> {
        let current_window = self.current_window.clone();
        let config = self.config.clone();
        let metrics = self.metrics.clone();
        let shutdown_signal = self.shutdown_signal.clone();

        tokio::spawn(async move {
            let mut interval_timer = {
                let config_guard = config.read().await;
                interval(Duration::from_millis(config_guard.window_update_interval_ms))
            };

            while !shutdown_signal.load(Ordering::Relaxed) {
                interval_timer.tick().await;

                if let Some(window_info) = Self::get_active_window_info() {
                    let mut current = current_window.write().await;

                    // Only update if window changed
                    let should_update = match &*current {
                        Some(current_info) => {
                            current_info.title != window_info.title ||
                            current_info.application != window_info.application
                        }
                        None => true,
                    };

                    if should_update {
                        debug!("🪟 Janela ativa: {} - {}", window_info.application, window_info.title);
                        *current = Some(window_info);
                        metrics.events_discarded.fetch_add(1, Ordering::Relaxed);
                    }
                }
            }

            info!("🪟 Detector de janelas finalizado");
        });

        Ok(())
    }

    /// Inicia o listener de teclas com timeout e tratamento robusto para macOS
    async fn start_key_listener(&self, tx: mpsc::UnboundedSender<KeyEvent>) -> Result<()> {
        let current_window = self.current_window.clone();
        let shutdown_signal = self.shutdown_signal.clone();

                // Para macOS, usamos uma abordagem mais cautelosa
        #[cfg(target_os = "macos")]
        {
            info!("🎯 Iniciando listener de teclas para macOS...");

            // Simplificado para evitar problemas com catch_unwind
            std::thread::spawn(move || {
                info!("🔍 Tentando iniciar captura de teclas no macOS...");

                match listen(move |event| {
                    if shutdown_signal.load(Ordering::Relaxed) {
                        return;
                    }

                    if let Err(e) = Self::handle_rdev_event(event, &tx, &current_window) {
                        error!("❌ Erro ao processar evento: {}", e);
                    }
                }) {
                    Ok(()) => {
                        info!("✅ Listener de teclas macOS finalizado normalmente");
                    }
                    Err(e) => {
                        error!("❌ Erro no listener de teclas macOS: {:?}", e);
                        error!("🚨 PERMISSÕES NECESSÁRIAS NO MACOS:");
                        error!("   1. Vá para Configurações do Sistema > Privacidade e Segurança");
                        error!("   2. Clique em 'Acessibilidade' na barra lateral");
                        error!("   3. Adicione 'Terminal' ou 'KeyAI Desktop' à lista de apps permitidos");
                        error!("   4. Reinicie o aplicativo após conceder as permissões");

                        // Tentar abrir as configurações
                        let _ = std::process::Command::new("open")
                            .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
                            .spawn();
                    }
                }
            });
        }

        // Para outras plataformas, usa a implementação original
        #[cfg(not(target_os = "macos"))]
        {
            std::thread::spawn(move || {
                info!("🎯 Iniciando thread de captura de teclas...");

                match listen(move |event| {
                    if shutdown_signal.load(Ordering::Relaxed) {
                        debug!("🛑 Sinal de shutdown recebido, parando listener");
                        return;
                    }

                    if let Err(e) = Self::handle_rdev_event(event, &tx, &current_window) {
                        error!("❌ Erro ao processar evento: {}", e);
                    }
                }) {
                    Ok(()) => {
                        info!("✅ Listener de teclas finalizado normalmente");
                    }
                    Err(e) => {
                        error!("❌ Erro no listener de teclas: {:?}", e);
                    }
                }
            });
        }

        Ok(())
    }

    /// Inicia o reporter de métricas
    async fn start_metrics_reporter(&self) -> Result<()> {
        let metrics = self.metrics.clone();
        let shutdown_signal = self.shutdown_signal.clone();

        tokio::spawn(async move {
            let mut interval_timer = interval(Duration::from_secs(60)); // Report every minute

            while !shutdown_signal.load(Ordering::Relaxed) {
                interval_timer.tick().await;

                let summary = metrics.get_summary();
                info!("📊 Métricas do Agente: {:?}", summary);
            }
        });

        Ok(())
    }

    /// Processa evento do rdev
    fn handle_rdev_event(
        event: Event,
        sender: &mpsc::UnboundedSender<KeyEvent>,
        current_window: &Arc<RwLock<Option<WindowInfo>>>
    ) -> Result<()> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| anyhow!("Erro ao obter timestamp: {}", e))?
            .as_secs();

        match event.event_type {
            EventType::KeyPress(key) | EventType::KeyRelease(key) => {
                let key_str = Self::key_to_string(key);
                let event_type = match event.event_type {
                    EventType::KeyPress(_) => "press",
                    EventType::KeyRelease(_) => "release",
                    _ => unreachable!(),
                };

                // Get current window info (non-blocking)
                let window_info = {
                    if let Ok(guard) = current_window.try_read() {
                        guard.clone()
                    } else {
                        None
                    }
                };

                let key_event = KeyEvent {
                    timestamp,
                    key: key_str,
                    event_type: event_type.to_string(),
                    window_info,
                    is_modifier: Self::is_modifier_key(key),
                    is_function_key: Self::is_function_key(key),
                };

                if let Err(e) = sender.send(key_event) {
                    error!("❌ Erro ao enviar evento: {}", e);
                    return Err(anyhow!("Erro ao enviar evento: {}", e));
                }
            }
            _ => {} // Ignore other event types
        }

        Ok(())
    }

    /// Verifica se o evento deve ser filtrado
    fn should_filter_event(event: &KeyEvent, config: &AgentConfig) -> bool {
        // Filter modifiers if not enabled
        if event.is_modifier && !config.capture_modifiers {
            return true;
        }

        // Filter function keys if not enabled
        if event.is_function_key && !config.capture_function_keys {
            return true;
        }

        // Filter by application
        if let Some(window_info) = &event.window_info {
            if config.ignored_applications.iter().any(|app| {
                window_info.application.to_lowercase().contains(&app.to_lowercase())
            }) {
                return true;
            }

            // Filter by window title patterns
            for pattern in &config.ignored_window_patterns {
                if let Ok(regex) = regex::Regex::new(pattern) {
                    if regex.is_match(&window_info.title) {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Flush eventos para o banco de dados
    async fn flush_events(
        database: &Arc<Database>,
        buffer: &mut Vec<KeyEvent>,
        metrics: &Arc<AgentMetrics>
    ) {
        if buffer.is_empty() {
            return;
        }

        match database.store_events(buffer).await {
            Ok(_) => {
                let count = buffer.len();
                metrics.events_processed.fetch_add(count as u64, Ordering::Relaxed);
                debug!("✅ {} eventos armazenados", count);
            }
            Err(e) => {
                error!("❌ Erro ao armazenar eventos: {}", e);
                metrics.events_discarded.fetch_add(buffer.len() as u64, Ordering::Relaxed);
            }
        }

        buffer.clear();
    }

    /// Converte Key do rdev para string
    pub fn key_to_string(key: Key) -> String {
        match key {
            // Modifier keys
            Key::Alt => "Alt".to_string(),
            Key::AltGr => "AltGr".to_string(),
            Key::ControlLeft => "CtrlLeft".to_string(),
            Key::ControlRight => "CtrlRight".to_string(),
            Key::ShiftLeft => "ShiftLeft".to_string(),
            Key::ShiftRight => "ShiftRight".to_string(),
            Key::MetaLeft => "MetaLeft".to_string(),
            Key::MetaRight => "MetaRight".to_string(),

            // Function keys
            Key::F1 => "F1".to_string(),
            Key::F2 => "F2".to_string(),
            Key::F3 => "F3".to_string(),
            Key::F4 => "F4".to_string(),
            Key::F5 => "F5".to_string(),
            Key::F6 => "F6".to_string(),
            Key::F7 => "F7".to_string(),
            Key::F8 => "F8".to_string(),
            Key::F9 => "F9".to_string(),
            Key::F10 => "F10".to_string(),
            Key::F11 => "F11".to_string(),
            Key::F12 => "F12".to_string(),

            // Special keys
            Key::Backspace => "Backspace".to_string(),
            Key::CapsLock => "CapsLock".to_string(),
            Key::Delete => "Delete".to_string(),
            Key::DownArrow => "DownArrow".to_string(),
            Key::End => "End".to_string(),
            Key::Escape => "Escape".to_string(),
            Key::Home => "Home".to_string(),
            Key::LeftArrow => "LeftArrow".to_string(),
            Key::PageDown => "PageDown".to_string(),
            Key::PageUp => "PageUp".to_string(),
            Key::Return => "Return".to_string(),
            Key::RightArrow => "RightArrow".to_string(),
            Key::Space => "Space".to_string(),
            Key::Tab => "Tab".to_string(),
            Key::UpArrow => "UpArrow".to_string(),
            Key::PrintScreen => "PrintScreen".to_string(),
            Key::ScrollLock => "ScrollLock".to_string(),
            Key::Pause => "Pause".to_string(),
            Key::NumLock => "NumLock".to_string(),
            Key::Insert => "Insert".to_string(),

            // Number row
            Key::BackQuote => "`".to_string(),
            Key::Num1 => "1".to_string(),
            Key::Num2 => "2".to_string(),
            Key::Num3 => "3".to_string(),
            Key::Num4 => "4".to_string(),
            Key::Num5 => "5".to_string(),
            Key::Num6 => "6".to_string(),
            Key::Num7 => "7".to_string(),
            Key::Num8 => "8".to_string(),
            Key::Num9 => "9".to_string(),
            Key::Num0 => "0".to_string(),
            Key::Minus => "-".to_string(),
            Key::Equal => "=".to_string(),

            // Letters
            Key::KeyQ => "q".to_string(),
            Key::KeyW => "w".to_string(),
            Key::KeyE => "e".to_string(),
            Key::KeyR => "r".to_string(),
            Key::KeyT => "t".to_string(),
            Key::KeyY => "y".to_string(),
            Key::KeyU => "u".to_string(),
            Key::KeyI => "i".to_string(),
            Key::KeyO => "o".to_string(),
            Key::KeyP => "p".to_string(),
            Key::LeftBracket => "[".to_string(),
            Key::RightBracket => "]".to_string(),
            Key::KeyA => "a".to_string(),
            Key::KeyS => "s".to_string(),
            Key::KeyD => "d".to_string(),
            Key::KeyF => "f".to_string(),
            Key::KeyG => "g".to_string(),
            Key::KeyH => "h".to_string(),
            Key::KeyJ => "j".to_string(),
            Key::KeyK => "k".to_string(),
            Key::KeyL => "l".to_string(),
            Key::SemiColon => ";".to_string(),
            Key::Quote => "'".to_string(),
            Key::BackSlash => "\\".to_string(),
            Key::IntlBackslash => "\\".to_string(),
            Key::KeyZ => "z".to_string(),
            Key::KeyX => "x".to_string(),
            Key::KeyC => "c".to_string(),
            Key::KeyV => "v".to_string(),
            Key::KeyB => "b".to_string(),
            Key::KeyN => "n".to_string(),
            Key::KeyM => "m".to_string(),
            Key::Comma => ",".to_string(),
            Key::Dot => ".".to_string(),
            Key::Slash => "/".to_string(),

            // Keypad
            Key::KpReturn => "KpReturn".to_string(),
            Key::KpMinus => "Kp-".to_string(),
            Key::KpPlus => "Kp+".to_string(),
            Key::KpMultiply => "Kp*".to_string(),
            Key::KpDivide => "Kp/".to_string(),
            Key::Kp0 => "Kp0".to_string(),
            Key::Kp1 => "Kp1".to_string(),
            Key::Kp2 => "Kp2".to_string(),
            Key::Kp3 => "Kp3".to_string(),
            Key::Kp4 => "Kp4".to_string(),
            Key::Kp5 => "Kp5".to_string(),
            Key::Kp6 => "Kp6".to_string(),
            Key::Kp7 => "Kp7".to_string(),
            Key::Kp8 => "Kp8".to_string(),
            Key::Kp9 => "Kp9".to_string(),
            Key::KpDelete => "KpDelete".to_string(),

            // Other
            Key::Function => "Function".to_string(),
            Key::Unknown(code) => format!("Unknown({})", code),
        }
    }

    /// Verifica se é uma tecla modificadora
    fn is_modifier_key(key: Key) -> bool {
        matches!(key,
            Key::Alt | Key::AltGr |
            Key::ControlLeft | Key::ControlRight |
            Key::ShiftLeft | Key::ShiftRight |
            Key::MetaLeft | Key::MetaRight |
            Key::CapsLock | Key::NumLock | Key::ScrollLock
        )
    }

    /// Verifica se é uma tecla de função
    fn is_function_key(key: Key) -> bool {
        matches!(key,
            Key::F1 | Key::F2 | Key::F3 | Key::F4 | Key::F5 | Key::F6 |
            Key::F7 | Key::F8 | Key::F9 | Key::F10 | Key::F11 | Key::F12
        )
    }

    /// Obtém informações da janela ativa (multiplataforma)
    fn get_active_window_info() -> Option<WindowInfo> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .ok()?
            .as_secs();

        #[cfg(target_os = "windows")]
        {
            Self::get_windows_active_window(timestamp)
        }

        #[cfg(target_os = "macos")]
        {
            Self::get_macos_active_window(timestamp)
        }

        #[cfg(target_os = "linux")]
        {
            Self::get_linux_active_window(timestamp)
        }

        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            None
        }
    }

    // Platform-specific implementations

    #[cfg(target_os = "windows")]
    fn get_windows_active_window(timestamp: u64) -> Option<WindowInfo> {
        use std::ptr;

        unsafe {
            let hwnd = winuser::GetForegroundWindow();
            if hwnd.is_null() {
                return None;
            }

            // Get window title
            let mut title_buffer = [0u16; 512];
            let title_len = winuser::GetWindowTextW(hwnd, title_buffer.as_mut_ptr(), title_buffer.len() as i32);
            let title = if title_len > 0 {
                String::from_utf16_lossy(&title_buffer[..title_len as usize])
            } else {
                "Unknown".to_string()
            };

            // Get process ID
            let mut process_id: u32 = 0;
            winuser::GetWindowThreadProcessId(hwnd, &mut process_id);

            // Get process name
            let process_handle = processthreadsapi::OpenProcess(
                psapi::PROCESS_QUERY_INFORMATION | psapi::PROCESS_VM_READ,
                0,
                process_id,
            );

            let application = if !process_handle.is_null() {
                let mut module_name = [0u16; 512];
                let name_len = psapi::GetModuleBaseNameW(
                    process_handle,
                    ptr::null_mut(),
                    module_name.as_mut_ptr(),
                    module_name.len() as u32,
                );

                handleapi::CloseHandle(process_handle);

                if name_len > 0 {
                    String::from_utf16_lossy(&module_name[..name_len as usize])
                } else {
                    "Unknown".to_string()
                }
            } else {
                "Unknown".to_string()
            };

            Some(WindowInfo {
                title,
                application,
                process_id: Some(process_id),
                timestamp,
            })
        }
    }

    #[cfg(target_os = "macos")]
    fn get_macos_active_window(timestamp: u64) -> Option<WindowInfo> {
        // Implementação simplificada para evitar crashes
        warn!("🚧 Detecção de janela ativa no macOS em modo simplificado");
        Some(WindowInfo {
            title: "macOS Window".to_string(),
            application: "Unknown App".to_string(),
            process_id: None,
            timestamp,
        })
    }

    #[cfg(target_os = "linux")]
    fn get_linux_active_window(timestamp: u64) -> Option<WindowInfo> {
        use std::ffi::CStr;
        use std::ptr;

        unsafe {
            let display = XOpenDisplay(ptr::null());
            if display.is_null() {
                return None;
            }

            let root = XDefaultRootWindow(display);

            // Get active window
            let mut active_window = 0;
            let mut actual_type = 0;
            let mut actual_format = 0;
            let mut nitems = 0;
            let mut bytes_after = 0;
            let mut prop_data: *mut u8 = ptr::null_mut();

            let net_active_window = x11::xlib::XInternAtom(
                display,
                b"_NET_ACTIVE_WINDOW\0".as_ptr() as *const i8,
                0,
            );

            let result = XGetWindowProperty(
                display,
                root,
                net_active_window,
                0,
                1,
                0,
                x11::xlib::XA_WINDOW,
                &mut actual_type,
                &mut actual_format,
                &mut nitems,
                &mut bytes_after,
                &mut prop_data,
            );

            if result == 0 && !prop_data.is_null() && nitems > 0 {
                active_window = *(prop_data as *const u64);
                x11::xlib::XFree(prop_data as *mut _);
            }

            if active_window == 0 {
                x11::xlib::XCloseDisplay(display);
                return None;
            }

            // Get window title
            let mut title_data: *mut u8 = ptr::null_mut();
            let net_wm_name = x11::xlib::XInternAtom(
                display,
                b"_NET_WM_NAME\0".as_ptr() as *const i8,
                0,
            );

            let title = if XGetWindowProperty(
                display,
                active_window,
                net_wm_name,
                0,
                1024,
                0,
                x11::xlib::XInternAtom(display, b"UTF8_STRING\0".as_ptr() as *const i8, 0),
                &mut actual_type,
                &mut actual_format,
                &mut nitems,
                &mut bytes_after,
                &mut title_data,
            ) == 0 && !title_data.is_null() {
                let title_str = CStr::from_ptr(title_data as *const i8)
                    .to_string_lossy()
                    .to_string();
                x11::xlib::XFree(title_data as *mut _);
                title_str
            } else {
                "Unknown".to_string()
            };

            // Get application name
            let mut class_data: *mut u8 = ptr::null_mut();
            let wm_class = x11::xlib::XInternAtom(
                display,
                b"WM_CLASS\0".as_ptr() as *const i8,
                0,
            );

            let application = if XGetWindowProperty(
                display,
                active_window,
                wm_class,
                0,
                1024,
                0,
                x11::xlib::XA_STRING,
                &mut actual_type,
                &mut actual_format,
                &mut nitems,
                &mut bytes_after,
                &mut class_data,
            ) == 0 && !class_data.is_null() {
                let class_str = CStr::from_ptr(class_data as *const i8)
                    .to_string_lossy()
                    .to_string();
                x11::xlib::XFree(class_data as *mut _);
                class_str
            } else {
                "Unknown".to_string()
            };

            x11::xlib::XCloseDisplay(display);

            Some(WindowInfo {
                title,
                application,
                process_id: None, // PID detection on Linux requires additional work
                timestamp,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::mpsc;
    use tempfile::NamedTempFile;

    async fn create_test_database() -> Result<Arc<Database>> {
        let temp_file = NamedTempFile::new().unwrap();
        Ok(Arc::new(Database::new(temp_file.path()).await?))
    }

    #[test]
    fn test_key_to_string() {
        assert_eq!(Agent::key_to_string(Key::KeyA), "a");
        assert_eq!(Agent::key_to_string(Key::Space), "Space");
        assert_eq!(Agent::key_to_string(Key::Return), "Return");
        assert_eq!(Agent::key_to_string(Key::Num1), "1");
        assert_eq!(Agent::key_to_string(Key::Unknown(999)), "Unknown(999)");
    }

    #[test]
    fn test_is_modifier_key() {
        assert!(Agent::is_modifier_key(Key::ControlLeft));
        assert!(Agent::is_modifier_key(Key::Alt));
        assert!(Agent::is_modifier_key(Key::ShiftRight));
        assert!(!Agent::is_modifier_key(Key::KeyA));
        assert!(!Agent::is_modifier_key(Key::Space));
    }

    #[test]
    fn test_is_function_key() {
        assert!(Agent::is_function_key(Key::F1));
        assert!(Agent::is_function_key(Key::F12));
        assert!(!Agent::is_function_key(Key::KeyA));
        assert!(!Agent::is_function_key(Key::Space));
    }

    #[tokio::test]
    async fn test_agent_creation() {
        let masker = Masker::new();
        let database = create_test_database().await.unwrap();

        let agent = Agent::new(masker, database).await.unwrap();
        assert!(!agent.is_running());
    }

    #[tokio::test]
    async fn test_agent_with_config() {
        let masker = Masker::new();
        let database = create_test_database().await.unwrap();
        let config = AgentConfig {
            buffer_size: 50,
            capture_modifiers: false,
            ..Default::default()
        };

        let agent = Agent::with_config(masker, database, config.clone()).await.unwrap();
        let retrieved_config = agent.get_config().await;

        assert_eq!(retrieved_config.buffer_size, 50);
        assert!(!retrieved_config.capture_modifiers);
    }

    #[tokio::test]
    async fn test_agent_start_stop() {
        let masker = Masker::new();
        let database = create_test_database().await.unwrap();

        let mut agent = Agent::new(masker, database).await.unwrap();

        // Test starting agent
        assert!(!agent.is_running());
        agent.start().await.unwrap();
        assert!(agent.is_running());

        // Test starting already running agent
        let result = agent.start().await;
        assert!(result.is_ok()); // Should not error, just warn

        // Test stopping agent
        agent.stop().await.unwrap();
        assert!(!agent.is_running());

        // Test stopping already stopped agent
        let result = agent.stop().await;
        assert!(result.is_ok()); // Should not error, just warn
    }

    #[test]
    fn test_should_filter_event() {
        let config = AgentConfig {
            capture_modifiers: false,
            capture_function_keys: false,
            ignored_applications: vec!["password".to_string()],
            ignored_window_patterns: vec![r".*[Pp]assword.*".to_string()],
            ..Default::default()
        };

        // Test modifier filtering
        let modifier_event = KeyEvent {
            timestamp: 0,
            key: "CtrlLeft".to_string(),
            event_type: "press".to_string(),
            window_info: None,
            is_modifier: true,
            is_function_key: false,
        };
        assert!(Agent::should_filter_event(&modifier_event, &config));

        // Test function key filtering
        let function_event = KeyEvent {
            timestamp: 0,
            key: "F1".to_string(),
            event_type: "press".to_string(),
            window_info: None,
            is_modifier: false,
            is_function_key: true,
        };
        assert!(Agent::should_filter_event(&function_event, &config));

        // Test application filtering
        let app_event = KeyEvent {
            timestamp: 0,
            key: "a".to_string(),
            event_type: "press".to_string(),
            window_info: Some(WindowInfo {
                title: "Login".to_string(),
                application: "password-manager".to_string(),
                process_id: None,
                timestamp: 0,
            }),
            is_modifier: false,
            is_function_key: false,
        };
        assert!(Agent::should_filter_event(&app_event, &config));

        // Test window pattern filtering
        let window_event = KeyEvent {
            timestamp: 0,
            key: "a".to_string(),
            event_type: "press".to_string(),
            window_info: Some(WindowInfo {
                title: "Password Entry".to_string(),
                application: "browser".to_string(),
                process_id: None,
                timestamp: 0,
            }),
            is_modifier: false,
            is_function_key: false,
        };
        assert!(Agent::should_filter_event(&window_event, &config));

        // Test normal event (should not be filtered)
        let normal_event = KeyEvent {
            timestamp: 0,
            key: "a".to_string(),
            event_type: "press".to_string(),
            window_info: Some(WindowInfo {
                title: "Document".to_string(),
                application: "editor".to_string(),
                process_id: None,
                timestamp: 0,
            }),
            is_modifier: false,
            is_function_key: false,
        };
        assert!(!Agent::should_filter_event(&normal_event, &config));
    }

    #[tokio::test]
    async fn test_metrics() {
        let masker = Masker::new();
        let database = create_test_database().await.unwrap();

        let agent = Agent::new(masker, database).await.unwrap();
        let metrics = agent.get_metrics();

        assert_eq!(metrics.get("events_captured").unwrap_or(&0), &0);
        assert_eq!(metrics.get("events_processed").unwrap_or(&0), &0);
        assert!(metrics.contains_key("uptime_seconds"));
    }

    #[tokio::test]
    async fn test_config_update() {
        let masker = Masker::new();
        let database = create_test_database().await.unwrap();

        let agent = Agent::new(masker, database).await.unwrap();

        let new_config = AgentConfig {
            buffer_size: 200,
            flush_interval_secs: 10,
            ..Default::default()
        };

        agent.update_config(new_config.clone()).await.unwrap();
        let retrieved_config = agent.get_config().await;

        assert_eq!(retrieved_config.buffer_size, 200);
        assert_eq!(retrieved_config.flush_interval_secs, 10);
    }

    #[test]
    fn test_handle_rdev_event() {
        use std::sync::Arc;
        use tokio::sync::RwLock;

        let (tx, mut rx) = mpsc::unbounded_channel::<KeyEvent>();
        let current_window = Arc::new(RwLock::new(None));

        // Test key press event
        let event = Event {
            time: SystemTime::now(),
            name: None,
            event_type: EventType::KeyPress(Key::KeyA),
        };

        Agent::handle_rdev_event(event, &tx, &current_window).unwrap();

        let received = rx.try_recv();
        assert!(received.is_ok());
        let key_event = received.unwrap();
        assert_eq!(key_event.key, "a");
        assert_eq!(key_event.event_type, "press");
        assert!(!key_event.is_modifier);
        assert!(!key_event.is_function_key);
    }
}
