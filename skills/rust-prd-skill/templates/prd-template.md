# PRD 模板 — [Replace with 产品名称]

> 复制此文件，替换所有 `[Replace with...]` 占位符。参考 SKILL.md Section 6 审核检查清单。
> Minimum viable PRD: fill Background + Users + Feature table (P0 only) + Tech constraints first. Remaining sections can be added iteratively.

## 1. 背景

[Replace with 量化数据 + 差距分析。例：目标用户日均执行 XX 操作，现有方案缺少 A/B/C 功能。竞品 X 仅支持平台 Y，竞品 Z 需联网且月费 $N。我们的机会：原生性能 + 离线优先 + 跨平台。]

## 2. 目标用户

- **主要用户**：[Replace with 画像 — 年龄/职业/核心工作流。例：自由摄影师，20-45 岁，管理 10K+ 素材]
- **次要用户**：[Replace with 次要群体，可为空]
- **桌面端场景**：[Replace with 高频/长时任务描述]
- **移动端场景**：[Replace with 低频/短时任务描述，如无移动端可删除]

## 3. 核心功能表

| 功能 | 优先级 | 描述 | 验收标准 |
|------|--------|------|----------|
| [Replace with P0 功能名] | P0 | [Replace with 简述] | AC1: [Replace with 可测试条件] |
| | | | AC2: [Replace with 可测试条件] |
| [Replace with P1 功能名] | P1 | [Replace with 简述] | AC1: [Replace with 可测试条件] |
| [Replace with P2 功能名] | P2 | [Replace with 简述] | AC1: [Replace with 可测试条件] |

> 每个 P0 功能至少 2 条 AC。优先级定义见 SKILL.md Section 2。

## 4. 离线策略

| 功能 | 离线级别 | 说明 |
|------|---------|------|
| [Replace with 核心功能] | 必须离线 | [Replace with 理由] |
| [Replace with 同步功能] | 降级可用 | 离线排队，联网后同步 |
| [Replace with 验证功能] | 降级可用 | 本地缓存 N 天 |

## 5. 平台差异

| 功能 | Win/Mac | Linux | iOS | Android |
|------|---------|-------|-----|---------|
| [Replace with 桌面特有功能] | P1 | P1 | N/A | N/A |
| [Replace with 通用功能] | P1 | P1 | P1 | P1 |
| [Replace with 移动特有功能] | N/A | N/A | P2 | P2 |

## 6. 渐进增强

- **基础（全平台）**：[Replace with 核心工作流，例：导入 → 浏览 → 搜索 → 标签]
- **增强（桌面端）**：[Replace with 进阶功能，例：批量操作 → 快捷键 → 拖拽 → 系统集成]
- **移动端增强**：[Replace with 移动特有功能，例：相机直连导入 → NFC 分享]

## 7. 性能指标

| 指标 | 目标值 | 测量方法 |
|------|--------|----------|
| 冷启动 | [Replace with，例：< 800ms] | [Replace with，例：双击图标到首屏可交互，Release build] |
| 热启动 | [Replace with，例：< 200ms] | [Replace with 测量方法] |
| 内存（空闲） | [Replace with，例：< 80MB] | [Replace with 测量方法] |
| IPC 延迟 | [Replace with，例：< 5ms P99] | 前端 → Rust 单次 invoke |

## 8. 技术约束

- Rust: [Replace with MSRV，例：1.80+]，MSRV 写入 CI
- Tauri: [Replace with 版本，例：v2.x]，前端 @tauri-apps/api v2
- 前端：[Replace with 框架，例：React 19 / Svelte 5 / Vue 3] + TypeScript 5.x + Vite 6 + Tailwind 4
- 最低系统：[Replace with，例：Win 10 21H2 / macOS 13 / Ubuntu 22.04 / iOS 16 / Android 13]
- 存储：[Replace with，例：本地 SQLite + tauri-plugin-store]
- 分发：[Replace with，例：Win NSIS/MSI / macOS DMG / Linux AppImage+deb]

## 8.5. 安全与隐私

- 数据收集：[Replace with 收集哪些用户数据]
- 本地存储加密：[Replace with 敏感数据加密策略，如 tauri-plugin-store + OS keychain]
- 网络传输：[Replace with TLS/HTTPS 要求]
- 权限声明：[Replace with Capabilities 最小权限集]
- 合规要求：[Replace with GDPR/CCPA/App Store 审核]
## 9. KPI（[Replace with 时间线，例：上线 3 个月]）

| 指标 | 目标 | 测量方式 |
|------|------|----------|
| DAU | [Replace with] | [Replace with] |
| 次日留存 | [Replace with，例：> 40%] | [Replace with] |
| 崩溃率 | [Replace with，例：< 0.1%] | [Replace with 崩溃上报工具] |

## 10. 发布计划

[Replace with 分阶段计划。例：V1.0: Win/Mac P0+P1 → V1.1: Linux + P2 → V1.2: iOS → V1.3: Android]
