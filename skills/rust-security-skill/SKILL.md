---
name: rust-security
description: Tauri v2 桌面应用安全审计和防护 — 敏感内容过滤、Capabilities 最小化、FTS 注入防护、FFI 安全、CSP 配置
tools: ['Read', 'Glob', 'Grep', 'Bash']
---

# Rust Security — Tauri v2 桌面应用安全

## Quick Start
1. 剪贴板存敏感内容？→ Section 1（敏感内容过滤）
2. Capabilities 太宽？→ Section 2（权限最小化）
3. FTS 搜索有拼接？→ Section 3（FTS5 注入防护）
4. arboard/enigo FFI？→ Section 4（FFI 安全）
5. CSP 配置？→ Section 5（WebView 安全）
6. 发布前审计？→ Section 6（综合审计）

## 适用范围
**适用于：** ClipVault 等 Tauri v2 桌面应用的安全审计与防护。
**不适用于：** 通用 Rust 安全（见 `rust-core` skill）、Web 应用安全、服务器安全。
**目标：** Tauri v2 + SQLite FTS5 + 系统 FFI（arboard/enigo）。

---

## Section 1: 敏感内容过滤

**触发信号：** 涉及剪贴板内容存储、搜索结果展示、`create_clip`、`preview` 字段。

### 决策树

```
剪贴板内容到达
  ├─ 内容类型?
  │   ├─ 文本 → 进入正则检测
  │   ├─ 图片 → 跳过文本过滤，检查 EXIF 元数据
  │   └─ 文件路径 → 检查路径穿越
  ├─ 正则匹配命中?
  │   ├─ 命中密码/密钥/Token → SensitivityLevel::Critical
  │   ├─ 命中信用卡/银行卡 → SensitivityLevel::Critical
  │   ├─ 命中邮箱/手机号 → SensitivityLevel::Medium
  │   └─ 未命中 → SensitivityLevel::None
  ├─ 处置策略
  │   ├─ Critical → 加密存储 + 不进 FTS 索引 + preview 显示 "***"
  │   ├─ Medium → 正常存储 + 标记标签 + preview 部分遮蔽
  │   └─ None → 正常存储 + 正常索引
  └─ 异常处理
      └─ 过滤失败 → 拒绝存储，日志记录原因
```

### 正则模式库

| 模式名 | 正则 | 级别 |
|--------|------|------|
| AWS Access Key | `AKIA[0-9A-Z]{16}` | Critical |
| AWS Secret Key | `[A-Za-z0-9/+=]{40}` 上下文关联 | Critical |
| GitHub Token | `gh[ps]_[A-Za-z0-9_]{36,}` | Critical |
| JWT | `eyJ[A-Za-z0-9-_]+\.eyJ[A-Za-z0-9-_]+` | Critical |
| 信用卡 | `\b4[0-9]{12}(?:[0-9]{3})?\b` 等 | Critical |
| 私钥头 | `-----BEGIN (?:RSA \|EC \|DSA )?PRIVATE KEY-----` | Critical |
| 邮箱 | `[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}` | Medium |
| 手机号 | `1[3-9]\d{9}` | Medium |

### 检测命令

```bash
# 检查是否有敏感内容过滤机制
rg -n "sensitivity\|SensitivityLevel\|sensitive\|filter.*content" src-tauri/src/
# 检查 preview 是否做了遮蔽
rg -n "preview.*\*\*\*\|mask\|redact" src-tauri/src/
```

→ 完整实现参考：[references/sensitive-content-filtering.md](references/sensitive-content-filtering.md)

---

## Section 2: Capabilities 权限最小化

**触发信号：** 涉及 `tauri.conf.json`、`capabilities/` 目录、新增 Tauri 命令。

### 决策树

```
新增功能需要权限?
  ├─ 功能需要什么 API?
  │   ├─ 窗口控制 → core:window:allow-{具体操作}
  │   ├─ 全局快捷键 → global-shortcut:allow-register/unregister
  │   ├─ 文件系统 → core:fs:allow-{具体路径+操作}（绝不 allow-*
  │   ├─ Shell → shell:allow-open（绝不 allow-execute
  │   └─ 存储 → store:allow-get/set/delete
  ├─ 权限声明检查
  │   ├─ 有通配符 `allow-*`? → 拆分为具体操作
  │   ├─ 有未使用的权限? → 删除
  │   └─ windows 字段限定? → 必须限定到具体窗口
  ├─ 与 tauri.conf.json 一致?
  │   ├─ 插件已注册? → 检查 .plugin() 调用
  │   └─ 命令已暴露? → 检查 invoke_handler
  └─ 验证
      └─ cargo build → 无 capability 警告
```

### ClipVault 当前权限分析

```json
// capabilities/default.json — 当前声明
"permissions": [
  "core:default",                    // OK — 基础权限集
  "core:window:allow-show",          // OK — 窗口显示
  "core:window:allow-hide",          // OK — 窗口隐藏
  "core:window:allow-set-focus",     // OK — 焦点设置
  "core:window:allow-close",         // OK — 窗口关闭
  "core:window:allow-set-title",     // OK — 标题设置
  "global-shortcut:allow-register",  // OK — 快捷键注册
  "global-shortcut:allow-unregister",// OK — 快捷键注销
  "global-shortcut:allow-is-registered", // OK — 快捷键查询
  "shell:allow-open",                // 风险 — 审计所有 open 调用
  "store:allow-*"                    // OK — 本地存储 CRUD
]
```

### 检测命令

```bash
# 检查 capabilities 目录
find src-tauri/capabilities -name "*.json" -exec echo "=== {} ===" \; -exec cat {} \;
# 查找通配符权限
rg 'allow-\*' src-tauri/capabilities/
# 对比注册的插件 vs 声明的权限
rg '\.plugin\(' src-tauri/src/lib.rs
```

→ 完整检查流程：[references/capabilities-hardening.md](references/capabilities-hardening.md)

---

## Section 3: FTS5 注入防护

**触发信号：** 涉及 `search_clips`、FTS 查询、`MATCH` 关键字、用户搜索输入。

### 决策树

```
用户搜索输入到达
  ├─ 输入来源?
  │   ├─ 前端搜索框 → 不信任，必须清洗
  │   └─ 内部调用 → 仍需验证，防御纵深
  ├─ 当前实现 (clip_repo.rs:115)?
  │   ├─ format!("\"{}\"*", query.replace('"', "\"\""))
  │   └─ 参数化传入 MATCH ? → OK，但引号转义需审计
  ├─ 清洗规则
  │   ├─ 双引号转义 → replace('"', "\"\"")  ✓ 当前已做
  │   ├─ 特殊 FTS 语法 → *, ^, +, -, AND, OR, NOT, NEAR
  │   │   └─ 包裹在引号内即可抑制所有 FTS 操作符 ✓
  │   ├─ 空查询 → 拒绝或返回空结果
  │   └─ 超长查询 → 截断至 200 字符
  └─ 验证
      └─ 单元测试覆盖: 正常词、引号注入、FTS 操作符、超长输入、空输入
```

### 当前实现安全评估

```rust
// clip_repo.rs:115 — 当前实现
let fts_query = format!("\"{}\"*", query.replace('"', "\"\""));
// ✅ 使用参数化查询 (MATCH ?)
// ✅ 双引号转义 ("" -> "）" ")
// ✅ 外层引号包裹抑制 FTS 操作符
// ⚠️ 未限制输入长度
// ⚠️ 未处理空查询
// ⚠️ 未处理仅含特殊字符的查询
```

### 检测命令

```bash
# 检查所有 SQL 拼接
rg 'format!.*SELECT\|format!.*INSERT\|format!.*DELETE\|format!.*UPDATE' src-tauri/src/
# 检查 MATCH 用法
rg 'MATCH\|fts\|FTS' src-tauri/src/
# 检查参数化查询
rg 'params!\|ToSql' src-tauri/src/
```

→ 完整防护方案：[references/fts-injection-prevention.md](references/fts-injection-prevention.md)

---

## Section 4: FFI 安全

**触发信号：** 涉及系统级 FFI 调用、`arboard`（剪贴板读写）、`enigo`（键盘模拟）、`unsafe` 块。

### 决策树

```
FFI 调用点
  ├─ 调用类型?
  │   ├─ arboard::Clipboard::new() → 系统剪贴板
  │   │   ├─ 可能 panic? → catch_unwind 包裹
  │   │   ├─ 线程安全? → Clipboard 不是 Send/Sync，需线程局部
  │   │   └─ 超时? → spawn_blocking + timeout(Duration::from_secs(5))
  │   ├─ enigo::Keyboard::text() → 键盘模拟
  │   │   ├─ 输入内容受限? → 仅允许 UTF-8 文本，禁止控制字符
  │   │   ├─ 速率限制? → 两次 paste 间隔 ≥ 100ms
  │   │   └─ 错误恢复? → 失败后清理键盘状态
  │   └─ 直接 unsafe FFI → 需要 SAFETY 注释文档化
  ├─ 资源管理
  │   ├─ Clipboard handle → Drop trait 自动释放
  │   ├─ 文件描述符 → RAII 包装，Drop 关闭
  │   └─ 内存 → 所有权明确，无手动 alloc/free
  └─ 测试
      ├─ FFI 调用 → catch_unwind 包裹的集成测试
      └─ 超时 → mock 测试 timeout 行为
```

### 检测命令

```bash
# 查找所有 unsafe 块
rg 'unsafe\s*\{' src-tauri/src/
# 查找 arboard/enigo 使用
rg 'arboard\|enigo' src-tauri/
# 查找 catch_unwind
rg 'catch_unwind' src-tauri/src/
# 查找 spawn_blocking
rg 'spawn_blocking' src-tauri/src/
```

→ 完整 FFI 安全指南：[references/ffi-safety.md](references/ffi-safety.md)

---

## Section 5: CSP 和 WebView 安全

**触发信号：** 涉及前端安全、`csp` 配置、`tauri.conf.json` 的 `app.security` 字段。

### 决策树

```
CSP 配置审计
  ├─ 当前 CSP (tauri.conf.json:25)?
  │   "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'"
  │   ├─ default-src 'self' → ✅ 好
  │   ├─ script-src 'self' → ✅ 无远程脚本
  │   ├─ style-src 'unsafe-inline' → ⚠️ 必要但需评估
  │   └─ 缺少 img-src/font-src/connect-src → 由 default-src 兜底
  ├─ freezePrototype: true → ✅ 防止原型污染
  ├─ 危险模式检查
  │   ├─ script-src 含 'unsafe-eval'? → 禁止
  │   ├─ 含 https:// 外部域名? → 需要白名单理由
  │   └─ 含 data: URI? → 仅 img-src 允许
  ├─ WebView 安全
  │   ├─ devtools 在生产环境? → 必须禁用
  │   ├─ navigation 到外部 URL? → 拦截或确认
  │   └─ IPC 消息验证? → 命令参数校验
  └─ 前端代码
      ├─ innerHTML? → 用 textContent
      ├─ v-html / dangerouslySetInnerHTML? → 消毒后使用
      └─ 评价/用户输入直接展示? → 转义处理
```

### 检测命令

```bash
# 检查 CSP 配置
rg 'csp\|security' src-tauri/tauri.conf.json
# 检查前端危险 API
rg 'innerHTML\|v-html\|dangerouslySetInnerHTML\|document\.write' src/
# 检查外部资源加载
rg 'https?://\|cdn\.' src/ --type html --type ts --type tsx --type vue
```

→ 完整 CSP 配置指南：[references/csp-and-permissions.md](references/csp-and-permissions.md)

---

## Section 6: 安全审计

**触发信号：** 发布前、重大变更后、安全 incident 后、定期审计（每季度）。

### 决策树

```
安全审计启动
  ├─ 确定范围
  │   ├─ 全量审计 → 覆盖全部 6 个维度
  │   └─ 增量审计 → 仅审计变更涉及维度
  ├─ 执行审计（使用 checklist 模板）
  │   ├─ P0 — 阻断发布
  │   │   ├─ 敏感内容明文存储
  │   │   ├─ SQL/FTS 注入漏洞
  │   │   ├─ 未限定的 shell:allow-execute
  │   │   └─ CSP 含 unsafe-eval
  │   ├─ P1 — 限期修复（下个版本）
  │   │   ├─ Capabilities 权限过度
  │   │   ├─ FFI 无超时保护
  │   │   └─ 前端未转义用户输入
  │   └─ P2 — 持续改进
  │       ├─ 敏感内容检测覆盖率
  │       ├─ 审计日志完整性
  │       └─ 安全测试自动化
  ├─ 输出
  │   ├─ 发现列表（P0/P1/P2 分级）
  │   ├─ 修复建议（每项附代码示例）
  │   └─ 复测计划
  └─ 关闭条件
      └─ P0 全部修复 + P0 复测通过
```

### 自动化审计命令

```bash
# 一键安全扫描
cd src-tauri && cargo audit 2>&1           # 依赖漏洞
cd src-tauri && cargo deny check 2>&1      # 许可证审计
rg 'unsafe\s*\{' src-tauri/src/             # unsafe 使用
rg 'unwrap\(\)' src-tauri/src/              # unwrap 使用
rg 'format!.*SELECT\|format!.*WHERE.*\{' src-tauri/src/  # SQL 拼接
```

→ 完整审计清单：[templates/security-audit-checklist.md](templates/security-audit-checklist.md)

---

## Section 7: 剪贴板管理器专项安全

**触发信号：** 剪贴板监控应用、paste 操作、临时文件预览、pasteboard 权限。

### 决策树

```
剪贴板管理器安全
  ├─ 临时文件预览
  │   ├─ 文件名可预测? → 必须用 UUID/随机名
  │   ├─ 权限 0o600? → 必须，防止其他用户读取
  │   ├─ create_new(true)? → 必须，防 symlink 攻击
  │   ├─ 清理机制? → delayed cleanup (tokio::spawn + sleep)
  │   └─ 位置? → temp_dir()，不放在应用目录
  ├─ Paste 操作竞态
  │   ├─ 并发粘贴? → AtomicBool reentry guard
  │   ├─ 监控器重捕获? → is_pasting flag 暂停监控
  │   ├─ 窗口隐藏时序? → hide → 150ms → set_clipboard → 50ms → simulate → 200ms → restore
  │   └─ 剪贴板还原? → paste 前保存 → paste 后恢复
  ├─ Pasteboard 权限
  │   ├─ macOS Accessibility → enigo 需要，启动时检测
  │   ├─ macOS Screen Recording → screencapture 需要
  │   └─ 权限缺失 → 明确提示用户去系统设置授权
  └─ 路径安全
      ├─ canonicalize → 防穿越
      ├─ 拒绝系统路径 → /System/, /private/var/
      └─ UUID 验证 → 防注入任意路径
```

### 检测命令

```bash
# 临时文件安全
rg 'temp_dir\(\)|NamedTempFile|/tmp/' src-tauri/src/
rg '0o600|create_new|mode\(' src-tauri/src/
rg 'remove_file|cleanup' src-tauri/src/

# Paste 竞态
rg 'is_pasting|AtomicBool|load.*SeqCst' src-tauri/src/
rg 'Ordering::SeqCst' src-tauri/src/

# 权限检测
rg 'AXIsProcessTrusted|accessibility|enigo' src-tauri/src/
rg 'canonicalize|starts_with.*System|starts_with.*private' src-tauri/src/
```

### 临时文件安全模板

```rust
// 安全的临时文件写入
fn write_temp_preview(id: &str, content: &[u8]) -> Result<PathBuf, AppError> {
    let path = std::env::temp_dir().join(format!("app-preview-{}.png", id));

    // UUID 验证防路径注入
    if uuid::Uuid::parse_str(id).is_err() {
        return Err(AppError::Validation("invalid id".into()));
    }

    use std::os::unix::fs::OpenOptionsExt;
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)  // 防 symlink
        .mode(0o600)       // 仅 owner 可读写
        .open(&path)
        .and_then(|mut f| std::io::Write::write_all(&mut f, content))?;

    Ok(path)
}

// 延迟清理
tokio::spawn(async move {
    tokio::time::sleep(std::time::Duration::from_secs(30)).await;
    let _ = tokio::fs::remove_file(&cleanup_path).await;
});
```

---

## References
| Topic | File |
|-------|------|
| 敏感内容过滤 | [references/sensitive-content-filtering.md](references/sensitive-content-filtering.md) |
| Capabilities 最小化 | [references/capabilities-hardening.md](references/capabilities-hardening.md) |
| FTS5 注入防护 | [references/fts-injection-prevention.md](references/fts-injection-prevention.md) |
| FFI 安全 | [references/ffi-safety.md](references/ffi-safety.md) |
| CSP 和 WebView 安全 | [references/csp-and-permissions.md](references/csp-and-permissions.md) |
| 安全审计清单 | [templates/security-audit-checklist.md](templates/security-audit-checklist.md) |
| 相关 Skill | [rust-async-patterns](../rust-async-patterns/skill.md) — spawn_blocking 时序 |
| 相关 Skill | [rust-crash-debug](../rust-crash-debug/skill.md) — enigo panic 诊断 |
