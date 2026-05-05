# Capabilities 权限最小化 — Tauri v2 安全参考

## 概述

Tauri v2 用 Capabilities 控制前端可调用的后端 API。最小权限原则：只声明实际使用的权限，拒绝通配符。

## 三者一致性

- `lib.rs` 注册的插件 <-> `capabilities/` 声明的权限 <-> 前端实际调用
- 前端只能调用三者交集

## BAD vs GOOD

### BAD — 通配符权限

```json
{
  "permissions": [
    "core:window:allow-*",
    "shell:allow-*",
    "fs:allow-*"
  ]
}
```

`shell:allow-*` 允许执行任意命令，最危险。`fs:allow-*` 允许访问所有路径。

### GOOD — ClipVault 最小权限集

```json
{
  "identifier": "default",
  "description": "ClipVault minimal permission set",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "core:window:allow-show",
    "core:window:allow-hide",
    "core:window:allow-set-focus",
    "core:window:allow-close",
    "core:window:allow-set-title",
    "global-shortcut:allow-register",
    "global-shortcut:allow-unregister",
    "global-shortcut:allow-is-registered",
    "shell:allow-open",
    "store:allow-get",
    "store:allow-set",
    "store:allow-delete",
    "store:allow-keys",
    "store:allow-load",
    "store:allow-save"
  ]
}
```

## 权限审计矩阵

| 功能 | 权限 | ClipVault 需要 |
|------|------|---------------|
| 显示窗口 | allow-show | 是 |
| 隐藏窗口 | allow-hide | 是 |
| 聚焦窗口 | allow-set-focus | 是 |
| 关闭窗口 | allow-close | 是 |
| 调整大小 | allow-set-size | 否 — 固定尺寸 |
| 移动窗口 | allow-set-position | 否 |
| Shell 执行 | shell:allow-execute | **永远不要** |
| Shell 打开 | shell:allow-open | 是 — 仅浏览器 |

## Shell 权限红线

- `shell:allow-open` — 仅在默认浏览器打开 URL
- `shell:allow-execute` — **永远不要声明**
- 外部 URL 应限制为白名单域名

## 新增权限流程

```
1. 确认最小权限（allow-{具体操作}，不用 allow-*）
2. 添加到 capabilities/default.json
3. 确认 lib.rs 注册了对应插件
4. 确认前端只使用了声明的操作
5. cargo build 无 capability 警告
6. Code review 重点关注权限变更
```

## 检测命令

```bash
# 通配符检查（必须修复）
rg 'allow-\*' src-tauri/capabilities/
# shell:allow-execute 检查（严禁）
rg 'allow-execute' src-tauri/capabilities/
# 对比插件注册 vs 权限声明
rg '\.plugin\(' src-tauri/src/lib.rs
# 前端是否有直接文件/网络调用（绕过 Capabilities）
rg '@tauri-apps/plugin-fs\|@tauri-apps/plugin-http' src/
# 未使用权限检查
for p in $(rg -o 'allow-[a-z-]+' src-tauri/capabilities/ | sort -u); do
  rg -q "${p#allow-}" src/ || echo "WARN: $p 可能未使用"
done
```
