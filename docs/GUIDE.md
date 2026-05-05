# Rust Agent Team v2.0 使用说明书

> 10 个专职 Agent + 18 个配套 Skills + 2 个桥接 Agent
> 覆盖 Rust/Tauri v2 桌面应用开发全生命周期

---

## 目录

- [1. 产品概述](#1-产品概述)
- [2. 系统架构](#2-系统架构)
- [3. 环境要求与安装](#3-环境要求与安装)
- [4. 快速上手（5 分钟）](#4-快速上手5-分钟)
- [5. Agent 详细参考](#5-agent-详细参考)
- [6. Skill 详细参考](#6-skill-详细参考)
- [7. 标准工作流](#7-标准工作流)
- [8. 安全机制](#8-安全机制)
- [9. 配置与定制](#9-配置与定制)
- [10. 常见问题](#10-常见问题)
- [11. 术语表](#11-术语表)

---

## 1. 产品概述

### 1.1 是什么

Rust Agent Team 是一套基于 Claude Code 的多 Agent 协作系统，专为 Rust/Tauri v2 桌面应用开发设计。它将软件开发全生命周期拆解为 10 个专职角色，通过 Hub-and-Spoke 架构协调工作。

### 1.2 解决什么问题

| 痛点 | 解决方案 |
|------|---------|
| 单一 AI 上下文爆炸，长任务丢失细节 | 每个 Agent 只聚焦一个领域，上下文精简 |
| AI 生成的代码缺少质量门控 | 6 道质量门（PRD→架构→UI→开发→审查→验收） |
| 安全风险（SQL 注入、硬编码密钥）| Guardrails 护栏自动暂停高风险操作 |
| 代码变更无法追溯到需求 | Surgical Changes 要求每个 diff 追溯到 AC |
| 多模型切换混乱 | 统一路由矩阵，自动分配最优模型 |

### 1.3 核心设计原则

**AI Coding 6 原则**（全团队强制遵守）：

```
┌─────────────────────────────────────────────────────┐
│  1. Think Before Acting  先想再做，输出假设列表       │
│  2. Simplicity First     极简主义，不过度设计         │
│  3. Surgical Changes     精准手术，diff 追溯到 AC     │
│  4. Goal-Driven          目标驱动，可验证目标         │
│  5. Risk-Level Routing   风险分级 (LOW/MED/HIGH)     │
│  6. Always Review Diff   始终审查变更                │
└─────────────────────────────────────────────────────┘
```

**3 个安全模式**（来自 agent-coding-playbook）：

- **Guardrails（护栏）** — 高风险场景自动暂停，请求人工确认
- **标准完成报告** — Changed / Verified / Not verified / Risks 四段式
- **不适用场景** — 明确每个 Agent 的边界，防止误调用

---

## 2. 系统架构

### 2.1 Hub-and-Spoke 拓扑

```
                        ┌──────────────┐
                        │  rust-team-  │
                        │    lead      │  ← 调度中枢（Hub）
                        │  (Claude)    │
                        └──────┬───────┘
                               │
          ┌────────────┬───────┼───────┬────────────┐
          ▼            ▼       ▼       ▼            ▼
    ┌──────────┐ ┌──────────┐ ┌─────┐ ┌──────────┐ ┌──────────┐
    │PM Agent  │ │Architect │ │ QA  │ │Reviewer  │ │UI Design │
    │(Claude)  │ │(Opus)    │ │(Cla)│ │(Sonnet)  │ │(Gemini)  │
    └──────────┘ └──────────┘ └─────┘ └──────────┘ └──────────┘
          │            │                       │
          │    ┌───────┴────────┐              │
          │    ▼                ▼              │
          │  ┌──────────┐ ┌──────────┐  ┌──────────┐
          │  │ Backend  │ │ Frontend │  │Integrate │
          │  │(Codex)   │ │(Codex)   │  │(Codex)   │
          │  └──────────┘ └──────────┘  └──────────┘
          │                                   │
          │  ┌──────────┐                     │
          └─►│  Build   │◄────────────────────┘
             │Resolver  │
             │(Sonnet)  │
             └──────────┘
```

**关键规则：**
- Team Lead 每次只激活 1 个 Agent，节省 Token
- 所有交接必须通过质量门
- 不确定的需求先追问再路由

### 2.2 多模型路由矩阵

| 模型 | 负责的 Agent | 选择理由 |
|------|-------------|---------|
| **Claude** (Sonnet/Opus) | Team Lead, Architect, QA, PM | 深度推理、架构决策、质量守门 |
| **Codex** (GPT-5) | Frontend, Backend, Integration, Reviewer | 代码生成快、重构强、成本低 |
| **Gemini** (2.5 Pro) | UI Designer, PM(文档分析) | 长上下文、多模态、设计理解 |

模型通过桥接 Agent 调用：
- `codex-bridge-agent` — Codex 代码执行入口
- `gemini-bridge-agent` — Gemini 分析入口

### 2.3 数据流

```
用户需求
   │
   ▼
[Team Lead] ─── 路由决策 ──→ [PM Agent] ─── PRD
   │                              │
   │                              ▼ 质量门: PRD Checklist 12/12
   │                         [Architect] ─── 技术方案 + IPC 契约
   │                              │
   │                              ▼ 质量门: 数据模型 + 契约表
   │                         [UI Designer] ─── 设计规格
   │                              │
   │                    ┌─────────┴─────────┐
   │                    ▼                   ▼
   │              [Backend]           [Frontend]
   │              (Codex)             (Codex)
   │                    └─────────┬─────────┘
   │                              │
   │                              ▼ 质量门: test + lint + Diff 自审
   │                         [QA Agent] ─── 验收报告
   │                              │
   │                              ▼ FAIL → 退回修复 | PASS → 完成
   │                           完成
   │
   └── 贯穿全程：[Reviewer] 代码审查 + [Build Resolver] 构建修复
```

---

## 3. 环境要求与安装

### 3.1 前置条件

| 依赖 | 最低版本 | 验证命令 |
|------|---------|---------|
| Claude Code CLI | 最新版 | `claude --version` |
| Git | 2.x | `git --version` |
| Rust toolchain | 1.80+ | `rustc --version` |
| Node.js | 18+ | `node --version` |
| pnpm / npm | 最新版 | `pnpm --version` |

### 3.2 安装步骤

#### 方式一：从 GitHub 克隆（推荐）

```bash
# 克隆仓库
git clone https://github.com/zolo1978/RustAgentTeam.git

# 安装 Agents
cp RustAgentTeam/agents/*.md ~/.claude/agents/

# 安装 Skills
cp -r RustAgentTeam/skills/* ~/.claude/skills/

# 验证安装
echo "Agents:" && ls ~/.claude/agents/rust-*.md | wc -l
echo "Skills:" && ls -d ~/.claude/skills/rust-* | wc -l
```

#### 方式二：从压缩包安装

```bash
# 解压
unzip RustAgentTeam-v2.0.zip -d /tmp/RustAgentTeam

# 安装
cp /tmp/RustAgentTeam/agents/*.md ~/.claude/agents/
cp -r /tmp/RustAgentTeam/skills/* ~/.claude/skills/
```

### 3.3 验证安装

安装完成后，在 Claude Code 中输入：

```
> 帮我看看 Rust Agent Team 的团队结构
```

如果 Team Lead 正确响应并展示路由表，说明安装成功。

### 3.4 目录结构

安装后的目录布局：

```
~/.claude/
├── agents/
│   ├── rust-team-lead.md          # 团队中枢
│   ├── rust-architect-agent.md    # 架构师
│   ├── rust-backend-agent.md      # 后端工程师
│   ├── rust-frontend-agent.md     # 前端工程师
│   ├── rust-ui-designer-agent.md  # UI 设计师
│   ├── rust-pm-agent.md           # 产品经理
│   ├── rust-qa-agent.md           # 质量守门员
│   ├── rust-reviewer.md           # 代码审查
│   ├── rust-integration-agent.md  # 集成专家
│   ├── rust-build-resolver.md     # 构建修复
│   ├── codex-bridge-agent.md      # Codex 桥接
│   └── gemini-bridge-agent.md     # Gemini 桥接
│
└── skills/
    ├── rust-arch/                 # 架构 Skill（含 references/ + templates/）
    ├── rust-backend/              # 后端 Skill
    ├── rust-frontend/             # 前端 Skill
    ├── rust-core/                 # Rust 核心模式
    ├── rust-ui-skill/             # UI 设计 Skill
    ├── rust-qa-skill/             # QA Skill
    ├── rust-integration-skill/    # 集成 Skill
    ├── rust-security-skill/       # 安全 Skill
    ├── rust-performance-skill/    # 性能 Skill
    ├── rust-prd-skill/            # PRD Skill
    ├── pmbok-master/              # 项目管理方法论
    ├── rust-async-patterns/       # 异步模式
    ├── rust-crash-debug/          # 崩溃调试
    ├── rust-fix-planner/          # 修复计划
    ├── rust-release-checklist/    # 发布检查
    ├── rust-tauri-testing/        # Tauri 测试
    ├── rust-team-plan/            # 团队计划
    └── rust-verify-checker/       # 验证检查器
```

---

## 4. 快速上手（5 分钟）

### 4.1 场景一：全栈功能开发

在 Claude Code 中输入：

```
帮我做一个文件批量管理功能，从需求到实现
```

Team Lead 将自动调度完整流水线：

```
PM(PRD) → 架构师(方案) → UI设计师(规格) → 前端+后端(并行) → QA(验收) → 完成
```

每步完成后会暂停征求你的确认。

### 4.2 场景二：单个角色任务

```
帮我写一个用户管理的 PRD
```

Team Lead 路由到 PM Agent，输出 PRD 文档。

```
帮我设计这个项目的数据库 schema
```

Team Lead 路由到 Architect Agent，输出技术方案。

### 4.3 场景三：紧急修复

```
线上崩溃了，panic in user handler
```

Team Lead 跳过 PM，直接路由到 Backend Agent 紧急修复。

### 4.4 场景四：构建报错

```
cargo build 报错 E0502 borrow checker
```

Build Resolver 自动接管，精准修复编译错误。

---

## 5. Agent 详细参考

### 5.1 rust-team-lead（团队中枢）

| 属性 | 值 |
|------|-----|
| 角色 | 调度中枢（Hub） |
| 默认模型 | Claude Sonnet |
| 工具 | Read, Write, Glob, Grep, Bash |
| 产出 | 路由决策、上下文摘要、质量门检查 |

**核心职责：**
- 解析用户意图，路由到正确的 Agent
- 管理质量门（PRD→架构→UI→开发→验收）
- 控制 Token 预算
- 错误状态处理（Agent 失败、输出被拒、上下文溢出）

**触发信号：**
- 精确：任何 Rust/Tauri 开发请求
- 模糊："帮我做XX"、"看看项目"

**Guardrails 高风险项：**
- 跨 Phase 架构变更（影响 > 2 个 Agent）
- 安全模型变更（Capabilities / Isolation / CSP）
- 数据迁移方案
- Token 预算超过 80%

**不适用：** 非 Rust/Tauri 项目、单文件小修改、纯样式调整

---

### 5.2 rust-pm-agent（产品经理）

| 属性 | 值 |
|------|-----|
| 角色 | 产品经理 |
| 默认模型 | Claude Sonnet |
| 工具 | Read, Write, Glob, Grep, Bash |
| 产出 | PRD、项目计划、验收报告、发布清单 |

**核心职责：**
- 需求分析（12 项 Intake 问卷）
- PRD 编写（使用 `rust-prd-skill` 模板）
- 项目计划（WBS 分解、三点估算、风险登记）
- 验收决策（对照 PRD 逐条检查）
- 发布管理

**关键约束：**
- PRD 无量化验收标准 = 不通过
- 项目计划无风险登记 = 不通过
- 不提技术方案，只提"要什么"

**触发信号：** "PRD"、"排期"、"验收"、"上线"、"需求分析"

---

### 5.3 rust-architect-agent（架构师）

| 属性 | 值 |
|------|-----|
| 角色 | 技术架构师 |
| 默认模型 | Claude Opus |
| 工具 | Read, Write, Glob, Grep, Bash |
| 产出 | 技术方案、IPC 契约表、依赖清单、脚手架代码 |

**核心职责：**
- PRD 分析与技术方案设计
- IPC 契约表输出（Rust↔TypeScript 类型映射）
- 依赖清单输出
- 脚手架代码搭建（可 `cargo check` 通过）
- 架构决策记录

**关键约束：**
- 每个决策标注"这是当前 PRD 的最小实现"
- 输出可编译的代码骨架，不只是文档
- 不过度抽象——只有第 3 处相似逻辑出现时才提取公共模块

**触发信号：** "架构"、"技术方案"、"选型"、"数据模型"

---

### 5.4 rust-backend-agent（后端工程师）

| 属性 | 值 |
|------|-----|
| 角色 | Rust 核心层开发者 |
| 默认模型 | Claude Sonnet（通过 Codex 执行） |
| 工具 | Read, Write, Glob, Grep, Bash |
| 产出 | 可编译 Rust 代码 + 测试（覆盖率 ≥ 80%） |

**核心职责：**
- Tauri Command 开发（薄层：参数校验 + 调用 Service）
- Service 层业务逻辑（可独立测试）
- Repository 层数据访问（spawn_blocking 包装）
- 数据库操作（rusqlite + 参数化查询）
- 错误处理（类型化 AppError）

**工作流：** 严格 TDD 红绿循环
```
写测试(RED) → cargo test(红) → 写实现(GREEN) → cargo test(绿) → clippy → fmt
```

**Guardrails 高风险项：**
- 认证/授权逻辑修改
- 加密/安全相关代码
- 数据库 migration（ALTER/DROP）
- unsafe 代码引入
- 公共 Command 接口变更

---

### 5.5 rust-frontend-agent（前端工程师）

| 属性 | 值 |
|------|-----|
| 角色 | WebView 前端层开发者 |
| 默认模型 | Claude Sonnet（通过 Codex 执行） |
| 工具 | Read, Write, Glob, Grep, Bash |
| 产出 | 页面/组件代码 + IPC 封装 + 前端测试 |

**核心职责：**
- React/Vue/Svelte 组件开发
- 类型安全 IPC 封装（safeInvoke + 自动重试 + 超时）
- 暗色模式实现（CSS 变量方案）
- 表单处理（验证 + 提交 + 错误展示）
- 错误边界和加载状态

**自审检查点：**
- 无 `any` 类型
- 无组件内直接 `invoke` 调用
- 所有异步操作有 loading/error 状态
- 暗色模式 CSS 变量已覆盖

**Guardrails 高风险项：**
- 认证/登录流程 UI
- 敏感数据展示
- 支付相关 UI
- CSP/security 配置

---

### 5.6 rust-ui-designer-agent（UI 设计师）

| 属性 | 值 |
|------|-----|
| 角色 | UI 设计师 |
| 默认模型 | Gemini 2.5 Pro |
| 工具 | Read, Glob, Grep, Bash |
| 产出 | 设计规格 Markdown（组件/Token/状态/交互） |

**核心职责：**
- 组件选型（优先 shadcn/ui）
- Design Token（颜色/间距/字体/亮暗主题）
- 状态设计（Store 拆分 + IPC 数据流）
- 交互规范（键盘/鼠标/快捷键）

**关键约束：**
- **只输出 Markdown 设计规格，不写任何代码**
- 必须包含组件/Token/状态/交互四节
- 图标使用 lucide-react，不用 emoji
- 间距必须是 4px 的倍数

---

### 5.7 rust-qa-agent（质量守门员）

| 属性 | 值 |
|------|-----|
| 角色 | QA 验收 |
| 默认模型 | Claude Sonnet |
| 工具 | Read, Glob, Grep, Bash |
| 产出 | 验收报告（PASS/FAIL/CONDITIONAL PASS） |

**核心职责：**
- PRD 验收清单（每条 AC → 可执行验证命令）
- IPC 契约验证（Rust serde vs TS interface 字段对齐）
- 反模式检测（6 条 grep 规则）
- Smoke Test
- Diff 可追溯性审计

**关键约束：**
- **不写代码，只验收**
- P0 问题标记 FAIL，阻塞后续
- 每条验证结果标注 PASS/FAIL，不写"基本通过"

**反模式检测规则：**

| 规则 | 级别 |
|------|------|
| `unwrap()` 在 command 函数中 | P0 |
| `todo!()` / `unimplemented!()` | P0 |
| `.clone()` 在循环中 | P1 |
| `unsafe` 代码块 | P1 |
| `String` 作为错误类型 | P2 |
| 硬编码路径/URL | P1 |

---

### 5.8 rust-reviewer（代码审查）

| 属性 | 值 |
|------|-----|
| 角色 | Rust 代码审查 |
| 默认模型 | Claude Sonnet |
| 工具 | Read, Grep, Glob, Bash |
| 产出 | 审查报告（Approve/Warning/Block） |

**审查优先级：**

| 等级 | 覆盖范围 |
|------|---------|
| **CRITICAL** | 安全（SQL 注入、命令注入、硬编码密钥、路径遍历）、错误处理 |
| **HIGH** | 所有权/生命周期、并发（阻塞 async、无界 channel、死锁）、代码质量 |
| **MEDIUM** | 性能（不必要分配、循环中 clone）、最佳实践 |

**审查维度（新增 AI Coding）：**
- Simplicity Check — 是否过度设计？
- Surgical Check — 是否有"顺手重构"？
- Diff Traceability — 每个文件是否追溯到具体需求？

---

### 5.9 rust-integration-agent（系统集成专家）

| 属性 | 值 |
|------|-----|
| 角色 | 系统集成 |
| 默认模型 | Claude Sonnet（通过 Codex 执行） |
| 工具 | Read, Write, Glob, Grep, Bash |
| 产出 | 集成代码 + 安全封装 + 集成检查清单 |

**5 个集成域：**

| 集成域 | 核心 crate | 参考 Skill |
|--------|-----------|-----------|
| 系统剪贴板 | arboard | rust-integration-skill |
| 全局热键 | tauri-plugin-global-shortcut | rust-integration-skill |
| 系统托盘 | 内置 TrayIconBuilder | rust-integration-skill |
| 窗口管理 | 内置 Window API | rust-integration-skill |
| 签名分发 | codesign/notarytool | rust-integration-skill |

**关键约束：**
- **所有系统集成默认 HIGH 风险**
- 所有 FFI 必须三层封装：catch_unwind + spawn_blocking + timeout
- V1 只覆盖 macOS + Tauri 官方插件

---

### 5.10 rust-build-resolver（构建修复）

| 属性 | 值 |
|------|-----|
| 角色 | 编译错误修复 |
| 默认模型 | Claude Sonnet |
| 工具 | Read, Write, Edit, Bash, Grep, Glob |
| 产出 | 编译修复报告 |

**覆盖范围：**
- `cargo build` / `cargo check` 错误
- Borrow checker 和 lifetime 错误
- Cargo.toml 依赖冲突
- `cargo clippy` 警告

**修复原则：**
- 只修报错的部分，不"顺手重构"
- 优先级：改一行 > 改一个函数 > 加新抽象 > 重构模块
- 同一错误 3 次修不好 → 停止，报告需要架构层面调整

---

## 6. Skill 详细参考

每个 Skill 位于 `~/.claude/skills/<name>/` 目录，由 `SKILL.md` 主文件 + `references/` 参考文档 + `templates/` 模板组成。

### 核心开发 Skills

| Skill | 文件数 | 核心内容 |
|-------|--------|---------|
| **rust-arch** | 12 | 分层架构、Capabilities ACL、IPC 模式、Rust↔TS 类型映射表、脚手架模板（error.rs/state.rs/lib.rs/commands.rs/ipc-contract.md） |
| **rust-backend** | 17 | 认证(JWT)、错误码枚举、数据库选型、迁移脚本、CI/CD、构建优化、自动更新、代码模板（tauri-command.rs/jwt-auth.rs/paged-list.rs） |
| **rust-frontend** | 14 | IPC 错误处理、事件处理、文件对话框、暗色模式、表单模式、组件架构、无障碍、测试 Mock、代码模板（safe-invoke.ts/use-form.ts/theme.ts） |
| **rust-core** | 8 | Rust 核心模式：错误处理、测试、安全（威胁模型/模糊测试/安全扫描/unsafe 审计）、模板（finding.md/incident-response.md） |

### 设计与 QA Skills

| Skill | 文件数 | 核心内容 |
|-------|--------|---------|
| **rust-ui-skill** | 6 | shadcn/ui 组件清单、Design Token（颜色/间距/字体）、状态管理方案、桌面端交互规范、设计规格模板 |
| **rust-qa-skill** | 5 | 验收清单生成、IPC 契约验证、反模式检测（6 条 grep 规则）、Smoke Test 检查清单 |
| **rust-prd-skill** | 5 | PRD 模板、BAD/GOOD 对比、竞品分析框架、复杂场景处理 |

### 专业领域 Skills

| Skill | 文件数 | 核心内容 |
|-------|--------|---------|
| **rust-security-skill** | 7 | Capabilities 加固、CSP 策略、FFI 安全、FTS 注入防护、敏感内容过滤、安全审计清单 |
| **rust-performance-skill** | 6 | 冷启动优化、内存控制、搜索性能、IPC 延迟、Rust 基准测试模板 |
| **rust-integration-skill** | 7 | 剪贴板/热键/托盘/窗口/签名分发参考实现、集成检查清单 |

### 辅助 Skills

| Skill | 核心内容 |
|-------|---------|
| **pmbok-master** | 项目管理方法论：WBS 分解、三点估算、风险登记、知识领域概览 |
| **rust-async-patterns** | Tokio 异步模式参考 |
| **rust-crash-debug** | Rust 崩溃调试流程 |
| **rust-fix-planner** | 修复计划模板 |
| **rust-release-checklist** | Tauri 应用发布检查清单 |
| **rust-tauri-testing** | Tauri 应用测试策略 |
| **rust-team-plan** | 团队项目计划模板 |
| **rust-verify-checker** | 变更验证检查器 |

---

## 7. 标准工作流

### 7.1 全栈功能开发（标准流水线）

这是最完整的流程，适用于"从零做一个新功能"。

```
Step 1: 需求理解 ──────────────────── PM Agent
  │  输入：用户原始需求
  │  产出：PRD（含 AC + 性能指标 + 风险登记）
  │  质量门：PRD Checklist 12/12 通过
  ▼
Step 2: 技术设计 ──────────────────── Architect Agent
  │  输入：PRD
  │  产出：技术方案 + IPC 契约表 + 依赖清单 + 脚手架
  │  质量门：cargo check 通过 + 契约表完整
  ▼
Step 3: UI 设计 ──────────────────── UI Designer Agent (Gemini)
  │  输入：技术方案 + PRD
  │  产出：设计规格（组件/Token/状态/交互）
  │  质量门：四节齐全 + Token 使用现有体系
  ▼
Step 4: 后端实现 ──────────────────── Backend Agent (Codex)
  │  输入：技术方案 + IPC 契约 + 数据模型
  │  产出：Rust 代码 + 测试
  │  工作流：TDD 红绿循环
  │  自审：Diff 可追溯性 + 无 unwrap + 无硬编码
  ▼
Step 5: 前端实现（可与 Step 4 并行）── Frontend Agent (Codex)
  │  输入：设计规格 + IPC 契约
  │  产出：组件/页面代码 + IPC 封装 + 测试
  │  自审：无 any + 无组件内 invoke + 暗色模式
  ▼
Step 6: 验收 ──────────────────────── QA Agent
  │  输入：PRD + 实现代码
  │  检查：PRD AC + IPC 契约 + 反模式 + Smoke Test
  │  产出：验收报告（PASS/FAIL）
  │  FAIL → 退回对应 Agent 修复
  ▼
Step 7: 完成 ──────────────────────── Team Lead 汇总
     输出：标准完成报告（Changed/Verified/Not verified/Risks）
```

### 7.2 紧急修复流程

```
用户："线上崩溃 / panic / 500"
  │
  ▼
Team Lead → 直接路由 Backend Agent（跳过 PM）
  │
  │  附加指令："紧急修复模式，优先恢复，事后补验收"
  │
  ▼
Backend Agent → 定位 → 最小修复 → cargo test
  │
  ▼
QA Agent → 快速验收（仅验证修复点）
  │
  ▼
完成 → 事后补 PRD + 全量验收
```

### 7.3 构建报错流程

```
用户："cargo build 报错 E0502"
  │
  ▼
Build Resolver 自动接管
  │
  │  1. cargo check → 解析错误
  │  2. Read 受影响文件 → 理解上下文
  │  3. 最小修复 → 只修报错部分
  │  4. cargo check → 验证修复
  │  5. cargo test → 确认无回归
  │
  ▼
输出：构建修复报告（Changed/Verified/Not verified/Risks）
```

### 7.4 代码审查流程

```
用户："review 这个 PR" / 开发完成后自动触发
  │
  ▼
Reviewer Agent
  │
  │  1. cargo check + clippy + fmt + test（任一失败则停止）
  │  2. git diff -- '*.rs' 查看变更
  │  3. 按优先级审查：CRITICAL → HIGH → MEDIUM
  │  4. AI Coding 审查：Simplicity + Surgical + Diff Traceability
  │
  ▼
输出：审查报告（Approve/Warning/Block + 具体意见）
```

### 7.5 会话分解策略

大任务按"一个会话一个焦点"原则拆解：

| 会话 | 焦点 | 产出 | 上下文管理 |
|------|------|------|-----------|
| 1 | 需求理解 | PRD | 完整上下文 |
| 2 | 技术设计 | 技术方案 + 契约 | PRD 摘要 + 关键决策 |
| 3 | 后端实现 | Rust 代码 + 测试 | 契约表 + 数据模型 |
| 4 | 前端实现 | UI 代码 + 测试 | 设计规格 + API 层 |
| 5 | 集成验证 | 验收报告 | 测试结果 + Diff |

**会话交接原则：**
- 每个会话通过**文件产物**交接（docs/ 下的文档、git commit）
- 不传原始对话上下文，只传结构化摘要
- 上游产出物路径在任务头明确标注

---

## 8. 安全机制

### 8.1 Guardrails（护栏）系统

每个 Agent 内置了高风险场景的自动暂停机制。当触发护栏时，Agent 必须**停止执行并请求人工确认**。

**各 Agent 的核心护栏：**

| Agent | 高风险场景 |
|-------|-----------|
| Team Lead | 跨 Phase 架构变更、安全模型变更、数据迁移、Token 超限 |
| Architect | 安全模式变更、schema 迁移、公共 API 变更、unsafe 引入 |
| Backend | 认证/授权逻辑、加密代码、DB migration、unsafe、Command 变更 |
| Frontend | 认证 UI、敏感数据展示、支付 UI、CSP 配置 |
| Reviewer | SQL 注入、命令注入、硬编码密钥、unsafe 无注释 |
| PM | 敏感业务（支付/医疗）、合规变更、数据迁移需求 |
| QA | 数据迁移安全、P0 安全问题、IPC 严重不对齐 |
| Integration | 系统权限、签名公证、隐私 API、跨平台数据丢失 |
| UI Designer | 可访问性合规、暗色对比度、支付交互、隐私展示 |
| Build Resolver | 依赖变更、unsafe、Capabilities、schema、连锁修改 |

### 8.2 标准完成报告

每个 Agent 完成任务后，必须输出四段式报告：

```markdown
## 完成报告

### Changed（变更）
具体做了什么：文件列表 + 变更摘要

### Verified（已验证）
哪些已通过验证：测试结果 + 检查结果

### Not verified（未验证）
哪些尚未验证：待联调/待 QA/待 Smoke Test 的项

### Risks（风险）
风险等级 + 风险描述 + 缓解方案
```

### 8.3 风险分级路由

所有任务在路由时自动标注风险等级：

| 等级 | 触发条件 | 审查要求 |
|------|---------|---------|
| **LOW** | UI 微调、文案、样式调整 | 自动检查（test + lint）即可交付 |
| **MEDIUM** | 新功能、API 新增、数据模型变更 | 自动检查 + QA Agent 审查 |
| **HIGH** | 安全相关、加密、权限、数据迁移、unsafe | 自动检查 + QA + 用户 Diff 审查 |

### 8.4 Diff 审查要求

- **每个 Agent** 完成任务后必须自审 `git diff`
- **QA Agent** 验收时必须检查 Diff 可追溯性
- **HIGH 风险任务**的 Diff 必须由用户审查后才能合并
- 无法追溯到具体 AC 的变更标记为 `UNNECESSARY_CHANGE`

---

## 9. 配置与定制

### 9.1 模型配置

每个 Agent 文件的 YAML frontmatter 中可修改默认模型：

```yaml
---
name: rust-backend-agent
model: sonnet  # 可选：sonnet, opus, haiku
---
```

模型选择建议：

| 场景 | 推荐模型 | 理由 |
|------|---------|------|
| 日常开发 | sonnet | 性价比最优，90% 场景够用 |
| 架构决策 | opus | 最深推理能力 |
| 轻量任务 | haiku | 3x 成本节省，适合频繁调用 |

### 9.2 添加自定义 Agent

1. 在 `~/.claude/agents/` 创建新文件 `my-agent.md`
2. 使用标准 YAML frontmatter：
```yaml
---
name: my-agent
description: '描述'
tools: ["Read", "Write", "Glob", "Grep", "Bash"]
model: sonnet
---
```
3. 在 `rust-team-lead.md` 的路由表中添加新条目

### 9.3 添加自定义 Skill

1. 在 `~/.claude/skills/` 创建新目录 `my-skill/`
2. 添加 `SKILL.md` 主文件（标准格式）
3. 可选添加 `references/` 和 `templates/` 子目录
4. 在对应 Agent 文件中引用

### 9.4 修改质量门

在 `rust-team-lead.md` 的"交接质量门"节修改门控条件。每道门列出了必须满足的条件，不满足则退回上游。

---

## 10. 常见问题

### Q1: 安装后 Claude Code 没有识别到 Agent

**A:** 确认文件路径正确：
```bash
ls ~/.claude/agents/rust-team-lead.md
# 应显示文件存在
```
Claude Code 会自动扫描 `~/.claude/agents/` 目录。

### Q2: Team Lead 没有正确路由

**A:** Team Lead 依赖关键词匹配。尽量使用明确的触发信号：
- 用 "PRD" 而不是 "文档"
- 用 "架构" 而不是 "设计"
- 用 "验收" 而不是 "检查一下"

### Q3: 某个 Agent 产出质量差

**A:** 检查以下几点：
1. 上游产出物是否完整（PRD 是否通过了 12 项 Checklist）
2. 上下文是否足够（参考"上下文工程指南"）
3. 该 Agent 的 Skill 是否正确安装
4. 模型是否合适（架构决策用 Opus，日常开发用 Sonnet）

### Q4: 如何跳过某个 Phase

**A:** Team Lead 支持以下快捷路径：
- 紧急修复：直接 Backend Agent（跳过 PM）
- 已有 PRD：直接 Architect Agent（跳过 PM）
- 只写代码：直接对应工程师 Agent（跳过上游）

### Q5: Guardrails 太严格，如何放宽

**A:** 编辑对应 Agent 文件的 `## Guardrails` 节，移除或注释不需要的护栏项。**不建议完全移除**，至少保留安全相关的护栏。

### Q6: Codex / Gemini 桥接 Agent 报错

**A:** 桥接 Agent 需要：
1. `codex-bridge-agent.md` 需要配置 Codex API 访问权限
2. `gemini-bridge-agent.md` 需要配置 Gemini API 访问权限
3. 检查对应桥接 Agent 文件中的配置说明

### Q7: 如何在已有项目中使用

**A:** Rust Agent Team 不要求项目从零开始。在已有项目中：
1. 安装 Agent + Skills
2. 让 PM Agent 分析现有代码库
3. Architect Agent 会判断"从零 vs 扩展"
4. 工程师 Agent 会尊重现有代码模式

### Q8: 支持哪些前端框架

**A:** Skills 中包含 React、Vue、Svelte 的参考。Agent 默认使用 React + TypeScript + Tailwind CSS + shadcn/ui，但可适配其他框架。

---

## 11. 术语表

| 术语 | 含义 |
|------|------|
| **Agent** | 专职 AI 角色，有明确的职责边界和工具权限 |
| **Skill** | 知识库文件，提供参考文档和代码模板 |
| **Hub-and-Spoke** | 中心调度架构，Team Lead 是 Hub，其他 Agent 是 Spoke |
| **IPC** | Inter-Process Communication，Tauri 中 Rust↔WebView 的通信机制 |
| **AC** | Acceptance Criteria，验收标准 |
| **PRD** | Product Requirements Document，产品需求文档 |
| **WBS** | Work Breakdown Structure，工作分解结构 |
| **TDD** | Test-Driven Development，测试驱动开发 |
| **Capabilities** | Tauri v2 的权限声明机制（替代 v1 allowlist） |
| **Isolation Pattern** | Tauri v2 的 IPC 加密模式（AES-GCM） |
| **Guardrails** | 高风险场景自动暂停机制 |
| **Diff Traceability** | 代码变更可追溯到具体需求 |
| **SafeInvoke** | 前端类型安全的 IPC 调用封装（含重试 + 超时） |

---

## 附录 A: 版本历史

| 版本 | 日期 | 变更 |
|------|------|------|
| v2.0 | 2026-05-05 | 新增 AI Coding 6 原则 + 3 个安全模式（Guardrails/标准报告/不适用场景）|
| v1.0 | 2026-04 | 初始版本：10 Agent + 18 Skill |

## 附录 B: 文件统计

| 类别 | 数量 | 总行数 |
|------|------|--------|
| Agent 文件 | 12 | ~3,277 行 |
| Skill 文件 | 98 | ~18,934 行 |
| 模板文件 | 含在 Skill 中 | — |
| **合计** | **110** | **~22,211 行** |

## 附录 C: 推荐阅读

- [Tauri v2 官方文档](https://v2.tauri.app/)
- [Rust 官方文档](https://doc.rust-lang.org/)
- [agent-coding-playbook](https://github.com/bravekingzhang/agent-coding-playbook) — Guardrails 模式来源
- [Claude Code 文档](https://docs.anthropic.com/en/docs/claude-code) — Agent 系统 参考

---

*Rust Agent Team v2.0 | 2026-05-05 | zolo1978*
