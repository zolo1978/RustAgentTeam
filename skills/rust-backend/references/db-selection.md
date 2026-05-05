# Database Selection Reference

## Decision Guide

```
Desktop local -> rusqlite (sync, zero-dep, spawn_blocking wrapper)
Server        -> sqlx (async, compile-time SQL check, connection pool)
```

## rusqlite (Desktop)

- Synchronous API, zero external dependencies
- Uses `rusqlite::Connection` directly (not a pool type)
- Wrap with `tokio::task::spawn_blocking` for async compatibility
- Ideal for Tauri desktop apps with local SQLite storage

```rust
use rusqlite::Connection;

pub async fn query_user(db: &Mutex<Connection>, id: Uuid) -> Result<Option<User>, AppError> {
    let id = id.to_string();
    // Lock is held only for the query duration inside spawn_blocking.
    // This is safe for desktop (single-user, low contention). For high-throughput
    // scenarios, consider r2d2 connection pooling to avoid lock contention.
    tokio::task::spawn_blocking(move || {
        let conn = db.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        conn.query_row(
            "SELECT id, name FROM users WHERE id = ?1",
            [&id],
            |row| Ok(User { id: row.get(0)?, name: row.get(1)? }),
        ).optional().map_err(|e| AppError::Internal(e.to_string()))
    }).await?
}
```

For connection pooling on desktop (rarely needed), use `r2d2::Pool<SqliteConnectionManager>`:

```rust
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

let manager = SqliteConnectionManager::file("app.db");
let pool: Pool<SqliteConnectionManager> = Pool::new(manager)?;

let conn = pool.get()?;
conn.query_row("SELECT 1", [], |row| row.get::<_, i32>(0))?;
```

## sqlx (Server)

- Fully async, compile-time SQL verification with `query!` macro
- Built-in connection pooling via `PgPoolOptions`
- Supports PostgreSQL, MySQL, SQLite

```rust
pub async fn query_user(pool: &PgPool, id: Uuid) -> Result<Option<User>, AppError> {
    sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", id)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))
}
```

## Rules

- Do NOT introduce sqlx async SQLite in desktop apps (over-engineered)
- Do NOT use rusqlite on the server (lacks connection pool and async support)
- Desktop single-connection: wrap `rusqlite::Connection` in `std::sync::Mutex`
- Desktop pooling: only when concurrent writes require it, use `r2d2`
