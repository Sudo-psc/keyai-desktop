use rdev::{listen, Event, EventType, Key};
use std::panic;
use std::process;

fn main() {
    println!("🔍 Debug Específico - Captura de Teclado KeyAI");
    println!("==============================================");
    
    // Configurar panic handler personalizado
    panic::set_hook(Box::new(|panic_info| {
        eprintln!("💥 PANIC DETECTADO:");
        eprintln!("   Localização: {:?}", panic_info.location());
        eprintln!("   Mensagem: {:?}", panic_info.payload().downcast_ref::<&str>());
        eprintln!("   Thread: {:?}", std::thread::current().name());
        
        // Tentar salvar informações de debug
        eprintln!("\n🔧 Informações de Debug:");
        eprintln!("   PID: {}", process::id());
        eprintln!("   Timestamp: {:?}", std::time::SystemTime::now());
        
        // Forçar exit
        process::exit(1);
    }));
    
    println!("⚙️ Configurando captura apenas de teclado...");
    println!("📝 Digite qualquer tecla (ESC para sair)");
    println!("🎯 Foco: Detectar onde exatamente ocorre o crash\n");
    
    // Contador de eventos para debug
    let mut event_count = 0;
    
    println!("🚀 Iniciando listener...");
    
    // Usar Result para capturar erros
    let result = std::panic::catch_unwind(|| {
        listen(move |event: Event| {
            event_count += 1;
            
            // Log detalhado de cada evento
            println!("📨 Evento #{}: {:?}", event_count, event.event_type);
            
            match event.event_type {
                EventType::KeyPress(key) => {
                    println!("  ⬇️ TECLA PRESSIONADA: {:?}", key);
                    
                    // Verificar se é ESC para sair
                    if matches!(key, Key::Escape) {
                        println!("🛑 ESC detectado - saindo...");
                        process::exit(0);
                    }
                    
                    // Verificar teclas problemáticas
                    match key {
                        Key::Num3 => println!("  ⚠️ Tecla 3 detectada (potencial problema)"),
                        Key::Num4 => println!("  ⚠️ Tecla 4 detectada (potencial problema)"),
                        Key::Space => println!("  ⚠️ Space detectada (potencial problema)"),
                        Key::Tab => println!("  ⚠️ Tab detectada (potencial problema)"),
                        _ => println!("  ✅ Tecla normal: {:?}", key),
                    }
                }
                EventType::KeyRelease(key) => {
                    println!("  ⬆️ TECLA LIBERADA: {:?}", key);
                }
                _ => {
                    // Ignorar eventos de mouse para este teste
                    return;
                }
            }
            
            // Flush stdout para garantir que vemos os logs
            use std::io::{self, Write};
            io::stdout().flush().unwrap();
        })
    });
    
    match result {
        Ok(listen_result) => {
            match listen_result {
                Ok(()) => println!("✅ Listen terminou normalmente"),
                Err(e) => eprintln!("❌ Erro no listen: {:?}", e),
            }
        }
        Err(panic_payload) => {
            eprintln!("💥 Panic capturado no catch_unwind:");
            eprintln!("   Payload: {:?}", panic_payload);
        }
    }
    
    println!("🏁 Programa finalizado");
} 