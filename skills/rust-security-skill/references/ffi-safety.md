# FFI 安全 — arboard / enigo 外部 crate

## 概述

ClipVault 使用 arboard（系统剪贴板）和 enigo（键盘模拟）等 crate。这些 crate 内部通过 FFI 调用系统 API，可能 panic 或挂起。

## 风险矩阵

| Crate | 操作 | 风险 | 概率 |
|-------|------|------|------|
| arboard | Clipboard::new() | 系统剪贴板不可用 | 低 |
| arboard | get_text() | 剪贴板数据格式异常 | 中 |
| enigo | key_press() | 无焦点窗口 | 中 |
| arboard | set_text() | 数据过大 | 低 |

## 安全模式：三层防护

### 1. catch_unwind — 防 panic

```rust
use std::panic::catch_unwind;
use std::panic::AssertUnwindSafe;

// BAD — 无 panic 处理
let text = clipboard.get_text()?;

// GOOD — catch_unwind 包裹
let result = catch_unwind(AssertUnwindSafe(|| {
    clipboard.get_text()
})).map_err(|_| AppError::Clipboard("clipboard operation panicked".into()))?;
```

### 2. spawn_blocking — 防阻塞 async runtime

```rust
// BAD — 在 async 中直接调用 FFI（阻塞 tokio 线程）
async fn read_clipboard() -> Result<String> {
    let mut cb = Clipboard::new()?;
    Ok(cb.get_text()?)
}

// GOOD — spawn_blocking 隔离
async fn read_clipboard() -> Result<String> {
    tokio::task::spawn_blocking(|| {
        let mut cb = Clipboard::new()
            .map_err(|e| AppError::Clipboard(e.to_string()))?;
        cb.get_text()
            .map_err(|e| AppError::Clipboard(e.to_string()))
    })
    .await
    .map_err(|e| AppError::Clipboard(format!("blocking task: {e}")))?
}
```

### 3. timeout — 防挂起

```rust
use tokio::time::{timeout, Duration};

async fn read_clipboard_safe() -> Result<String, AppError> {
    timeout(Duration::from_secs(2), async {
        tokio::task::spawn_blocking(|| {
            catch_unwind(AssertUnwindSafe(|| {
                let mut cb = Clipboard::new()
                    .map_err(|e| AppError::Clipboard(e.to_string()))?;
                cb.get_text()
                    .map_err(|e| AppError::Clipboard(e.to_string()))
            }))
            .map_err(|_| AppError::Clipboard("panicked".into()))?
        })
        .await
        .map_err(|e| AppError::Clipboard(format!("join: {e}")))?
    })
    .await
    .map_err(|_| AppError::Clipboard("timeout".into()))?
}
```

## 完整安全封装

```rust
/// 安全的 FFI 调用封装
pub async fn safe_ffi<F, T>(label: &str, f: F) -> Result<T, AppError>
where
    F: FnOnce() -> Result<T, AppError> + Send + 'static,
    T: Send + 'static,
{
    timeout(Duration::from_secs(3), async {
        tokio::task::spawn_blocking(move || {
            catch_unwind(AssertUnwindSafe(f))
                .map_err(|_| AppError::FFI(format!("{label} panicked")))?
        })
        .await
        .map_err(|e| AppError::FFI(format!("{label} join: {e}")))?
    })
    .await
    .map_err(|_| AppError::FFI(format!("{label} timeout")))?
}
```

## 资源管理

- `arboard::Clipboard` 持有系统句柄 → 短生命周期，用完即丢
- `enigo::Enigo` 持有系统连接 → 单例 + `Mutex`
- 不需要手动 Drop（Rust RAII 自动处理）

## 检测命令

```bash
# 检查 FFI 调用是否有 catch_unwind
rg -n 'Clipboard::new\|Enigo::new' src-tauri/src/

# 检查是否有 spawn_blocking
rg -n 'spawn_blocking' src-tauri/src/

# 检查是否有超时
rg -n 'timeout\|Duration' src-tauri/src/
```
