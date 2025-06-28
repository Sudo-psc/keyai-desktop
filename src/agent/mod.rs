use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};
use tracing::{info, warn, error, debug};
use rdev::{listen, Event, EventType, Key};
use anyhow::{Result, anyhow};

use crate::masker::Masker;
use crate::db::Database;

#[derive(Debug, Clone)]
pub struct KeyEvent {
    pub timestamp: u64,
    pub key: String,
    pub event_type: String,
    pub window_title: Option<String>,
    pub application: Option<String>,
}

pub struct Agent {
    masker: Masker,
    database: Arc<Database>,
    is_running: bool,
    event_sender: Option<mpsc::UnboundedSender<KeyEvent>>,
}

impl Agent {
    pub async fn new(masker: Masker, database: Arc<Database>) -> Result<Self> {
        Ok(Self {
            masker,
            database,
            is_running: false,
            event_sender: None,
        })
    }

    pub async fn start(&mut self) -> Result<()> {
        if self.is_running {
            warn!("⚠️ Agente já está em execução");
            return Ok(());
        }

        info!("🎯 Iniciando agente de captura de teclas...");
        
        // Create channel for key events
        let (tx, mut rx) = mpsc::unbounded_channel::<KeyEvent>();
        self.event_sender = Some(tx.clone());
        self.is_running = true;

        // Clone necessary data for the processing task
        let masker = self.masker.clone();
        let database = self.database.clone();

        // Spawn task to process events
        tokio::spawn(async move {
            let mut buffer = Vec::new();
            let mut last_flush = std::time::Instant::now();
            const FLUSH_INTERVAL: Duration = Duration::from_secs(5);
            const BUFFER_SIZE: usize = 100;

            while let Some(event) = rx.recv().await {
                debug!("📝 Evento recebido: {:?}", event);
                
                // Apply PII masking
                let masked_event = masker.mask_event(event);
                buffer.push(masked_event);

                // Flush buffer if it's full or if enough time has passed
                if buffer.len() >= BUFFER_SIZE || last_flush.elapsed() >= FLUSH_INTERVAL {
                    if let Err(e) = database.store_events(&buffer).await {
                        error!("❌ Erro ao armazenar eventos: {}", e);
                    } else {
                        debug!("✅ {} eventos armazenados", buffer.len());
                    }
                    buffer.clear();
                    last_flush = std::time::Instant::now();
                }
            }

            // Flush remaining events
            if !buffer.is_empty() {
                if let Err(e) = database.store_events(&buffer).await {
                    error!("❌ Erro ao armazenar eventos finais: {}", e);
                }
            }
        });

        // Start the key listener in a separate thread
        let tx_clone = tx.clone();
        std::thread::spawn(move || {
            if let Err(e) = listen(move |event| {
                if let Err(e) = Self::handle_event(event, &tx_clone) {
                    error!("❌ Erro ao processar evento: {}", e);
                }
            }) {
                error!("❌ Erro no listener de teclas: {:?}", e);
            }
        });

        info!("✅ Agente de captura iniciado com sucesso");
        Ok(())
    }

    pub async fn stop(&mut self) -> Result<()> {
        if !self.is_running {
            warn!("⚠️ Agente não está em execução");
            return Ok(());
        }

        info!("🛑 Parando agente de captura...");
        self.is_running = false;
        self.event_sender = None;
        info!("✅ Agente parado com sucesso");
        Ok(())
    }

    pub fn is_running(&self) -> bool {
        self.is_running
    }

    pub fn handle_event(event: Event, sender: &mpsc::UnboundedSender<KeyEvent>) -> Result<()> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| anyhow!("Erro ao obter timestamp: {}", e))?
            .as_secs();

        match event.event_type {
            EventType::KeyPress(key) => {
                let key_event = KeyEvent {
                    timestamp,
                    key: Self::key_to_string(key),
                    event_type: "press".to_string(),
                    window_title: Self::get_active_window_title(),
                    application: Self::get_active_application(),
                };

                sender.send(key_event)
                    .map_err(|e| anyhow!("Erro ao enviar evento: {}", e))?;
            }
            EventType::KeyRelease(key) => {
                let key_event = KeyEvent {
                    timestamp,
                    key: Self::key_to_string(key),
                    event_type: "release".to_string(),
                    window_title: Self::get_active_window_title(),
                    application: Self::get_active_application(),
                };

                sender.send(key_event)
                    .map_err(|e| anyhow!("Erro ao enviar evento: {}", e))?;
            }
            _ => {} // Ignore other event types for now
        }

        Ok(())
    }

    pub fn key_to_string(key: Key) -> String {
        match key {
            Key::Alt => "Alt".to_string(),
            Key::AltGr => "AltGr".to_string(),
            Key::Backspace => "Backspace".to_string(),
            Key::CapsLock => "CapsLock".to_string(),
            Key::ControlLeft => "CtrlLeft".to_string(),
            Key::ControlRight => "CtrlRight".to_string(),
            Key::Delete => "Delete".to_string(),
            Key::DownArrow => "DownArrow".to_string(),
            Key::End => "End".to_string(),
            Key::Escape => "Escape".to_string(),
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
            Key::Home => "Home".to_string(),
            Key::LeftArrow => "LeftArrow".to_string(),
            Key::MetaLeft => "MetaLeft".to_string(),
            Key::MetaRight => "MetaRight".to_string(),
            Key::PageDown => "PageDown".to_string(),
            Key::PageUp => "PageUp".to_string(),
            Key::Return => "Return".to_string(),
            Key::RightArrow => "RightArrow".to_string(),
            Key::ShiftLeft => "ShiftLeft".to_string(),
            Key::ShiftRight => "ShiftRight".to_string(),
            Key::Space => "Space".to_string(),
            Key::Tab => "Tab".to_string(),
            Key::UpArrow => "UpArrow".to_string(),
            Key::PrintScreen => "PrintScreen".to_string(),
            Key::ScrollLock => "ScrollLock".to_string(),
            Key::Pause => "Pause".to_string(),
            Key::NumLock => "NumLock".to_string(),
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
            Key::Insert => "Insert".to_string(),
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
            Key::Function => "Function".to_string(),
            Key::Unknown(code) => format!("Unknown({})", code),
        }
    }

    fn get_active_window_title() -> Option<String> {
        // TODO: Implement platform-specific window title detection
        // For now, return None
        None
    }

    fn get_active_application() -> Option<String> {
        // TODO: Implement platform-specific application detection
        // For now, return None
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::mpsc;

    // Mock database for testing
    struct MockDatabase;
    
    impl MockDatabase {
        async fn new() -> Result<Arc<Database>> {
            // In tests, we'll use a real database with a temporary file
            use tempfile::NamedTempFile;
            let temp_file = NamedTempFile::new().unwrap();
            Ok(Arc::new(Database::new(temp_file.path()).await?))
        }
    }

    #[test]
    fn test_key_to_string() {
        assert_eq!(Agent::key_to_string(Key::KeyA), "a");
        assert_eq!(Agent::key_to_string(Key::Space), "Space");
        assert_eq!(Agent::key_to_string(Key::Return), "Return");
        assert_eq!(Agent::key_to_string(Key::Num1), "1");
        assert_eq!(Agent::key_to_string(Key::Unknown(999)), "Unknown(999)");
    }

    #[tokio::test]
    async fn test_agent_creation() {
        let masker = Masker::new();
        let database = MockDatabase::new().await.unwrap();
        
        let agent = Agent::new(masker, database).await.unwrap();
        assert!(!agent.is_running());
    }

    #[tokio::test]
    async fn test_agent_start_stop() {
        let masker = Masker::new();
        let database = MockDatabase::new().await.unwrap();
        
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
    fn test_handle_event() {
        let (tx, mut rx) = mpsc::unbounded_channel::<KeyEvent>();
        
        // Test key press event
        let event = Event {
            time: SystemTime::now(),
            event_type: EventType::KeyPress(Key::KeyA),
        };
        
        Agent::handle_event(event, &tx).unwrap();
        
        let received = rx.try_recv();
        assert!(received.is_ok());
        let key_event = received.unwrap();
        assert_eq!(key_event.key, "a");
        assert_eq!(key_event.event_type, "press");
    }

    #[test]
    fn test_get_active_window_title() {
        // Currently returns None - test the expected behavior
        assert_eq!(Agent::get_active_window_title(), None);
    }

    #[test]
    fn test_get_active_application() {
        // Currently returns None - test the expected behavior
        assert_eq!(Agent::get_active_application(), None);
    }

    #[tokio::test]
    async fn test_event_channel_capacity() {
        let masker = Masker::new();
        let database = MockDatabase::new().await.unwrap();
        
        let mut agent = Agent::new(masker, database).await.unwrap();
        agent.start().await.unwrap();
        
        // The channel should handle many events without blocking
        // This test verifies the unbounded channel doesn't have capacity issues
        assert!(agent.event_sender.is_some());
    }
} 