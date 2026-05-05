# 敏感内容过滤 — ClipVault 安全参考

## 概述

剪贴板管理器捕获所有系统剪贴板内容。必须在存储前检测并分级，防止密码/密钥/Token 明文泄露。

## 内容分级

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum SensitivityLevel {
    None,       // 正常存储和索引
    Medium,     // 个人信息 — 存储但标记
    Critical,   // 密钥/密码 — 加密存储，不进 FTS
}

pub struct FilterResult {
    pub level: SensitivityLevel,
    pub matched_patterns: Vec<String>,
    pub sanitized_preview: String,
    pub should_encrypt: bool,
    pub should_index: bool,
}
```

## BAD vs GOOD

### BAD — 无过滤直接存储

```rust
// 密码 "AWS_SECRET=AKIAIOSFODNN7EXAMPLE" 明文进 DB 和 FTS
pub fn create_clip(conn: &Connection, content: Vec<u8>) -> Result<Clip> {
    let preview = String::from_utf8_lossy(&content).to_string();
    insert_clip(conn, &id, "text", content, &preview, &hash, now)
}
```

### GOOD — RegexSet 批量检测 + 分级处理

```rust
use regex::RegexSet;

pub struct SensitiveFilter {
    patterns: RegexSet,
    levels: Vec<SensitivityLevel>,
    names: Vec<&'static str>,
}

impl SensitiveFilter {
    pub fn new() -> Self {
        let patterns = vec![
            (r"AKIA[0-9A-Z]{16}", SensitivityLevel::Critical, "aws_key"),
            (r"gh[ps]_[A-Za-z0-9_]{36,}", SensitivityLevel::Critical, "github_token"),
            (r"eyJ[A-Za-z0-9-_]+\.eyJ[A-Za-z0-9-_]+", SensitivityLevel::Critical, "jwt"),
            (r"-----BEGIN (?:RSA |EC |DSA )?PRIVATE KEY-----", SensitivityLevel::Critical, "private_key"),
            (r"\b4[0-9]{12}(?:[0-9]{3})?\b", SensitivityLevel::Critical, "visa"),
            (r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}", SensitivityLevel::Medium, "email"),
            (r"1[3-9]\d{9}", SensitivityLevel::Medium, "phone_cn"),
        ];
        let (regexes, levels, names) = patterns.into_iter().multiunzip();
        Self { patterns: RegexSet::new(regexes).unwrap(), levels, names }
    }

    pub fn analyze(&self, text: &str) -> FilterResult {
        let matches: Vec<usize> = self.patterns.matches(text).into_iter().collect();
        if matches.is_empty() {
            return FilterResult {
                level: SensitivityLevel::None, matched_patterns: vec![],
                sanitized_preview: truncate(text, 200), should_encrypt: false, should_index: true,
            };
        }
        let max_level = /* 取 matches 中最高级别 */;
        let is_critical = max_level == SensitivityLevel::Critical;
        FilterResult {
            level: max_level, matched_patterns: /* names */, should_encrypt: is_critical,
            should_index: !is_critical,
            sanitized_preview: if is_critical { "***".into() } else { mask_pii(text) },
        }
    }
}
```

## 集成到 ClipService

```rust
pub fn create_clip(conn: &Connection, content: Vec<u8>) -> Result<Clip, AppError> {
    let text = String::from_utf8_lossy(&content);
    let result = SensitiveFilter::new().analyze(&text);

    let stored = if result.should_encrypt { encrypt(&content)? } else { content };
    let clip = insert_clip(conn, &id, "text", stored, &result.sanitized_preview, &hash, now)?;

    if result.level != SensitivityLevel::None {
        set_sensitivity_tag(conn, &clip.id, &result.level, &result.matched_patterns)?;
    }
    Ok(clip)
}
```

## 性能

- `RegexSet` 编译一次，运行时 O(n) 并行匹配
- 文本 >1MB 跳过正则扫描
- AES-256-GCM 加密开销 ~0.1ms/KB

## 检测命令

```bash
# 敏感内容过滤是否存在
rg -n "SensitiveFilter\|SensitivityLevel\|encrypt_content" src-tauri/src/
# preview 是否可能泄露
rg -n "preview.*content\|String::from_utf8_lossy.*preview" src-tauri/src/
# FTS 是否排除了敏感内容
rg -n "should_index\|clips_fts" src-tauri/src/
```
