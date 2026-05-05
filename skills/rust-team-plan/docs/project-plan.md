# Rust 团队 Skill/Agent 生态补齐计划

> 基于 ClipVault 模拟项目复盘结论，补齐缺失角色和知识体系。

## 项目总览

| 维度 | 内容 |
|------|------|
| 目标 | 新增 3 Agent + 5 Skill，增强 2 Agent + 2 Skill |
| 工作包 | 20 个（每个 ≤ 2h） |
| 预估总工时 | 26h |
| 关键路径 | 7.5h |
| 执行批次 | 5 批 |
| 最大并行度 | 7（批次 1） |

---

## 复盘根因

ClipVault 模拟暴露的核心问题不是代码能力，是**角色缺失**：

| 缺失角色 | 导致的问题 |
|----------|-----------|
| 质量守门员 | PRD 写完没人验收，编译通过就算"完成" |
| UI 设计师 | 前端直接写代码没设计，emoji 当按钮 |
| 系统集成专家 | 遇到 OS 层 FFI 就写 TODO 跳过 |
| 安全参考 | 剪贴板敏感内容、Capabilities、FTS 注入全是盲区 |
| 性能参考 | PRD 10 项量化指标没人测量 |

---

## 交付物清单

### 新增 Agent（3 个）

| Agent | 文件路径 | 职责 |
|-------|---------|------|
| rust-qa-agent | `~/.claude/agents/rust-qa-agent.md` | PRD 验收、IPC 契约验证、Smoke Test、反模式检测 |
| rust-ui-designer-agent | `~/.claude/agents/rust-ui-designer-agent.md` | 输出设计规格（不写代码），组件选型、Token、交互规范 |
| rust-integration-agent | `~/.claude/agents/rust-integration-agent.md` | 系统剪贴板、全局热键、系统托盘、平台抽象 |

### 新增 Skill（5 个）

| Skill | 目录 | 核心内容 |
|-------|------|---------|
| rust-qa-skill | `~/.claude/skills/rust-qa-skill/` | 验收清单模板、IPC 契约验证、反模式检测、Smoke Test |
| rust-ui-skill | `~/.claude/skills/rust-ui-skill/` | shadcn/ui 组件库、Design Token、状态设计、桌面端交互 |
| rust-integration-skill | `~/.claude/skills/rust-integration-skill/` | arboard 剪贴板、tauri-plugin-global-shortcut、TrayIcon、签名分发 |
| rust-security-skill | `~/.claude/skills/rust-security-skill/` | 敏感内容过滤、Capabilities 最小化、FTS 注入防护、FFI 安全、CSP |
| rust-performance-skill | `~/.claude/skills/rust-performance-skill/` | 冷启动优化、内存控制、搜索性能、IPC 延迟、基准测试 |

### 增强 Agent（2 个）

| Agent | 文件路径 | 增强内容 |
|-------|---------|---------|
| rust-team-lead | `~/.claude/agents/rust-team-lead.md` | 门控机制（每个 phase 后 QA 验收）、UI 设计阶段路由、困难上报机制 |
| rust-architect-agent | `~/.claude/agents/rust-architect-agent.md` | Step 2.5 接口契约输出、Step 2.6 依赖清单输出 |

### 增强 Skill（2 个）

| Skill | 文件路径 | 增强内容 |
|-------|---------|---------|
| rust-arch | `~/.claude/skills/rust-arch/SKILL.md` | 接口契约模板、IPC 类型映射表、平台抽象模式 |
| rust-frontend | `~/.claude/skills/rust-frontend/SKILL.md` | 文档输出规范 |

---

## WBS 工作分解

### 第一层：基础 Skill 增强（前置依赖）

| 编号 | 名称 | 交付物 | 依赖 | 工时 | 优先级 |
|------|------|--------|------|------|--------|
| WP-01 | rust-arch 增加接口契约模板 | `skills/rust-arch/templates/ipc-contract.md` + SKILL.md 新增 Section | 无 | 1.5h | P0 |
| WP-02 | rust-arch 增加 IPC 映射表 | `skills/rust-arch/references/ipc-mapping-table.md` + SKILL.md 新增 Section | 无 | 1.5h | P0 |
| WP-03 | rust-arch 增加平台抽象模式 | `skills/rust-arch/references/platform-abstraction.md` + SKILL.md 新增 Section | 无 | 1.5h | P0 |
| WP-04 | rust-architect-agent 增强契约输出 | agents/rust-architect-agent.md 增加 Step 2.5 + 2.6 | WP-01, WP-02 | 1.5h | P0 |
| WP-05 | rust-frontend 增加文档输出要求 | skills/rust-frontend/SKILL.md 新增 Section | 无 | 1h | P1 |

### 第二层：新增 Skill

| 编号 | 名称 | 交付物 | 依赖 | 工时 | 优先级 |
|------|------|--------|------|------|--------|
| WP-07 | 新建 rust-qa-skill | `skills/rust-qa-skill/SKILL.md` + references/(3) + templates/(1) | 无 | 2h | P0 |
| WP-08 | 新建 rust-ui-skill | `skills/rust-ui-skill/SKILL.md` + references/(4) + templates/(1) | WP-05 | 2h | P0 |
| WP-09 | 新建 rust-integration-skill | `skills/rust-integration-skill/SKILL.md` + references/(5) + templates/(1) | WP-03 | 2h | P1 |
| WP-10 | 新建 rust-security-skill | `skills/rust-security-skill/SKILL.md` + references/(5) + templates/(1) | 无 | 2h | P0 |
| WP-11 | 新建 rust-performance-skill | `skills/rust-performance-skill/SKILL.md` + references/(4) + templates/(1) | 无 | 1.5h | P2 |

### 第三层：新增 Agent

| 编号 | 名称 | 交付物 | 依赖 | 工时 | 优先级 |
|------|------|--------|------|------|--------|
| WP-12 | 新建 rust-qa-agent | agents/rust-qa-agent.md | WP-07 | 1.5h | P0 |
| WP-13 | 新建 rust-ui-designer-agent | agents/rust-ui-designer-agent.md | WP-08 | 1.5h | P0 |
| WP-14 | 新建 rust-integration-agent | agents/rust-integration-agent.md | WP-09, WP-10 | 1.5h | P1 |
| WP-06 | 增强 rust-team-lead | agents/rust-team-lead.md 增加门控 + UI 路由 | WP-10 | 1.5h | P1 |

### 第四层：集成验证

| 编号 | 名称 | 交付物 | 依赖 | 工时 | 优先级 |
|------|------|--------|------|------|--------|
| WP-15 | 验证 QA Agent | 模拟 PRD → 验收报告 | WP-12, WP-04 | 1h | P0 |
| WP-16 | 验证 UI Designer Agent | 模拟需求 → 设计规格 | WP-13, WP-06 | 1h | P0 |
| WP-17 | 验证 Integration Agent | 模拟热键需求 → 实现 | WP-14 | 1h | P1 |
| WP-18 | 验证 team-lead 路由 | 全栈请求正确路由 | WP-06, WP-15, WP-16, WP-17 | 1h | P0 |
| WP-19 | 验证 architect 契约输出 | 模拟 PRD → 契约+依赖清单 | WP-04 | 0.5h | P0 |
| WP-20 | 全局回归验证 | 所有 Agent/Skill 一致性检查 | WP-18, WP-19 | 0.5h | P0 |

---

## 关键路径

```
WP-03 → WP-09 → WP-14 → WP-17 → WP-18 → WP-20
(1.5h)  (2h)    (1.5h)   (1h)    (1h)    (0.5h) = 7.5h
```

**瓶颈**：WP-01/02/03（arch Skill 增强）是多个下游的共同前置。

---

## 执行批次

### 批次 1：基础层（7 并行）— 预估 2h（取最长）

```
并行启动：
  WP-01 rust-arch 接口契约模板        1.5h
  WP-02 rust-arch IPC 映射表           1.5h
  WP-03 rust-arch 平台抽象模式         1.5h
  WP-05 rust-frontend 文档输出         1h
  WP-07 rust-qa-skill 新建             2h
  WP-10 rust-security-skill 新建       2h
  WP-11 rust-performance-skill 新建    1.5h

完成后串行：
  WP-04 rust-architect-agent 增强      1.5h（依赖 WP-01/02）
```

### 批次 2：依赖层（4 并行）— 预估 2h

```
并行启动：
  WP-08 rust-ui-skill 新建             2h（依赖 WP-05）
  WP-09 rust-integration-skill 新建    2h（依赖 WP-03）
  WP-12 rust-qa-agent 新建             1.5h（依赖 WP-07）
  WP-06 rust-team-lead 增强            1.5h（依赖 WP-10）
```

### 批次 3：Agent 层（2 并行）— 预估 1.5h

```
并行启动：
  WP-13 rust-ui-designer-agent 新建    1.5h（依赖 WP-08）
  WP-14 rust-integration-agent 新建    1.5h（依赖 WP-09/10）
```

### 批次 4：验证层（4 并行 + 1 串行）— 预估 2h

```
并行启动：
  WP-15 验证 QA Agent                  1h
  WP-16 验证 UI Designer Agent         1h
  WP-17 验证 Integration Agent         1h
  WP-19 验证 Architect Agent           0.5h

完成后串行：
  WP-18 验证 team-lead 路由            1h
```

### 批次 5：全局回归 — 预估 0.5h

```
串行：
  WP-20 全局一致性检查                  0.5h
```

**总工期：约 8-10h（含并行优化）**

---

## 里程碑

### M1：基础层完成（批次 1 结束）

**验收标准：**
- [ ] `rust-arch/SKILL.md` 新增 3 个 Section
- [ ] `rust-architect-agent.md` 新增 Step 2.5 + 2.6
- [ ] `rust-qa-skill/` 目录存在，SKILL.md 格式校验通过
- [ ] `rust-security-skill/` 目录存在，SKILL.md 格式校验通过
- [ ] `rust-performance-skill/` 目录存在，SKILL.md 格式校验通过
- [ ] `rust-frontend/SKILL.md` 新增"文档输出规范"Section

### M2：所有 Agent/Skill 创建完成（批次 3 结束）

**验收标准：**
- [ ] `rust-qa-agent.md` 存在且引用 rust-qa-skill
- [ ] `rust-ui-designer-agent.md` 存在且引用 rust-ui-skill，明确"不写代码"
- [ ] `rust-integration-agent.md` 存在且引用 rust-integration-skill + rust-security-skill
- [ ] `rust-team-lead.md` 路由表包含 7+ Agent
- [ ] 所有新增 Skill 的 references/ 和 templates/ 非空

### M3：全局集成验证通过（批次 5 结束）

**验收标准：**
- [ ] WP-15~19 五个验证工作包全部通过
- [ ] 全栈模拟请求走通：PM → Architect(契约) → UI Designer(规格) → Backend/Frontend → QA(验收) → 完成
- [ ] 所有 Agent 引用的 Skill 文件存在，无死链
- [ ] 自动化验收脚本 0 FAIL

---

## 风险登记

| # | 风险 | 概率 | 影响 | 等级 | 缓解方案 |
|---|------|------|------|------|---------|
| R1 | arch Skill 增强范围蔓延，接口契约和平台抽象深度难界定 | 高 | 高 | **极高** | 每个 Section 控制在 80 行以内；超时拆两轮迭代 |
| R2 | UI Designer Agent 边界模糊，容易跟前端 Agent 职责重叠 | 中 | 高 | **高** | 明确"只输出设计规格 Markdown，不写 TSX/Rust" |
| R3 | Integration Agent 覆盖面太广，5 个系统集成域每个都是深坑 | 中 | 中 | **中** | V1 只覆盖 Tauri 官方插件 + macOS，Linux/Windows 放 P2 |
| R4 | QA Agent 验收标准不可执行，写成文档而非可运行检查 | 中 | 高 | **高** | 验证门绑定具体命令（cargo test/clippy/npm test） |
| R5 | team-lead 路由表膨胀，从 4 Agent 扩展到 7+ | 低 | 中 | **中** | 路由按精确触发信号优先匹配，新增信号与现有无交集 |

---

## 验收方法

### 每个 Agent 的验证清单

| Agent | 检查项 |
|-------|--------|
| rust-qa-agent | (1) 文件存在 (2) frontmatter 正确 (3) Skill 引用存在 (4) 模拟 PRD 触发→输出验收报告 (5) 验证门绑定命令 |
| rust-ui-designer-agent | (1)-(3) 同上 (4) 触发→输出设计规格 Markdown (5) 输出含组件/Token/状态/交互 |
| rust-integration-agent | (1)-(3) 同上 (4) 触发→覆盖 macOS (5) 引用 security-skill |
| rust-team-lead(增强) | (1) 路由表 7+ 行 (2) 新增触发信号无交集 (3) 质量门含 UI→前端交接门 |
| rust-architect-agent(增强) | (1) Step 2.5/2.6 存在 (2) 模拟→输出 IPC 契约表 + 依赖 crate 列表 |

### 每个 Skill 的验证清单

| Skill | 检查项 |
|-------|--------|
| rust-qa-skill | (1) frontmatter + Quick Start + 适用范围 (2) references/ 3 文件 (3) templates/ 1 文件 (4) 决策树格式 |
| rust-ui-skill | 同上 + templates/ui-spec-template.md 含组件/Token/状态/交互四节 |
| rust-integration-skill | 同上 + references/ 5 文件对应 5 集成域 |
| rust-security-skill | 同上 + 含 FTS 注入防护和 FFI 安全 |
| rust-performance-skill | 同上 + templates/benchmark-template.rs 语法正确 |

### 自动化验收脚本

```bash
# WP-20 执行时运行

# Agent 文件存在 + frontmatter
for agent in rust-qa-agent rust-ui-designer-agent rust-integration-agent; do
  file="$HOME/.claude/agents/${agent}.md"
  [ -f "$file" ] || echo "FAIL: $file missing"
  head -5 "$file" | grep -q "name:" || echo "FAIL: $file missing name"
  head -5 "$file" | grep -q "tools:" || echo "FAIL: $file missing tools"
done

# Skill 目录完整
for skill in rust-qa-skill rust-ui-skill rust-integration-skill rust-security-skill rust-performance-skill; do
  dir="$HOME/.claude/skills/${skill}"
  [ -f "${dir}/SKILL.md" ] || echo "FAIL: ${dir}/SKILL.md missing"
  [ -d "${dir}/references" ] || echo "FAIL: ${dir}/references/ missing"
  [ -d "${dir}/templates" ] || echo "FAIL: ${dir}/templates/ missing"
done

# 增强 Section 存在
grep -q "接口契约模板" "$HOME/.claude/skills/rust-arch/SKILL.md" || echo "FAIL: arch missing 接口契约模板"
grep -q "IPC 映射表" "$HOME/.claude/skills/rust-arch/SKILL.md" || echo "FAIL: arch missing IPC 映射表"
grep -q "平台抽象模式" "$HOME/.claude/skills/rust-arch/SKILL.md" || echo "FAIL: arch missing 平台抽象模式"
grep -q "文档输出规范" "$HOME/.claude/skills/rust-frontend/SKILL.md" || echo "FAIL: frontend missing 文档输出规范"

# team-lead 路由完整
grep -q "rust-qa-agent" "$HOME/.claude/agents/rust-team-lead.md" || echo "FAIL: team-lead missing QA route"
grep -q "rust-ui-designer" "$HOME/.claude/agents/rust-team-lead.md" || echo "FAIL: team-lead missing UI Designer route"
grep -q "rust-integration" "$HOME/.claude/agents/rust-team-lead.md" || echo "FAIL: team-lead missing Integration route"

echo "All checks complete."
```

---

## 增强后的团队阵型

```
9 Agents：
  现有 6：Team Lead, PM, Architect, Backend, Frontend, Reviewer, Build Resolver
  新增 3：QA Agent, UI Designer Agent, Integration Agent
  增强 2：Team Lead(门控), Architect(契约)

8 Skills：
  现有 5：rust-core, rust-arch, rust-backend, rust-frontend, rust-prd
  新增 5：rust-qa, rust-ui, rust-integration, rust-security, rust-performance
  增强 2：rust-arch(契约+映射+抽象), rust-frontend(文档)

完整工作流：
  PM → UI Designer → Architect → Integration → Backend → Frontend → Reviewer → QA → Build → 发布
         ↑ 新增         ↑ 增强      ↑ 新增                              ↑ 新增(验收)
```

---

## 任务看板

### 批次 1 — 基础层

- [ ] WP-01: rust-arch 接口契约模板
- [ ] WP-02: rust-arch IPC 映射表
- [ ] WP-03: rust-arch 平台抽象模式
- [ ] WP-05: rust-frontend 文档输出要求
- [ ] WP-07: rust-qa-skill
- [ ] WP-10: rust-security-skill
- [ ] WP-11: rust-performance-skill
- [ ] WP-04: rust-architect-agent 增强

### 批次 2 — 依赖层

- [ ] WP-08: rust-ui-skill
- [ ] WP-09: rust-integration-skill
- [ ] WP-12: rust-qa-agent
- [ ] WP-06: rust-team-lead 增强

### 批次 3 — Agent 层

- [ ] WP-13: rust-ui-designer-agent
- [ ] WP-14: rust-integration-agent

### 批次 4 — 验证层

- [ ] WP-15: 验证 QA Agent
- [ ] WP-16: 验证 UI Designer Agent
- [ ] WP-17: 验证 Integration Agent
- [ ] WP-19: 验证 Architect Agent
- [ ] WP-18: 验证 team-lead 路由

### 批次 5 — 回归

- [ ] WP-20: 全局一致性检查

---

## ClipVault V1 复盘追加（已完成）

> 基于 ClipVault V1 实战暴露的问题，追加 4 个专项 Skill 和 Skill 交叉引用机制。

### 新增 Skill（4 个，已完成）

| Skill | 目录 | 核心内容 | 状态 |
|-------|------|---------|------|
| rust-async-patterns | `~/.claude/skills/rust-async-patterns/` | spawn_blocking 时序、Mutex 选型、竞态防护、临时文件安全 | done |
| rust-crash-debug | `~/.claude/skills/rust-crash-debug/` | 5 类闪退决策树（panic/WebView/系统级/DB死锁/状态风暴） | done |
| rust-release-checklist | `~/.claude/skills/rust-release-checklist/` | 9 步发版清单（编译→格式→lint→测试→类型→安全→构建→冒烟→发布） | done |
| rust-tauri-testing | `~/.claude/skills/rust-tauri-testing/` | Tauri v2 Rust 单元/集成测试 + 前端 Vitest 测试 | done |

### 增强 Skill（3 个，已完成）

| Skill | 增强内容 | 状态 |
|-------|---------|------|
| rust-security-skill | 新增 Section 7（剪贴板管理器专项安全：临时文件生命周期、Paste 竞态、pasteboard 权限） | done |
| rust-frontend | 新增组件拆分硬约束（>400行拆分、>3 useState 提取 hook、ErrorBoundary 规则） | done |
| rust-core | 新增临时文件安全节（5 项约束：权限/随机名/防 symlink/清理/路径验证） | done |

### 交叉引用机制（已完成）

所有 Skill 的 References 区域新增 `相关 Skill` 表格，形成引用网：

```
rust-core ─────────┬── rust-async-patterns
                   ├── rust-crash-debug
                   ├── rust-tauri-testing
                   └── rust-security-skill

rust-async-patterns ── rust-security-skill
                     ├── rust-crash-debug
                     ├── rust-core
                     └── rust-tauri-testing

rust-crash-debug ── rust-async-patterns
                  ├── rust-security-skill
                  ├── rust-core
                  └── rust-release-checklist

rust-release-checklist ── rust-tauri-testing
                       ├── rust-security-skill
                       ├── rust-crash-debug
                       └── rust-async-patterns

rust-tauri-testing ── rust-release-checklist
                    ├── rust-async-patterns
                    ├── rust-frontend
                    └── rust-core
```

---

## Skill 调用链

### 开发阶段

```
新功能/修复请求
  │
  ├─ rust-prd-skill ── 需求分析
  │
  ├─ rust-arch ── 架构设计
  │   └─ rust-async-patterns ── async 方案决策
  │
  ├─ rust-security-skill ── 安全审计
  │   └─ rust-core（临时文件安全） ── 写文件时的安全约束
  │
  ├─ rust-backend ── Rust 实现
  │   └─ rust-integration-skill ── 系统 FFI 集成
  │
  ├─ rust-frontend ── 前端实现
  │   └─ rust-ui-skill ── UI 设计规格
  │
  └─ rust-tauri-testing ── 测试编写
```

### 调试阶段

```
应用闪退/异常
  │
  ├─ rust-crash-debug ── 诊断决策树
  │   ├─ panic → rust-core（unwrap 替换）
  │   ├─ WebView 崩溃 → rust-security-skill（CSP）
  │   ├─ DB 死锁 → rust-async-patterns（Mutex）
  │   └─ 状态风暴 → rust-frontend（事件去重）
  │
  └─ rust-async-patterns ── async 反模式检测
```

### 发版阶段

```
准备发版
  │
  └─ rust-release-checklist（9 步强制清单）
      ├─ Step 1-3: 编译/格式/Lint → rust-core
      ├─ Step 4: 测试 → rust-tauri-testing
      ├─ Step 6: 安全 → rust-security-skill
      ├─ Step 7: 构建 → rust-backend
      └─ Step 8: 冒烟 → rust-crash-debug
```
