# Rust <-> TypeScript 完整类型映射表

## 基础类型映射

| Rust 类型 | serde JSON | TypeScript 类型 | 备注 |
|-----------|-----------|----------------|------|
| String | "hello" | string | |
| &str | "hello" | string | 很少直接用于 Command 参数 |
| u8 / u16 / u32 | 42 | number | JS 安全整数范围内 |
| i8 / i16 / i32 | -42 | number | |
| u64 | 42 | number | 超过 2^53 有精度丢失风险 |
| i64 | -42 | number | 同上，大数用 String 传输 |
| f32 / f64 | 3.14 | number | |
| bool | true | boolean | |
| () | null | void / undefined | |

## 容器映射

| Rust 类型 | serde JSON | TypeScript 类型 | 备注 |
|-----------|-----------|----------------|------|
| Vec<T> | [t1, t2] | T[] | |
| Option<T> | t / null | T \| null | serde 默认 None -> null |
| HashMap<String, V> | {"k": v} | Record<string, V> | key 必须是 String |
| HashMap<K, V> (K != String) | [["k",v]] | [K, V][] | 非 String key 用 array 序列化 |
| HashSet<T> | [t1, t2] | T[] | 无序，去重由 Rust 保证 |
| tuple (A, B) | [a, b] | [A, B] | |
| Unit struct {} | null | null | |

## 枚举映射

### 简单枚举（无变体数据）

```rust
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ClipType {
    Text,
    Image,
    File,
    Url,
}
```
```typescript
type ClipType = "text" | "image" | "file" | "url";
```

### 带数据的枚举

```rust
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum Event {
    Click { x: f64, y: f64 },
    KeyPress { key: String },
    Resize { width: u32, height: u32 },
}
```
```typescript
type Event =
  | { type: "click"; x: number; y: number }
  | { type: "keyPress"; key: string }
  | { type: "resize"; width: number; height: number };
```

### 变体含不同数据类型

```rust
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum Value {
    String(String),
    Number(f64),
    Bool(bool),
    Null,
}
```
```typescript
type Value = string | number | boolean | null;
```

## 特殊映射

| Rust 类型 | serde 策略 | TypeScript 类型 | 配置 |
|-----------|-----------|----------------|------|
| Vec<u8> | base64 | string | `serde(with = "base64_serde")` 或传输时手动编码 |
| chrono::DateTime<Utc> | Unix timestamp (i64) | number | `#[serde(with = "ts_milliseconds")]` |
| chrono::NaiveDate | "2024-01-15" | string | ISO 8601 字符串，前端用 Date 解析 |
| PathBuf | "/path/to/file" | string | 跨平台路径用 String 传输 |
| Decimal (rust_decimal) | "3.14" | string | 精确金额用字符串，避免浮点误差 |
| uuid::Uuid | "550e8400-..." | string | 标准格式 |

### Vec<u8> base64 映射实现

```rust
mod base64_serde {
    use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(data: &Vec<u8>, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&BASE64.encode(data))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<u8>, D::Error> {
        let s = String::deserialize(d)?;
        BASE64.decode(&s).map_err(serde::de::Error::custom)
    }
}

#[derive(Serialize, Deserialize)]
pub struct ImageClip {
    #[serde(with = "base64_serde")]
    pub data: Vec<u8>,
    pub mime_type: String,
}
```

### chrono DateTime 映射

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ClipItem {
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub created_at: DateTime<Utc>,
}
```

```typescript
interface ClipItem {
  createdAt: number; // Unix milliseconds
}
// 前端使用
const date = new Date(clip.createdAt);
```

## 常见陷阱

### 1. 命名风格不一致

```rust
// Rust 默认 snake_case
#[derive(Serialize)]
pub struct User {
    pub user_name: String,      // JSON: "userName" (有 rename_all)
    pub email_address: String,   // JSON: "emailAddress"
}
// 必须加 #[serde(rename_all = "camelCase")]
// 否则前端收到 "user_name"，不符合 JS 惯例
```

**规则：** 所有跨 IPC 边界的 struct 统一加 `#[serde(rename_all = "camelCase")]`。

### 2. Option 字段省略 vs null

```rust
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub theme: Option<String>,        // None -> JSON 中字段不存在 (默认行为)
}

// 如果前端需要区分 "未设置" 和 "设为 null"：
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    #[serde(skip_serializing_if = "Option::is_none")]  // 默认行为，可省略
    pub theme: Option<String>,        // None -> 字段不存在
}

// 如果希望 None -> null（前端始终收到该字段）：
pub struct Config {
    #[serde(serialize_with = "serialize_option_as_null")]
    pub theme: Option<String>,        // None -> "theme": null
}
```

**规则：** 默认行为（None -> 字段不存在）即可。前端用 `field ?? defaultValue` 处理。

### 3. 嵌套结构忘加 rename_all

```rust
// BAD：内层 struct 没加 rename_all
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Order {
    pub shipping_info: ShippingInfo, // 内层仍然是 snake_case
}

#[derive(Serialize)]
// 忘了 rename_all!
pub struct ShippingInfo {
    pub postal_code: String, // JSON: "postal_code"（不一致）
}

// GOOD：每层都加
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShippingInfo {
    pub postal_code: String, // JSON: "postalCode"
}
```

**规则：** 每个跨 IPC 边界的 struct 都要独立标注 `rename_all = "camelCase"`，不会从外层继承。

### 4. 大整数精度丢失

JavaScript Number 安全整数范围: -(2^53 - 1) 到 2^53 - 1。

```rust
// BAD：u64/i64 超过安全范围会丢失精度
pub struct FileMeta {
    pub size: u64, // 文件可能 > 8 PB，前端丢失精度
}

// GOOD：大数用 String 传输
pub struct FileMeta {
    #[serde(serialize_with = "serialize_u64_as_string")]
    pub size: u64, // JSON: "9223372036854775807"
}
```

## 决策树：遇到新类型如何映射

```
新类型 T 出现
  |
  +--> T 是标准 Rust 类型？ --> 查上表直接映射
  |
  +--> T 是枚举？
  |      +--> 变体无数据？ --> TS union type ("a" | "b" | "c")
  |      +--> 变体有数据？ --> TS discriminated union + #[serde(tag = "type")]
  |      +--> 变体类型互斥？ --> TS union (string | number | ...) + #[serde(untagged)]
  |
  +--> T 包含二进制数据？ --> base64 编码为 string
  |
  +--> T 是时间类型？
  |      +--> 需要精度？ --> string (ISO 8601 / RFC 3339)
  |      +--> 只需排序/比较？ --> number (Unix timestamp)
  |
  +--> T 是大整数 (> 2^53)？ --> string 序列化
  |
  +--> T 是自定义 struct？ --> 逐字段递归应用本决策树，每层加 rename_all = "camelCase"
  |
  +--> 仍不确定？ --> serde_json::to_string(&sample) 查看实际 JSON 输出，据此写 TS 类型
```
