---
name: rust-arch
description: 'Tauri v2 架构设计：分层架构、Capabilities ACL、移动端适配、IPC 通信模式。面向架构师 Agent。决策树格式。'
type: skill
---

# Tauri v2 架构设计

## Quick Start
1. 新项目？Copy `templates/error.rs` + `templates/state.rs` + `templates/lib.rs` → 骨架就绪
2. 要权限？Copy capabilities JSON 模板（见 Section 2）
3. 移动端？看 Section 3 生命周期差异表
4. 流式数据？Copy `templates/commands.rs` 中的 Channel 示例

## 适用范围
**适用于：** Tauri v2 架构决策——分层设计、权限模型、IPC 模式、移动端适配、脚手架搭建。
**不适用于：** Rust 核心开发模式（见 `rust-core`）、后端实现细节（见 `rust-backend`）、前端 UI（见 `rust-frontend`）、PRD（见 `rust-prd-skill`）。
**v1 用户：** 本 Skill 仅覆盖 Tauri v2。v1→v2 迁移请参考 [官方迁移指南](https://v2.tauri.app/start/migrate/)（核心差异：allowlist→Capabilities ACL、`tauri::api`→plugin 系统、IPC Channel 新 API）。
**目标：** Tauri >= 2.0.0, Rust 1.80+ MSRV。

## 1. 架构

**推荐方案：** 分层架构——Rust Service 层做业务逻辑，Tauri Command 和 Axum Endpoint 各自是薄适配器。
**理由：** 逻辑在 Rust 层可测试、可复用，前端只做 UI 展示，API 只做传输。
**怎么做：**

```
Frontend (WebView) → Tauri IPC → Rust Core (Commands+Tokio) → Plugins/OS
                                  ↘ Axum REST (if needed)
```

```rust
// Service 层（纯逻辑，与 Tauri/Axum 无关）
pub async fn create_order(req: CreateOrderReq, db: &Db) -> Result<Order, AppError> {
    // 业务逻辑
}

// Tauri Command（薄适配器）
#[tauri::command]
async fn create_order(req: CreateOrderReq, state: State<'_, AppState>) -> Result<Order, AppError> {
    state.order_svc.create_order(req, &state.db).await
}

// Axum Endpoint（薄适配器）
async fn create_order(
    Json(req): Json<CreateOrderReq>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<Order>>, AppError> {
    let order = state.order_svc.create_order(req, &state.db).await?;
    Ok(Json(ApiResponse::success(order)))
}
```

**例外：** 纯桌面应用不需要 Axum 层，只用 Tauri Command。

**子决策：**
| 场景 | 方案 |
|------|------|
| 纯桌面应用 | Service + Tauri Command，不需要 Axum |
| 桌面 + REST API | Service + Tauri Command + Axum Endpoint（共享 Service 层） |
| 多窗口 | 每个窗口独立 Capabilities，共享一个 AppState（见下方示例） |
| 大型应用 | 按领域拆分 Service（见下方目录结构） |

**多窗口 Capabilities 示例：**

```json
// capabilities/main.json — 主窗口
{
  "identifier": "main-cap",
  "windows": ["main"],
  "permissions": ["core:default", "dialog:allow-open", "fs:allow-read-text-file"]
}

// capabilities/editor.json — 编辑器窗口（更宽松的文件权限）
{
  "identifier": "editor-cap",
  "windows": ["editor"],
  "permissions": [
    "core:default",
    { "identifier": "fs:allow-read-text-file", "allow": [{ "path": "$HOME/**" }] },
    { "identifier": "fs:allow-write-text-file", "allow": [{ "path": "$HOME/**" }] }
  ]
}

// capabilities/settings.json — 设置窗口（最小权限）
{
  "identifier": "settings-cap",
  "windows": ["settings"],
  "permissions": ["core:default", "store:allow-get", "store:allow-set"]
}
```

**大型应用目录结构：**

```
src-tauri/src/
├── commands/
│   ├── mod.rs
│   ├── order.rs        # 订单领域 Command
│   ├── auth.rs         # 认证 Command
│   └── settings.rs     # 设置 Command
├── models/
│   ├── mod.rs
│   ├── order.rs        # Order + OrderDto + CreateOrderReq
│   ├── user.rs         # User + UserDto
│   └── common.rs       # PagedResult, ApiResponse
├── services/
│   ├── mod.rs
│   ├── order.rs        # OrderService（纯逻辑）
│   ├── auth.rs         # AuthService
│   └── sync.rs         # SyncService（后台同步）
├── repositories/
│   ├── mod.rs
│   ├── order.rs        # spawn_blocking 包装
│   └── user.rs
├── error.rs
├── state.rs
├── lib.rs
└── main.rs
```

→ 脚手架模板：`templates/error.rs`, `templates/state.rs`, `templates/lib.rs`

## 2. 权限与安全

**推荐方案：** Tauri v2 Capabilities ACL——每个窗口显式声明最小权限，永不 `core:all`。安全敏感应用启用 Isolation Pattern。
**理由：** v2 默认无 IPC 权限，显式声明是最小权限原则的直接实现。
**怎么做：**

```json
// src-tauri/capabilities/default.json
{
  "identifier": "default",
  "windows": ["main"],
  "permissions": ["core:default", "dialog:allow-open", "fs:allow-read-text-file"]
}
```

```json
// tauri.conf.json CSP
{
  "security": {
    "csp": "default-src 'self'; script-src 'self'",
    "freezePrototype": true
  }
}
```

**例外：** 开发阶段可以放宽 CSP，但发布前必须收紧。

**子决策：**
| 场景 | 方案 |
|------|------|
| 普通应用 | Capabilities JSON + CSP |
| 处理敏感数据（密码管理器等） | 加 Isolation Pattern（`build > beforeBuildCommand` 分离） |
| 第三方内容嵌入 | `dangerousDisableAssetCorsModification: false` + 严格 CSP |
| 移动端 | Capabilities 按 `platform: ["iOS", "android"]` 过滤 |

→ 深入：[Capabilities ACL + Isolation](references/arch-capabilities-acl.md)

## 3. 移动端

**推荐方案：** 移动端与桌面共享 Rust Core，但生命周期和平台 API 不同。原子操作 + 本地持久化，不依赖后台长驻。
**理由：** iOS/Android 会挂起/杀死后台应用。必须在操作返回前持久化。
**怎么做：**

```rust
#[tauri::command]
async fn save_draft(draft: Draft, state: State<'_, AppState>) -> Result<(), AppError> {
    state.db.save_draft(&draft).await?;  // 立即持久化
    let _ = state.sync_client.push(&draft).await;  // 同步尽力而为
    Ok(())
}
```

**子决策：**

| 桌面方案 | 移动替代 |
|----------|---------|
| Tauri Updater | App Store / Play Store |
| 常驻后台 | 用户触发 + 本地队列 |
| WebSocket 长连接 | 前台长连接 + 推送唤醒 |
| 任意文件路径 | 系统 Picker（沙盒限制） |
| window.confirm | `@tauri-apps/plugin-dialog` |

## 4. IPC 通信

**推荐方案：** 批量 Command（一次 IPC 传完整数据），不用逐字段 invoke。流式数据用 `tauri::ipc::Channel<T>`。
**理由：** 每次 invoke 有序列化开销，批量减少往返次数。Channel 避免前端轮询。
**怎么做：**

```rust
// BAD: 每个字段一次 invoke
// await invoke("update_name", { name });
// await invoke("update_email", { email });

// GOOD: 批量提交
#[tauri::command]
async fn update_profile(
    req: UpdateProfileReq,
    state: State<'_, AppState>,
) -> Result<Profile, AppError> {
    state.profile_svc.update(req).await
}
```

**Channel 背压：** 高频流式场景（日志、传感器数据）Rust 端需要限速，否则内部缓冲区膨胀。

```rust
#[tauri::command]
async fn stream_sensor(ch: tauri::ipc::Channel<SensorData>) -> Result<(), AppError> {
    let mut interval = tokio::time::interval(Duration::from_millis(100)); // 10Hz 限速
    loop {
        interval.tick().await;
        let data = read_sensor()?;
        ch.send(data).map_err(|e| AppError::Internal(e.to_string()))?;
    }
}
```

**平台差异化权限：**

```json
// capabilities/mobile.json — 移动端独有权限
{
  "identifier": "mobile-cap",
  "windows": ["main"],
  "permissions": ["core:default", "biometric:allow-authenticate", "nfc:allow-scan"],
  "platforms": ["iOS", "android"]
}
// capabilities/desktop.json — 桌面端独有权限
{
  "identifier": "desktop-cap",
  "windows": ["main"],
  "permissions": ["core:default", "fs:allow-read-text-file", "shell:allow-open"],
  "platforms": ["macOS", "windows", "linux"]
}
```

→ 深入：[IPC Channel 流式](references/arch-ipc-channel.md)

## 5. 接口契约模板

**推荐方案：** 每个 Tauri Command 实现前定义完整契约——参数约束、返回类型、错误码、跨语言类型映射。
**理由：** 前后端并行开发需要明确的接口约定；类型不匹配是 IPC bug 的首要来源。
**怎么做：**

```rust
// Rust：带 serde rename_all 的请求/响应 struct
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListClipsReq {
    pub query: Option<String>,       // 可选，最大 200 字符
    pub clip_type: Option<ClipType>,  // 可选，null = 全部
    pub page: u32,                    // 必填，>= 1
    pub page_size: u32,               // 必填，1..=100
}

#[tauri::command]
async fn list_clips(
    req: ListClipsReq,
    state: State<'_, AppState>,
) -> Result<PagedResult<ClipItem>, AppError> {
    state.clip_svc.list(req).await
}
```

```typescript
// TypeScript：与 Rust struct 严格对应的 interface
interface ListClipsReq {
  query?: string | null;
  clipType?: ClipType | null;
  page: number;
  pageSize: number;
}
```

**例外：** 一次性原型可以跳过契约文档，但进入迭代前必须补齐。

**子决策：**
| 场景 | 方案 |
|------|------|
| 新增 Command | 先写契约表头，再实现 Rust，最后生成 TS interface |
| 修改已有 Command | 先更新契约，确认向后兼容，再改代码 |
| 复杂参数（>4 字段） | 封装为 Request struct，不要在 Command 参数中逐个罗列 |
| 错误处理 | 用 thiserror enum + serde tag，前端 match error.code |

→ 完整模板：[ipc-contract.md](templates/ipc-contract.md)

## 6. IPC 类型映射表

**推荐方案：** Rust struct 统一 `#[serde(rename_all = "camelCase")]`，TypeScript interface 逐字段手写保持同步。用决策树处理边界情况。
**理由：** 自动生成工具（ts-rs 等）引入构建依赖，手写维护成本低且完全可控。
**怎么做：**

核心映射速查：
```
String <-> string        u32 <-> number        bool <-> boolean
Vec<T> <-> T[]           Option<T> <-> T | null
HashMap<String,V> <-> Record<string,V>
chrono::DateTime (ts_milliseconds) <-> number
Vec<u8> (base64) <-> string
```

**例外：** 大型项目（50+ Command）考虑 `ts-rs` 或 `specta` 自动生成 TS 类型，减少手动同步负担。

**子决策：**
| 场景 | 方案 |
|------|------|
| 枚举无数据 | `#[serde(rename_all = "camelCase")]` -> TS union (`"text" \| "image"`) |
| 枚举有数据 | `#[serde(tag = "type")]` -> TS discriminated union |
| 大整数 (>2^53) | 序列化为 string，前端 BigInt 或保持 string |
| 嵌套 struct | 每层独立加 `rename_all = "camelCase"`，不继承 |

```rust
// 枚举映射示例
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ClipType { Text, Image, File, Url }
// TS: type ClipType = "text" | "image" | "file" | "url"
```

→ 完整映射表 + 决策树：[ipc-mapping-table.md](references/ipc-mapping-table.md)

## 7. 平台抽象模式

**推荐方案：** 自定义 trait + cfg(target_os) 分文件 + 工厂函数。优先用 Tauri 官方插件覆盖基础场景，原生 API 需求才自定义。
**理由：** trait 抽象保证 Command/Service 层平台无关；cfg 分文件让每个平台实现独立编译，互不干扰。
**怎么做：**

```
src-tauri/src/platform/
    mod.rs              # trait 定义 + cfg re-export + 工厂函数
    macos.rs            # macOS 实现（仅 cfg(target_os = "macos") 编译）
    windows.rs          # Windows 实现
```

```rust
// mod.rs — 调用方只看到 trait，不知道底层平台
pub trait Clipboard: Send + Sync + 'static {
    fn read_text(&self) -> Result<Option<String>>;
    fn write_text(&self, text: &str) -> Result<()>;
}

pub fn clipboard() -> Box<dyn Clipboard> {
    #[cfg(target_os = "macos")]
    { Box::new(macos::MacClipboard::new()) }
    #[cfg(target_os = "windows")]
    { Box::new(windows::WinClipboard::new()) }
}
```

**例外：** 仅移动端 vs 桌面端 UI 差异（非 API 差异）不需要 Rust 抽象，在 Capabilities JSON 的 `platforms` 字段 + 前端条件渲染即可。

**子决策：**
| 场景 | 方案 |
|------|------|
| 剪贴板/快捷键/通知 | 先查 Tauri 插件，有就用插件 |
| 需要原生 API 细粒度控制 | 自定义 trait + cfg 分文件 |
| 功能简单，shell 可搞定 | `std::process::Command` + cfg（仅限原型） |
| Cargo.toml 依赖 | `[target.'cfg(target_os = "macos")'.dependencies]` 隔离 |

→ 完整模式 + 示例：[platform-abstraction.md](references/platform-abstraction.md)

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

# Security
argon2 = "0.5"
zeroize = { version = "1", features = ["derive"] }
```

## References

| Topic | File |
|-------|------|
| Capabilities ACL + Isolation | [arch-capabilities-acl.md](references/arch-capabilities-acl.md) |
| IPC Channel 流式 | [arch-ipc-channel.md](references/arch-ipc-channel.md) |
| IPC 类型映射表 | [ipc-mapping-table.md](references/ipc-mapping-table.md) |
| 平台抽象模式 | [platform-abstraction.md](references/platform-abstraction.md) |
| Tauri 安全 | [security-tauri.md](references/security-tauri.md) |
| WebSocket | [websocket.md](references/websocket.md) |

## Templates

| 用途 | 文件 |
|------|------|
| 错误类型定义 | [error.rs](templates/error.rs) |
| 应用状态 | [state.rs](templates/state.rs) |
| lib.rs 入口 | [lib.rs](templates/lib.rs) |
| Command 脚手架 + 流式日志 | [commands.rs](templates/commands.rs) |
| IPC 接口契约 | [ipc-contract.md](templates/ipc-contract.md) |
