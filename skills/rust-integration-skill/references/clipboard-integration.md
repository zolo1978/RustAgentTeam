# 系统剪贴板集成 — arboard

## 概述

使用 `arboard` crate 读写系统剪贴板。必须通过 `spawn_blocking` + `catch_unwind` + `timeout` 三层封装。

## Cargo.toml

```toml
[dependencies]
arboard = "3"
tokio = { version = "1", features = ["full"] }
```

## 安全封装

```rust
use arboard::Clipboard;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::time::Duration;
use tokio::time::timeout;

async fn clipboard_read() -> Result<String, AppError> {
    timeout(Duration::from_secs(3), async {
        tokio::task::spawn_blocking(|| {
            catch_unwind(AssertUnwindSafe(|| {
                let mut cb = Clipboard::new()
                    .map_err(|e| AppError::Clipboard(e.to_string()))?;
                cb.get_text()
                    .map_err(|e| AppError::Clipboard(e.to_string()))
            }))
            .map_err(|_| AppError::Clipboard("clipboard panicked".into()))?
        })
        .await
        .map_err(|e| AppError::Clipboard(format!("join error: {e}")))?
    })
    .await
    .map_err(|_| AppError::Clipboard("clipboard timeout".into()))?
}

async fn clipboard_write(text: String) -> Result<(), AppError> {
    timeout(Duration::from_secs(3), async {
        tokio::task::spawn_blocking(move || {
            catch_unwind(AssertUnwindSafe(|| {
                let mut cb = Clipboard::new()
                    .map_err(|e| AppError::Clipboard(e.to_string()))?;
                cb.set_text(text)
                    .map_err(|e| AppError::Clipboard(e.to_string()))
            }))
            .map_err(|_| AppError::Clipboard("clipboard panicked".into()))?
        })
        .await
        .map_err(|e| AppError::Clipboard(format!("join error: {e}")))?
    })
    .await
    .map_err(|_| AppError::Clipboard("clipboard timeout".into()))?
}
```

## 剪贴板监听（轮询模式）

```rust
use tokio::sync::mpsc;

pub fn start_clipboard_monitor(tx: mpsc::Sender<String>) {
    tokio::spawn(async move {
        let mut last_hash = String::new();
        loop {
            tokio::time::sleep(Duration::from_millis(500)).await;
            if let Ok(text) = clipboard_read().await {
                let hash = format!("{:x}", md5::compute(&text));
                if hash != last_hash && !text.is_empty() {
                    last_hash = hash;
                    let _ = tx.send(text).await;
                }
            }
        }
    });
}
```

## 平台差异

| 特性 | macOS | Windows | Linux |
|------|-------|---------|-------|
| 文本读写 | ✅ | ✅ | ✅ |
| 图片读写 | ✅ | ✅ | ✅ |
| 剪贴板历史 | ❌ 无 | ✅ | ❌ |
| 并发安全 | ✅ 自动 | ✅ 自动 | ⚠️ 需 GDK |

## 检测命令

```bash
# 检查 arboard 使用方式
rg -n 'Clipboard::new\|get_text\|set_text' src-tauri/src/
# 检查安全封装
rg -n 'spawn_blocking\|catch_unwind\|timeout' src-tauri/src/
```
