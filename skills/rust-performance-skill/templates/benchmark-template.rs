// ClipVault Performance Benchmarks
// Usage: cargo bench
// Report: target/criterion/report/index.html
//
// Cargo.toml configuration:
// [dev-dependencies]
// criterion = { version = "0.5", features = ["html_reports"] }
// rusqlite = { version = "0.31", features = ["bundled"] }
// serde = { version = "1", features = ["derive"] }
// serde_json = "1"
// bincode = "2"
//
// [[bench]]
// name = "clipvault_bench"
// harness = false
// path = "benches/clipvault_bench.rs"

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

// ============================================================
// Data types
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ClipEntry {
    id: i64,
    content: String,
    content_type: String,
    timestamp: i64,
    tags: String,
}

// ============================================================
// Fixtures
// ============================================================

fn create_test_db() -> Connection {
    let conn = Connection::open_in_memory().expect("failed to open in-memory db");
    conn.execute_batch(
        "PRAGMA journal_mode=WAL;
         PRAGMA synchronous=NORMAL;
         PRAGMA cache_size=-4000;
         CREATE TABLE clips (
             id INTEGER PRIMARY KEY AUTOINCREMENT,
             content TEXT NOT NULL,
             content_type TEXT NOT NULL DEFAULT 'text',
             timestamp INTEGER NOT NULL,
             tags TEXT DEFAULT ''
         );
         CREATE VIRTUAL TABLE clips_fts USING fts5(
             content, tags,
             content='clips',
             content_rowid='id',
             tokenize='unicode61 remove_diacritics 2'
         );
         CREATE TRIGGER clips_ai AFTER INSERT ON clips BEGIN
             INSERT INTO clips_fts(rowid, content, tags) VALUES (new.id, new.content, new.tags);
         END;",
    )
    .expect("failed to create schema");
    conn
}

fn seed_clips(conn: &Connection, count: usize) {
    let tx = conn.unchecked_transaction().expect("tx failed");
    for i in 0..count {
        tx.execute(
            "INSERT INTO clips (content, content_type, timestamp, tags) VALUES (?1, 'text', ?2, ?3)",
            rusqlite::params![
                format!("Clipboard content #{} with some text for testing search performance", i),
                1700000000 + i as i64,
                format!("tag{}", i % 10),
            ],
        )
        .expect("insert failed");
    }
    tx.finish().expect("commit failed");
}

fn generate_entries(count: usize) -> Vec<ClipEntry> {
    (0..count)
        .map(|i| ClipEntry {
            id: i as i64,
            content: format!("Clipboard content #{} with varying length text for benchmark", i),
            content_type: "text".to_string(),
            timestamp: 1700000000 + i as i64,
            tags: format!("tag{}", i % 10),
        })
        .collect()
}

// ============================================================
// Benchmark 1: SQLite Query Performance
// ============================================================

fn bench_sqlite_inserts(c: &mut Criterion) {
    let mut group = c.benchmark_group("sqlite_inserts");
    for size in [100, 1000, 10000] {
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            b.iter(|| {
                let conn = create_test_db();
                seed_clips(&conn, size);
                black_box(&conn);
            });
        });
    }
    group.finish();
}

fn bench_sqlite_fts_search(c: &mut Criterion) {
    let conn = create_test_db();
    seed_clips(&conn, 10_000);

    let mut group = c.benchmark_group("sqlite_fts_search");
    for query in ["content", "clipboard", "testing", "nonexistent"] {
        group.bench_with_input(
            BenchmarkId::new("10k_records", query),
            &query,
            |b, &query| {
                b.iter(|| {
                    let mut stmt = conn
                        .prepare(
                            "SELECT c.id, c.content, c.timestamp
                             FROM clips c
                             JOIN clips_fts f ON c.id = f.rowid
                             WHERE clips_fts MATCH ?1
                             ORDER BY f.rank LIMIT 50",
                        )
                        .expect("prepare failed");
                    let results: Vec<(i64, String, i64)> = stmt
                        .query_map(rusqlite::params![format!("\"{}\"*", query)], |row| {
                            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
                        })
                        .expect("query failed")
                        .filter_map(|r| r.ok())
                        .collect();
                    black_box(&results);
                });
            },
        );
    }
    group.finish();
}

fn bench_sqlite_paginated_load(c: &mut Criterion) {
    let conn = create_test_db();
    seed_clips(&conn, 10_000);

    let mut group = c.benchmark_group("sqlite_paginated_load");
    for page in [0u32, 10, 50, 100] {
        group.bench_with_input(
            BenchmarkId::new("page", page),
            &page,
            |b, &page| {
                b.iter(|| {
                    let offset = page * 50;
                    let mut stmt = conn
                        .prepare(
                            "SELECT id, content, timestamp FROM clips
                             ORDER BY timestamp DESC LIMIT 50 OFFSET ?1",
                        )
                        .expect("prepare failed");
                    let results: Vec<(i64, String, i64)> = stmt
                        .query_map(rusqlite::params![offset], |row| {
                            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
                        })
                        .expect("query failed")
                        .filter_map(|r| r.ok())
                        .collect();
                    black_box(&results);
                });
            },
        );
    }
    group.finish();
}

// ============================================================
// Benchmark 2: IPC Serialization
// ============================================================

fn bench_serde_json_serialize(c: &mut Criterion) {
    let mut group = c.benchmark_group("serde_json_serialize");
    for count in [10, 50, 100, 500] {
        let entries = generate_entries(count);
        group.throughput(Throughput::Elements(count as u64));
        group.bench_with_input(BenchmarkId::from_parameter(count), &entries, |b, entries| {
            b.iter(|| {
                let json = serde_json::to_string(black_box(entries)).expect("serialize failed");
                black_box(&json);
            });
        });
    }
    group.finish();
}

fn bench_serde_json_deserialize(c: &mut Criterion) {
    let mut group = c.benchmark_group("serde_json_deserialize");
    for count in [10, 50, 100, 500] {
        let entries = generate_entries(count);
        let json = serde_json::to_string(&entries).expect("serialize failed");
        group.throughput(Throughput::Elements(count as u64));
        group.bench_with_input(BenchmarkId::from_parameter(count), &json, |b, json| {
            b.iter(|| {
                let parsed: Vec<ClipEntry> =
                    serde_json::from_str(black_box(json)).expect("deserialize failed");
                black_box(&parsed);
            });
        });
    }
    group.finish();
}

fn bench_bincode_serialize(c: &mut Criterion) {
    let mut group = c.benchmark_group("bincode_serialize");
    for count in [10, 50, 100, 500] {
        let entries = generate_entries(count);
        group.throughput(Throughput::Elements(count as u64));
        group.bench_with_input(BenchmarkId::from_parameter(count), &entries, |b, entries| {
            b.iter(|| {
                let bytes = bincode::serialize(black_box(entries)).expect("serialize failed");
                black_box(&bytes);
            });
        });
    }
    group.finish();
}

fn bench_bincode_deserialize(c: &mut Criterion) {
    let mut group = c.benchmark_group("bincode_deserialize");
    for count in [10, 50, 100, 500] {
        let entries = generate_entries(count);
        let bytes = bincode::serialize(&entries).expect("serialize failed");
        group.throughput(Throughput::Elements(count as u64));
        group.bench_with_input(BenchmarkId::from_parameter(count), &bytes, |b, bytes| {
            b.iter(|| {
                let parsed: Vec<ClipEntry> =
                    bincode::deserialize(black_box(bytes)).expect("deserialize failed");
                black_box(&parsed);
            });
        });
    }
    group.finish();
}

// ============================================================
// Benchmark 3: String Processing
// ============================================================

fn bench_fts_query_sanitization(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_sanitization");
    let inputs = vec![
        ("simple", "hello world".to_string()),
        ("special_chars", "test*:\"()^content".to_string()),
        ("long", "a".repeat(1000)),
        ("mixed", "hello *world* (test) \"quoted\"".to_string()),
    ];

    for (name, input) in inputs {
        group.bench_with_input(BenchmarkId::new("sanitize", name), &input, |b, input| {
            b.iter(|| {
                let safe: String = black_box(input)
                    .chars()
                    .filter(|c| !"*:\"()^".contains(*c))
                    .collect::<String>()
                    .trim()
                    .to_string();
                let fts_query = format!("\"{}\"*", safe);
                black_box(&fts_query);
            });
        });
    }
    group.finish();
}

fn bench_clip_content_truncation(c: &mut Criterion) {
    let mut group = c.benchmark_group("content_truncation");

    group.bench_function("truncate_10kb_to_500", |b| {
        let long_content = "x".repeat(10_000);
        b.iter(|| {
            let truncated = if black_box(&long_content).len() > 500 {
                &long_content[..500]
            } else {
                &long_content
            };
            black_box(truncated);
        });
    });

    group.bench_function("truncate_unicode_10kb", |b| {
        let unicode_content: String = "你好世界".repeat(2500); // ~10KB
        b.iter(|| {
            let char_count = black_box(&unicode_content).chars().count();
            let truncated: String = black_box(&unicode_content)
                .chars()
                .take(500.min(char_count))
                .collect();
            black_box(&truncated);
        });
    });

    group.finish();
}

// ============================================================
// Criterion configuration and main
// ============================================================

criterion_group!(
    benches,
    bench_sqlite_inserts,
    bench_sqlite_fts_search,
    bench_sqlite_paginated_load,
    bench_serde_json_serialize,
    bench_serde_json_deserialize,
    bench_bincode_serialize,
    bench_bincode_deserialize,
    bench_fts_query_sanitization,
    bench_clip_content_truncation,
);

criterion_main!(benches);
