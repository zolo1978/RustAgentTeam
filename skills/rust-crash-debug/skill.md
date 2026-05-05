---
name: rust-crash-debug
description: 'Tauri v2 桌面应用闪退/panic 诊断：async panic、WebView crash、macOS abort、DB lock、state storm。决策树格式。'
type: skill
tools: ['Read', 'Glob', 'Grep', 'Bash']
---

# Tauri v2 闪退诊断

## 决策树

```
应用闪退？
├─ panic 信息可见（终端/日志）
│   ├─ "called `unwrap()` on a None value" → 查 unwrap 位置
│   ├─ "called `Result::unwrap()` on an `Err`" → 查 expect/unwrap 调用
│   ├─ "cannot call `blocking_lock` from async context" → Mutex 混用
│   ├─ "not currently running on the Tokio runtime" → block_in_place 上下文错误
│   └─ "panicked at" + spawn_blocking → 检查闭包内 panic
│
├─ 无 panic，窗口直接消失
│   ├─ macOS: app exit code 0 → 检查 app.exit(0) 调用点
│   ├─ macOS: SIGKILL → 系统杀进程（内存/权限）
│   └─ macOS: 无日志 → 检查 enigo/CGEvent accessibility 权限
│
├─ WebView 白屏/内容消失
│   ├─ CSP 违规 → 检查 tauri.conf.json CSP 配置
│   ├─ data URI 渲染失败 → 检查 WKWebView 兼容性
│   └─ JS 错误循环 → 检查 event listener 链式触发
│
└─ 挂起/无响应
    ├─ DB Mutex 死锁 → 检查 spawn_blocking 并发数 + 持锁时间
    ├─ async 阻塞 → 检查 std::fs/std::thread::sleep 在 async fn 中
    └─ 事件风暴 → 检查 emit → listen → setState 循环
```

---

## 1. Panic 定位

### 检测命令

```bash
# 查找所有非启动路径的 unwrap/expect
rg '\.unwrap\(\)' --type rust src-tauri/src/ | grep -v "test" | grep -v "expect("
rg '\.expect\(' --type rust src-tauri/src/ | grep -v "lib.rs" | grep -v "main.rs"

# 查找可能 panic 的数组索引
rg '\[\d\]' --type rust src-tauri/src/ | grep -v "test"
rg '\[.*\]' --type rust src-tauri/src/ | grep -v "let " | grep -v "fn "

# 查找 block_in_place 调用
rg 'block_in_place' --type rust src-tauri/src/
```

### 常见 panic 源

| 位置 | 模式 | 修复 |
|------|------|------|
| `arboard::Clipboard::new().unwrap()` | 剪贴板不可用 | 用 `.ok()?` 或 `map_err` |
| `enigo::Enigo::new().unwrap()` | 无辅助功能权限 | 返回明确错误提示用户授权 |
| `db.lock().unwrap()` | Mutex poisoned | `db.lock().map_err(\|e\| ...)` |
| `spawn_blocking` 内 panic | 闭包内 unwrap | spawn_blocking panic 会传播 |
| `block_in_place` 非 tokio 上下文 | 单线程 runtime | 用 `spawn_blocking` 替代 |

---

## 2. WebView 崩溃

### 检测命令

```bash
# CSP 配置检查
rg '"csp"' src-tauri/tauri.conf.json

# data URI 使用
rg 'data:' --type ts src/
rg 'createObjectURL' --type ts src/

# 事件循环检测
rg 'listen\(' --type ts src/
rg 'emit\(' --type rust src-tauri/src/
```

### WKWebView 专项

| 症状 | 原因 | 修复 |
|------|------|------|
| 大图不显示 | WKWebView 对 data URI 有限制 | 转 blob URL |
| 图片闪烁 | CSP 阻塞 | CSP 添加 `img-src 'self' data: blob:` |
| 白屏 | JS 错误未捕获 | 检查 console error |
| 页面重载循环 | event listener 触发 setState 循环 | 添加去重或防抖 |

### 调试方法

```bash
# macOS 系统日志
log stream --predicate 'subsystem == "com.apple.WebKit"'

# Tauri dev 模式下打开 WebView inspector
# 在 macOS: 右键 → Inspect Element
```

---

## 3. macOS 系统级崩溃

### 检测命令

```bash
# 查看崩溃报告
ls ~/Library/Logs/DiagnosticReports/ | grep ClipVault

# 查看最新崩溃
cat ~/Library/Logs/DiagnosticReports/ClipVault*.crash | head -50

# 检查 accessibility 权限相关代码
rg 'enigo|CGEvent|axusted' --type rust src-tauri/src/
```

### 常见系统级崩溃

| 场景 | 原因 | 修复 |
|------|------|------|
| enigo 使用时闪退 | 无 Accessibility 权限 | 启动时检测，提示用户授权 |
| `screencapture` 失败 | 无屏幕录制权限 | 捕获错误，提示授权 |
| `transparent: true` 崩溃 | 缺少 `macOSPrivateApi` | tauri.conf.json 添加 |
| App Store 审核拒绝 | 使用私有 API | 文档标注，构建条件编译 |

### Accessibility 权限检测模板

```rust
#[cfg(target_os = "macos")]
fn check_accessibility() -> Result<(), AppError> {
    let trusted = unsafe { AXIsProcessTrusted() };
    if !trusted {
        return Err(AppError::Validation(
            "请在系统设置 → 隐私与安全性 → 辅助功能中授权 ClipVault".into(),
        ));
    }
    Ok(())
}
```

---

## 4. 数据库死锁

### 检测命令

```bash
# 查找所有 DB 访问点
rg 'db\.lock\(\)' --type rust src-tauri/src/ | wc -l

# 查找 spawn_blocking 中的 DB 操作
rg 'spawn_blocking' --type rust src-tauri/src/ -A 3 | grep 'db'

# 查找长时间持锁的代码
rg 'db\.lock\(\)' --type rust src-tauri/src/ -A 10 | grep -c "await"
```

### 死锁模式

| 模式 | 后果 | 修复 |
|------|------|------|
| 多个 spawn_blocking 同时 lock | 排队等待，响应慢 | 缩小锁粒度或用连接池 |
| lock 内调用 async 操作 | 编译错误或死锁 | lock 内只做同步操作 |
| lock 持有时间过长 | 其他命令超时 | 拆分事务 |
| Mutex poisoning | 后续 lock 失败 | `lock().map_err` 处理 |

---

## 5. 前端状态风暴

### 检测命令

```bash
# 查找 event listener
rg 'listen\(' --type ts src/ -A 2

# 查找 setState 调用
rg 'set Clips\|setLoading\|setError\|setQuery' --type ts src/

# 查找 useEffect 链
rg 'useEffect' --type tsx src/ -A 3
```

### 风暴模式

| 模式 | 触发链 | 修复 |
|------|--------|------|
| emit → listen → setState → re-render | clip-created → loadClips → setLoading | 用 loadClipsSilent 或 debounce |
| search → setState → useEffect → search | query change → search → setQuery | 防抖 300ms |
| filter → setState → useEffect → load | filterType → loadClips → setLoading | silent mode |
| paste → window hide → 事件丢失 | paste_clip 隐藏窗口后事件不触发 | 在 paste 完成后手动 emit |

---

## 6. 快速诊断清单

闪退时按顺序检查：

```
1. log stream --predicate 'process == "ClipVault"' --level debug
   → 有 panic 信息？→ 第 1 节
   → 无输出？→ 第 3 节

2. 打开 Safari Web Inspector → Console
   → 有 JS error？→ 第 2 节
   → 无 error？→ 第 4 节

3. 检查最近修改的文件
   git diff HEAD~1 -- src-tauri/src/ | grep -E "unwrap|expect|lock|block_in_place"

4. 检查 async 反模式
   rg "std::fs::" --type rust src-tauri/src/
   rg "std::thread::sleep" --type rust src-tauri/src/
```

## 相关 Skill

| Skill | 关联场景 |
|-------|---------|
| [rust-async-patterns](../rust-async-patterns/skill.md) | spawn_blocking 时序、Mutex 死锁 |
| [rust-security-skill](../rust-security-skill/SKILL.md) | enigo 权限、临时文件清理 |
| [rust-core](../rust-core/SKILL.md) | unwrap/expect 替换、错误处理 |
| [rust-release-checklist](../rust-release-checklist/skill.md) | 发版前冒烟测试 |
