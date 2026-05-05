# 系统集成检查清单

> V1 覆盖 macOS + Tauri 官方插件。Linux/Windows 放 P2。

## 1. 剪贴板（macOS ✅ | Windows P2 | Linux P2）

- [ ] arboard 读写文本正常
- [ ] 剪贴板监听：轮询模式 ≤ 500ms 检测间隔
- [ ] 安全封装：catch_unwind + spawn_blocking + timeout 3s
- [ ] 内容去重：hash 比对，相同内容不重复存储
- [ ] 粘贴流程：写入剪贴板 → 模拟 Cmd+V → 隐藏窗口
```bash
rg -n 'Clipboard::new\|get_text\|set_text' src-tauri/src/
rg -n 'spawn_blocking\|catch_unwind\|timeout' src-tauri/src/
```

## 2. 全局热键（macOS ✅ | Windows P2 | Linux P2）

- [ ] tauri-plugin-global-shortcut 注册成功
- [ ] Cmd+Shift+V 切换窗口显示/隐藏
- [ ] 热键在应用无焦点时响应
- [ ] 冲突检测：注册前检查 is_registered
- [ ] Capabilities 包含 global-shortcut 权限
```bash
rg -n 'global_shortcut\|GlobalShortcutExt\|register' src-tauri/src/
rg 'global-shortcut' src-tauri/capabilities/
```

## 3. 系统托盘（macOS ✅ | Windows P2 | Linux P2）

- [ ] 托盘图标显示
- [ ] macOS template icon 适配亮暗色
- [ ] 右键菜单：显示/偏好设置/退出
- [ ] macOS 左键点击显示窗口
- [ ] tauri.conf.json trayIcon 配置正确
```bash
rg -n 'TrayIconBuilder\|trayIcon' src-tauri/src/ src-tauri/tauri.conf.json
rg -n 'menu_event\|MenuItem' src-tauri/src/
```

## 4. 窗口管理（macOS ✅ | Windows P2 | Linux P2）

- [ ] 无边框窗口 + 自定义拖拽区域
- [ ] alwaysOnTop 生效
- [ ] 显示/隐藏切换正常
- [ ] 失去焦点自动隐藏
- [ ] skipTaskbar 不在任务栏显示
```bash
rg -n 'decorations\|alwaysOnTop\|skipTaskbar' src-tauri/tauri.conf.json
rg -n 'data-tauri-drag-region' src/
rg -n 'window\(\)\.hide\|window\(\)\.show\|set_focus' src-tauri/src/
```

## 5. 签名分发（macOS ✅ | Windows P2）

- [ ] macOS codesign 签名通过
- [ ] macOS notarize 公证通过
- [ ] .app + .dmg 打包成功
- [ ] Windows 代码签名（P2）
```bash
codesign --verify --deep --strict target/release/bundle/macos/ClipVault.app
```

## 总结

| 集成域 | macOS | Windows | Linux |
|--------|-------|---------|-------|
| 剪贴板 | V1 | P2 | P2 |
| 全局热键 | V1 | P2 | P2 |
| 系统托盘 | V1 | P2 | P2 |
| 窗口管理 | V1 | P2 | P2 |
| 签名分发 | V1 | P2 | P2 |
