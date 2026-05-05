# IPC 契约验证操作手册

## 完整映射表

### Rust -> serde JSON -> TypeScript

| Rust 类型 | serde 注解影响 | JSON 输出 | TS 类型 | 注意事项 |
|-----------|--------------|----------|---------|---------|
| `String` | 无 | `"hello"` | `string` | |
| `&str` | 无 | `"hello"` | `string` | |
| `i32` / `u32` / `i64` | 无 | `42` | `number` | TS 无整型区分 |
| `f64` | 无 | `3.14` | `number` | |
| `bool` | 无 | `true` | `boolean` | |
| `Option<T>` | 无 | `T` 或 `null` | `T \| null \| undefined` | TS 三态 |
| `Vec<T>` | 无 | `[T, T]` | `T[]` | |
| `Vec<u8>` | 无 | `[0, 1, 2]` | `number[]` | 非 Uint8Array |
| `HashMap<K, V>` | K 须为 String | `{"k": v}` | `Record<string, V>` | |
| `serde_json::Value` | 无 | any JSON | `unknown` | 禁止用 `any` |
| `chrono::NaiveDateTime` | feature = "serde" | `"2024-01-01T00:00:00"` | `string` | 手动解析 |
| `chrono::DateTime<Utc>` | feature = "serde" | `"2024-01-01T00:00:00Z"` | `string` | 手动解析 |
| `uuid::Uuid` | feature = "serde" | `"550e8400-..."` | `string` | |
| `enum { A, B }` (unit) | rename_all 影响大小写 | `"A"` 或 `"a"` | `"A" \| "a"` | 确认 casing |
| `enum { A(u32), B(String) }` | [untagged] 改结构 | 变体内容 | 联合类型 | |
| `struct` | rename_all 影响字段名 | `{ "fieldName": ... }` | interface | 核心检查点 |

## 验证步骤

### Step 1: 读取 Rust struct

```bash
rg '#\[serde\(' src-tauri/src/ -A 3
rg 'pub struct \w+' src-tauri/src/ -A 20
```

关注点：
- `rename_all` 的值 (camelCase / snake_case / PascalCase)
- `rename = "..."` 个别字段覆盖
- `skip_serializing_if` 条件字段

### Step 2: 读取 TS interface

```bash
rg 'interface \w+' src/ -A 20
rg 'type \w+ =' src/ -A 10
```

### Step 3: 逐字段对比

对每个 struct/interface：
1. 字段名是否匹配 serde 输出
2. 类型是否对应映射表
3. Option/null/undefined 是否对齐
4. 枚举值是否完全覆盖

## 常见不对齐案例

### 案例 1: rename_all 遗漏

```rust
#[derive(Serialize)]
// 忘记 #[serde(rename_all = "camelCase")]
pub struct ClipItem {
    pub created_at: i64,  // JSON: "created_at"
}
```
```typescript
interface ClipItem {
    createdAt: number;  // 期望 "createdAt"
}
```
**结果**: TS 收到 undefined，因为 JSON 字段是 `created_at`

### 案例 2: Option 映射错误

```rust
pub struct Config {
    pub theme: Option<String>,  // JSON: null 或 "dark"
}
```
```typescript
// BAD: 不允许 null
interface Config { theme: string; }
// GOOD:
interface Config { theme: string | null; }
```

### 案例 3: Vec<u8> 误解

```rust
pub struct Response {
    pub data: Vec<u8>,  // JSON: [72, 101, 108, ...]
}
```
```typescript
// BAD: 期望 Uint8Array
interface Response { data: Uint8Array; }
// GOOD:
interface Response { data: number[]; }
// 或者 Rust 侧用 tauri::ipc::Response 直接传二进制
```

### 案例 4: 枚举大小写

```rust
#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType { ClipboardChanged, WindowFocused }
// JSON: "clipboard_changed", "window_focused"
```
```typescript
// BAD:
type EventType = "ClipboardChanged" | "WindowFocused";
// GOOD:
type EventType = "clipboard_changed" | "window_focused";
```

### 案例 5: 时间格式

```rust
pub struct ClipItem {
    pub created_at: chrono::NaiveDateTime,
}
// JSON: "2024-01-01T00:00:00"
```
```typescript
// BAD:
interface ClipItem { createdAt: Date; }
// GOOD:
interface ClipItem { createdAt: string; }  // ISO 8601 string
```

### 案例 6: 嵌套结构 rename_all 不一致

```rust
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Outer {
    pub inner_data: Inner,  // JSON: "innerData": { "raw_value": 1 }
}

#[derive(Serialize)]
// Inner 没有 rename_all，保持 snake_case
pub struct Inner {
    pub raw_value: i32,
}
```
```typescript
interface Outer { innerData: Inner; }
interface Inner { raw_value: number; }  // 注意这里是 snake_case
```

## 自动化检测思路

```bash
# 提取所有 Rust struct 的 serde 注解和字段
rg '#\[serde\(rename_all\s*=\s*"(\w+)"\)\]' src-tauri/src/ -n

# 提取所有 TS interface 字段
rg '^\s+(\w+)\??\s*:' src/types/ -n

# 对比脚本（伪代码）：
# 1. 解析 Rust struct: serde casing + 字段列表
# 2. 按 casing 规则转换 Rust 字段名为 JSON 名
# 3. 解析 TS interface 字段列表
# 4. diff 两个字段名集合
```
