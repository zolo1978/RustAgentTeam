---
name: rust-prd-skill
description: 'Rust 桌面/App PRD 专业编写指南。含各节填写规范、P0/P1/P2 严格定义、用户故事格式、竞品分析模板、成功指标。面向 Tauri v2。'
type: skill
---

# Rust 桌面/App PRD 编写指南

## Scope

**For:** Writing PRDs for Tauri v2-based Rust desktop/mobile applications.
**Not for:** CLI tools, pure Rust backend services, non-Tauri frameworks (Electron/Flutter/Qt), Web SaaS products.
**Requires:** Tauri v2 basics (WebView, IPC, Capabilities). All constraints and targets assume Tauri v2 + Rust.
**Target:** PRDs with quantified metrics, testable AC, and cross-platform strategy.

## Quick Start
1. Open `templates/prd-template.md` as starting point
2. Fill: Background (2+ data points) → Users (1+ persona) → Features (all P0 with AC)
3. Validate: Run Section 6 audit checklist
4. **Minimum viable PRD** = Background + Users + P0 Features + Tech Constraints. If any missing, ask first.

## Directory

| Topic | Summary | Reference |
|-------|---------|-----------|
| 竞品分析 + KPI | 竞品对比表模板、业务 KPI 指标模板 | [references/competitor-analysis.md](references/competitor-analysis.md) |
| 复杂场景 | 离线策略、平台特定功能、渐进增强 | [references/complex-scenarios.md](references/complex-scenarios.md) |
| BAD/GOOD 对比 | 功能描述、性能目标、跨平台策略等 6 类正反例 | [references/bad-good-comparisons.md](references/bad-good-comparisons.md) |

| Template | Description | File |
|----------|-------------|------|
| PRD 模板 | 带占位符的完整 PRD 结构，可复制作为起点 | [templates/prd-template.md](templates/prd-template.md) |

## 1. 各节填写规范

> 详细 BAD/GOOD 示例见 [references/bad-good-comparisons.md](references/bad-good-comparisons.md)

### 1.1 需求背景
包含量化数据 + 差距分析。BAD："市场上这类产品不多"。GOOD："用户日均处理 200+ 文件，竞品 A 仅 macOS，竞品 B 月费 $15，机会：离线优先+跨平台"。

### 1.2 目标用户
具体画像 + 场景区分。BAD："所有需要管理文件的用户"。GOOD："自由摄影师 20-45 岁管理 10K+ 素材（桌面高频 30min+），设计团队 3-10 人共享标签"。

### 1.3 功能设计
可测试的功能描述 + 验收标准表格。BAD："文件导入功能"。GOOD："批量导入 P0，AC1: 1000 文件 < 5s，AC2: 可取消保留已导入，AC3: MD5 去重"。

### 1.4 性能指标
具体数字 + 测量方法。BAD："应用应该快速"。GOOD："冷启动 < 800ms（Release build 首屏可交互），IPC < 5ms P99，搜索 < 200ms/10K 文件"。

### 1.5 技术约束
完整约束矩阵。BAD："用 Rust + Tauri"。GOOD："Rust 1.80+ MSRV 写 CI / Tauri v2 / React 19 + TS 5.x / Win 10 21H2+ macOS 13+ / SQLite 离线优先"。

## 2. 优先级严格定义

- **P0 = MVP 阻塞项**：没有此功能应用无法运行或完全无用。P0 缺失 = 不能发布。
- **P1 = V1.0 必需**：发布时必须有，否则体验残缺。P1 缺失 = 产品不完整。
- **P2 = 可推迟至 V1.x**：锦上添花，发布后迭代加入。

**判断方法**：砍掉这个功能，用户能否完成核心工作流？不能 = P0。能但不爽 = P1。无所谓 = P2。

## 3. 用户故事格式

```
As a [角色], I want [动作], so that [收益]

验收标准（至少 2 条 AC）：
- AC1: [可测试的具体条件]
- AC2: [可测试的具体条件]
```

**示例**：
```markdown
As a 摄影师, I want 通过拖拽导入 RAW+JPEG 文件对,
so that 我可以快速将拍摄素材纳入管理而不需要逐个选择。

AC1: 拖拽包含 500 个 RAW+JPEG 对的文件夹，5 秒内完成索引。
AC2: 系统自动识别配对关系（相同文件名不同后缀），在 UI 中合并展示。
AC3: 导入过程中显示进度条，支持取消操作。
AC4: 取消后已导入的文件保留在库中，不回滚。
```

## 4. 竞品分析 & 成功指标
→ Details: [references/competitor-analysis.md](references/competitor-analysis.md)

## 5. 复杂场景指导
→ Details: [references/complex-scenarios.md](references/complex-scenarios.md)

## 6. PRD 审核检查清单

- [ ] 背景包含量化数据和差距分析（非空话）
- [ ] 目标用户有具体画像和场景区分
- [ ] 所有 P0 功能有 ≥ 2 条验收标准
- [ ] 性能指标有具体数值和测量方法
- [ ] 技术约束包含 MSRV、Tauri 版本、最低系统要求
- [ ] 跨平台差异已标注每平台优先级
- [ ] 离线策略已标注每个功能的离线级别
- [ ] 竞品分析 ≥ 3 个维度有实际数据
- [ ] KPI 有测量方式和时间线
- [ ] 无未定义的缩写或内部术语

## 7. Usage Guide

**Minimal input required:** App type + target platforms + 1-2 core features.
**Step 1:** Start with templates/prd-template.md.
**Step 2:** Fill background → user profile → feature table (with AC).
**Step 3:** Add performance targets + technical constraints.
**Step 4:** Run Section 6 audit checklist.

**Minimum viable PRD:** Background (2+ data points) + Target users (1+ persona) + Feature table (all P0 with AC) + Tech constraints (MSRV + Tauri version). If any missing, ask before generating.

When requirements are vague, ask:
1. What problem does this app solve?
2. Who are the primary users?
3. Which platforms must V1.0 support?

## 8. Scope Guard

When the request is outside scope:
- **Non-Tauri desktop** (Electron/Flutter/Qt): "This Skill targets Tauri v2. For [Electron|Flutter], adapt the PRD structure but replace Tauri-specific sections (IPC, Capabilities, tauri-plugin-*) with your framework's equivalents."
- **Web SaaS**: "This Skill targets desktop/mobile apps. For web products, use the PRD structure but remove offline/local storage constraints and add deployment/infrastructure sections."
- **CLI/backend**: "This Skill targets user-facing applications. For CLI or backend services, focus on functional requirements and API contracts rather than UX metrics."
