use rdev::{listen, Event, EventType, Key};
use std::time::Duration;
use std::thread;

fn main() {
    println!("🔍 Teste de Captura de Teclas - KeyAI");
    println!("================================================");
    println!("Este teste verifica se conseguimos capturar teclas no macOS.");
    println!("Digite algumas teclas e pressione ESC para sair.\n");
    
    // Verificar permissões primeiro
    println!("⏳ Verificando permissões de acessibilidade...");
    thread::sleep(Duration::from_secs(2));
    
    let (tx, rx) = std::sync::mpsc::channel();
    
    // Thread para captura
    let handle = thread::spawn(move || {
        println!("🎯 Iniciando captura de eventos...\n");
        
        if let Err(error) = listen(move |event: Event| {
            match event.event_type {
                EventType::KeyPress(key) => {
                    let key_str = format_key(key);
                    println!("⌨️  Tecla pressionada: {}", key_str);
                    
                    if key == Key::Escape {
                        println!("\n🛑 ESC pressionado - encerrando teste");
                        let _ = tx.send(());
                    }
                },
                EventType::KeyRelease(key) => {
                    let key_str = format_key(key);
                    println!("⬆️  Tecla liberada: {}", key_str);
                },
                _ => {} // Ignorar outros eventos
            }
        }) {
            eprintln!("❌ Erro ao iniciar listener: {:?}", error);
            eprintln!("\n🚨 AÇÃO NECESSÁRIA:");
            eprintln!("1. Vá para: Configurações > Privacidade e Segurança > Acessibilidade");
            eprintln!("2. Adicione o Terminal (ou este app) à lista");
            eprintln!("3. Execute o teste novamente");
        }
    });
    
    // Aguardar sinal de saída
    let _ = rx.recv();
    
    println!("\n✅ Teste concluído!");
}

fn format_key(key: Key) -> String {
    match key {
        Key::Alt => "Alt".to_string(),
        Key::AltGr => "AltGr".to_string(),
        Key::Backspace => "Backspace".to_string(),
        Key::CapsLock => "CapsLock".to_string(),
        Key::ControlLeft => "Ctrl Esquerdo".to_string(),
        Key::ControlRight => "Ctrl Direito".to_string(),
        Key::Delete => "Delete".to_string(),
        Key::DownArrow => "Seta Baixo".to_string(),
        Key::End => "End".to_string(),
        Key::Escape => "ESC".to_string(),
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
        Key::LeftArrow => "Seta Esquerda".to_string(),
        Key::MetaLeft => "Cmd Esquerdo".to_string(),
        Key::MetaRight => "Cmd Direito".to_string(),
        Key::PageDown => "Page Down".to_string(),
        Key::PageUp => "Page Up".to_string(),
        Key::Return => "Enter".to_string(),
        Key::RightArrow => "Seta Direita".to_string(),
        Key::ShiftLeft => "Shift Esquerdo".to_string(),
        Key::ShiftRight => "Shift Direito".to_string(),
        Key::Space => "Espaço".to_string(),
        Key::Tab => "Tab".to_string(),
        Key::UpArrow => "Seta Cima".to_string(),
        Key::PrintScreen => "Print Screen".to_string(),
        Key::ScrollLock => "Scroll Lock".to_string(),
        Key::Pause => "Pause".to_string(),
        Key::NumLock => "Num Lock".to_string(),
        Key::Insert => "Insert".to_string(),
        Key::KeyA => "A".to_string(),
        Key::KeyB => "B".to_string(),
        Key::KeyC => "C".to_string(),
        Key::KeyD => "D".to_string(),
        Key::KeyE => "E".to_string(),
        Key::KeyF => "F".to_string(),
        Key::KeyG => "G".to_string(),
        Key::KeyH => "H".to_string(),
        Key::KeyI => "I".to_string(),
        Key::KeyJ => "J".to_string(),
        Key::KeyK => "K".to_string(),
        Key::KeyL => "L".to_string(),
        Key::KeyM => "M".to_string(),
        Key::KeyN => "N".to_string(),
        Key::KeyO => "O".to_string(),
        Key::KeyP => "P".to_string(),
        Key::KeyQ => "Q".to_string(),
        Key::KeyR => "R".to_string(),
        Key::KeyS => "S".to_string(),
        Key::KeyT => "T".to_string(),
        Key::KeyU => "U".to_string(),
        Key::KeyV => "V".to_string(),
        Key::KeyW => "W".to_string(),
        Key::KeyX => "X".to_string(),
        Key::KeyY => "Y".to_string(),
        Key::KeyZ => "Z".to_string(),
        Key::Num0 => "0".to_string(),
        Key::Num1 => "1".to_string(),
        Key::Num2 => "2".to_string(),
        Key::Num3 => "3".to_string(),
        Key::Num4 => "4".to_string(),
        Key::Num5 => "5".to_string(),
        Key::Num6 => "6".to_string(),
        Key::Num7 => "7".to_string(),
        Key::Num8 => "8".to_string(),
        Key::Num9 => "9".to_string(),
        _ => format!("{:?}", key),
    }
} 