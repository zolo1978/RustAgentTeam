# Rust Agent Team v2.0 (Tauri v2)

10 个专职 Agent + 18 个配套 Skills + 2 个桥接 Agent，覆盖 Rust/Tauri 桌面应用开发全生命周期。

## 快速安装

```bash
# 1. 复制 Agents
cp agents/*.md ~/.claude/agents/

# 2. 复制 Skills
cp -r skills/* ~/.claude/skills/

# 3. 验证
ls ~/.claude/agents/rust-*.md   # 应显示 10 个文件
ls ~/.claude/skills/rust-*      # 应显示 17 个目录
```

## Agent 清单

| # | Agent | 角色 | 默认模型 | 核心职责 |
|---|-------|------|---------|---------|
| 1 | rust-team-lead | 团队中枢 | Claude | 路由调度、质量门、Token 预算 |
| 2 | rust-pm-agent | 产品经理 | Claude | PRD、项目计划、验收决策、风险量化 |
| 3 | rust-architect-agent | 架构师 | Opus | 技术方案、分层架构、IPC 契约、脚手架 |
| 4 | rust-backend-agent | 后端工程师 | Sonnet | Rust Commands、Service、Repository、TDD |
| 5 | rust-frontend-agent | 前端工程师 | Sonnet | WebView UI、IPC 封装、暗色模式、TDD |
| 6 | rust-ui-designer-agent | UI 设计师 | Gemini | 设计规格（组件/Token/状态/交互） |
| 7 | rust-integration-agent | 集成专家 | Sonnet | 剪贴板、热键、托盘、签名分发 |
| 8 | rust-qa-agent | 质量守门员 | Claude | PRD 验收、IPC 契约验证、反模式检测 |
| 9 | rust-reviewer | 代码审查 | Sonnet | 所有权、安全、性能、最佳实践 |
| 10 | rust-build-resolver | 构建修复 | Sonnet | cargo build 错误、borrow checker 修复 |

## 标准协作流

```
用户需求 → PM(PRD) → 架构师(方案) → UI设计师(规格) → 前端+后端(并行) → QA(验收) → 完成
```

## AI Coding 6 原则（全团队强制）

1. **Think Before Acting** — 先想再做，输出假设列表
2. **Simplicity First** — 极简主义，不过度设计
3. **Surgical Changes** — 精准手术，每个 diff 追溯到 AC
4. **Goal-Driven Execution** — 目标驱动，可验证目标
5. **Risk-Level Routing** — 风险分级路由 (LOW/MEDIUM/HIGH)
6. **Always Review Diff** — 始终审查 Diff

## 3 个安全模式（来自 agent-coding-playbook）

- **Guardrails（护栏）** — 高风险场景自动暂停，请求人工确认
- **标准完成报告** — Changed / Verified / Not verified / Risks 四段式
- **不适用场景** — 明确每个 Agent 的边界，防止误调用

## Skill 清单

| Skill | 用途 |
|-------|------|
| rust-arch | 分层架构、Capabilities ACL、IPC 模式、类型映射 |
| rust-backend | 认证、错误处理、数据库、DevOps、发布 |
| rust-frontend | IPC 封装、表单、暗色模式、无障碍、测试 |
| rust-core | Rust 核心：错误处理、测试、安全、所有权、并发 |
| rust-ui-skill | shadcn/ui 组件、Design Token、状态管理、交互 |
| rust-qa-skill | 验收清单、IPC 契约验证、反模式检测、Smoke Test |
| rust-integration-skill | 剪贴板、热键、托盘、窗口、签名分发 |
| rust-security-skill | Capabilities、FFI 安全、CSP、敏感内容 |
| rust-performance-skill | 冷启动、内存、搜索、IPC 延迟、基准测试 |
| rust-prd-skill | PRD 模板、BAD/GOOD 示例 |
| pmbok-master | WBS、三点估算、风险登记 |
| rust-async-patterns | Tokio 异步模式参考 |
| rust-crash-debug | 崩溃调试参考 |
| rust-fix-planner | 修复计划模板 |
| rust-release-checklist | 发布检查清单 |
| rust-tauri-testing | Tauri 测试策略 |
| rust-team-plan | 团队计划模板 |
| rust-verify-checker | 验证检查器 |

## 桥接 Agent

| Agent | 用途 |
|-------|------|
| codex-bridge-agent | Codex (GPT-5) 代码执行入口 |
| gemini-bridge-agent | Gemini 2.5 Pro 分析入口 |

## 版本信息

- **版本**: v2.0 (2026-05-05)
- **适配**: Tauri v2 + React/Vue/Svelte + SQLite
- **平台**: macOS (V1), Windows/Linux (P2)
