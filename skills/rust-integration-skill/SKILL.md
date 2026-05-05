---
name: rust-integration
description: 'Tauri v2 桌面应用系统集成 — 剪贴板、全局热键、系统托盘、窗口管理、签名分发'
type: reference
---

# Tauri v2 系统集成

## Quick Start

遇到系统集成问题时，从匹配的 Section 开始。每个 Section 都是独立的决策树。

## 适用范围

**适用于：** Tauri v2 桌面应用的系统集成（剪贴板、热键、托盘、窗口、签名）。
**不适用于：** 通用 Rust 后端（见 `rust-backend`）、前端 UI（见 `rust-ui-skill`）。
**目标：** V1 覆盖 macOS + Tauri 官方插件。Linux/Windows 放 P2。

## 1. 系统剪贴板

**触发信号：** 剪贴板读写、监听

**决策树：**

```
剪贴板操作
  ├─ 读取内容
  │   ├─ 文本 → arboard::Clipboard::get_text()
  │   ├─ 图片 → arboard::Clipboard::get_image()
  │   └─ 文件路径 → 解析文本中的路径模式
  ├─ 写入内容
  │   ├─ 文本 → arboard::Clipboard::set_text()
  │   └─ 图片 → arboard::Clipboard::set_image()
  ├─ 监听变化
  │   ├─ 轮询模式 → spawn_blocking + 500ms 间隔
  │   └─ 事件模式 → macOS NSClipboard 通知（V2）
  └─ 安全封装
      ├─ catch_unwind — 防 panic
      ├─ spawn_blocking — 防阻塞
      └─ timeout 3s — 防挂起
```

**依赖：** `arboard = "3"`

**关键代码：** 见 `references/clipboard-integration.md`

## 2. 全局热键

**触发信号：** 快捷键、热键

**决策树：**

```
需要全局热键
  ├─ 选择快捷键
  │   ├─ macOS → Cmd+Shift+V（推荐）
  │   ├─ Windows → Ctrl+Shift+V
  │   └─ 自定义 → 用户偏好设置
  ├─ 注册热键
  │   ├─ Tauri 插件 → tauri-plugin-global-shortcut
  │   └─ 格式 → Shortcut::new(Modifiers::SUPER | SHIFT, Code::KeyV)
  ├─ 响应热键
  │   ├─ 窗口可见 → hide
  │   └─ 窗口隐藏 → show + focus
  └─ 冲突处理
      ├─ 检测 → is_registered()
      └─ 用户自定义 → 偏好设置界面
```

**依赖：** `tauri-plugin-global-shortcut`

**关键代码：** 见 `references/global-shortcut.md`

## 3. 系统托盘

**触发信号：** 托盘、后台运行

**决策树：**

```
需要系统托盘
  ├─ 创建托盘图标
  │   ├─ macOS → template icon (iconAsTemplate: true)
  │   └─ Windows → .ico 文件
  ├─ 托盘菜单
  │   ├─ 显示窗口
  │   ├─ 偏好设置
  │   ├─ 分隔线
  │   └─ 退出
  ├─ 点击行为
  │   ├─ macOS → 左键显示窗口
  │   └─ Windows → 左键显示菜单
  └─ 配置
      ├─ tauri.conf.json → trayIcon 节
      └─ Rust → TrayIconBuilder
```

**关键代码：** 见 `references/system-tray.md`

## 4. 窗口管理

**触发信号：** 窗口配置、窗口操作

**决策树：**

```
窗口需求
  ├─ 无边框 → decorations: false
  │   └─ 拖拽区域 → data-tauri-drag-region
  ├─ 置顶 → alwaysOnTop: true
  ├─ 跳过任务栏 → skipTaskbar: true
  ├─ 固定大小 → resizable: false
  └─ 显示/隐藏
      ├─ show/hide → Window API
      └─ 焦点 → set_focus()
```

**配置：** `tauri.conf.json` → `app.windows[]`

**关键代码：** 见 `references/window-management.md`

## 5. 签名与分发

**触发信号：** 发布、打包、分发

**决策树：**

```
发布目标
  ├─ macOS
  │   ├─ codesign → identity + entitlements
  │   ├─ notarize → notarytool (Apple ID + app password)
  │   ├─ 打包 → .app + .dmg
  │   └─ CI → GitHub Actions + macos-latest
  ├─ Windows
  │   ├─ signtool → code signing certificate
  │   ├─ 打包 → .msi + .exe (NSIS)
  │   └─ CI → GitHub Actions + windows-latest
  └─ Linux（P2）
      ├─ AppImage
      └─ deb/rpm
```

**关键代码：** 见 `references/signing-distribution.md`

## 6. 集成验证

**触发信号：** 集成开发完成、发布前

**使用模板：** `templates/integration-checklist.md`

**决策树：**

```
验证系统集成
  ├─ 剪贴板 → 读写测试、监听测试
  ├─ 热键 → 注册/响应/冲突测试
  ├─ 托盘 → 图标显示、菜单、点击
  ├─ 窗口 → 显示/隐藏/置顶/拖拽
  └─ 签名 → codesign --verify、spctl --assess
```

## 参考资料

| 文件 | 内容 |
|------|------|
| `references/clipboard-integration.md` | arboard 剪贴板集成 |
| `references/global-shortcut.md` | 全局热键注册和响应 |
| `references/system-tray.md` | 系统托盘创建和菜单 |
| `references/window-management.md` | 窗口管理和配置 |
| `references/signing-distribution.md` | macOS/Windows 签名和分发 |

## 模板

| 文件 | 用途 |
|------|------|
| `templates/integration-checklist.md` | 系统集成检查清单 |
