---
name: rust-release-checklist
description: 'Tauri v2 发版前强制检查清单：编译、测试、lint、安全扫描、手动冒烟。自动化 + 人工验证。'
type: skill
tools: ['Read', 'Glob', 'Grep', 'Bash']
---

# Tauri v2 发版检查清单

## 执行顺序

```
发版前必须全部通过：
1. 编译检查 → 2. 格式检查 → 3. Lint → 4. 测试 → 5. 类型检查 → 6. 安全扫描 → 7. 构建验证 → 8. 手动冒烟
```

---

## 1. 编译检查

```bash
cargo check --manifest-path src-tauri/Cargo.toml
# 必须: exit code 0, 0 errors
# 允许: warnings（但应修复）
```

---

## 2. 格式检查

```bash
cargo fmt --check --manifest-path src-tauri/Cargo.toml
# 必须: exit code 0
# 失败修复: cargo fmt --manifest-path src-tauri/Cargo.toml
```

---

## 3. Lint（Clippy）

```bash
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
# 必须: exit code 0, 0 warnings
```

常见 clippy 问题速查：

| Lint | 修复 |
|------|------|
| `redundant_closure` | `.map(\|x\| f(x))` → `.map(f)` |
| `clone_on_copy` | 移除不必要的 `.clone()` |
| `let_and_return` | 直接返回表达式 |
| `not_unsafe_ptr_arg_deref` | 添加 unsafe 块或重构 |
| `unused_imports` | 删除未使用的 import |

---

## 4. 测试

```bash
# Rust 测试
cargo test --manifest-path src-tauri/Cargo.toml
# 必须: 0 failures, 覆盖率 ≥80%

# 前端测试
npx vitest run
# 必须: 0 failures

# 覆盖率报告
npx vitest run --coverage
```

### 最低覆盖率要求

| 模块 | 最低 |
|------|------|
| repositories | 90% |
| services | 80% |
| commands | 70% |
| 前端 utils | 90% |
| 前端 hooks | 80% |

---

## 5. 类型检查

```bash
npx tsc --noEmit
# 必须: 0 errors
```

---

## 6. 安全扫描

### 6.1 敏感数据

```bash
# 检查硬编码密钥
rg -i 'api_key|secret|password|token|bearer|authorization|private_key|credential' --type rust src-tauri/src/
rg -i 'api_key|secret|password|token' --type ts src/
# 必须: 0 匹配（排除注释和变量名）
```

### 6.2 临时文件审计

```bash
# 检查所有临时文件写入
rg 'temp_dir\(\)|NamedTempFile|/tmp/' --type rust src-tauri/src/
# 验证每处都有:
#   - 0o600 权限或 tempfile crate
#   - 清理逻辑（删除或 delayed cleanup）
```

### 6.3 CSP 审计

```bash
# 检查 CSP 配置
rg '"csp"' src-tauri/tauri.conf.json
# 禁止: data: 或 blob: 在 default-src（应限制到 img-src）
# 禁止: unsafe-eval 在 script-src
# 推荐: 添加 object-src 'none'; base-uri 'self'; form-action 'none'
```

### 6.4 Capabilities 审计

```bash
# 检查权限声明
cat src-tauri/capabilities/default.json
# 验证: 每个权限都被实际使用
# 禁止: shell:allow-open（除非前端直接使用）
```

### 6.5 SQL 注入审计

```bash
# 检查非参数化查询
rg 'format!\(.*SELECT|format!\(.*INSERT|format!\(.*DELETE|format!\(.*UPDATE' --type rust src-tauri/src/
# 必须: 0 匹配（所有 SQL 用 params![]）
```

---

## 7. 构建验证

```bash
# 前端构建
npm run build
# 必须: 0 errors

# Release 构建
npx tauri build
# 必须: 生成 .app + .dmg 无错误

# 验证产物
ls -lh src-tauri/target/release/bundle/dmg/
ls -lh src-tauri/target/release/bundle/macos/
```

---

## 8. 手动冒烟测试

### 核心流程（全部通过才能发版）

| # | 测试 | 通过标准 |
|---|------|----------|
| 1 | 启动应用 | 窗口出现，圆角，无白边 |
| 2 | 复制文字 | 自动出现在列表 |
| 3 | 粘贴文字 | 窗口隐藏，文字粘贴到目标 app |
| 4 | Cmd+Shift+V | 窗口重新出现 |
| 5 | 搜索 | 列表实时过滤 |
| 6 | 切换 tab | 无闪烁 |
| 7 | 收藏/删除 | 状态正确更新 |
| 8 | 截图 | 截取后出现在列表 |
| 9 | 关闭按钮 | 弹出确认对话框 |
| 10 | 退出 | 应用完全退出 |
| 11 | 重启 | 正常启动 |
| 12 | 托盘菜单 | 显示/退出正常 |

### 性能基线

| 指标 | 阈值 |
|------|------|
| 冷启动 | <2s |
| 搜索响应 | <100ms |
| 内存（闲置） | <50MB |
| 粘贴延迟 | <500ms（从点击到目标 app 收到） |

---

## 9. 发版操作

全部检查通过后：

```bash
# 1. 更新版本号
# package.json + Cargo.toml: "0.1.0" → "0.1.1"

# 2. 提交
git add -A
git commit -m "release: v0.1.1 — <简短描述>"

# 3. 打 tag
git tag -a v0.1.1 -m "v0.1.1: <变更摘要>"

# 4. 推送
git push && git push origin v0.1.1

# 5. 创建 Release
gh release create v0.1.1 \
  --repo zolo1978/clipvault \
  --title "v0.1.1" \
  --notes "<Release Notes>" \
  src-tauri/target/release/bundle/dmg/ClipVault_0.1.1_aarch64.dmg

# 6. 设置 latest
gh release edit v0.1.1 --repo zolo1978/clipvault --latest
```

---

## 快速执行脚本

```bash
# 一键运行所有自动检查
cargo fmt --check --manifest-path src-tauri/Cargo.toml && \
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings && \
cargo test --manifest-path src-tauri/Cargo.toml && \
npx tsc --noEmit && \
npx vitest run && \
npm run build && \
echo "✅ All automated checks passed"
```

## 相关 Skill

| Skill | 关联场景 |
|-------|---------|
| [rust-tauri-testing](../rust-tauri-testing/skill.md) | 测试编写模式、覆盖率要求 |
| [rust-security-skill](../rust-security-skill/SKILL.md) | 安全扫描（步骤 6）详细方法 |
| [rust-crash-debug](../rust-crash-debug/skill.md) | 冒烟测试中的闪退诊断 |
| [rust-async-patterns](../rust-async-patterns/skill.md) | 构建前的 async 反模式检查 |
