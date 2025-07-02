use criterion::{black_box, criterion_group, criterion_main, Criterion};
use keyai_desktop::agent::{Agent, KeyEvent};
use keyai_desktop::masker::Masker;
use keyai_desktop::db::Database;
use std::sync::Arc;
use tempfile::NamedTempFile;
use tokio::runtime::Runtime;

fn benchmark_key_to_string(c: &mut Criterion) {
    use rdev::Key;

    c.bench_function("Agent::key_to_string", |b| {
        b.iter(|| {
            Agent::key_to_string(black_box(Key::KeyA));
            Agent::key_to_string(black_box(Key::Space));
            Agent::key_to_string(black_box(Key::Return));
            Agent::key_to_string(black_box(Key::Unknown(999)));
        })
    });
}

fn benchmark_event_processing(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let temp_file = NamedTempFile::new().unwrap();

    let masker = Masker::new();
    let database = rt.block_on(async {
        Arc::new(Database::new(temp_file.path()).await.unwrap())
    });

    c.bench_function("Agent event processing", |b| {
        b.iter(|| {
            let events = vec![
                KeyEvent {
                    timestamp: 1000,
                    key: "test@example.com".to_string(),
                    event_type: "press".to_string(),
                    window_title: Some("Test Window".to_string()),
                    application: Some("Test App".to_string()),
                },
                KeyEvent {
                    timestamp: 1001,
                    key: "123.456.789-01".to_string(),
                    event_type: "press".to_string(),
                    window_title: Some("Test Window".to_string()),
                    application: Some("Test App".to_string()),
                },
            ];

            let masked_events: Vec<KeyEvent> = events
                .into_iter()
                .map(|e| masker.mask_event(e))
                .collect();

            black_box(masked_events);
        })
    });
}

fn benchmark_batch_event_store(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let temp_file = NamedTempFile::new().unwrap();

    let database = rt.block_on(async {
        Arc::new(Database::new(temp_file.path()).await.unwrap())
    });

    let mut group = c.benchmark_group("Database store events");

    for size in [10, 100, 1000].iter() {
        group.bench_with_input(format!("batch_size_{}", size), size, |b, &size| {
            let events: Vec<KeyEvent> = (0..size).map(|i| KeyEvent {
                timestamp: i as u64,
                key: format!("key{}", i % 26),
                event_type: "press".to_string(),
                window_title: Some("Benchmark Window".to_string()),
                application: Some("Benchmark App".to_string()),
            }).collect();

            b.to_async(&rt).iter(|| async {
                database.store_events(black_box(&events)).await.unwrap();
            });
        });
    }

    group.finish();
}

criterion_group!(benches,
    benchmark_key_to_string,
    benchmark_event_processing,
    benchmark_batch_event_store
);
criterion_main!(benches);
