# CSP 和 WebView 安全

## 概述

Tauri 使用系统 WebView 渲染前端。CSP（Content Security Policy）控制 WebView 可加载的资源。

## ClipVault CSP 配置

```json
// tauri.conf.json
{
  "app": {
    "security": {
      "csp": "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'",
      "freezePrototype": true
    }
  }
}
```

## 各指令说明

| CSP 指令 | 值 | 原因 |
|---------|---|------|
| `default-src 'self'` | 仅本地 | 默认拒绝所有外部资源 |
| `script-src 'self'` | 仅本地 | 禁止 inline script 和 eval |
| `style-src 'self' 'unsafe-inline'` | 本地 + inline | Tailwind CSS 需要 inline style |

## 安全边界

### 允许

- `self` 来源的所有资源（打包在 app 内）
- inline CSS（Tailwind JIT 编译需要）
- `unsafe-inline` 仅限 `style-src`（不影响 JS 安全）

### 拒绝

- 外部 CDN（fonts、scripts、styles）
- `eval()` 和 `new Function()`
- inline `<script>` 标签
- `javascript:` 伪协议
- `data:` URI 中的脚本

## BAD vs GOOD

### BAD — 开发便利但危险的 CSP

```json
{
  "csp": "default-src 'self' 'unsafe-inline' 'unsafe-eval'; img-src * data:;"
}
```

问题：`unsafe-inline` 在 `default-src` 中允许了 inline script，`unsafe-eval` 允许了 eval。

### GOOD — 最严格的 CSP

```json
{
  "csp": "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self'"
}
```

## freezePrototype

`freezePrototype: true` 冻结 JavaScript 原型链，防止 prototype pollution 攻击。

```javascript
// 被 freezePrototype 阻止
Object.prototype.evil = "pwned"; // 静默失败，不影响现有对象
```

## WebView 安全检查清单

- [ ] CSP 不包含 `unsafe-eval`
- [ ] `script-src` 不包含 `unsafe-inline`
- [ ] `style-src` 的 `unsafe-inline` 是唯一例外
- [ ] `freezePrototype: true` 已启用
- [ ] 无 `<script>` 内联标签
- [ ] 无 `eval()` / `new Function()` 调用
- [ ] 无外部资源引用（CDN）

## 检测命令

```bash
# 验证 CSP 配置
grep -A3 '"security"' src-tauri/tauri.conf.json

# 检查 inline script
rg '<script>' src/ --include="*.html" --include="*.tsx"

# 检查 eval 使用
rg 'eval\(|new Function\(' src/ --include="*.ts" --include="*.tsx"

# 检查外部资源引用
rg 'https?://' src/ --include="*.html" --include="*.tsx" --include="*.css"
```
