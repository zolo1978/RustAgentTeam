# 搜索性能 (SQLite FTS5) — ClipVault (Tauri v2)

## 目标
10K 条记录搜索 < 30ms。

## 数据量分级

| 记录数 | 方案 | 预期延迟 |
|--------|------|---------|
| < 1K | SQLite LIKE | < 5ms |
| 1K-100K | FTS5 索引 | < 30ms |
| > 100K | FTS5 + 分页 + 缓存 | < 50ms |

## FTS5 索引配置

### 建表
```sql
-- 主表
CREATE TABLE IF NOT EXISTS clips (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    content TEXT NOT NULL,
    content_type TEXT NOT NULL DEFAULT 'text',  -- text/image/code
    tags TEXT DEFAULT '',
    source TEXT DEFAULT '',       -- 来源应用名
    timestamp INTEGER NOT NULL,
    pinned INTEGER DEFAULT 0,
    is_deleted INTEGER DEFAULT 0
);

-- FTS5 虚拟表（content table 模式，避免数据冗余）
CREATE VIRTUAL TABLE IF NOT EXISTS clips_fts USING fts5(
    content,
    tags,
    content='clips',
    content_rowid='id',
    tokenize='unicode61 remove_diacritics 2'
);
```

### 同步触发器
```sql
CREATE TRIGGER IF NOT EXISTS clips_fts_insert AFTER INSERT ON clips BEGIN
    INSERT INTO clips_fts(rowid, content, tags)
        VALUES (new.id, new.content, new.tags);
END;

CREATE TRIGGER IF NOT EXISTS clips_fts_delete AFTER DELETE ON clips BEGIN
    INSERT INTO clips_fts(clips_fts, rowid, content, tags)
        VALUES ('delete', old.id, old.content, old.tags);
END;

CREATE TRIGGER IF NOT EXISTS clips_fts_update AFTER UPDATE ON clips BEGIN
    INSERT INTO clips_fts(clips_fts, rowid, content, tags)
        VALUES ('delete', old.id, old.content, old.tags);
    INSERT INTO clips_fts(rowid, content, tags)
        VALUES (new.id, new.content, new.tags);
END;
```

### PRAGMA 优化
```sql
PRAGMA journal_mode=WAL;        -- 读不阻塞写
PRAGMA synchronous=NORMAL;      -- WAL 模式下足够安全
PRAGMA cache_size=-4000;        -- 4MB 缓存
PRAGMA temp_store=MEMORY;
```

## 查询优化

### EXPLAIN QUERY PLAN 验证
```sql
-- 应该看到 SCAN TABLE clips_fts VIRTUAL TABLE INDEX
EXPLAIN QUERY PLAN
SELECT c.id, c.content, c.timestamp
FROM clips c JOIN clips_fts f ON c.id = f.rowid
WHERE clips_fts MATCH 'test'
ORDER BY f.rank LIMIT 50;

-- 如果看到 SCAN TABLE clips (全表扫描)，说明索引没命中
```

### 分页策略
```sql
-- 方案 1: LIMIT/OFFSET（简单，适合前几页）
SELECT c.* FROM clips c
JOIN clips_fts f ON c.id = f.rowid
WHERE clips_fts MATCH ?1
ORDER BY f.rank
LIMIT 50 OFFSET 0;

-- 方案 2: cursor-based（适合深分页，更高效）
SELECT c.* FROM clips c
JOIN clips_fts f ON c.id = f.rowid
WHERE clips_fts MATCH ?1 AND c.id < ?last_seen_id
ORDER BY f.rank
LIMIT 50;
```

## Rust 实现

```rust
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

struct SearchResult {
    entries: Vec<ClipEntry>,
    total_count: u32,
    has_more: bool,
}

fn search_clips(
    pool: &Pool<SqliteConnectionManager>,
    query: &str,
    page: u32,
) -> Result<SearchResult> {
    let conn = pool.get()?;

    // 安全处理 FTS5 特殊字符
    let safe_query: String = query
        .chars()
        .filter(|c| !"*:\"()^".contains(*c))
        .collect::<String>()
        .trim()
        .to_string();
    if safe_query.is_empty() {
        return Ok(SearchResult {
            entries: vec![], total_count: 0, has_more: false,
        });
    }

    let fts_query = format!("\"{}\"*", safe_query); // 前缀匹配
    let per_page: u32 = 50;
    let offset = page * per_page;

    let mut stmt = conn.prepare(
        "SELECT c.id, c.content, c.content_type, c.timestamp, c.tags
         FROM clips c
         JOIN clips_fts f ON c.id = f.rowid
         WHERE clips_fts MATCH ?1 AND c.is_deleted = 0
         ORDER BY f.rank
         LIMIT ?2 OFFSET ?3"
    )?;

    let entries: Vec<ClipEntry> = stmt
        .query_map(rusqlite::params![fts_query, per_page + 1, offset], |row| {
            Ok(ClipEntry {
                id: row.get(0)?,
                content: row.get(1)?,
                content_type: row.get(2)?,
                timestamp: row.get(3)?,
                tags: row.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let has_more = entries.len() > per_page as usize;
    let result: Vec<ClipEntry> = entries.into_iter().take(per_page as usize).collect();

    // 总数估算（避免 count(*) 全表扫描）
    let total = offset as usize + result.len()
        + if has_more { 1 } else { 0 };

    Ok(SearchResult {
        entries: result,
        total_count: total as u32,
        has_more,
    })
}
```

## 性能测试

```bash
# 插入测试数据
sqlite3 clipvault.db "WITH RECURSIVE c(x) AS (SELECT 1 UNION ALL SELECT x+1 FROM c WHERE x<10000)
    INSERT INTO clips (content, tags, timestamp) SELECT 'test content ' || x, 'tag' || (x%10), x FROM c;"

# 查询计划检查
sqlite3 clipvault.db "EXPLAIN QUERY PLAN SELECT * FROM clips_fts WHERE clips_fts MATCH 'test'"

# 延迟测量
time sqlite3 clipvault.db "SELECT count(*) FROM clips_fts WHERE clips_fts MATCH 'content'"

# criterion 基准
cargo bench -- search
```

## 常见问题

| 问题 | 原因 | 解决 |
|------|------|------|
| LIKE '%xxx%' 慢 | 全表扫描 | 换 FTS5 |
| FTS5 中文搜不到 | 默认 tokenizer 不支持中文 | unicode61 + remove_diacritics 2 |
| 搜索结果少 | 精确匹配 | 加 `*` 前缀匹配 |
| 深分页慢 | OFFSET 扫描前面所有行 | cursor-based 分页 |
| 索引膨胀 | 未重建 FTS | `INSERT INTO clips_fts(clips_fts) VALUES('rebuild')` |

## 验证清单
- [ ] 10K 条记录搜索 < 30ms
- [ ] EXPLAIN QUERY PLAN 命中 FTS 索引
- [ ] 同步触发器正确维护索引
- [ ] 分页加载无卡顿
- [ ] 特殊字符输入不崩溃
