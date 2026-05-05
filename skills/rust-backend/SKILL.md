---
name: rust-backend
description: 'Tauri v2 后端开发：认证、错误处理、数据库、DevOps、发布。面向后端 Agent。决策树格式。'
type: skill
---

# Tauri v2 后端开发

## Quick Start
1. 要 API？Copy `templates/tauri-command.rs`（桌面）+ `templates/axum-endpoint.rs`（REST）
2. 认证？Copy `templates/jwt-auth.rs`（49 行完整 JWT 实现）
3. 分页？Copy `templates/paged-list.rs`（Cursor-based）
4. 测试？Copy `templates/test-helpers.rs`（Tauri + Axum 测试模式）
5. 发布？Follow Release Checklist (Section 5)

## 适用范围
**适用于：** Tauri v2 后端开发——Tauri Command、Service/Repo 分层、数据库、认证、DevOps、发布流程。
**不适用于：** 架构决策和脚手架（见 `rust-arch`）、Rust 语言核心（见 `rust-core`）、前端 UI（见 `rust-frontend`）、PRD（见 `rust-prd-skill`）。
**目标：** Tauri >= 2.0.0, Rust 1.80+ MSRV。

## 1. 认证

**推荐方案：** 桌面用 `tauri-plugin-authenticator`（系统 keychain），服务端用 JWT Bearer token。
**理由：** 桌面应用不应自管理密钥——OS keychain 有硬件支持和系统级安全。
**怎么做：**

```rust
// 桌面：系统 keychain 存储 credentials
use tauri_plugin_authenticator::AuthenticatorExt;
let authenticator = app.authenticator()?;
authenticator.save_credential("my-app", "api-token", &token)?;

// 服务端：JWT + FromRequestParts（完整代码见 templates/jwt-auth.rs）
use jsonwebtoken::{encode, decode, Header, Validation};
let token = encode(&Header::default(), &claims, &jwt_secret)?;
```

→ 模板：`templates/jwt-auth.rs`（完整 JWT 认证实现，49 行）

**例外：** 无需认证的本地工具可以跳过。

**子决策：**
| 场景 | 方案 |
|------|------|
| 本地工具（无服务端） | 系统 keychain（macOS Keychain / Windows Credential Manager） |
| 桌面 + 后端 API | keychain 存 refresh token + JWT 做 API 认证 |
| 移动端 | iOS Keychain / Android Keystore + 生物识别 |
| 多设备同步 | 后端验证 + 短期 access token + 长期 refresh token |

→ 深入：[JWT 认证](references/auth.md)

## 2. 错误处理

**推荐方案：** `AppError` 枚举 + `ErrorCode` 目录 + `ApiResponse<T>` 统一信封。
**理由：** 前端需要知道是客户端错误（4xx）还是服务端错误（5xx）以及具体错误码来展示对应 UI。
**怎么做：**

```rust
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    Validation(String),
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    Auth(String),
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

#[derive(serde::Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}
```

→ 深入：[错误码目录](references/error-codes.md)

**子决策：**
| 场景 | 方案 |
|------|------|
| Tauri Command 返回错误 | `Result<T, AppError>` — Tauri 自动序列化为 JSON |
| Axum REST 返回错误 | `AppError` 实现 `IntoResponse`，自动设置 HTTP 状态码 |
| 前端错误恢复 | `safeInvoke<T>` 自动重试 transient 错误 → `rust-frontend` templates |
| 错误码扩展 | 在 `ErrorCode` 枚举中添加变体，前端按 code 分支处理 |

## 3. 数据库

**推荐方案：** 桌面/移动用 rusqlite（同步、零依赖、`spawn_blocking`），服务端用 sqlx（异步、编译时 SQL 检查、连接池）。
**理由：** 桌面应用嵌入 SQLite 是唯一合理选择；服务端需要连接池和异步 IO。
**怎么做：**

```rust
// 桌面：rusqlite + spawn_blocking
tokio::task::spawn_blocking(move || {
    let conn = rusqlite::Connection::open(path)?;
    conn.execute("INSERT INTO orders (id) VALUES (?1)", [&id])?;
    Ok(())
}).await?
```

**例外：** 桌面应用如果未来需要多进程并发写入，考虑迁移到 sqlx + SQLite WAL 模式。

**子决策：**
| 场景 | 方案 |
|------|------|
| 纯桌面/移动 | rusqlite + `spawn_blocking`，零外部依赖 |
| 桌面 + REST API | 桌面用 rusqlite，API 层用 sqlx + PostgreSQL |
| 需要全文搜索 | rusqlite + `fts5` feature |
| 数据迁移 | Refinery 嵌入 SQL 文件 → `references/database-migration.md` |

→ 深入：[数据库选型](references/db-selection.md)
→ 迁移：[数据库迁移](references/database-migration.md)

## 4. DevOps

**推荐方案：** GitHub Actions 多平台矩阵 + 代码签名 + canary 分阶段放量。
**理由：** 人工构建不可复现，全量发布没有回滚窗口。
**怎么做：**

```yaml
jobs:
  release:
    strategy:
      matrix:
        include:
          - { platform: macos-latest, target: aarch64-apple-darwin }
          - { platform: windows-latest, target: x86_64-pc-windows-msvc }
          - { platform: ubuntu-22.04, target: x86_64-unknown-linux-gnu }
```

**子决策：**

| 需求 | 方案 |
|------|------|
| 环境 | `.env` + `TAURI_ENV` → 见 `templates/config.rs` |
| 监控 | Sentry (Rust + 前端) + `tracing` |
| 自动更新 | Tauri Updater + 签名密钥 + canary → 前端实现见 `rust-frontend` Skill `templates/check-for-update.ts` |
| 构建优化 | `opt-level = "z"`, LTO, strip, `codegen-units = 1` |
| CI 缓存 | `Swatinem/rust-cache@v2` + Cargo check 缓存 |

→ CI YAML：[CI/CD](references/ci-cd.md)
→ 自动更新：[自动更新](references/auto-update.md)
→ 构建优化：[构建优化](references/build-optimization.md)
→ 故障排除：[故障排除](references/troubleshooting.md)

## 5. Release Checklist

```
## Pre-Release
- [ ] 版本号：Cargo.toml + package.json + tauri.conf.json 三处一致
- [ ] CHANGELOG.md 更新所有用户可见变更
- [ ] 测试全绿（unit + integration + E2E）
- [ ] `cargo audit` + `cargo deny check` 无问题
- [ ] 签名证书未过期
- [ ] 数据库迁移正向测试通过
## Build
- [ ] 4 平台构建通过（macOS arm64/x64, Windows, Linux）
- [ ] macOS: signed + notarized
- [ ] Windows: Authenticode signed
- [ ] 移动端: .ipa + .aab signed
- [ ] 更新签名生成 (TAURI_SIGNING_PRIVATE_KEY)
## Deploy
- [ ] Update JSON 上传（签名 + URL 正确）
- [ ] GitHub Release 创建（artifacts + checksums）
- [ ] Canary 5% 开始
## Post-Release
- [ ] Sentry 无新增崩溃
- [ ] 24h 后 Canary → 25% → 100%
```

## Crate Dependencies

```toml
[dependencies]
tauri = "2"
serde = { version = "1", features = ["derive"] }
thiserror = "2"
anyhow = "1"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
tokio = { version = "1", features = ["full"] }
validator = { version = "0.18", features = ["derive"] }

# Database (desktop)
rusqlite = { version = "0.31", features = ["bundled"] }
refinery = { version = "0.8", features = ["rusqlite"] }

# Axum (if REST API needed)
axum = "0.7"
jsonwebtoken = "9"
tower-http = { version = "0.6", features = ["cors", "trace", "timeout"] }

# Security
argon2 = "0.5"
zeroize = { version = "1", features = ["derive"] }

[dev-dependencies]
mockall = "0.13"
rstest = "0.23"
proptest = "1"
criterion = { version = "0.5", features = ["html_reports"] }
```

## References

| Topic | File |
|-------|------|
| JWT 认证 | [auth.md](references/auth.md) |
| 错误码目录 | [error-codes.md](references/error-codes.md) |
| BAD/GOOD API 对比 | [bad-good-comparisons.md](references/bad-good-comparisons.md) |
| 测试架构 | [arch-testing.md](references/arch-testing.md) |
| 数据库选型 | [db-selection.md](references/db-selection.md) |
| 数据库迁移 | [database-migration.md](references/database-migration.md) |
| 构建优化 | [build-optimization.md](references/build-optimization.md) |
| CI/CD | [ci-cd.md](references/ci-cd.md) |
| 自动更新 | [auto-update.md](references/auto-update.md) |
| 故障排除 | [troubleshooting.md](references/troubleshooting.md) |

## Templates

| 用途 | 文件 |
|------|------|
| 单个 Command 模板 | [tauri-command.rs](templates/tauri-command.rs) |
| Axum Endpoint 模板 | [axum-endpoint.rs](templates/axum-endpoint.rs) |
| JWT 认证实现 | [jwt-auth.rs](templates/jwt-auth.rs) |
| 分页列表（Cursor-based） | [paged-list.rs](templates/paged-list.rs) |
| 测试辅助（Tauri + Axum） | [test-helpers.rs](templates/test-helpers.rs) |
| 环境配置 | [config.rs](templates/config.rs) |
