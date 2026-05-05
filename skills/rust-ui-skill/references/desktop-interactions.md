# 桌面端交互 — Tauri 特有模式

## 概述

桌面端有很多 Web 端没有的交互模式：无边框窗口、全局快捷键、系统托盘、原生操作。

## 无边框窗口

### 配置

```json
// tauri.conf.json
{
  "app": {
    "windows": [{
      "decorations": false,
      "width": 400,
      "height": 600
    }]
  }
}
```

### 自定义拖拽区域

```tsx
// 标题栏作为拖拽区域
<header data-tauri-drag-region className="flex items-center h-8 px-4">
  <span data-tauri-drag-region className="text-sm font-medium">ClipVault</span>
  <div className="ml-auto">
    <button onClick={hide}>—</button>
    <button onClick={quit}>×</button>
  </div>
</header>
```

注意：`data-tauri-drag-region` 不能放在 button 等交互元素上，否则会阻止点击。

## 全局快捷键

### 注册（Rust 侧）

```rust
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut};

fn setup_shortcut(app: &App) -> Result<(), Box<dyn std::error::Error>> {
    let shortcut = Shortcut::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::KeyV);
    app.plugin(tauri_plugin_global_shortcut::Builder::new())
        .build()?;
    app.global_shortcut().on_shortcut(shortcut, |app, _, event| {
        // 切换窗口显示/隐藏
        toggle_window(app);
    })?;
    Ok(())
}
```

### 前端快捷键

```tsx
useEffect(() => {
  const handleKeyDown = (e: KeyboardEvent) => {
    // 列表导航
    if (e.key === 'ArrowDown') { e.preventDefault(); selectNext(); }
    if (e.key === 'ArrowUp') { e.preventDefault(); selectPrev(); }
    if (e.key === 'Enter') { e.preventDefault(); pasteSelected(); }
    if (e.key === 'Escape') { hideWindow(); }
  };
  window.addEventListener('keydown', handleKeyDown);
  return () => window.removeEventListener('keydown', handleKeyDown);
}, []);
```

## 系统托盘

### Rust 侧

```rust
use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::TrayIconBuilder,
};

fn setup_tray(app: &App) -> Result<(), Box<dyn std::error::Error>> {
    let show = MenuItemBuilder::with_id("show", "显示 ClipVault").build(app)?;
    let quit = MenuItemBuilder::with_id("quit", "退出").build(app)?;
    let menu = MenuBuilder::new(app).items(&[&show, &quit]).build()?;

    TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => show_window(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .build(app)?;
    Ok(())
}
```

## 窗口显示/隐藏切换

```rust
use tauri::Manager;

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

## 原生操作（通过 Rust）

### 复制到剪贴板

```rust
async fn paste_clip(id: String, app: AppHandle) -> Result<(), AppError> {
    let clip = get_clip(&db, &id)?;
    // 通过 arboard 写入系统剪贴板
    safe_ffi("paste", || {
        let mut cb = Clipboard::new()?;
        cb.set_text(&clip.content)?;
        Ok(())
    }).await?;
    // 通过 enigo 模拟 Cmd+V
    safe_ffi("simulate_paste", || {
        let mut enigo = Enigo::new()?;
        enigo.key(Key::Control, Press)?;
        enigo.key(Key::Layout('v'), Click)?;
        enigo.key(Key::Control, Release)?;
        Ok(())
    }).await?;
    // 隐藏窗口
    toggle_window(&app);
    Ok(())
}
```
