use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use tempfile::NamedTempFile;

use keyai_desktop::agent::{Agent, AgentConfig, KeyEvent, WindowInfo};
use keyai_desktop::masker::Masker;
use keyai_desktop::db::Database;

async fn create_test_database() -> anyhow::Result<Arc<Database>> {
    let temp_file = NamedTempFile::new().unwrap();
    Ok(Arc::new(Database::new(temp_file.path()).await?))
}

#[tokio::test]
async fn test_agent_lifecycle() {
    let masker = Masker::new();
    let database = create_test_database().await.unwrap();

    let mut agent = Agent::new(masker, database).await.unwrap();

    // Test initial state
    assert!(!agent.is_running());

    // Test starting agent
    agent.start().await.unwrap();
    assert!(agent.is_running());

    // Give agent time to initialize
    sleep(Duration::from_millis(100)).await;

    // Test metrics
    let metrics = agent.get_metrics();
    assert!(metrics.contains_key("events_captured"));
    assert!(metrics.contains_key("uptime_seconds"));

    // Test stopping agent
    agent.stop().await.unwrap();
    assert!(!agent.is_running());
}

#[tokio::test]
async fn test_agent_configuration() {
    let masker = Masker::new();
    let database = create_test_database().await.unwrap();

    let custom_config = AgentConfig {
        buffer_size: 200,
        flush_interval_secs: 10,
        capture_modifiers: false,
        ignored_applications: vec!["test-app".to_string()],
        ..Default::default()
    };

    let agent = Agent::with_config(masker, database, custom_config.clone()).await.unwrap();

    let retrieved_config = agent.get_config().await;
    assert_eq!(retrieved_config.buffer_size, 200);
    assert_eq!(retrieved_config.flush_interval_secs, 10);
    assert!(!retrieved_config.capture_modifiers);
    assert!(retrieved_config.ignored_applications.contains(&"test-app".to_string()));
}

#[tokio::test]
async fn test_agent_config_update() {
    let masker = Masker::new();
    let database = create_test_database().await.unwrap();

    let agent = Agent::new(masker, database).await.unwrap();

    let new_config = AgentConfig {
        buffer_size: 150,
        capture_function_keys: false,
        ..Default::default()
    };

    agent.update_config(new_config.clone()).await.unwrap();

    let updated_config = agent.get_config().await;
    assert_eq!(updated_config.buffer_size, 150);
    assert!(!updated_config.capture_function_keys);
}

#[tokio::test]
async fn test_event_filtering() {
    use keyai_desktop::agent::Agent;

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
async fn test_agent_metrics() {
    let masker = Masker::new();
    let database = create_test_database().await.unwrap();

    let mut agent = Agent::new(masker, database).await.unwrap();

    // Start agent to initialize metrics
    agent.start().await.unwrap();
    sleep(Duration::from_millis(50)).await;

    let metrics = agent.get_metrics();

    // Check that all expected metrics are present
    assert!(metrics.contains_key("events_captured"));
    assert!(metrics.contains_key("events_processed"));
    assert!(metrics.contains_key("events_stored"));
    assert!(metrics.contains_key("events_filtered"));
    assert!(metrics.contains_key("window_updates"));
    assert!(metrics.contains_key("uptime_seconds"));

    // Initial values should be 0 or low
    assert_eq!(metrics.get("events_captured").unwrap(), &0);
    assert_eq!(metrics.get("events_processed").unwrap(), &0);

    agent.stop().await.unwrap();
}

#[tokio::test]
async fn test_window_info_structure() {
    let window_info = WindowInfo {
        title: "Test Window".to_string(),
        application: "test-app".to_string(),
        process_id: Some(1234),
        timestamp: 1234567890,
    };

    assert_eq!(window_info.title, "Test Window");
    assert_eq!(window_info.application, "test-app");
    assert_eq!(window_info.process_id, Some(1234));
    assert_eq!(window_info.timestamp, 1234567890);

    // Test serialization
    let json = serde_json::to_string(&window_info).unwrap();
    let deserialized: WindowInfo = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.title, window_info.title);
    assert_eq!(deserialized.application, window_info.application);
    assert_eq!(deserialized.process_id, window_info.process_id);
    assert_eq!(deserialized.timestamp, window_info.timestamp);
}

#[tokio::test]
async fn test_key_event_structure() {
    let key_event = KeyEvent {
        timestamp: 1234567890,
        key: "a".to_string(),
        event_type: "press".to_string(),
        window_info: Some(WindowInfo {
            title: "Test".to_string(),
            application: "test-app".to_string(),
            process_id: None,
            timestamp: 1234567890,
        }),
        is_modifier: false,
        is_function_key: false,
    };

    assert_eq!(key_event.key, "a");
    assert_eq!(key_event.event_type, "press");
    assert!(!key_event.is_modifier);
    assert!(!key_event.is_function_key);
    assert!(key_event.window_info.is_some());

    // Test serialization
    let json = serde_json::to_string(&key_event).unwrap();
    let deserialized: KeyEvent = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.key, key_event.key);
    assert_eq!(deserialized.event_type, key_event.event_type);
    assert_eq!(deserialized.is_modifier, key_event.is_modifier);
    assert_eq!(deserialized.is_function_key, key_event.is_function_key);
}

#[test]
fn test_key_classification() {
    use rdev::Key;
    use keyai_desktop::agent::Agent;

    // Test modifier keys
    assert!(Agent::is_modifier_key(Key::ControlLeft));
    assert!(Agent::is_modifier_key(Key::Alt));
    assert!(Agent::is_modifier_key(Key::ShiftRight));
    assert!(Agent::is_modifier_key(Key::CapsLock));
    assert!(!Agent::is_modifier_key(Key::KeyA));
    assert!(!Agent::is_modifier_key(Key::Space));

    // Test function keys
    assert!(Agent::is_function_key(Key::F1));
    assert!(Agent::is_function_key(Key::F12));
    assert!(!Agent::is_function_key(Key::KeyA));
    assert!(!Agent::is_function_key(Key::Space));
    assert!(!Agent::is_function_key(Key::ControlLeft));
}

#[test]
fn test_key_to_string_conversion() {
    use rdev::Key;
    use keyai_desktop::agent::Agent;

    // Test letters
    assert_eq!(Agent::key_to_string(Key::KeyA), "a");
    assert_eq!(Agent::key_to_string(Key::KeyZ), "z");

    // Test numbers
    assert_eq!(Agent::key_to_string(Key::Num1), "1");
    assert_eq!(Agent::key_to_string(Key::Num0), "0");

    // Test special keys
    assert_eq!(Agent::key_to_string(Key::Space), "Space");
    assert_eq!(Agent::key_to_string(Key::Return), "Return");
    assert_eq!(Agent::key_to_string(Key::Backspace), "Backspace");

    // Test modifiers
    assert_eq!(Agent::key_to_string(Key::ControlLeft), "CtrlLeft");
    assert_eq!(Agent::key_to_string(Key::Alt), "Alt");
    assert_eq!(Agent::key_to_string(Key::ShiftRight), "ShiftRight");

    // Test function keys
    assert_eq!(Agent::key_to_string(Key::F1), "F1");
    assert_eq!(Agent::key_to_string(Key::F12), "F12");

    // Test unknown key
    assert_eq!(Agent::key_to_string(Key::Unknown(999)), "Unknown(999)");
}

#[tokio::test]
async fn test_agent_double_start_stop() {
    let masker = Masker::new();
    let database = create_test_database().await.unwrap();

    let mut agent = Agent::new(masker, database).await.unwrap();

    // Test starting twice
    agent.start().await.unwrap();
    assert!(agent.is_running());

    // Should not error when starting already running agent
    agent.start().await.unwrap();
    assert!(agent.is_running());

    // Test stopping twice
    agent.stop().await.unwrap();
    assert!(!agent.is_running());

    // Should not error when stopping already stopped agent
    agent.stop().await.unwrap();
    assert!(!agent.is_running());
}
