# 窗口管理

## 概述

ClipVault 是一个 400×600 的紧凑窗口应用。需要无边框、置顶、显示/隐藏切换。

## tauri.conf.json 配置

```json
{
  "app": {
    "windows": [{
      "label": "main",
      "title": "ClipVault",
      "width": 400,
      "height": 600,
      "decorations": false,
      "alwaysOnTop": true,
      "visible": true,
      "skipTaskbar": true,
      "resizable": true
    }]
  }
}
```

## 无边框窗口拖拽

```tsx
// 前端：自定义标题栏拖拽区域
<header data-tauri-drag-region className="flex items-center h-8 px-4 select-none">
  <span data-tauri-drag-region className="text-sm font-medium">ClipVault</span>
  <div className="ml-auto flex gap-1">
    <button onClick={hide} className="px-2 py-1 text-xs hover:bg-gray-200 dark:hover:bg-gray-700">—</button>
    <button onClick={quit} className="px-2 py-1 text-xs hover:bg-red-200 dark:hover:bg-red-800">×</button>
  </div>
</header>
```

注意：交互元素（button/input）不能有 `data-tauri-drag-region`，否则无法点击。

## 窗口操作（Rust 侧）

```rust
use tauri::Manager;

#[tauri::command]
async fn toggle_window(app: AppHandle) -> Result<(), AppError> {
    let window = app.get_webview_window("main")
        .ok_or(AppError::Window("窗口未找到".into()))?;

    if window.is_visible().unwrap_or(false) {
        window.hide().map_err(|e| AppError::Window(e.to_string()))?;
    } else {
        window.show().map_err(|e| AppError::Window(e.to_string()))?;
        window.set_focus().map_err(|e| AppError::Window(e.to_string()))?;
    }
    Ok(())
}

#[tauri::command]
async fn hide_window(app: AppHandle) -> Result<(), AppError> {
    let window = app.get_webview_window("main")
        .ok_or(AppError::Window("窗口未找到".into()))?;
    window.hide().map_err(|e| AppError::Window(e.to_string()))
}
```

## 窗口操作（前端调用）

```typescript
import { getCurrentWindow } from '@tauri-apps/api/window';

async function hideWindow() {
  await getCurrentWindow().hide();
}

async function showWindow() {
  const win = getCurrentWindow();
  await win.show();
  await win.setFocus();
}
```

## 失去焦点自动隐藏

```typescript
// 前端监听窗口失焦事件
import { getCurrentWindow } from '@tauri-apps/api/window';

const unlisten = await getCurrentWindow().onFocusChanged(({ payload: focused }) => {
  if (!focused) {
    getCurrentWindow().hide();
  }
});
```

## 平台差异

| 特性 | macOS | Windows | Linux |
|------|-------|---------|-------|
| decorations: false | ✅ | ✅ | ✅ |
| alwaysOnTop | ✅ | ✅ | ⚠️ WM 相关 |
| skipTaskbar | ✅ | ✅ | ⚠️ |
| set_focus | ✅ | ✅ | ✅ |
| drag-region | ✅ | ✅ | ✅ |
