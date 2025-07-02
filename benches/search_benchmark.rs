use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::sync::Arc;
use tempfile::NamedTempFile;
use tokio::runtime::Runtime;

use keyai_desktop::db::Database;
use keyai_desktop::search::{SearchEngine, SearchOptions};
use keyai_desktop::agent::KeyEvent;

async fn setup_test_data(database: Arc<Database>, event_count: usize) {
    let mut events = Vec::new();
    let words = vec![
        "hello", "world", "test", "benchmark", "performance", "search", "database",
        "keyai", "desktop", "application", "user", "interface", "system", "data",
        "privacy", "security", "encryption", "local", "storage", "fast", "efficient"
    ];

    for i in 0..event_count {
        let word = &words[i % words.len()];
        events.push(KeyEvent {
            timestamp: i as u64,
            key: word.to_string(),
            event_type: "press".to_string(),
            window_title: Some(format!("Test Window {}", i)),
            application: Some("Test App".to_string()),
        });

        // Store in batches to avoid memory issues
        if events.len() >= 1000 {
            database.store_events(&events).await.unwrap();
            events.clear();
        }
    }

    // Store remaining events
    if !events.is_empty() {
        database.store_events(&events).await.unwrap();
    }
}

fn search_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    // Setup test databases with different sizes
    let sizes = vec![1000, 10000, 100000];
    let mut databases = Vec::new();
    let mut search_engines = Vec::new();

    for &size in &sizes {
        let temp_file = NamedTempFile::new().unwrap();
        let database = rt.block_on(async {
            let db = Arc::new(Database::new(temp_file.path()).await.unwrap());
            setup_test_data(db.clone(), size).await;
            db
        });

        let search_engine = rt.block_on(async {
            SearchEngine::new(database.clone()).await.unwrap()
        });

        databases.push(database);
        search_engines.push(search_engine);
    }

    // Benchmark text search
    let mut group = c.benchmark_group("text_search");
    for (i, &size) in sizes.iter().enumerate() {
        group.bench_with_input(
            BenchmarkId::new("search_text", size),
            &size,
            |b, &_size| {
                let search_engine = &search_engines[i];
                let options = SearchOptions::default();

                b.to_async(&rt).iter(|| async {
                    let results = search_engine
                        .search_text(black_box("test"), &options)
                        .await
                        .unwrap();
                    black_box(results)
                });
            },
        );
    }
    group.finish();

    // Benchmark semantic search
    let mut group = c.benchmark_group("semantic_search");
    for (i, &size) in sizes.iter().enumerate() {
        group.bench_with_input(
            BenchmarkId::new("search_semantic", size),
            &size,
            |b, &_size| {
                let search_engine = &search_engines[i];
                let options = SearchOptions::default();

                b.to_async(&rt).iter(|| async {
                    let results = search_engine
                        .search_semantic(black_box("test application"), &options)
                        .await
                        .unwrap();
                    black_box(results)
                });
            },
        );
    }
    group.finish();

    // Benchmark hybrid search
    let mut group = c.benchmark_group("hybrid_search");
    for (i, &size) in sizes.iter().enumerate() {
        group.bench_with_input(
            BenchmarkId::new("search_hybrid", size),
            &size,
            |b, &_size| {
                let search_engine = &search_engines[i];
                let options = SearchOptions::default();

                b.to_async(&rt).iter(|| async {
                    let results = search_engine
                        .search_hybrid(black_box("test performance"), &options)
                        .await
                        .unwrap();
                    black_box(results)
                });
            },
        );
    }
    group.finish();
}

fn database_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("database_operations");

    // Benchmark event storage
    group.bench_function("store_events_batch_100", |b| {
        b.to_async(&rt).iter(|| async {
            let temp_file = NamedTempFile::new().unwrap();
            let database = Arc::new(Database::new(temp_file.path()).await.unwrap());

            let events: Vec<KeyEvent> = (0..100)
                .map(|i| KeyEvent {
                    timestamp: i,
                    key: format!("key_{}", i),
                    event_type: "press".to_string(),
                    window_title: Some("Test Window".to_string()),
                    application: Some("Test App".to_string()),
                })
                .collect();

            database.store_events(black_box(&events)).await.unwrap();
        });
    });

    // Benchmark database stats
    group.bench_function("get_stats", |b| {
        b.to_async(&rt).iter(|| async {
            let temp_file = NamedTempFile::new().unwrap();
            let database = Arc::new(Database::new(temp_file.path()).await.unwrap());
            setup_test_data(database.clone(), 1000).await;

            let stats = database.get_stats().await.unwrap();
            black_box(stats)
        });
    });

    group.finish();
}

criterion_group!(benches, search_benchmark, database_benchmark);
criterion_main!(benches);
