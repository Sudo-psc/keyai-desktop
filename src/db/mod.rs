use std::path::Path;
use std::sync::Arc;
use rusqlite::{Connection, Result as SqliteResult, params};
use tokio::sync::Mutex;
use anyhow::{Result, anyhow};
use tracing::{info, error, debug};
use serde::{Serialize, Deserialize};

use crate::agent::KeyEvent;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredEvent {
    pub id: i64,
    pub timestamp: u64,
    pub key: String,
    pub event_type: String,
    pub window_title: Option<String>,
    pub application: Option<String>,
    pub text_content: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: i64,
    pub content: String,
    pub timestamp: u64,
    pub relevance_score: f64,
    pub context: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseStats {
    pub total_events: i64,
    pub total_size_bytes: i64,
    pub oldest_event: Option<u64>,
    pub newest_event: Option<u64>,
}

pub struct Database {
    connection: Arc<Mutex<Connection>>,
}

impl Database {
    pub async fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        info!("🗄️ Inicializando banco de dados: {:?}", db_path.as_ref());
        
        let conn = Connection::open(db_path)?;
        
        // Set SQLCipher password (in production, this should come from secure storage)
        conn.execute("PRAGMA key = 'keyai-desktop-secret-key'", [])?;
        
        // Enable WAL mode for better concurrency
        conn.execute("PRAGMA journal_mode = WAL", [])?;
        
        // Enable foreign keys
        conn.execute("PRAGMA foreign_keys = ON", [])?;
        
        // Optimize for performance
        conn.execute("PRAGMA synchronous = NORMAL", [])?;
        conn.execute("PRAGMA cache_size = 10000", [])?;
        conn.execute("PRAGMA temp_store = MEMORY", [])?;
        
        let database = Self {
            connection: Arc::new(Mutex::new(conn)),
        };
        
        database.initialize_schema().await?;
        
        info!("✅ Banco de dados inicializado com sucesso");
        Ok(database)
    }

    async fn initialize_schema(&self) -> Result<()> {
        let conn = self.connection.lock().await;
        
        // Create events table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp INTEGER NOT NULL,
                key TEXT NOT NULL,
                event_type TEXT NOT NULL,
                window_title TEXT,
                application TEXT,
                text_content TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                UNIQUE(timestamp, key, event_type)
            )",
            [],
        )?;

        // Create text_content table for full-text search
        conn.execute(
            "CREATE VIRTUAL TABLE IF NOT EXISTS text_search USING fts5(
                content,
                timestamp,
                application,
                window_title,
                content='events',
                content_rowid='id'
            )",
            [],
        )?;

        // Create embeddings table for semantic search
        conn.execute(
            "CREATE TABLE IF NOT EXISTS embeddings (
                id INTEGER PRIMARY KEY,
                event_id INTEGER NOT NULL,
                embedding BLOB NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (event_id) REFERENCES events (id) ON DELETE CASCADE
            )",
            [],
        )?;

        // Create indexes for performance
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_events_timestamp ON events (timestamp)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_events_application ON events (application)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_embeddings_event_id ON embeddings (event_id)",
            [],
        )?;

        // Create triggers to keep FTS5 table in sync
        conn.execute(
            "CREATE TRIGGER IF NOT EXISTS events_ai AFTER INSERT ON events BEGIN
                INSERT INTO text_search(rowid, content, timestamp, application, window_title)
                VALUES (new.id, new.text_content, new.timestamp, new.application, new.window_title);
            END",
            [],
        )?;

        conn.execute(
            "CREATE TRIGGER IF NOT EXISTS events_ad AFTER DELETE ON events BEGIN
                INSERT INTO text_search(text_search, rowid, content, timestamp, application, window_title)
                VALUES ('delete', old.id, old.text_content, old.timestamp, old.application, old.window_title);
            END",
            [],
        )?;

        conn.execute(
            "CREATE TRIGGER IF NOT EXISTS events_au AFTER UPDATE ON events BEGIN
                INSERT INTO text_search(text_search, rowid, content, timestamp, application, window_title)
                VALUES ('delete', old.id, old.text_content, old.timestamp, old.application, old.window_title);
                INSERT INTO text_search(rowid, content, timestamp, application, window_title)
                VALUES (new.id, new.text_content, new.timestamp, new.application, new.window_title);
            END",
            [],
        )?;

        debug!("✅ Schema do banco de dados inicializado");
        Ok(())
    }

    pub async fn store_events(&self, events: &[KeyEvent]) -> Result<()> {
        if events.is_empty() {
            return Ok(());
        }

        let conn = self.connection.lock().await;
        let tx = conn.unchecked_transaction()?;

        for event in events {
            // Reconstruct text content from key events
            let text_content = if event.key.len() == 1 && event.event_type == "press" {
                Some(event.key.clone())
            } else {
                None
            };

            tx.execute(
                "INSERT OR IGNORE INTO events 
                (timestamp, key, event_type, window_title, application, text_content)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    event.timestamp,
                    event.key,
                    event.event_type,
                    event.window_title,
                    event.application,
                    text_content
                ],
            )?;
        }

        tx.commit()?;
        debug!("✅ {} eventos armazenados no banco de dados", events.len());
        Ok(())
    }

    pub async fn search_text(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        let conn = self.connection.lock().await;
        
        let mut stmt = conn.prepare(
            "SELECT e.id, e.text_content, e.timestamp, 
                    rank, e.application, e.window_title
             FROM text_search ts
             JOIN events e ON e.id = ts.rowid
             WHERE text_search MATCH ?1
             ORDER BY rank
             LIMIT ?2"
        )?;

        let rows = stmt.query_map(params![query, limit], |row| {
            Ok(SearchResult {
                id: row.get(0)?,
                content: row.get::<_, Option<String>>(1)?.unwrap_or_default(),
                timestamp: row.get(2)?,
                relevance_score: row.get::<_, f64>(3)?,
                context: row.get::<_, Option<String>>(4)?,
            })
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }

        debug!("🔍 Busca textual retornou {} resultados para: {}", results.len(), query);
        Ok(results)
    }

    pub async fn search_by_timerange(&self, start_timestamp: u64, end_timestamp: u64, limit: usize) -> Result<Vec<StoredEvent>> {
        let conn = self.connection.lock().await;
        
        let mut stmt = conn.prepare(
            "SELECT id, timestamp, key, event_type, window_title, application, text_content, created_at
             FROM events
             WHERE timestamp BETWEEN ?1 AND ?2
             ORDER BY timestamp DESC
             LIMIT ?3"
        )?;

        let rows = stmt.query_map(params![start_timestamp, end_timestamp, limit], |row| {
            Ok(StoredEvent {
                id: row.get(0)?,
                timestamp: row.get(1)?,
                key: row.get(2)?,
                event_type: row.get(3)?,
                window_title: row.get(4)?,
                application: row.get(5)?,
                text_content: row.get(6)?,
                created_at: row.get(7)?,
            })
        })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }

        debug!("📅 Busca por período retornou {} resultados", results.len());
        Ok(results)
    }

    pub async fn get_stats(&self) -> Result<DatabaseStats> {
        let conn = self.connection.lock().await;
        
        let total_events: i64 = conn.query_row(
            "SELECT COUNT(*) FROM events",
            [],
            |row| row.get(0)
        )?;

        let total_size_bytes: i64 = conn.query_row(
            "SELECT page_count * page_size FROM pragma_page_count(), pragma_page_size()",
            [],
            |row| row.get(0)
        ).unwrap_or(0);

        let oldest_event: Option<u64> = conn.query_row(
            "SELECT MIN(timestamp) FROM events",
            [],
            |row| row.get(0)
        ).ok();

        let newest_event: Option<u64> = conn.query_row(
            "SELECT MAX(timestamp) FROM events",
            [],
            |row| row.get(0)
        ).ok();

        Ok(DatabaseStats {
            total_events,
            total_size_bytes,
            oldest_event,
            newest_event,
        })
    }

    pub async fn clear_all_data(&self) -> Result<()> {
        let conn = self.connection.lock().await;
        
        conn.execute("DELETE FROM embeddings", [])?;
        conn.execute("DELETE FROM events", [])?;
        conn.execute("DELETE FROM text_search", [])?;
        
        // Vacuum to reclaim space
        conn.execute("VACUUM", [])?;
        
        info!("🗑️ Todos os dados foram removidos do banco de dados");
        Ok(())
    }

    pub async fn store_embedding(&self, event_id: i64, embedding: &[f32]) -> Result<()> {
        let conn = self.connection.lock().await;
        
        // Convert f32 array to bytes
        let embedding_bytes: Vec<u8> = embedding
            .iter()
            .flat_map(|&x| x.to_le_bytes().to_vec())
            .collect();

        conn.execute(
            "INSERT OR REPLACE INTO embeddings (event_id, embedding) VALUES (?1, ?2)",
            params![event_id, embedding_bytes],
        )?;

        debug!("🧠 Embedding armazenado para evento {}", event_id);
        Ok(())
    }

    pub async fn get_embedding(&self, event_id: i64) -> Result<Option<Vec<f32>>> {
        let conn = self.connection.lock().await;
        
        let embedding_bytes: Option<Vec<u8>> = conn.query_row(
            "SELECT embedding FROM embeddings WHERE event_id = ?1",
            params![event_id],
            |row| row.get(0)
        ).ok();

        if let Some(bytes) = embedding_bytes {
            // Convert bytes back to f32 array
            let embedding: Vec<f32> = bytes
                .chunks_exact(4)
                .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
                .collect();
            Ok(Some(embedding))
        } else {
            Ok(None)
        }
    }

    pub async fn vacuum(&self) -> Result<()> {
        let conn = self.connection.lock().await;
        conn.execute("VACUUM", [])?;
        info!("🧹 Banco de dados otimizado (VACUUM executado)");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_database_creation() {
        let temp_file = NamedTempFile::new().unwrap();
        let db = Database::new(temp_file.path()).await.unwrap();
        
        let stats = db.get_stats().await.unwrap();
        assert_eq!(stats.total_events, 0);
    }

    #[tokio::test]
    async fn test_store_and_search_events() {
        let temp_file = NamedTempFile::new().unwrap();
        let db = Database::new(temp_file.path()).await.unwrap();
        
        let events = vec![
            KeyEvent {
                timestamp: 1000,
                key: "h".to_string(),
                event_type: "press".to_string(),
                window_title: Some("Test Window".to_string()),
                application: Some("Test App".to_string()),
            },
            KeyEvent {
                timestamp: 1001,
                key: "e".to_string(),
                event_type: "press".to_string(),
                window_title: Some("Test Window".to_string()),
                application: Some("Test App".to_string()),
            },
            KeyEvent {
                timestamp: 1002,
                key: "l".to_string(),
                event_type: "press".to_string(),
                window_title: Some("Test Window".to_string()),
                application: Some("Test App".to_string()),
            },
            KeyEvent {
                timestamp: 1003,
                key: "l".to_string(),
                event_type: "press".to_string(),
                window_title: Some("Test Window".to_string()),
                application: Some("Test App".to_string()),
            },
            KeyEvent {
                timestamp: 1004,
                key: "o".to_string(),
                event_type: "press".to_string(),
                window_title: Some("Test Window".to_string()),
                application: Some("Test App".to_string()),
            },
        ];

        db.store_events(&events).await.unwrap();
        
        let stats = db.get_stats().await.unwrap();
        assert_eq!(stats.total_events, 5);
        assert_eq!(stats.oldest_event, Some(1000));
        assert_eq!(stats.newest_event, Some(1004));
    }

    #[tokio::test]
    async fn test_search_by_timerange() {
        let temp_file = NamedTempFile::new().unwrap();
        let db = Database::new(temp_file.path()).await.unwrap();
        
        let events = vec![
            KeyEvent {
                timestamp: 1000,
                key: "a".to_string(),
                event_type: "press".to_string(),
                window_title: None,
                application: None,
            },
            KeyEvent {
                timestamp: 2000,
                key: "b".to_string(),
                event_type: "press".to_string(),
                window_title: None,
                application: None,
            },
            KeyEvent {
                timestamp: 3000,
                key: "c".to_string(),
                event_type: "press".to_string(),
                window_title: None,
                application: None,
            },
        ];

        db.store_events(&events).await.unwrap();
        
        // Search for middle event
        let results = db.search_by_timerange(1500, 2500, 10).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].key, "b");
        
        // Search for all events
        let results = db.search_by_timerange(0, 5000, 10).await.unwrap();
        assert_eq!(results.len(), 3);
    }

    #[tokio::test]
    async fn test_clear_all_data() {
        let temp_file = NamedTempFile::new().unwrap();
        let db = Database::new(temp_file.path()).await.unwrap();
        
        let events = vec![
            KeyEvent {
                timestamp: 1000,
                key: "test".to_string(),
                event_type: "press".to_string(),
                window_title: None,
                application: None,
            },
        ];

        db.store_events(&events).await.unwrap();
        
        // Verify data exists
        let stats = db.get_stats().await.unwrap();
        assert_eq!(stats.total_events, 1);
        
        // Clear data
        db.clear_all_data().await.unwrap();
        
        // Verify data is gone
        let stats = db.get_stats().await.unwrap();
        assert_eq!(stats.total_events, 0);
    }

    #[tokio::test]
    async fn test_embedding_storage() {
        let temp_file = NamedTempFile::new().unwrap();
        let db = Database::new(temp_file.path()).await.unwrap();
        
        // Store an event first
        let events = vec![
            KeyEvent {
                timestamp: 1000,
                key: "test".to_string(),
                event_type: "press".to_string(),
                window_title: None,
                application: None,
            },
        ];
        db.store_events(&events).await.unwrap();
        
        // Get the event ID (should be 1 for the first event)
        let event_id = 1;
        
        // Store embedding
        let embedding = vec![0.1, 0.2, 0.3, 0.4, 0.5];
        db.store_embedding(event_id, &embedding).await.unwrap();
        
        // Retrieve embedding
        let retrieved = db.get_embedding(event_id).await.unwrap();
        assert!(retrieved.is_some());
        let retrieved_embedding = retrieved.unwrap();
        
        // Check values (allowing for floating point precision)
        assert_eq!(retrieved_embedding.len(), embedding.len());
        for (a, b) in embedding.iter().zip(retrieved_embedding.iter()) {
            assert!((a - b).abs() < 0.0001);
        }
    }

    #[tokio::test]
    async fn test_duplicate_events() {
        let temp_file = NamedTempFile::new().unwrap();
        let db = Database::new(temp_file.path()).await.unwrap();
        
        let event = KeyEvent {
            timestamp: 1000,
            key: "a".to_string(),
            event_type: "press".to_string(),
            window_title: None,
            application: None,
        };
        
        // Store same event twice
        db.store_events(&[event.clone()]).await.unwrap();
        db.store_events(&[event]).await.unwrap();
        
        // Should only have one event due to UNIQUE constraint
        let stats = db.get_stats().await.unwrap();
        assert_eq!(stats.total_events, 1);
    }

    #[tokio::test]
    async fn test_empty_events_store() {
        let temp_file = NamedTempFile::new().unwrap();
        let db = Database::new(temp_file.path()).await.unwrap();
        
        // Storing empty vec should not error
        db.store_events(&[]).await.unwrap();
        
        let stats = db.get_stats().await.unwrap();
        assert_eq!(stats.total_events, 0);
    }

    #[tokio::test]
    async fn test_vacuum() {
        let temp_file = NamedTempFile::new().unwrap();
        let db = Database::new(temp_file.path()).await.unwrap();
        
        // Add and remove data
        let events = vec![
            KeyEvent {
                timestamp: 1000,
                key: "test".to_string(),
                event_type: "press".to_string(),
                window_title: None,
                application: None,
            },
        ];
        db.store_events(&events).await.unwrap();
        db.clear_all_data().await.unwrap();
        
        // Vacuum should not error
        db.vacuum().await.unwrap();
    }

    #[tokio::test]
    async fn test_get_embedding_nonexistent() {
        let temp_file = NamedTempFile::new().unwrap();
        let db = Database::new(temp_file.path()).await.unwrap();
        
        // Try to get embedding for non-existent event
        let result = db.get_embedding(999).await.unwrap();
        assert!(result.is_none());
    }
} 