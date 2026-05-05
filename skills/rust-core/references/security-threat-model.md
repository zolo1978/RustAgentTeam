# STRIDE Threat Model — 通用 Rust 应用

| Threat | Rust Surface | Mitigation |
|--------|-------------|------------|
| Spoofing | 伪造输入身份（API token、user ID） | 输入验证 + 类型化 ID（newtype）|
| Tampering | 未验证的外部输入（CLI args、HTTP body、文件内容） | Parse, don't validate — 在边界转换为类型化结构体 |
| Repudiation | 缺少审计日志 | `tracing` 结构化日志 + 请求 ID |
| Info Disclosure | 硬编码密钥、日志泄露、错误信息暴露内部结构 | 环境变量 + 日志过滤 + 错误信息脱敏 |
| Denial of Service | 无界输入、无限循环、资源耗尽 | 输入长度限制 + 超时 + 背压 |
| Elevation of Privilege | unsafe 代码、依赖链中的恶意 crate | unsafe 审计 + `cargo audit` + `cargo deny` |

## 攻击场景与缓解措施

### Spoofing — 身份伪造

攻击：攻击者伪造 user_id 参数访问他人数据。

缓解：用 newtype 区分不同 ID，在边界验证所有权。

```rust
struct UserId(u64);
struct OrderId(u64);

fn get_order(user: UserId, order: OrderId) -> Result<Order, AppError> {
    let order = db.find_order(&order)?;
    if order.owner != user {
        return Err(AppError::Forbidden);
    }
    Ok(order)
}
```

### Tampering — 输入篡改

攻击：HTTP 请求体或 CLI 参数包含恶意数据（SQL 注入、路径穿越、命令注入）。

缓解：在系统边界 parse 为类型化结构体，用 `validator` crate 校验。

```rust
use validator::Validate;

#[derive(Deserialize, Validate)]
struct CreateUser {
    #[validate(length(min = 1, max = 100))]
    name: String,
    #[validate(email)]
    email: String,
}

fn create_user(input: CreateUser) -> Result<User, AppError> {
    input.validate()?;
    // 类型已保证 name 非空、email 合法
}
```

SQL 注入：永远参数化查询。

```rust
// BAD
let q = format!("SELECT * FROM users WHERE id = '{}}'", id);

// GOOD
db.query("SELECT * FROM users WHERE id = $1", &[&id]).await?;
```

路径穿越：canonicalize + starts_with 检查。

```rust
fn safe_read(base: &Path, name: &str) -> Result<Vec<u8>, AppError> {
    let resolved = base.join(name).canonicalize()?;
    if !resolved.starts_with(&base.canonicalize()?) {
        return Err(AppError::Forbidden("path traversal blocked"));
    }
    Ok(std::fs::read(&resolved)?)
}
```

### Repudiation — 审计缺失

攻击：管理员执行破坏性操作后无法追溯。

缓解：`tracing` 结构化日志 + 请求 ID + 可选的 append-only 审计存储。

```rust
use tracing;
use uuid::Uuid;

let request_id = Uuid::new_v4();
tracing::info!(
    request_id = %request_id,
    user_id = %uid,
    action = "delete_record",
    record_id = %rid
);
```

### Information Disclosure — 信息泄露

攻击场景：API key 硬编码到源码 → 进入 Git 历史；错误信息包含数据库 schema。

缓解：

```rust
// 密钥：永远从环境变量读取
let key = env::var("API_KEY").map_err(|_| AppError::Config("API_KEY not set"))?;

// 日志：永远脱敏
tracing::debug!(key = "***REDACTED***");

// 错误：对外脱敏，对内详细
match result {
    Err(e) => {
        tracing::error!(error = %e, "internal error");  // 日志记录详情
        return Err(AppError::Internal("operation failed".into()));  // 对外只返回通用信息
    }
}

// 敏感数据清理
use zeroize::Zeroize;
#[derive(Zeroize)]
#[zeroize(drop)]
struct SecretKey(Vec<u8>);
```

### Denial of Service — 拒绝服务

攻击：10 GB 字符串输入耗尽内存；无限循环阻塞线程。

缓解：输入限制 + 超时 + 背压。

```rust
// 输入长度限制
fn process(data: &str) -> Result<String, AppError> {
    if data.len() > 10_000 {
        return Err(AppError::InputTooLong);
    }
    Ok(data.to_uppercase())
}

// 超时
let result = tokio::time::timeout(
    Duration::from_secs(5),
    slow_operation(),
).await;

// 背压（bounded channel）
let (tx, rx) = mpsc::sync_channel(16);
```

### Elevation of Privilege — 权限提升

攻击：unsafe 代码绕过安全检查；恶意依赖在构建时执行代码。

缓解：

**unsafe 审计：** 所有 unsafe 块必须有 `// SAFETY:` 注释说明为什么安全。

```rust
// SAFETY: index is always < len due to the loop bound
unsafe { slice.get_unchecked(index) }

// BAD: 无 SAFETY 注释
unsafe { *ptr }  // 为什么 ptr 有效？为什么对齐？未说明。
```

**依赖审计：**

```bash
cargo audit              # 已知漏洞
cargo deny check         # 许可证 + 来源
cargo geiger             # unsafe 使用统计
cargo tree -d            # 检测重复依赖
```

## 安全门检查清单（CI 必跑）

```bash
cargo audit                        # 已知漏洞
cargo deny check                   # 许可证 + 来源审计
cargo geiger --all-features        # unsafe 统计
cargo fuzz run TARGET -- -max_total_time=300  # 5 分钟 fuzzing
```

手动检查：
- [ ] 所有 unsafe 块有 `// SAFETY:` 注释
- [ ] 无 `unsafe impl Send/Sync`（除非有充分理由）
- [ ] 无 Mutex lock 跨 `.await` 点
- [ ] 无硬编码密钥（grep `api_key|password|secret|token`）
- [ ] 敏感结构体实现 `Zeroize` + `#[zeroize(drop)]`
- [ ] TLS 证书验证未禁用（grep `danger_accept_invalid_certs`）
