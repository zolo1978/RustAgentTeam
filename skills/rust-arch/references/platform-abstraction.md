# 平台差异抽象模式

## 分文件组织模式

按平台拆分实现文件，通过 mod.rs 统一导出。

```
src-tauri/src/platform/
    mod.rs              # 公共 trait + cfg re-export
    macos.rs            # macOS 实现
    windows.rs          # Windows 实现
    linux.rs            # Linux 实现 (可选，cfg(target_os = "linux"))
```

```rust
// src-tauri/src/platform/mod.rs
pub trait Clipboard: Send + Sync + 'static {
    fn read_text(&self) -> Result<Option<String>>;
    fn write_text(&self, text: &str) -> Result<()>;
    fn read_image(&self) -> Result<Option<Vec<u8>>>;
    fn write_image(&self, data: &[u8], width: u32, height: u32) -> Result<()>;
}

pub trait Hotkey: Send + Sync + 'static {
    fn register(&self, key: &str, callback: Box<dyn Fn() + Send>) -> Result<()>;
    fn unregister(&self, key: &str) -> Result<()>;
}

// cfg re-export：编译时只引入当前平台的实现
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::{MacClipboard, MacHotkey};

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use windows::{WinClipboard, WinHotkey};

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::{LinuxClipboard, LinuxHotkey};

// 工厂函数：运行时获取当前平台实例
pub fn clipboard() -> Box<dyn Clipboard> {
    #[cfg(target_os = "macos")]
    { Box::new(macos::MacClipboard::new()) }
    #[cfg(target_os = "windows")]
    { Box::new(windows::WinClipboard::new()) }
    #[cfg(target_os = "linux")]
    { Box::new(linux::LinuxClipboard::new()) }
}

pub fn hotkey() -> Box<dyn Hotkey> {
    #[cfg(target_os = "macos")]
    { Box::new(macos::MacHotkey::new()) }
    #[cfg(target_os = "windows")]
    { Box::new(windows::WinHotkey::new()) }
    #[cfg(target_os = "linux")]
    { Box::new(linux::LinuxHotkey::new()) }
}
```

## macOS 实现（示例）

```rust
// src-tauri/src/platform/macos.rs
use super::{Clipboard, Hotkey};
use anyhow::Result;

pub struct MacClipboard;

impl MacClipboard {
    pub fn new() -> Self {
        Self
    }
}

impl Clipboard for MacClipboard {
    fn read_text(&self) -> Result<Option<String>> {
        // 使用 cocoa/appkit 或 tauri-plugin-clipboard-manager
        // 此处为架构示例，实际用 plugin 或 NSPasteboard FFI
        let output = std::process::Command::new("pbpaste")
            .output()?;
        if output.stdout.is_empty() {
            Ok(None)
        } else {
            Ok(Some(String::from_utf8(output.stdout)?))
        }
    }

    fn write_text(&self, text: &str) -> Result<()> {
        std::process::Command::new("pbcopy")
            .stdin(std::process::Stdio::piped())
            .spawn()?
            .stdin
            .take()
            .map(|mut stdin| {
                use std::io::Write;
                let _ = stdin.write_all(text.as_bytes());
            });
        Ok(())
    }

    fn read_image(&self) -> Result<Option<Vec<u8>>> {
        // macOS: NSPasteboard -> NSImage -> PNG data
        // 实际实现用 objc2-app-kit 或类似 crate
        Ok(None)
    }

    fn write_image(&self, _data: &[u8], _width: u32, _height: u32) -> Result<()> {
        // macOS: NSImage -> NSPasteboard
        Ok(())
    }
}

pub struct MacHotkey;

impl MacHotkey {
    pub fn new() -> Self {
        Self
    }
}

impl Hotkey for MacHotkey {
    fn register(&self, _key: &str, _callback: Box<dyn Fn() + Send>) -> Result<()> {
        // macOS: 使用 CGEventTap 或 tauri-plugin-global-shortcut
        Ok(())
    }

    fn unregister(&self, _key: &str) -> Result<()> {
        Ok(())
    }
}
```

## 条件编译最佳实践

### 规则 1：用模块级别 cfg，不用函数级别

```rust
// BAD：每个函数都包 cfg，散乱难维护
#[cfg(target_os = "macos")]
fn get_clipboard() -> ... { /* macos */ }
#[cfg(target_os = "windows")]
fn get_clipboard() -> ... { /* windows */ }

// GOOD：整个文件一个 cfg，通过 trait 多态分发
// mod.rs 统一导出，调用方只看到 trait
```

### 规则 2：测试也分平台

```rust
#[cfg(test)]
#[cfg(target_os = "macos")]
mod tests {
    use super::*;
    #[test]
    fn test_mac_clipboard_roundtrip() { /* ... */ }
}
```

### 规则 3：共享逻辑放 common 模块

```rust
// src-tauri/src/platform/common.rs
// 跨平台共享的辅助函数，不被 cfg 限制
pub fn normalize_hotkey_str(key: &str) -> String {
    key.to_lowercase().replace("cmd", "super")
}
```

### 规则 4：避免在 trait 定义中使用平台特定类型

```rust
// BAD：trait 依赖 macOS 类型
pub trait Clipboard {
    fn get_pasteboard(&self) -> objc2_app_kit::NSPasteboard; // 错误
}

// GOOD：trait 只用标准类型
pub trait Clipboard {
    fn read_text(&self) -> Result<Option<String>>; // 正确
}
```

## Cargo.toml 平台依赖配置

```toml
# src-tauri/Cargo.toml

[dependencies]
# 跨平台依赖
tauri = "2"
serde = { version = "1", features = ["derive"] }
anyhow = "1"

# macOS 专用依赖
[target.'cfg(target_os = "macos")'.dependencies]
objc2 = "0.6"
objc2-app-kit = { version = "0.3", features = ["NSPasteboard"] }
objc2-foundation = { version = "0.3", features = ["NSString", "NSData"] }
cocoa = "0.26"

# Windows 专用依赖
[target.'cfg(target_os = "windows")'.dependencies]
windows = { version = "0.58", features = [
    "Win32_System_DataExchange",  # Clipboard API
    "Win32_UI_Input_KeyboardAndMouse",
] }

# Linux 专用依赖
[target.'cfg(target_os = "linux")'.dependencies]
x11-clipboard = "0.9"

# 可选：用 features 控制平台编译
[features]
default = []
platform-macos = []
platform-windows = []
platform-linux = []
```

## 完整示例：剪贴板访问的平台抽象

### Service 层（平台无关）

```rust
// src-tauri/src/services/clipboard_service.rs
use crate::platform::{clipboard, Clipboard};
use crate::models::ClipItem;
use anyhow::Result;
use std::sync::Arc;

pub struct ClipboardService {
    platform: Box<dyn Clipboard>,
}

impl ClipboardService {
    pub fn new() -> Self {
        Self {
            platform: clipboard(), // 工厂函数返回当前平台实现
        }
    }

    pub async fn read_current(&self) -> Result<Option<ClipItem>> {
        let text = self.platform.read_text()?;
        Ok(text.map(|content| ClipItem::new_text(content)))
    }

    pub async fn paste(&self, clip: &ClipItem) -> Result<()> {
        match &clip.content {
            crate::models::ClipContent::Text(text) => {
                self.platform.write_text(text)?;
            }
            crate::models::ClipContent::Image(data) => {
                self.platform.write_image(&data.bytes, data.width, data.height)?;
            }
        }
        Ok(())
    }
}
```

### Command 层（平台无关）

```rust
// src-tauri/src/commands/clipboard.rs
use crate::error::AppError;
use crate::services::ClipboardService;
use crate::models::ClipItem;
use tauri::State;

#[tauri::command]
pub async fn read_clipboard(
    state: State<'_, crate::state::AppState>,
) -> Result<Option<ClipItem>, AppError> {
    state.clipboard.read_current().await.map_err(AppError::from)
}

#[tauri::command]
pub async fn paste_clip(
    clip_id: String,
    state: State<'_, crate::state::AppState>,
) -> Result<(), AppError> {
    let clip = state.clip_svc.get(&clip_id).await?;
    state.clipboard.paste(&clip).await?;
    Ok(())
}
```

### AppState 注入

```rust
// src-tauri/src/state.rs
use crate::services::ClipboardService;
use std::sync::Arc;

pub struct AppState {
    pub clipboard: Arc<ClipboardService>,
    // ... 其他 services
}

impl AppState {
    pub fn new() -> Self {
        Self {
            clipboard: Arc::new(ClipboardService::new()),
        }
    }
}
```

## 决策树：平台抽象方案选择

```
需要平台特定功能？
  |
  +--> Tauri 官方插件已覆盖？ --> 用插件（clipboard-manager, global-shortcut 等）
  |      优点：维护成本低，API 稳定
  |      缺点：功能受限，可能不满足定制需求
  |
  +--> 需要原生 API 细粒度控制？ --> 自定义 trait + cfg 分文件
  |      优点：完全控制，可做平台优化
  |      缺点：需为每个平台写实现
  |
  +--> 功能简单，shell 命令可搞定？ --> std::process::Command + cfg
  |      优点：零依赖
  |      缺点：性能差，错误处理粗糙
  |      适用：原型开发、非性能敏感场景
  |
  +--> 仅移动端 vs 桌面端差异？ --> capabilities platforms 过滤 + 前端条件渲染
         不需要 Rust 侧抽象，在 JSON 和 UI 层处理
```
