# 系统托盘 — TrayIconBuilder

## 概述

Tauri v2 内置 TrayIcon 支持，不需要额外插件。

## tauri.conf.json 配置

```json
{
  "app": {
    "trayIcon": {
      "iconPath": "icons/icon.png",
      "iconAsTemplate": true
    }
  }
}
```

- macOS: `iconAsTemplate: true` 使用 template icon（自动适配亮暗色）
- Windows: 使用 `.ico` 格式

## Rust 创建托盘

```rust
use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder},
    Manager,
};

pub fn setup_tray(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    let show_item = MenuItemBuilder::with_id("show", "显示 ClipVault").build(app)?;
    let prefs_item = MenuItemBuilder::with_id("prefs", "偏好设置...").build(app)?;
    let quit_item = MenuItemBuilder::with_id("quit", "退出 ClipVault").build(app)?;

    let menu = MenuBuilder::new(app)
        .items(&[&show_item, &prefs_item])
        .separator()
        .items(&[&quit_item])
        .build()?;

    TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .tooltip("ClipVault — 剪贴板管理器")
        .menu(&menu)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => {
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.show();
                    let _ = w.set_focus();
                }
            }
            "prefs" => {
                // 打开偏好设置
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            // macOS: 单击显示窗口
            if let tauri::tray::TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.show();
                    let _ = w.set_focus();
                }
            }
        })
        .build(app)?;

    Ok(())
}
```

## 平台差异

| 特性 | macOS | Windows | Linux |
|------|-------|---------|-------|
| 左键点击 | 触发事件 | 触发事件 | 触发事件 |
| 右键点击 | 显示菜单 | 显示菜单 | 显示菜单 |
| Template icon | ✅ 推荐 | N/A | N/A |
| Tooltip | ✅ | ✅ | ✅ |
| 动态图标 | ✅ set_icon | ✅ | ⚠️ |

## 启动时隐藏窗口

```rust
// 启动时隐藏主窗口，只在托盘显示
app.get_webview_window("main")
    .map(|w| w.hide().ok());
```
