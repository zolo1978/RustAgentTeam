# 全局热键 — tauri-plugin-global-shortcut

## 概述

使用 Tauri 官方插件注册系统级全局快捷键，即使应用不在前台也能响应。

## Cargo.toml

```toml
[dependencies]
tauri-plugin-global-shortcut = "2"
```

## 注册和使用

```rust
use tauri_plugin_global_shortcut::{
    Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState,
};

pub fn setup_shortcuts(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    let shortcut = Shortcut::new(
        Some(Modifiers::SUPER | Modifiers::SHIFT),
        Code::KeyV,
    );

    app.plugin(
        tauri_plugin_global_shortcut::Builder::new()
            .with_handler(move |app, shortcut, event| {
                if event.state == ShortcutState::Pressed {
                    toggle_window(app);
                }
            })
            .build(),
    )?;

    app.global_shortcut().register(shortcut)?;
    Ok(())
}

fn toggle_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
        } else {
            let _ = window.show();
            let _ = window.set_focus();
        }
    }
}
```

## 快捷键格式

| 修饰键 | macOS | Windows/Linux |
|--------|-------|--------------|
| SUPER | Cmd | Win |
| ALT | Option | Alt |
| SHIFT | Shift | Shift |
| CONTROL | Control | Ctrl |

**推荐快捷键：** `Cmd/Ctrl + Shift + V`（剪贴板管理器标配）

## 用户自定义

```rust
// 从 Store 读取用户配置的快捷键
fn parse_user_shortcut(key_str: &str) -> Result<Shortcut, AppError> {
    // key_str 格式: "CommandOrControl+Shift+V"
    let parts: Vec<&str> = key_str.split('+').collect();
    let mut modifiers = Modifiers::empty();
    let mut code = None;

    for part in parts {
        match part.trim() {
            "CommandOrControl" | "CmdOrCtrl" => modifiers |= Modifiers::SUPER,
            "Shift" => modifiers |= Modifiers::SHIFT,
            "Alt" => modifiers |= Modifiers::ALT,
            "V" => code = Some(Code::KeyV),
            "C" => code = Some(Code::KeyC),
            _ => {}
        }
    }

    Ok(Shortcut::new(
        if modifiers.is_empty() { None } else { Some(modifiers) },
        code.ok_or(AppError::Config("无效快捷键".into()))?,
    ))
}
```

## Capabilities 配置

```json
{
  "permissions": [
    "global-shortcut:allow-register",
    "global-shortcut:allow-unregister",
    "global-shortcut:allow-is-registered"
  ]
}
```

## 冲突检测

```rust
// 注册前检查
if app.global_shortcut().is_registered(shortcut)? {
    return Err(AppError::Config("快捷键已被占用".into()));
}
```
