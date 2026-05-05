# FTS5 注入防护 — SQLite 全文搜索安全参考

## 概述

ClipVault 使用 SQLite FTS5 进行剪贴板内容的全文搜索。FTS5 的 MATCH 查询有自己的查询语法，
用户输入如果未经清洗，可能导致意外查询行为或信息泄露。

## FTS5 查询语法风险

FTS5 MATCH 支持以下特殊语法：
- `"phrase"` — 短语匹配
- `token*` — 前缀匹配
- `AND`, `OR`, `NOT` — 逻辑操作符
- `NEAR` — 邻近匹配
- `^` — 列起始匹配
- `+`, `-` — 修饰符
- `column:token` — 列限定匹配

恶意利用后果：
1. 构造复杂查询消耗 CPU（DoS）
2. 通过布尔逻辑推断其他记录内容
3. 利用错误信息泄露数据库结构

## ClipVault 当前实现分析

```rust
// clip_repo.rs:115 — 当前实现
let fts_query = format!("\"{}\"*", query.replace('"', "\"\""));
// ✅ 使用参数化查询 (MATCH ?)
// ✅ 双引号转义 ("" -> 字面双引号）
// ✅ 外层引号包裹抑制 FTS 操作符
// ⚠️ 未限制输入长度
// ⚠️ 未处理空查询
// ⚠️ 未处理仅含特殊字符的查询
```

当前实现的核心思路正确：外层引号将查询包裹为短语，尾部 `*` 启用前缀匹配。
需要增加输入验证层。

## BAD vs GOOD

### BAD — 字符串拼接 FTS 查询

```rust
// 危险：用户输入直接拼入 MATCH 表达式
pub fn search_clips(conn: &Connection, query: &str) -> Result<Vec<ClipSummary>> {
    let sql = format!(
        "SELECT c.id FROM clips c \
         JOIN clips_fts f ON f.rowid = c.rowid \
         WHERE clips_fts MATCH '{}'", query  // 直接拼接！
    );
    let mut stmt = conn.prepare(&sql)?;
    // 用户输入 "hello' OR '1'='1" 可泄露所有记录
    stmt.query_map([], row_to_summary)
}
```

### GOOD — 参数化查询 + 引号包裹 + 输入验证

```rust
use rusqlite::{params, types::ToSql};

const MAX_QUERY_LEN: usize = 200;
const MAX_SEARCH_RESULTS: u32 = 100;

/// 验证搜索查询输入
pub fn validate_search_query(query: &str) -> Result<&str, AppError> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return Err(AppError::Validation("搜索查询不能为空".into()));
    }
    if trimmed.len() > MAX_QUERY_LEN {
        return Ok(&trimmed[..MAX_QUERY_LEN]);
    }
    if trimmed.chars().all(|c| !c.is_alphanumeric()) {
        return Err(AppError::Validation("搜索查询必须包含字母或数字".into()));
    }
    Ok(trimmed)
}

/// 转义 FTS5 查询中的双引号并包裹
fn escape_fts_query(query: &str) -> String {
    // FTS5 中 "" 表示字面双引号
    // 外层双引号将整个查询变为短语匹配
    // 尾部 * 启用前缀匹配
    format!("\"{}\"*", query.replace('"', "\"\""))
}

/// FTS5 全文搜索（安全版）
pub fn search_clips(
    conn: &Connection,
    query: &str,
    content_type: Option<&str>,
    limit: u32,
) -> Result<Vec<ClipSummary>, AppError> {
    // 1. 输入验证
    let query = validate_search_query(query)?;
    let limit = limit.min(MAX_SEARCH_RESULTS);

    // 2. 转义并构建 FTS 查询
    let fts_query = escape_fts_query(query);

    // 3. 参数化查询 — 永远不拼接
    let mut sql = String::from(
        "SELECT c.id, c.content_type, c.preview, c.is_favorite, c.created_at \
         FROM clips c \
         JOIN clips_fts f ON f.rowid = c.rowid \
         WHERE clips_fts MATCH ?",
    );
    let mut param_boxed: Vec<Box<dyn ToSql>> = vec![Box::new(fts_query)];

    if let Some(ct) = content_type {
        sql.push_str(" AND c.content_type = ?");
        param_boxed.push(Box::new(ct.to_string()));
    }

    sql.push_str(" ORDER BY f.rank LIMIT ?");
    param_boxed.push(Box::new(limit));

    let param_refs: Vec<&dyn ToSql> =
        param_boxed.iter().map(|p| p.as_ref()).collect();

    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt
        .query_map(param_refs.as_slice(), row_to_summary)?
        .filter_map(|r| r.ok())
        .collect();
    Ok(rows)
}
```

## 防护四层体系

### Layer 1: 输入验证

```rust
pub fn validate_search_query(query: &str) -> Result<&str, AppError> {
    let trimmed = query.trim();
    if trimmed.is_empty() { return Err(/* ... */); }
    if trimmed.len() > 200 { return Ok(&trimmed[..200]); }
    if trimmed.chars().all(|c| !c.is_alphanumeric()) { return Err(/* ... */); }
    Ok(trimmed)
}
```

### Layer 2: 引号转义

```rust
fn escape_fts_query(query: &str) -> String {
    format!("\"{}\"*", query.replace('"', "\"\""))
}
```

外层双引号将整个查询变为短语匹配，所有 FTS 操作符（AND、OR、NOT、NEAR）被视为字面文本。
尾部 `*` 启用前缀匹配。

### Layer 3: 参数化查询

所有 FTS 查询使用 `?` 占位符和 rusqlite 的 `ToSql` trait 传参，禁止字符串拼接。

### Layer 4: 查询限制

```rust
const MAX_SEARCH_RESULTS: u32 = 100;
let limit = limit.min(MAX_SEARCH_RESULTS);

// 数据库层超时
conn.busy_timeout(std::time::Duration::from_secs(5))?;
```

## 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fts_escape_normal() {
        assert_eq!(escape_fts_query("hello"), "\"hello\"*");
    }

    #[test]
    fn fts_escape_double_quotes() {
        assert_eq!(escape_fts_query("say \"hi\""), "\"say \"\"hi\"\"\"*");
    }

    #[test]
    fn fts_neutralizes_operators() {
        assert_eq!(escape_fts_query("a OR b"), "\"a OR b\"*");
        assert_eq!(escape_fts_query("NOT x"), "\"NOT x\"*");
        assert_eq!(escape_fts_query("a NEAR b"), "\"a NEAR b\"*");
    }

    #[test]
    fn validate_rejects_empty() {
        assert!(validate_search_query("").is_err());
        assert!(validate_search_query("   ").is_err());
    }

    #[test]
    fn validate_rejects_special_only() {
        assert!(validate_search_query("***").is_err());
    }

    #[test]
    fn validate_accepts_normal() {
        assert!(validate_search_query("hello").is_ok());
        assert!(validate_search_query("ClipVault 测试").is_ok());
    }

    #[test]
    fn validate_truncates_long() {
        let long = "a".repeat(300);
        assert_eq!(validate_search_query(&long).unwrap().len(), 200);
    }
}
```

## 清洗规则速查

| FTS5 语法 | 处理方式 | 原因 |
|-----------|---------|------|
| `"` | 转义为 `""` | 防止逃逸引号包裹 |
| `AND OR NOT` | 引号包裹后变为字面文本 | 用户不需要布尔逻辑 |
| `*` | 引号包裹后变为字面文本 | 防止前缀通配 DoS |
| `NEAR` | 引号包裹后变为字面文本 | 不需要邻近搜索 |
| `:` | 引号包裹后变为字面文本 | 防止列过滤 |
| `^` | 引号包裹后变为字面文本 | 防止开头匹配 |
| `()` | 引号包裹后变为字面文本 | 防止子表达式 |

## 检测命令

```bash
# 检查所有 SQL 拼接（高风险）
rg 'format!.*SELECT\|format!.*INSERT\|format!.*DELETE\|format!.*UPDATE' src-tauri/src/

# 检查 MATCH 用法
rg 'MATCH' src-tauri/src/

# 检查参数化查询
rg 'params!\|ToSql' src-tauri/src/

# 检查输入长度限制
rg 'MAX_QUERY\|MAX_SEARCH\|\.len\(\).*200\|truncate' src-tauri/src/
```
