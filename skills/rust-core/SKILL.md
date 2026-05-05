---
name: rust-core
description: 'Rust 核心开发：错误处理、测试、安全、所有权、并发。决策树格式——给结论，不列菜单。'
type: skill
---

# Rust 核心开发

## Quick Start
1. 写新函数？先写 `#[test]`（TDD: RED → GREEN → REFACTOR）
2. 错误处理？库用 `thiserror`，应用用 `anyhow`，永远不要 `unwrap()`
3. 安全？`cargo audit && cargo deny check`，无 SAFETY 注释不许写 unsafe
4. 并发？`Arc<Mutex<T>>` 共享状态，channel 消息传递，tokio async
5. Code review？看底部反模式速查表

## 适用范围
**适用于：** 任何 Rust 项目的核心开发模式——错误处理、测试、安全、所有权、并发。
**不适用于：** Tauri v2 特定架构（见 `rust-tauri`）、PRD 编写（见 `rust-prd`）、Rust 语言入门知识问答、跨语言比较。
**目标：** Rust 1.80+ MSRV。

## 1. 错误处理

**推荐方案：** 库用 `thiserror` 定义结构化错误枚举，应用用 `anyhow` 做灵活错误传播。
**理由：** 库的消费者需要匹配具体错误变体（typed errors）；应用的调用者是顶层 main/tests，只需打印和日志（flexible errors）。
**怎么做：**
```rust
// 库代码 — 结构化错误
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("record not found: {id}")]
    NotFound { id: String },
    #[error("connection failed")]
    Connection(#[from] std::io::Error),
}

// 应用代码 — 灵活错误
use anyhow::{Context, Result};
fn load_config(path: &str) -> Result<Config> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read config from {path}"))?;
    Ok(toml::from_str(&content)?)
}
```
**例外：** 原型/POC 阶段可以全用 `anyhow`，稳定后再拆出 `thiserror` 错误类型。

## 2. 测试

**推荐方案：** TDD（RED-GREEN-REFACTOR），`#[cfg(test)] mod tests` 放在源文件内，覆盖率 ≥ 80%。
**理由：** 测试先写保证你理解需求，覆盖率门槛防止回归。
**怎么做：**
```rust
pub fn add(a: i32, b: i32) -> i32 { a + b }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn adds_two_numbers() { assert_eq!(add(2, 3), 5); }
}
```

**子决策：**
| 需求 | 方案 | 理由 |
|------|------|------|
| Mock 外部依赖 | `mockall` | 最成熟的 Rust mock 框架 |
| 参数化测试 | `rstest` | 比 手写 循环更清晰 |
| 属性测试 | `proptest` | 随机输入发现边界 bug |
| 基准测试 | `criterion` | 统计显著的性能测量 |
| 覆盖率 | `cargo llvm-cov --fail-under-lines 80` | CI 集成方便 |

```toml
[dependencies]
thiserror = "2"
anyhow = "1"
serde = { version = "1", features = ["derive"] }
tokio = { version = "1", features = ["full"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
validator = { version = "0.18", features = ["derive"] }
argon2 = "0.5"
zeroize = { version = "1", features = ["derive"] }

[dev-dependencies]
mockall = "0.13"
rstest = "0.23"
proptest = "1"
criterion = { version = "0.5", features = ["html_reports"] }
```

**例外：** FFI 绑定层测试可以跳过覆盖率要求，用集成测试代替。

## 3. 安全

**推荐方案：** 发布前必须过安全门：`cargo audit` + `cargo deny check` + unsafe 审计 + fuzzing。
**理由：** 供应链攻击和 内存安全 是 Rust 应用的两大风险面。
**怎么做：**
```bash
# 安全门（CI 必跑）
cargo audit                    # 已知漏洞检查
cargo deny check               # 许可证+来源审计
cargo geiger --all-features    # 统计 unsafe 使用
cargo fuzz run parse_input -- -max_total_time=300  # 5分钟模糊测试
```

**子决策：**
| 场景 | 方案 |
|------|------|
| 密码存储 | `argon2` 哈希，永不明文 |
| 敏感数据清理 | `zeroize` + `#[zeroize(drop)]` |
| SQL 查询 | 参数化（`$1`, `$2`），永不拼接 |
| 文件路径 | `canonicalize` + `starts_with` 防穿越 |
| 密钥管理 | 环境变量，永不硬编码 |

**例外：** 内部工具/脚本可以跳过 fuzzing，但 audit + deny 仍然必须。

### 临时文件安全

**规则：** 任何写入临时文件的代码必须遵守 5 项约束。
**理由：** 临时文件是桌面应用最常见的本地攻击面——权限泄露、symlink 竞态、残留泄露。

| 约束 | 实现 | 检测 |
|------|------|------|
| 限制权限 | `mode(0o600)` 仅 owner 读写 | `rg 'mode\(0o' src/` |
| 随机文件名 | UUID 或 `tempfile::NamedTempFile` | `rg 'format!.*tmp\|temp_dir' src/` |
| 防 symlink | `create_new(true)` 不覆盖已有文件 | `rg 'create_new' src/` |
| 注册清理 | `tokio::spawn` + 延迟 `remove_file`，或 `tempfile` 自动 Drop | `rg 'remove_file\|drop.*temp' src/` |
| 路径验证 | `canonicalize` + 白名单前缀 | `rg 'canonicalize' src/` |

```rust
// 安全模式
use std::os::unix::fs::OpenOptionsExt;
std::fs::OpenOptions::new()
    .write(true)
    .create_new(true)   // 防 symlink
    .mode(0o600)        // 仅 owner
    .open(&path)?;

// 延迟清理
tokio::spawn(async move {
    tokio::time::sleep(Duration::from_secs(30)).await;
    let _ = tokio::fs::remove_file(&path).await;
});
```

**例外：** 使用 `tempfile::NamedTempFile` 时权限和清理由 crate 处理，但仍需确保 Drop 被调用。

→ 完整安全评审流程：[references/security-threat-model.md]
→ fuzzing 配置：[references/security-fuzzing.md]
→ unsafe 审计规范：[references/security-unsafe-audit.md]

## 4. 所有权与借用

**推荐方案：** 默认传引用 `&T`，只在需要存储或消费时取所有权。避免 clone 满足借用检查器。
**理由：** clone 掩盖了设计问题——如果需要 clone，通常意味着所有权模型需要重新思考。
**怎么做：**
```rust
// 好：借用
fn process(data: &[u8]) -> usize { data.len() }

// 好：需要灵活所有权时用 Cow
fn normalize(input: &str) -> Cow<'_, str> {
    if input.contains(' ') { Cow::Owned(input.replace(' ', "_")) }
    else { Cow::Borrowed(input) }
}

// 坏：无谓 clone
fn process_bad(data: &Vec<u8>) -> usize {
    let cloned = data.clone(); // 浪费
    cloned.len()
}
```
**例外：** 并发场景中 clone Arc/小结构体 是合理的（Arc::clone 只增加引用计数）。

## 5. 并发

**推荐方案：** 共享可变状态用 `Arc<Mutex<T>>`，消息传递用 bounded channel，IO 用 tokio async。
**理由：** 这是 Rust 并发的安全三角——编译器保证无数据竞争。
**怎么做：**
```rust
// 共享状态
let counter = Arc::new(Mutex::new(0));
// 多线程中: let mut num = counter.lock().expect("poisoned"); *num += 1;

// 消息传递（有背压）
let (tx, rx) = mpsc::sync_channel(16);

// async IO
async fn fetch(url: &str) -> Result<String> {
    tokio::time::timeout(Duration::from_secs(5), reqwest::get(url))
        .await.context("timeout")?
        .context("request failed")?
        .text().await.context("read body")
}
```
**例外：** 极端性能场景可以用 `parking_lot` 替代 `std::sync`（更快的锁实现），或用 `rayon` 并行迭代器。

## 6. 类型设计

**推荐方案：** 用 枚举 让非法状态不可表示，用 newtype 防止参数混用，exhaustive match 不用 `_`。
**理由：** 类型系统是最好的文档和验证。
**怎么做：**
```rust
// 枚举建模状态
enum ConnectionState {
    Disconnected,
    Connecting { attempt: u32 },
    Connected { session_id: String },
    Failed { reason: String, retries: u32 },
}

// newtype 防混用
struct UserId(u64);
struct OrderId(u64);
fn get_order(user: UserId, order: OrderId) -> Result<Order> { todo!() }
```
**例外：** 原型阶段可以先用类型别名 `type UserId = u64`，稳定后升级为 newtype。

## 7. 模块组织

**推荐方案：** 按领域组织（`auth/`, `orders/`），不按类型组织（`models/`, `handlers/`）。最小 `pub` 暴露面。
**理由：** 按领域组织让内聚性高、耦合度低，改一个功能只动一个目录。
**怎么做：**
```
my_app/
├── src/
│   ├── main.rs
│   ├── lib.rs
│   ├── auth/           # 领域模块
│   │   ├── mod.rs
│   │   ├── token.rs
│   │   └── middleware.rs
│   └── orders/         # 领域模块
│       ├── mod.rs
│       └── service.rs
```
**例外：** 小项目（<5 个模块）可以扁平结构，不需要子目录。

## 8. trait 设计

**推荐方案：** 接受泛型参数（`impl Trait`），返回具体类型。需要动态分发时才用 `Box<dyn Trait>`。用 `From` trait 实现类型转换。
**理由：** 泛型零成本（编译时单态化），trait object 有运行时开销。`From` 自动获得 `Into`，启用 `?` 操作符。
**怎么做：**

```rust
// 好：泛型输入，具体输出
fn read_all(reader: &mut impl Read) -> io::Result<Vec<u8>> {
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf)?;
    Ok(buf)
}

// 好：用 From 启用 ? 操作符
impl From<io::Error> for AppError {
    fn from(e: io::Error) -> Self { AppError::Internal(e.to_string()) }
}
fn read_config(path: &str) -> Result<Config, AppError> {
    let content = std::fs::read_to_string(path)?; // ? 调用 From::from
    Ok(parse(&content))
}

// 好：需要异构集合时用 trait object
struct Router { handlers: Vec<Box<dyn Handler>> }

// 坏：不需要动态分发时用 dyn
fn fast_process<H: Handler>(h: &H, req: &Request) -> Response { h.handle(req) }
```
**例外：** 插件系统、运行时注册等必须用 `dyn Trait` 的场景。

## 反模式速查表

| 反模式 | 正确做法 |
|--------|---------|
| `unwrap()` 生产代码 | `?` 或显式错误处理 |
| clone 满足借用检查器 | 重新思考所有权设计 |
| `String` 当 `&str` 够用 | 默认 `&str`，需要所有权时才 `String` |
| `Box<dyn Error>` 在库里 | `thiserror` 定义具体错误类型 |
| `_` 通配符 match 业务枚举 | exhaustive match 每个变体 |
| async 中 `std::thread::sleep` | `tokio::time::sleep().await` |
| 一切 `pub` | `pub(crate)` 最小暴露 |
| 忽略 `#[must_use]` 返回值 | 显式处理或 `let _ =` 附注释 |

## 工具命令速查

```bash
cargo check              # 快速类型检查
cargo clippy             # lint
cargo fmt                # 格式化
cargo test               # 跑测试
cargo test -- --nocapture # 显示 println
cargo llvm-cov --fail-under-lines 80  # 覆盖率
cargo audit              # 安全审计
cargo bench              # 基准测试
```

## References
| Topic | File |
|-------|------|
| STRIDE 威胁模型 | [references/security-threat-model.md](references/security-threat-model.md) |
| 自动化扫描 | [references/security-scanning.md](references/security-scanning.md) |
| Fuzzing | [references/security-fuzzing.md](references/security-fuzzing.md) |
| unsafe 审计 | [references/security-unsafe-audit.md](references/security-unsafe-audit.md) |
| 密码/清理实现 | [references/security-implementations.md](references/security-implementations.md) |
| 安全发现模板 | [templates/finding.md](templates/finding.md) |
| 事件响应模板 | [templates/incident-response.md](templates/incident-response.md) |

## 相关 Skill

| Skill | 关联场景 |
|-------|---------|
| [rust-async-patterns](../rust-async-patterns/skill.md) | spawn_blocking、Mutex 选型、async 反模式 |
| [rust-crash-debug](../rust-crash-debug/skill.md) | panic 定位、unwrap 替换 |
| [rust-tauri-testing](../rust-tauri-testing/skill.md) | TDD 测试编写、覆盖率工具 |
| [rust-security-skill](../rust-security-skill/SKILL.md) | Tauri v2 安全专项审计 |
