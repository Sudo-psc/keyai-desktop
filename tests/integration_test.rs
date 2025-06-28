use std::sync::Arc;
use tempfile::NamedTempFile;
use tokio::sync::Mutex;
use keyai_desktop::{AppState, agent::Agent, masker::Masker, db::Database, search::SearchEngine};
use keyai_desktop::agent::KeyEvent;
use keyai_desktop::search::SearchOptions;

#[tokio::test]
async fn test_full_flow_integration() {
    // Setup
    let temp_file = NamedTempFile::new().unwrap();
    let database = Arc::new(Database::new(temp_file.path()).await.unwrap());
    let search_engine = Arc::new(SearchEngine::new(database.clone()).await.unwrap());
    let masker = Masker::new();
    let agent = Arc::new(Mutex::new(Agent::new(masker.clone(), database.clone()).await.unwrap()));
    
    let _app_state = AppState {
        agent,
        database: database.clone(),
        search_engine: search_engine.clone(),
    };

    // Simulate key events with PII
    let events = vec![
        KeyEvent {
            timestamp: 1000,
            key: "M".to_string(),
            event_type: "press".to_string(),
            window_title: Some("Email Client".to_string()),
            application: Some("Thunderbird".to_string()),
        },
        KeyEvent {
            timestamp: 1001,
            key: "y".to_string(),
            event_type: "press".to_string(),
            window_title: Some("Email Client".to_string()),
            application: Some("Thunderbird".to_string()),
        },
        KeyEvent {
            timestamp: 1002,
            key: " ".to_string(),
            event_type: "press".to_string(),
            window_title: Some("Email Client".to_string()),
            application: Some("Thunderbird".to_string()),
        },
        KeyEvent {
            timestamp: 1003,
            key: "e".to_string(),
            event_type: "press".to_string(),
            window_title: Some("Email Client".to_string()),
            application: Some("Thunderbird".to_string()),
        },
        KeyEvent {
            timestamp: 1004,
            key: "m".to_string(),
            event_type: "press".to_string(),
            window_title: Some("Email Client".to_string()),
            application: Some("Thunderbird".to_string()),
        },
        KeyEvent {
            timestamp: 1005,
            key: "a".to_string(),
            event_type: "press".to_string(),
            window_title: Some("Email Client".to_string()),
            application: Some("Thunderbird".to_string()),
        },
        KeyEvent {
            timestamp: 1006,
            key: "i".to_string(),
            event_type: "press".to_string(),
            window_title: Some("Email Client".to_string()),
            application: Some("Thunderbird".to_string()),
        },
        KeyEvent {
            timestamp: 1007,
            key: "l".to_string(),
            event_type: "press".to_string(),
            window_title: Some("Email Client".to_string()),
            application: Some("Thunderbird".to_string()),
        },
        KeyEvent {
            timestamp: 1008,
            key: " ".to_string(),
            event_type: "press".to_string(),
            window_title: Some("Email Client".to_string()),
            application: Some("Thunderbird".to_string()),
        },
        KeyEvent {
            timestamp: 1009,
            key: "test@example.com".to_string(),
            event_type: "press".to_string(),
            window_title: Some("Email Client".to_string()),
            application: Some("Thunderbird".to_string()),
        },
    ];

    // Apply masking and store
    let masked_events: Vec<KeyEvent> = events
        .into_iter()
        .map(|e| masker.mask_event(e))
        .collect();
    
    database.store_events(&masked_events).await.unwrap();

    // Test search functionality
    let options = SearchOptions::default();
    let results = search_engine.search_text("email", &options).await.unwrap();
    
    // Should find results related to email
    assert!(!results.is_empty());
    
    // Verify PII was masked
    let all_events = database.search_by_timerange(0, u64::MAX, 100).await.unwrap();
    for event in all_events {
        if event.key.contains("@") {
            assert!(event.key.contains("***@"));
        }
    }
}

#[tokio::test]
async fn test_database_persistence() {
    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path().to_path_buf();
    
    // First session - store data
    {
        let database = Arc::new(Database::new(&path).await.unwrap());
        
        let events = vec![
            KeyEvent {
                timestamp: 1000,
                key: "persistent".to_string(),
                event_type: "press".to_string(),
                window_title: None,
                application: None,
            },
        ];
        
        database.store_events(&events).await.unwrap();
        
        let stats = database.get_stats().await.unwrap();
        assert_eq!(stats.total_events, 1);
    }
    
    // Second session - verify data persists
    {
        let database = Arc::new(Database::new(&path).await.unwrap());
        
        let stats = database.get_stats().await.unwrap();
        assert_eq!(stats.total_events, 1);
        
        let events = database.search_by_timerange(0, u64::MAX, 10).await.unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].key, "persistent");
    }
}

#[tokio::test]
async fn test_search_with_special_characters() {
    let temp_file = NamedTempFile::new().unwrap();
    let database = Arc::new(Database::new(temp_file.path()).await.unwrap());
    let search_engine = Arc::new(SearchEngine::new(database.clone()).await.unwrap());
    
    // Store events with special characters
    let events = vec![
        KeyEvent {
            timestamp: 1000,
            key: "test+query".to_string(),
            event_type: "press".to_string(),
            window_title: None,
            application: None,
        },
        KeyEvent {
            timestamp: 1001,
            key: "test@domain.com".to_string(),
            event_type: "press".to_string(),
            window_title: None,
            application: None,
        },
    ];
    
    database.store_events(&events).await.unwrap();
    
    // Search should handle special characters
    let options = SearchOptions::default();
    let results = search_engine.search_text("test", &options).await.unwrap();
    
    assert_eq!(results.len(), 2);
}

#[tokio::test]
async fn test_concurrent_access() {
    let temp_file = NamedTempFile::new().unwrap();
    let database = Arc::new(Database::new(temp_file.path()).await.unwrap());
    
    // Spawn multiple tasks writing to database
    let mut handles = vec![];
    
    for i in 0..5 {
        let db = database.clone();
        let handle = tokio::spawn(async move {
            let events = vec![
                KeyEvent {
                    timestamp: (i * 1000) as u64,
                    key: format!("task{}", i),
                    event_type: "press".to_string(),
                    window_title: None,
                    application: None,
                },
            ];
            db.store_events(&events).await.unwrap();
        });
        handles.push(handle);
    }
    
    // Wait for all tasks
    for handle in handles {
        handle.await.unwrap();
    }
    
    // Verify all events were stored
    let stats = database.get_stats().await.unwrap();
    assert_eq!(stats.total_events, 5);
}

#[tokio::test]
async fn test_large_batch_insert() {
    let temp_file = NamedTempFile::new().unwrap();
    let database = Arc::new(Database::new(temp_file.path()).await.unwrap());
    
    // Create a large batch of events
    let mut events = Vec::new();
    for i in 0..1000 {
        events.push(KeyEvent {
            timestamp: i as u64,
            key: format!("key{}", i % 26),
            event_type: "press".to_string(),
            window_title: Some("Test Window".to_string()),
            application: Some("Test App".to_string()),
        });
    }
    
    // Store and verify
    database.store_events(&events).await.unwrap();
    
    let stats = database.get_stats().await.unwrap();
    assert_eq!(stats.total_events, 1000);
} 