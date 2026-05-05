---
name: rust-async-patterns
description: 'Tauri v2 异步模式：spawn_blocking 时机、Mutex 选型、避免 blocking_lock、临时文件安全、竞态防护。决策树格式——给结论，不列菜单。'
type: skill
tools: ['Read', 'Glob', 'Grep', 'Bash', 'Edit', 'Write']
---

# Rust 异步模式（Tauri v2）

## 决策树

```
遇到异步问题？
├─ 阻塞操作在 async fn 中 → 用 spawn_blocking 包装
├─ 需要锁？ → 选 std::sync::Mutex（DB 连接等 Send 资源）
│            └─ 绝不用 tokio::sync::Mutex + blocking_lock
├─ 多个异步操作共享状态 → 用 AtomicBool/AtomicPtr 或 channel
│                      └─ 不用 Mutex<bool>
├─ 临时文件 → 用 tempfile crate 或 0o600 权限 + 注册清理
├─ 剪贴板/系统 API → 总是 spawn_blocking（macOS pasteboard 是同步的）
└─ 读写文件 → tokio::fs（不要在 async 中用 std::fs）
```

---

## 1. spawn_blocking 使用时机

### 规则：凡是在 async fn 中调用同步阻塞 API，必须用 spawn_blocking

```rust
// ❌ 错误：阻塞 tokio worker
async fn my_command(state: State<'_, AppState>) -> Result<(), AppError> {
    let mut cb = arboard::Clipboard::new()?;
    let text = cb.get_text()?;
    // ...
}

// ✅ 正确：spawn_blocking 包装
async fn my_command(state: State<'_, AppState>) -> Result<(), AppError> {
    let text = tokio::task::spawn_blocking(|| {
        let mut cb = arboard::Clipboard::new()?;
        cb.get_text()
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))?
    .map_err(|e| AppError::Internal(e.to_string()))?;
    // ...
}
```

### 必须用 spawn_blocking 的操作

| 操作 | 原因 |
|------|------|
| `arboard::Clipboard::new/get/set` | macOS pasteboard 同步调用 |
| `rusqlite::Connection` 操作 | 同步 C 库 |
| `std::fs::*` 文件操作 | 同步系统调用 |
| `std::net::*` 网络（如需要） | 同步 socket |
| `std::thread::sleep` | 阻塞线程 |
| 任何 C FFI 调用 | 通常是同步的 |

### 可以直接 await 的操作

| 操作 | 原因 |
|------|------|
| `tokio::fs::*` | tokio 的异步文件 API |
| `tokio::time::sleep` | 异步定时器 |
| `tokio::process::Command` | 异步进程管理 |
| `reqwest` 等 async HTTP | 天然 async |
| `tauri::AppHandle::emit` | Tauri 异步事件 |

---

## 2. Mutex 选型

### 规则

```
需要锁？
├─ 锁内持有 Send 资源（Connection, HashMap 等）→ std::sync::Mutex
│   └─ 在 spawn_blocking 中 lock()
├─ 需要跨 .await 持锁 → tokio::sync::Mutex（罕见，通常是设计问题）
├─ 只是 bool 标志 → std::sync::atomic::AtomicBool
├─ 读多写少 → std::sync::RwLock（非 tokio）
└─ Tauri State 中存储 → 看内容：
    ├─ Connection → std::sync::Mutex<Connection>
    ├─ 配置 → tokio::sync::RwLock<AppConfig>
    └─ 服务实例 → std::sync::Mutex（配合 spawn_blocking）
```

### 绝对禁止

```rust
// ❌ 绝不用 blocking_lock
let mut monitor = state.monitor.blocking_lock();
// ↑ 在 tokio runtime 内调用 = 潜在死锁

// ❌ 不混用 std::Mutex 和 tokio::Mutex 持有同一数据
// ❌ 不在 async fn 中用 std::sync::Mutex.lock().unwrap() 跨 await
```

### 正确模式

```rust
// ✅ std::sync::Mutex + spawn_blocking
pub struct AppState {
    pub db: Arc<std::sync::Mutex<Connection>>,
}

async fn query(state: State<'_, AppState>) -> Result<Vec<Clip>, AppError> {
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || {
        let conn = db.lock().unwrap();
        clip_repo::list_clips(&conn, limit, offset)
    })
    .await?
    .map_err(Into::into)
}
```

---

## 3. 竞态防护

### 常见竞态场景

| 场景 | 防护 |
|------|------|
| paste 时 monitor 同时读剪贴板 | `AtomicBool` 标志：`is_pasting` |
| 多个 command 同时写 DB | `Mutex<Connection>` 已保护 |
| monitor stop 后任务仍在运行 | `oneshot::channel` + `.await` 确认 |
| 配置更新与读取并发 | `RwLock` 已保护 |

### paste-监控竞态修复模板

```rust
// state.rs
pub struct AppState {
    pub is_pasting: std::sync::atomic::AtomicBool,
    // ...
}

// clipboard.rs — paste_clip
async fn paste_clip(id: String, state: State<'_, AppState>) -> Result<(), AppError> {
    state.is_pasting.store(true, Ordering::SeqCst);
    // ... 执行粘贴 ...
    // 延迟恢复，确保目标应用已读取剪贴板
    tokio::time::sleep(Duration::from_millis(200)).await;
    // ... 恢复原剪贴板 ...
    state.is_pasting.store(false, Ordering::SeqCst);
    Ok(())
}

// monitor_service.rs — run_loop
if state.is_pasting.load(Ordering::SeqCst) {
    continue; // 跳过本次采集
}
```

---

## 4. 临时文件安全

### 规则

```
需要临时文件？
├─ 敏感数据（剪贴板内容） → 限制权限 + 确保清理
│   ├─ 用 tempfile::NamedTempFile（自动清理）
│   └─ 或手动：0o600 权限 + 注册清理
├─ 非敏感数据 → tempfile crate 仍然推荐
└─ 绝对不要 → 可预测文件名（时间戳、序号）
```

### 安全临时文件模式

```rust
use std::os::unix::fs::OpenOptionsExt;

// ✅ 安全模式
fn write_temp_image(data: &[u8], id: &str) -> Result<PathBuf, AppError> {
    let tmp = std::env::temp_dir().join(format!("clipvault-preview-{id}.png"));

    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)        // 防止符号链接攻击
        .mode(0o600)             // 仅 owner 可读写
        .open(&tmp)
        .and_then(|mut f| f.write_all(data))?;

    // 注册延迟清理
    let path = tmp.clone();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_secs(30));
        let _ = std::fs::remove_file(&path);
    });

    Ok(tmp)
}

// ❌ 危险模式
let tmp = std::env::temp_dir().join(format!("clipvault-snip-{}.png", timestamp));
// 可预测路径 + 默认权限 + 无清理
```

### 应用退出时统一清理

```rust
// lib.rs setup 中
let tmp_dir = std::env::temp_dir();
app.on_cleanup(move || {
    // 清理所有 clipvault 临时文件
    if let Ok(entries) = std::fs::read_dir(&tmp_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str.starts_with("clipvault-") {
                let _ = std::fs::remove_file(entry.path());
            }
        }
    }
});
```

---

## 5. ?? 双重解包模式

### 规则：禁止 `??`，用显式中间变量

```rust
// ❌ 可读性差，维护者容易误解
let result = tokio::task::spawn_blocking(move || {
    some_fallible_operation()
})
.await??;

// ✅ 显式分步
let join_result = tokio::task::spawn_blocking(move || {
    some_fallible_operation()
})
.await
.map_err(|e| AppError::Internal(format!("task failed: {e}")))?;

join_result.map_err(Into::into)
```

---

## 6. 错误处理检查清单

在 async 代码中遇到以下模式时立即标记：

| 反模式 | 问题 | 修复 |
|--------|------|------|
| `let _ =` 忽略结果 | 静默失败 | 至少 log 错误 |
| `.ok()?` | 吞掉错误上下文 | 用 `map_err` 保留信息 |
| `async fn` 中 `std::fs::*` | 阻塞 worker | 改 `tokio::fs::*` 或 `spawn_blocking` |
| `blocking_lock()` | 死锁风险 | 改 `std::sync::Mutex` + `spawn_blocking` |
| `unwrap()` 在非启动路径 | panic | 用 `?` 或 `map_err` |
| `expect()` 在非启动路径 | panic | 同上 |
| 无超时的 `.await` | 可能永远挂起 | 加 `tokio::time::timeout` |

---

## 7. 检测命令

```bash
# 查找 async fn 中的阻塞调用
rg "std::fs::" --type rust -A 0 src-tauri/src/
rg "std::thread::sleep" --type rust src-tauri/src/
rg "blocking_lock" --type rust src-tauri/src/

# 查找静默错误忽略
rg "let _ =" --type rust src-tauri/src/
rg "\.ok\(\)" --type rust src-tauri/src/

# 查找非启动路径的 unwrap/expect
rg "\.unwrap\(\)" --type rust src-tauri/src/ | grep -v "test"
rg "\.expect(" --type rust src-tauri/src/ | grep -v "lib.rs" | grep -v "main.rs"

# 查找 ?? 模式
rg '\?\?' --type rust src-tauri/src/
```

## 相关 Skill

| Skill | 关联场景 |
|-------|---------|
| [rust-security-skill](../rust-security-skill/SKILL.md) | 临时文件安全、FFI 安全审计 |
| [rust-crash-debug](../rust-crash-debug/skill.md) | spawn_blocking panic、block_in_place 上下文错误 |
| [rust-core](../rust-core/SKILL.md) | 临时文件 5 项约束、Arc<Mutex> 选型 |
| [rust-tauri-testing](../rust-tauri-testing/skill.md) | async 函数测试、spawn_blocking mock |
