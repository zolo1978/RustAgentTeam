# 反模式检测规则集

## 严重程度定义

| 级别 | 含义 | 处理要求 |
|------|------|---------|
| P0 | 阻塞 | 必须修复，阻塞验收 |
| P1 | 警告 | 应修复，验收前需评估 |
| P2 | 建议 | 建议修复，可延后 |

## P0 规则（阻塞验收）

### P0-1: 空/Stub 函数体

**模式**: 函数只有 `todo!()`, `unimplemented!()`, 或空花括号

```bash
rg -n 'todo!\(\)|unimplemented!\(\)' src-tauri/src/
rg -n 'fn \w+[^;]*\{\s*\}' src-tauri/src/
```

**BAD:**

```rust
fn delete_clip(id: &str) -> bool {
    todo!()
}
```

**GOOD:**

```rust
fn delete_clip(id: &str) -> Result<bool> {
    let deleted = db::delete(id)?;
    Ok(deleted > 0)
}
```

**修复**: 实现完整逻辑或标注明确原因（如依赖外部服务未就绪）

### P0-2: 生产代码中的 unwrap()

**模式**: `.unwrap()` 出现在非 test 代码中

```bash
rg -n '\.unwrap\(\)' src-tauri/src/ --glob '!*test*'
```

**BAD:**

```rust
let config = fs::read_to_string(path).unwrap();
```

**GOOD:**

```rust
let config = fs::read_to_string(path)
    .context(format!("Failed to read config from {}", path))?;
```

**例外**: `#[cfg(test)]` 和 `#[test]` 函数中的 unwrap() 为 P2

**修复**: 使用 `?` + `Context` 或 `unwrap_or_default()` / `unwrap_or_else()`

## P1 规则（警告）

### P1-1: TODO/FIXME 注释

**模式**: 代码中残留 TODO 或 FIXME 标记

```bash
rg -n 'TODO|FIXME' src-tauri/src/ src/ --glob '!*.md'
```

**处理**: 逐条评估。如果是已知的合理延后项，标注 issue 编号；否则修复或删除。

### P1-2: filter_map 静默吞错误

**模式**: `.filter_map(|r| r.ok())` 将错误静默丢弃

```bash
rg -n 'filter_map.*\.ok\(\)' src-tauri/src/
```

**BAD:**

```rust
let items: Vec<ClipItem> = lines
    .iter()
    .map(|l| serde_json::from_str(l))
    .filter_map(|r| r.ok())  // 解析失败被静默丢弃
    .collect();
```

**GOOD:**

```rust
let items: Vec<ClipItem> = lines
    .iter()
    .filter_map(|l| {
        serde_json::from_str(l)
            .map_err(|e| log::warn!("Parse failed for line: {e}"))
            .ok()
    })
    .collect();
```

**修复**: 至少 log 错误信息，不要完全静默

### P1-3: expect() 缺少上下文

**模式**: `.expect("...")` 的消息不足以定位问题

```bash
rg -n '\.expect\("' src-tauri/src/
```

**BAD:**

```rust
let port = config.port.expect("missing");  // 什么 missing？
```

**GOOD:**

```rust
let port = config.port.expect("config.port is required -- check config.json");
```

**修复**: expect 消息应包含：什么值缺失 + 在哪里配置 + 如何修复

### P1-4: 前端 any 类型

**模式**: TypeScript 中使用 `any` 类型

```bash
rg -n ': any\b|as any' src/ --glob '!*.d.ts'
```

**BAD:**

```typescript
const data: any = await safeInvoke('get_clips');
```

**GOOD:**

```typescript
const data: ClipItem[] = await safeInvoke<ClipItem[]>('get_clips');
```

**修复**: 为所有 IPC 返回值指定具体类型

## P2 规则（建议）

### P2-1: 测试中 unwrap()

```bash
rg -n '\.unwrap\(\)' src-tauri/src/ --glob '*test*'
```

测试中的 unwrap() 可以接受，但 `assert!` 更清晰：

**BAD:**

```rust
let result = search("test").unwrap();
```

**GOOD:**

```rust
let result = search("test").expect("search should succeed in test");
```

### P2-2: 过长函数

```bash
# 查找超过 50 行的函数（粗略检测）
rg -n 'fn \w+' src-tauri/src/
```

**修复**: 拆分为小函数，每个函数做一件事

### P2-3: 硬编码魔法值

```bash
rg -n '"\d{2,}|:\d{4,}|localhost:\d+' src-tauri/src/
```

**BAD:**

```rust
let url = "http://localhost:3000/api";
```

**GOOD:**

```rust
let url = config.api_url.as_str();
```

**修复**: 提取为常量或配置项

## 执行顺序

1. 先运行所有 P0 规则，有任何命中则阻塞
2. 运行 P1 规则，汇总报告
3. 运行 P2 规则，生成建议清单
4. 生成报告：规则编号 / 文件:行号 / 严重程度 / 描述
