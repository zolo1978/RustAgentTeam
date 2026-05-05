# IPC 接口契约模板

## 契约表头

每个 Tauri Command 必须在实现前定义以下契约：

```
Command:  <name>
参数:     <struct> (<必填>/<可选> 字段列表)
返回:     Result<T, E>  — T 为成功类型, E 为错误类型
错误码:   见下方错误码表
备注:     <额外约束、副作用、平台差异>
```

## 参数约束

```
字段名         类型         必填/可选    约束
------------------------------------------------------
<field>       <RustType>   required     <范围/格式>
<field>       <RustType>   optional     <默认值/None语义>
```

- 必填字段：Rust struct 中不用 Option，serde 反序列化时缺失会报错。
- 可选字段：用 Option<T>，前端传 null/undefined 或省略。
- 校验：用 `validator` derive 在 Rust 侧做服务端校验，不信任前端。

## 返回类型定义

```rust
// 成功响应统一包装
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandResult<T: Serialize> {
    pub data: T,
}

// 分页响应
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PagedResult<T: Serialize> {
    pub items: Vec<T>,
    pub total: u32,
    pub page: u32,
    pub page_size: u32,
}
```

## 错误码表

| 错误码 | 含义 | 触发条件 | HTTP 等价 |
|--------|------|----------|-----------|
| VALIDATION_FAILED | 参数校验失败 | 字段缺失/越界/格式错误 | 400 |
| NOT_FOUND | 资源不存在 | id 对应记录为空 | 404 |
| CONFLICT | 状态冲突 | 并发修改/重复创建 | 409 |
| PERMISSION_DENIED | 权限不足 | Capabilities 未授权 | 403 |
| INTERNAL | 内部错误 | 未预期异常 | 500 |

```rust
#[derive(Debug, thiserror::Error, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE", tag = "code")]
pub enum AppError {
    #[error("{message}")]
    ValidationFailed { message: String },
    #[error("{resource} not found")]
    NotFound { resource: String },
    #[error("{message}")]
    Conflict { message: String },
    #[error("permission denied")]
    PermissionDenied,
    #[error("{0}")]
    Internal(String),
}
```

## Rust -> serde JSON -> TypeScript 完整映射示例

```rust
// Rust struct
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClipItem {
    pub id: String,
    pub content: String,
    pub clip_type: ClipType,
    pub tags: Vec<String>,
    pub source_app: Option<String>,
    pub created_at: i64,           // Unix timestamp ms
    pub pinned: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ClipType {
    Text,
    Image,
    File,
    Url,
}
```

```json
// serde JSON 输出
{
  "id": "clip-001",
  "content": "hello world",
  "clipType": "text",
  "tags": ["greeting"],
  "sourceApp": null,
  "createdAt": 1714358400000,
  "pinned": false
}
```

```typescript
// TypeScript interface（由前端维护，与 Rust struct 保持同步）
interface ClipItem {
  id: string;
  content: string;
  clipType: "text" | "image" | "file" | "url";
  tags: string[];
  sourceApp: string | null;
  createdAt: number;
  pinned: boolean;
}
```

## 示例契约：list_clips

```
Command:  list_clips
参数:
  字段名          类型           必填/可选    约束
  -----------------------------------------------
  query          String         optional     最大 200 字符
  clip_type      Option<ClipType> optional  null = 全部类型
  page           u32            required     >= 1
  page_size      u32            required     1..=100
返回:     Result<PagedResult<ClipItem>, AppError>
错误码:   VALIDATION_FAILED (page/page_size 越界)
备注:     结果按 created_at DESC 排序
```

```rust
#[tauri::command]
async fn list_clips(
    query: Option<String>,
    clip_type: Option<ClipType>,
    page: u32,
    page_size: u32,
    state: State<'_, AppState>,
) -> Result<PagedResult<ClipItem>, AppError> {
    if page == 0 || page_size == 0 || page_size > 100 {
        return Err(AppError::ValidationFailed {
            message: "page must >= 1, page_size must be 1..=100".into(),
        });
    }
    state.clip_svc.list(query, clip_type, page, page_size).await
}
```

```typescript
// 前端调用
const result = await safeInvoke<PagedResult<ClipItem>>("list_clips", {
  query: searchText || null,
  clipType: filterType ?? null,
  page: 1,
  pageSize: 20,
});
```

## 示例契约：paste_clip

```
Command:  paste_clip
参数:
  字段名          类型           必填/可选    约束
  -----------------------------------------------
  clip_id        String         required     UUID 格式
  target_window  Option<String> optional     null = 当前焦点窗口
返回:     Result<(), AppError>
错误码:   NOT_FOUND (clip_id 不存在)
          PERMISSION_DENIED (无剪贴板写入权限)
备注:     调用平台剪贴板 API，可能触发系统权限弹窗
```

```rust
#[tauri::command]
async fn paste_clip(
    clip_id: String,
    target_window: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    let clip = state.clip_svc.get(&clip_id).await?;
    state.clipboard.write(&clip.content).await?;
    Ok(())
}
```

```typescript
// 前端调用
await safeInvoke<void>("paste_clip", {
  clipId: selectedClipId,
  targetWindow: null,
});
```
