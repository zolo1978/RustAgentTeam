# Tauri 事件处理

## Rust→前端推送

Rust 端用 `app.emit()` 向前端推送事件：

```rust
use tauri::{AppHandle, Emitter};

#[tauri::command]
async fn start_download(url: String, app: AppHandle) -> Result<(), AppError> {
    let total = get_content_length(&url).await?;
    let mut downloaded = 0u64;
    // ... 下载循环中
    app.emit("download-progress", ProgressPayload { downloaded, total }).ok();
    Ok(())
}
```

前端用 `listen` 接收：

```typescript
import { listen } from '@tauri-apps/api/event';

interface ProgressPayload {
  downloaded: number;
  total: number;
}

const unlisten = await listen<ProgressPayload>('download-progress', (event) => {
  console.log(`${event.payload.downloaded}/${event.payload.total}`);
});
// 组件卸载时必须调用 unlisten()
```

## 跨窗口事件

```typescript
// 窗口 A 发送定向事件
import { emitTo } from '@tauri-apps/api/event';
await emitTo('settings-window', 'config-changed', { key: 'theme', value: 'dark' });

// 窗口 B 监听
const unlisten = await listen('config-changed', handler);
```

## 事件泄漏防范

每次 `listen` 返回一个 `unlisten` 函数，**必须在组件卸载时调用**：

```typescript
// BAD: 忘记清理
useEffect(() => {
  listen('event', handler);  // 每次渲染都注册，从不清理
}, []);

// GOOD: useEffect 清理
useEffect(() => {
  const promise = listen('event', handler);
  return () => { promise.then(fn => fn()); };
}, []);
```

## listen vs invoke vs Channel

| 场景 | 工具 | 特点 |
|------|------|------|
| 前端请求→Rust 返回 | `invoke` (Command) | 请求-响应，有返回值 |
| Rust 主动推送→前端 | `emit` + `listen` | 单向推送，无返回值 |
| 流式数据 | `Channel<T>` | 高频、有序、背压控制 |
| 窗口间通信 | `emitTo` + `listen` | 定向推送 |
