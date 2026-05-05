---
name: rust-qa
description: 'Tauri v2 项目质量保证：PRD 验收清单、IPC 契约验证、Smoke Test、反模式检测。面向 QA Agent。决策树格式。'
type: reference
---

# Tauri v2 项目质量保证

## Quick Start

你是一个 QA Agent，负责在 PRD 实现完成后逐项验证。从 Section 1 开始，按顺序执行每个 Section 的检查项。

## 适用范围

**适用于：** Tauri v2 桌面应用的验收测试、契约验证、反模式检测、依赖审计、权限审计、Smoke Test。
**不适用于：** 前端单元测试（见 `rust-frontend`）、Rust 单元测试（见 `rust-backend`）、架构设计（见 `rust-arch`）。
**目标：** 确保实现与 PRD 一致，IPC 契约对齐，无反模式残留。

## 1. PRD 验收清单生成

**推荐方案：** 从 PRD 提取 Acceptance Criteria，逐条生成可执行的检查项表格。
**理由：** PRD 中的 AC 是唯一权威的完成标准。没有清单就无法判断"完成"。
**怎么做：**

1. 读取 PRD 文件，提取每个功能的 Acceptance Criteria
2. 对每条 AC 生成：验证方法（手动/自动/grep）、预期结果、实际结果
3. 每个功能必须至少有 1 条可自动验证的检查

**BAD：** AC 写成"搜索功能正常" -- 不可量化，无法自动验证

**GOOD：**

```
AC-3.2: 搜索响应时间 < 200ms（10 条记录）
  验证方法: 自动 -- 计时 safeInvoke('search_clips', { query: 'test' })
  预期: < 200ms
  实际: ___
  状态: [PASS/FAIL]
```

**例外：** UI 交互类 AC（如动画流畅度）只能手动验证，标注 `[MANUAL]`。

→ 模板：`templates/smoke-test-checklist.md`
→ 参考：`references/acceptance-checklist.md`

## 2. IPC 契约验证

**推荐方案：** Rust serde 序列化名 与 TypeScript 接口逐字段对比。
**理由：** IPC 跨越 Rust/TS 边界时，类型不对齐是最高频的运行时 bug 源头。
**怎么做：**

1. 读 Rust struct 的 `#[serde(rename_all = "...")]` 注解
2. 读 TS interface 的字段名
3. 逐字段对比：名称、类型、可选性
4. 特别注意映射规则表：

| Rust 类型 | serde JSON | TypeScript 类型 | 陷阱 |
|-----------|-----------|----------------|------|
| `String` | `"..."` | `string` | 无 |
| `i32/u32/i64` | `number` | `number` | TS 无 int/uint 区分 |
| `bool` | `true/false` | `boolean` | 无 |
| `Option<T>` | `T \| null` | `T \| null \| undefined` | TS 多了 undefined |
| `Vec<T>` | `[T]` | `T[]` | 无 |
| `Vec<u8>` | `[number]` | `number[]` | **BAD**: TS 期望 `Uint8Array` |
| `HashMap<String, T>` | `{[key: string]: T}` | `Record<string, T>` | key 必须是 string |
| `chrono::DateTime<Utc>` | `"2024-01-01T00:00:00Z"` | `string` | TS 需要手动解析 |
| `enum { A, B }` (unit) | `"A" \| "B"` | `"A" \| "B"` | 确认 rename_all |
| `serde_json::Value` | any JSON | `unknown` | **BAD**: 用 `any` |

**BAD：**

```rust
// Rust: snake_case by default
pub struct ClipItem {
    pub created_at: i64,
}
```

```typescript
// TS: 忘记 camelCase 映射
interface ClipItem {
    created_at: number;  // 如果 serde 用了 rename_all = "camelCase"，这里应该是 createdAt
}
```

**GOOD：**

```rust
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClipItem {
    pub created_at: i64,
}
// JSON 输出: { "createdAt": 1234567890 }
```

```typescript
interface ClipItem {
    createdAt: number;  // 与 serde 输出对齐
}
```

**例外：** 当 Rust struct 没有使用 `rename_all` 时，JSON 字段名就是 Rust 原始字段名（snake_case），TS interface 也应使用相同命名。

→ 参考：`references/contract-verification.md`

## 3. 反模式检测

**推荐方案：** 用 grep 规则扫描代码，按严重程度分级报告。
**理由：** TODO/FIXME/unwrap/空函数体是未完成实现的信号，上线前必须清零。
**怎么做：**

运行以下检测，记录每个命中的文件和行号：

| 规则 | grep 命令 | 严重程度 |
|------|----------|---------|
| TODO/FIXME | `rg -n 'TODO\|FIXME' src-tauri/src/ src/` | P1 |
| unwrap() | `rg -n '\.unwrap\(\)' src-tauri/src/` | P0 (非 test) |
| 空函数体 | `rg -n 'fn \w+.*\{\s*\}' src-tauri/src/` | P0 |
| stub 实现 | `rg -n 'todo!\(\)\|unimplemented!\(\)' src-tauri/src/` | P0 |
| filter_map 吞错误 | `rg -n 'filter_map.*\.ok\(\)' src-tauri/src/` | P1 |
| expect() 无上下文 | `rg -n '\.expect\("' src-tauri/src/` | P2 |

**BAD：**

```rust
fn search_clips(query: &str) -> Vec<ClipItem> {
    todo!()  // P0: 阻塞验收
}

fn get_config() -> AppConfig {
    let config = fs::read_to_string("config.json").unwrap();  // P0: 生产代码禁止裸 unwrap
    let parsed: AppConfig = serde_json::from_str(&config).unwrap();
    parsed
}
```

**GOOD：**

```rust
fn search_clips(query: &str) -> Vec<ClipItem> {
    db::search(query).unwrap_or_else(|e| {
        log::error!("search failed: {e}");
        vec![]
    })
}

fn get_config() -> Result<AppConfig> {
    let config = fs::read_to_string("config.json")
        .context("Failed to read config.json")?;
    serde_json::from_str(&config).context("Failed to parse config")
}
```

**例外：** `unwrap()` 在 `#[cfg(test)]` 模块中是 P2（建议级别），不阻塞。

→ 参考：`references/anti-patterns.md`

## 4. 依赖审计

**推荐方案：** Cargo.toml 声明 vs 代码实际 use/import 交叉验证。
**理由：** 未使用的依赖增大二进制体积和编译时间；缺少声明的依赖会导致隐式依赖未来版本冲突。
**怎么做：**

**Rust 侧：**

```bash
# 检查未使用的依赖
cargo +nightly udeps 2>/dev/null || cargo tree --duplicates
# 手动交叉验证
rg '^use ' src-tauri/src/ | sed 's/.*use //;s/::.*//' | sort -u
# 对比 Cargo.toml [dependencies]
```

**前端侧：**

```bash
# 检查未使用的依赖
npx depcheck --json 2>/dev/null
# 手动交叉验证
rg "from ['\"]" src/ | sed "s/.*from ['\"]//;s/['\"].*//" | sort -u
# 对比 package.json dependencies
```

**BAD：** Cargo.toml 声明了 `reqwest`，但代码中没有任何 `use reqwest`

**GOOD：** 每个声明的依赖在代码中至少有一个 `use` 引用（标准库和 proc-macro crate 除外）

**例外：** Build-dependencies (`[build-dependencies]`) 和 proc-macro crate 不一定在 `src/` 中有 `use` 语句，需单独验证。

## 5. Capabilities 权限审计

**推荐方案：** 对比 `src-tauri/capabilities/` 声明的权限与 Rust 代码实际使用的 Tauri API。
**理由：** 过度授权违反最小权限原则；权限不足导致运行时 panic。
**怎么做：**

1. 读取 `src-tauri/capabilities/*.json`，提取所有 `permissions` 列表
2. 在 Rust 代码中搜索实际使用的 Tauri API：

```bash
# 检测实际使用的 API
rg 'app\.\w+\(|state\.\w+\(|window\.\w+\(' src-tauri/src/
rg 'tauri::(command|State|App|Manager|WebviewWindow)' src-tauri/src/
```

3. 对比：声明的权限是否覆盖了实际使用，是否有多余声明

**BAD：**

```json
// capabilities/default.json -- 过度授权
{ "permissions": ["core:default", "shell:allow-execute", "fs:allow-read"] }
```

```rust
// 实际代码只用到了剪贴板，shell 和 fs 权限多余
fn read_clipboard(app: AppHandle) -> Result<String> { ... }
```

**GOOD：**

```json
{ "permissions": ["core:default", "clipboard-manager:allow-read"] }
```

**例外：** 开发环境可能需要额外权限（如 `shell:allow-execute` 用于热重载），通过 `identifier: "dev"` 的 capability 文件单独管理。

## 6. Smoke Test

**推荐方案：** 覆盖核心用户流程的端到端验证清单。
**理由：** 单元测试通过不代表应用能正常启动和使用。Smoke Test 是最后一道防线。
**怎么做：**

按以下顺序执行，每步记录 PASS/FAIL：

1. **启动**: `cargo tauri dev` 或双击应用图标，应用窗口 5 秒内出现
2. **监控**: 剪贴板变化被捕获并显示在列表中
3. **搜索**: 输入关键词，结果实时过滤，响应 < 200ms
4. **粘贴**: 点击列表项，内容写入剪贴板
5. **退出**: Cmd+Q / Alt+F4，无报错，无残留进程

性能基线：

| 指标 | 阈值 | 验证方法 |
|------|------|---------|
| 冷启动时间 | < 3s | 计时 `cargo tauri dev` 到窗口可见 |
| 内存占用（空闲） | < 100MB | Activity Monitor / Task Manager |
| 搜索响应 | < 200ms | `console.time` / `Instant::now` |
| 剪贴板监听延迟 | < 500ms | 复制后到列表更新 |

安全基线：

| 检查项 | 验证方法 |
|--------|---------|
| Capabilities 最小权限 | Section 5 审计通过 |
| CSP 无 `unsafe-inline` | 检查 `tauri.conf.json` 的 `security.csp` |
| 无硬编码密钥 | `rg -i 'api_key\|secret\|password\|token' src/` |

→ 模板：`templates/smoke-test-checklist.md`
