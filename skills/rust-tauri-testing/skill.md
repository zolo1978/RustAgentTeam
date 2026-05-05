---
name: rust-tauri-testing
description: 'Tauri v2 应用测试：Rust 单元/集成测试、前端 Vitest 组件测试、IPC 契约测试。决策树格式——给结论，不列菜单。'
type: skill
tools: ['Read', 'Glob', 'Grep', 'Bash', 'Edit', 'Write']
---

# Tauri v2 测试

## 决策树

```
需要测试？
├─ 纯函数/工具函数 → Rust #[test] 单元测试
├─ 数据库查询 → 内存 SQLite + #[test] 单元测试
├─ Tauri command（有 AppState）→ 集成测试，用 state::new_test()
├─ 前端 hook → Vitest + @testing-library/react
├─ 前端工具函数 → Vitest 纯函数测试
├─ IPC 契约（前后端接口一致性）→ rust-qa-skill 契约验证
└─ E2E 用户流程 → Playwright（见 e2e-testing skill）
```

---

## 1. Rust 后端测试

### 1.1 纯函数 — 直接 `#[test]`

适用：`compute_hash`、`make_preview`、`ContentType::from_str`、FTS sanitize 逻辑。

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compute_hash_returns_consistent_sha256() {
        let h1 = compute_hash("hello");
        let h2 = compute_hash("hello");
        assert_eq!(h1, h2);
        assert!(h1.starts_with("2cf24dba")); // SHA-256 前缀
    }

    #[test]
    fn make_preview_truncates_long_text() {
        let long = "a".repeat(500);
        let preview = make_preview(&long);
        assert!(preview.len() <= 203); // 200 chars + "..."
    }
}
```

### 1.2 数据库操作 — 内存 SQLite

适用：`clip_repo` 的所有查询。

**前提**：`state.rs` 必须有 `new_test()` 方法返回内存数据库的 `AppState`。

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::AppState;

    fn test_db() -> Arc<Mutex<Connection>> {
        let state = AppState::new_test();
        state.db
    }

    #[test]
    fn insert_and_query_clip() {
        let db = test_db();
        let conn = db.lock().unwrap();
        // 运行 migration
        crate::state::run_migrations(&conn).unwrap();

        let id = uuid::Uuid::now_v7().to_string();
        clip_repo::insert_clip(&conn, &Clip { id: id.clone(), .. });
        let result = clip_repo::get_clip(&conn, &id).unwrap();
        assert!(result.is_some());
    }
}
```

### 1.3 Tauri Commands — 集成测试

适用：验证 command 层的输入验证、错误处理。

```rust
// tests/commands_test.rs
use clipvault_lib::state::AppState;

fn setup() -> AppState {
    AppState::new_test()
}

#[tokio::test]
async fn get_clip_returns_not_found_for_invalid_id() {
    let state = setup();
    let result = commands::get_clip("nonexistent".into(), state.db.clone()).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn list_clips_respects_limit() {
    let state = setup();
    // 插入 20 条，请求 limit=5
    let result = commands::list_clips(ListParams { limit: 5, offset: 0, content_type: None }, state.db.clone()).await;
    assert!(result.unwrap().len() <= 5);
}
```

### 1.4 测试覆盖要求

| 层 | 最低覆盖率 | 关键测试点 |
|----|-----------|-----------|
| repositories | 90% | CRUD、FTS 搜索、purge、batch delete |
| services | 80% | 去重逻辑、缩略图生成、hash 计算 |
| commands | 70% | 输入验证、错误映射 |
| models | 90% | 序列化/反序列化、ContentType 解析 |

运行覆盖率：
```bash
cargo tarpaulin --skip-clean --out Html --output-dir target/coverage
```

---

## 2. 前端测试

### 2.1 工具函数 — Vitest 纯测试

适用：`formatTime`、`HighlightText`、`dataUriToBlob`、`safeInvoke`。

```typescript
// src/lib/__tests__/utils.test.ts
import { describe, it, expect } from 'vitest';

describe('dataUriToBlob', () => {
  it('converts valid data URI to blob URL', () => {
    const b64 = btoa('test');
    const uri = `data:image/png;base64,${b64}`;
    const result = dataUriToBlob(uri);
    expect(result).toMatch(/^blob:/);
    URL.revokeObjectURL(result); // 清理
  });

  it('returns original string on invalid input', () => {
    expect(dataUriToBlob('not-a-data-uri')).toBe('not-a-data-uri');
  });
});

describe('formatTime', () => {
  it('shows "刚刚" for <60s', () => {
    const now = Date.now();
    expect(formatTime(now)).toBe('刚刚');
  });

  it('shows "N分钟前" for <60min', () => {
    const fiveMinAgo = Date.now() - 5 * 60 * 1000;
    expect(formatTime(fiveMinAgo)).toContain('分钟前');
  });
});
```

### 2.2 Hook 测试 — @testing-library/react

```typescript
// src/hooks/__tests__/useClips.test.ts
import { renderHook, act } from '@testing-library/react';
import { useClips } from '../useClips';

// mock safeInvoke
vi.mock('../../lib/safe-invoke', () => ({
  safeInvoke: vi.fn().mockResolvedValue([
    { id: '1', content_type: 'text', preview: 'hello', is_favorite: false, created_at: Date.now() }
  ]),
}));

describe('useClips', () => {
  it('loads clips on mount', async () => {
    const { result } = renderHook(() => useClips());
    await waitFor(() => expect(result.current.loading).toBe(false));
    expect(result.current.clips).toHaveLength(1);
  });

  it('merge loadClips and loadClipsSilent via silent param', () => {
    // 验证 silent=true 不设置 loading
  });
});
```

### 2.3 组件测试 — @testing-library/react

```typescript
// src/views/__tests__/ClipItem.test.tsx
import { render, screen, fireEvent } from '@testing-library/react';

describe('ClipItem', () => {
  it('shows preview text', () => {
    render(<ClipItem clip={mockClip} query="" onAction={vi.fn()} />);
    expect(screen.getByText('hello')).toBeInTheDocument();
  });

  it('calls onAction on double click', () => {
    const onAction = vi.fn();
    render(<ClipItem clip={mockClip} query="" onAction={onAction} />);
    fireEvent.doubleClick(screen.getByRole('listitem'));
    expect(onAction).toHaveBeenCalledWith(mockClip);
  });
});
```

### 2.4 前端覆盖要求

| 类型 | 最低覆盖率 |
|------|-----------|
| 工具函数 | 90% |
| Hooks | 80% |
| 组件 | 70% |

运行：`npx vitest run --coverage`

---

## 3. CI 集成

```yaml
# .github/workflows/ci.yml 中添加
- name: Run Rust tests
  working-directory: src-tauri
  run: cargo test

- name: Run frontend tests
  run: npx vitest run
```

---

## 4. 反模式检测清单

在 code review 时，遇到以下情况立即标记：

| 反模式 | 问题 | 修复 |
|--------|------|------|
| `#[test]` 不存在 | 零覆盖 | 为每个 pub 函数写测试 |
| `unwrap()` 在测试外的测试中 | 测试本身会 panic | 用 `assert!` 或 `expect` |
| 测试依赖文件系统/网络 | 不稳定 | mock 或用内存替代 |
| `#[ignore]` 测试从不运行 | 死测试 | 删除或修复 |
| 快照测试过于严格 | 脆弱 | 用语义断言替代 |
| mock 过多 | 测试无意义 | 用真实实现或集成测试 |
| 测试之间共享状态 | 顺序依赖 | 每个测试独立 setup |
| 只测 happy path | 边界未覆盖 | 添加错误、空值、边界测试 |

---

## 5. 检查命令

```bash
# Rust
cargo test                              # 运行所有测试
cargo test -- --nocapture               # 显示 println! 输出
cargo test clip_repo                    # 只跑 clip_repo 模块
cargo tarpaulin --skip-clean            # 覆盖率报告

# Frontend
npx vitest run                          # 运行所有前端测试
npx vitest run --coverage               # 覆盖率报告
npx vitest run src/lib                  # 只跑 lib 目录

# 验证
cargo test 2>&1 | grep "test result"   # 应显示 >0 passed
npx vitest run 2>&1 | grep "Tests"     # 应显示 >0 passed
```

## 相关 Skill

| Skill | 关联场景 |
|-------|---------|
| [rust-release-checklist](../rust-release-checklist/skill.md) | 发版前测试执行顺序 |
| [rust-async-patterns](../rust-async-patterns/skill.md) | async 函数 spawn_blocking 测试 |
| [rust-frontend](../rust-frontend/SKILL.md) | 前端测试 mock 模式 |
| [rust-core](../rust-core/SKILL.md) | TDD 流程、覆盖率工具 |
