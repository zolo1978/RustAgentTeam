# IPC 错误处理（重试 + 超时 + 用户友好消息）

> 代码模板：[templates/safe-invoke.ts](../templates/safe-invoke.ts)

## 核心组件

- **IpcError**：自定义错误类，携带 `code`、`retryable` 标志和用户友好 `message`。
- **safeInvoke\<T\>**：类型安全的 invoke 封装，内置重试（指数退避）和超时（Promise.race）。
- **isTransient**：判断错误是否可重试（timeout / network / busy）。
- **toUserMessage**：将技术错误消息转换为用户可理解的中文提示。

## 错误分类与处理策略

| 错误类型 | 来源 | 可重试 | 处理策略 |
|----------|------|--------|----------|
| PermissionDenied | Capabilities 未授权 | 否 | 引导用户检查权限设置 |
| NotFound | 数据/资源不存在 | 否 | 显示 404 UI |
| Validation | 输入校验失败 | 否 | 高亮表单字段 + 错误提示 |
| Timeout | IPC 超时（默认 10s） | 是 | 自动重试 2 次，指数退避 |
| Network | 网络不可达 | 是 | 自动重试 + 显示离线提示 |
| Busy | 后端正在处理 | 是 | 自动重试 + 显示 loading |
| Internal | 服务端未知错误 | 否 | 显示通用错误 + 上报 Sentry |

## 重试策略

```typescript
// 指数退避：500ms * 2^attempt
// 默认最多重试 2 次（共 3 次尝试）
// 仅对 isTransient() 返回 true 的错误重试

const retryDelays = [500, 1000]; // 2 次重试的间隔
for (let attempt = 0; attempt <= maxRetries; attempt++) {
  try {
    return await Promise.race([
      invoke<T>(cmd, args),
      timeout(ms)  // 默认 10000ms
    ]);
  } catch (err) {
    if (!isTransient(err) || attempt === maxRetries) throw toIpcError(err);
    await sleep(retryDelays[attempt]);
  }
}
```

## 超时实现

Tauri `invoke` 不支持 `AbortSignal`，用 `Promise.race` + `setTimeout`：

```typescript
function withTimeout<T>(promise: Promise<T>, ms: number): Promise<T> {
  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => reject(new IpcError('TIMEOUT', '操作超时', true)), ms);
    promise.then(v => { clearTimeout(timer); resolve(v); })
           .catch(e => { clearTimeout(timer); reject(e); });
  });
}
```

## 用户友好消息映射

```typescript
function toUserMessage(code: string): string {
  const map: Record<string, string> = {
    'PERMISSION_DENIED': '没有权限执行此操作，请检查应用设置',
    'NOT_FOUND': '请求的内容不存在',
    'VALIDATION': '输入信息有误，请检查后重试',
    'TIMEOUT': '操作超时，请检查网络后重试',
    'NETWORK': '网络连接失败，请检查网络设置',
    'BUSY': '正在处理中，请稍候',
    'INTERNAL': '操作失败，请稍后重试',
  };
  return map[code] ?? '未知错误';
}
```

## Rust 端：Command 错误序列化

```rust
// AppError 自动序列化为 JSON，前端 safeInvoke 解析
impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: serde::Serializer {
        serializer.serialize_str(&self.to_string())
    }
}

// Tauri Command 返回 Result<T, AppError>，序列化后前端收到：
// { "code": "VALIDATION", "message": "邮箱格式不正确", "retryable": false }
```

## 最佳实践

1. **前端永远不裸调用 `invoke`** — 统一走 `safeInvoke<T>`
2. **重试仅用于 transient 错误** — PermissionDenied/NotFound 重试无意义
3. **超时设置按场景调整** — 文件操作 30s，普通查询 10s，搜索 20s
4. **错误上报 Sentry** — 非 transient 错误上报，transient 仅在重试耗尽后上报
5. **本地先校验** — 表单提交前先前端校验，减少无效 IPC 调用
