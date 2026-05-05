# Database Migration Deployment — Refinery

## Migration SQL Files

```sql
-- migrations/V1__create_users.sql
CREATE TABLE users (id INTEGER PRIMARY KEY, email TEXT NOT NULL UNIQUE);

-- migrations/V2__add_orders.sql
CREATE TABLE orders (id INTEGER PRIMARY KEY, user_id INTEGER REFERENCES users(id));
```

## Cargo.toml Dependency

```toml
[dependencies]
refinery = { version = "0.8", features = ["rusqlite"] }
```

## Embedding and Running Migrations

```rust
use refinery::Runner;

// 在 lib.rs 或 main.rs 中嵌入迁移文件
refinery::embed_migrations!("./migrations");

fn run_migrations(conn: &mut rusqlite::Connection) -> Result<(), AppError> {
    Runner::new(&migrations::MIGRATIONS)
        .set_abort_divergent(true)
        .run(conn)?;
    Ok(())
}
```

## Rollback Rules

- **Data changes (ALTER, INSERT)**: forward-only, no rollback.
- **Schema changes**: provide a paired down migration.
- **Never modify a released migration** — add a new one instead.
