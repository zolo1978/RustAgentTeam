---
name: rust-fix-planner
description: Standardized bug fix workflow for Rust/Tauri projects — intake, root cause, plan, execute, verify
---

# Rust Fix Planner

Standardized bug fix workflow for Rust/Tauri desktop projects.

## Trigger

Use when the user reports a bug, regression, or requests a fix. This skill drives the entire fix lifecycle.

## Workflow

### Phase 1: Intake (analysis mode)

1. **Restate** the bug in one sentence, confirm severity (P0-P3)
2. **Locate** the affected files with targeted searches (max 5 tool calls)
3. **Identify** root cause — distinguish between:
   - Logic bug (wrong condition, missing branch)
   - Integration bug (frontend/backend contract mismatch)
   - Platform bug (macOS/Linux/Windows specific)
   - Config bug (missing setup, wrong defaults)
4. **Scope** — files that need to change, blast radius

### Phase 2: Plan (plan mode)

1. Write the fix plan to `.claude/fix-plan.md`:
   ```
   ## Bug: [title]
   Severity: P0-P3
   Root cause: [one line]
   
   ### Changes
   - [ ] File 1: what changes
   - [ ] File 2: what changes
   
   ### Risk
   - [side effects]
   
   ### Verify
   - [ ] [test scenario]
   ```

### Phase 3: Execute (execution mode)

1. Apply fixes via Edit tool (not codex — direct for precision)
2. Each edit tagged with the plan item it addresses
3. After each edit: `cargo check` to verify compilation
4. If compilation fails: fix immediately, don't accumulate errors

### Phase 4: Verify (analysis mode)

1. Start `tauri dev` (or `cargo build` for backend-only)
2. Check for:
   - Compilation: 0 errors, 0 new warnings
   - Runtime: feature works as expected
   - Regression: existing features still work
3. Self-reflect against rubric:
   - Fix addresses root cause (not symptoms)
   - No new warnings introduced
   - Edge cases handled
   - Security not degraded

### Phase 5: Close

1. Delete `.claude/fix-plan.md`
2. Report to user: what was fixed, which files, any remaining risk

## MCP Restrictions

During analysis phases (1, 2, 4),禁止调用 context7、exa、web-search-prime、open-websearch、web_reader 等 MCP 工具。仅使用 Read/Glob/Grep/Bash 进行本地分析。

## Severity Guide

| Level | Meaning | Response Time |
|-------|---------|---------------|
| P0 | Data loss, crash, security breach | Fix now, ship immediately |
| P1 | Feature broken for all users | Fix this session |
| P2 | Feature broken for edge case | Fix within 24h |
| P3 | Cosmetic, UX annoyance | Batch fix |

## Fix Principles

1. **Root cause over symptoms** — fix the source, not the manifestation
2. **Minimal change** — smallest diff that fixes the bug
3. **No regression** — verify existing features after fix
4. **Document why** — if the fix is non-obvious, add a comment
5. **Test the fix** — at minimum, manual verification; ideally add a regression test
