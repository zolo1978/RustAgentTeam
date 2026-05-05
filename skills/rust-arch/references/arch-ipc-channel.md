# IPC Channel 流式传输

## Rust 端：流式 Command

```rust
use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, BufReader};

#[tauri::command]
async fn stream_log(
    path: PathBuf,
    ch: tauri::ipc::Channel<String>,
) -> Result<(), AppError> {
    let mut reader = BufReader::new(tokio::fs::File::open(&path).await?);
    let mut line = String::new();
    loop {
        line.clear();
        if reader.read_line(&mut line).await? == 0 {
            break;
        }
        ch.send(line.clone())
            .map_err(|e| AppError::Internal(e.to_string()))?;
    }
    Ok(())
}
```

关键点：
- `tauri::ipc::Channel<T>` 是 v2 新增 API，需要 Tauri >= 2.0。
- `ch.send()` 将消息写入内部缓冲区；高频场景仍需在 Rust 侧自行限速，避免消息堆积。
- 适用于日志流、进度条、大文件逐行处理等场景。

## 前端 TypeScript 消费端

### invoke 封装

```typescript
// lib/ipc.ts
import { invoke } from '@tauri-apps/api/core';

export async function safeInvoke<T>(
    cmd: string,
    args?: Record<string, unknown>,
): Promise<T> {
    try {
        return await invoke<T>(cmd, args);
    } catch (err) {
        throw new Error(`IPC ${cmd} failed: ${err}`);
    }
}

// 使用示例
const data = await safeInvoke<GetDataResponse>('get_data', { id: '123' });
```

### Channel 消费

```typescript
// lib/stream.ts
import { Channel } from '@tauri-apps/api/core';

const ch = new Channel<string>();
ch.onmessage = (line) => {
    console.log(line);
};
await invoke('stream_log', { path: '/var/log/app.log', ch });
```

### 事件监听（替代 WebSocket）

```typescript
// lib/events.ts
import { listen } from '@tauri-apps/api/event';

const unlisten = await listen<ProgressEvent>('upload-progress', (event) => {
    console.log(`Progress: ${event.payload.percent}%`);
});
// 组件卸载时调用 unlisten() 清理
```

## IPC 粒度最佳实践

- **批量提交**：将多个字段更新合并为一次 invoke，避免 N 次 IPC 往返。
- **流式场景用 Channel**：避免轮询，用 Channel 实现服务端推送。
- **类型安全**：泛型 `safeInvoke<T>` 保证前后端类型一致。
