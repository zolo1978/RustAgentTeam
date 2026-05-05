---
name: rust-frontend
description: 'Tauri v2 前端开发：React+TS+Vite、IPC 封装、事件监听、表单、暗色模式、文件对话框、键盘快捷键、无障碍、前端测试、自动更新。面向前端 Agent。决策树格式。'
type: skill
---

# Tauri v2 前端开发

## Quick Start
1. IPC 调用？Copy `templates/safe-invoke.ts` — 永不裸调用 invoke
2. 表单？Copy `templates/use-form.ts` — 泛型 hook
3. 暗色模式？Copy `templates/theme.ts` — CSS 变量 + 持久化
4. 更新检查？Copy `templates/check-for-update.ts`

## 适用范围
**适用于：** Tauri v2 WebView 前端开发——React + TypeScript + Vite、IPC 封装、表单、暗色模式、事件监听、文件对话框、无障碍、前端测试。
**不适用于：** Rust 后端开发（见 `rust-backend`）、架构决策（见 `rust-arch`）、Rust 语言核心（见 `rust-core`）、PRD（见 `rust-prd-skill`）。
**目标：** React 19 + TS 5.x + Vite, Tauri >= 2.0.0。
**其他框架：** Vue/Svelte 适配见 `references/vue-svelte-migration.md`——用 Composition API 等价替换 Hooks，核心模式（IPC、事件、对话框）完全相同。

## 1. IPC 调用

**推荐方案：** `safeInvoke<T>` 封装所有 IPC 调用，内置重试（指数退避）和超时（Promise.race）。
**理由：** 类型安全的 IPC 防止运行时错误，自动重试处理 transient 错误，超时防止无限等待。
**怎么做：**

```typescript
import { safeInvoke } from './safe-invoke';

// 永不裸调用 invoke
export async function createUser(data: CreateUserInput): Promise<User> {
  return safeInvoke<User>('create_user', data);
}
```

**子决策：**
| 场景 | 方案 |
|------|------|
| 普通查询 | `safeInvoke<T>` 默认 10s 超时 |
| 文件操作 | 增加超时到 30s `safeInvoke<T>(cmd, args, { timeoutMs: 30000 })` |
| 不需要重试 | `safeInvoke<T>(cmd, args, { retries: 0 })` |
| 批量操作 | 单次 invoke 传完整对象，不逐字段调用 |

→ 深入：[IPC 错误处理](references/ipc-error-handling.md)

## 2. 表单

**推荐方案：** `useForm<T>` 泛型 hook 管理表单状态、校验和提交。
**理由：** 泛型表单减少重复代码，类型安全防止字段遗漏。
**怎么做：**

```typescript
const { values, errors, handleSubmit, handleChange } = useForm<CreateUserInput>({
  initialValues: { name: '', email: '' },
  onSubmit: async (values) => { await createUser(values); },
});
```

→ 模板：`templates/use-form.ts`

## 3. 暗色模式

**推荐方案：** CSS 变量 + `@tauri-apps/plugin-store` 持久化 + 系统偏好检测。
**理由：** CSS 变量零依赖、性能好；Store 持久化跨重启保持；系统检测无需用户手动设置。
**怎么做：**

```typescript
import { initTheme, setTheme, useTheme } from './theme';

// 初始化（App.tsx 中调用一次）
await initTheme();

// 切换
await setTheme('dark');

// 组件中读取
const { theme, setTheme } = useTheme();
```

→ 模板：`templates/theme.ts`
→ 深入：[暗色模式](references/theme.md)

## 4. 状态管理

**推荐方案：** Zustand 管理全局状态，按需订阅。
**理由：** Zustand 比 prop drilling 更可维护，比 Redux 更轻量，按需订阅避免不必要渲染。
**怎么做：**

```typescript
const name = useUserStore(s => s.user?.name);
```

**子决策：**
| 场景 | 方案 |
|------|------|
| 全局共享状态 | Zustand store |
| 组件内状态 | `useState` / `useReducer` |
| 服务端数据 | React Query 或自定义 hook + `safeInvoke` |
| 表单状态 | `useForm<T>` hook |

## 5. 无障碍

**推荐方案：** 语义 HTML + ARIA 属性 + 键盘导航。
**理由：** 桌面应用的键盘用户比例高，屏幕阅读器依赖语义标记。
**怎么做：**

```typescript
// 按钮：有语义和键盘支持
<button onClick={handleSave} aria-label="保存用户" type="submit">

// 错误提示：role="alert" 让屏幕阅读器立即播报
<div role="alert">{error}</div>

// 表单：label 关联
<label htmlFor="name">姓名</label>
<input id="name" type="text" aria-invalid={!!errors.name} />
```

→ 深入：[无障碍](references/a11y.md)

## 6. Event 事件

**推荐方案：** `listen` / `emit` 做双向事件通信，组件卸载时必须 `unlisten`。
**理由：** Rust 后端推送状态变更（下载进度、后台同步完成）需要事件机制，比前端轮询高效 10 倍。
**怎么做：**

```typescript
import { listen } from '@tauri-apps/api/event';

function useDownloadProgress() {
  const [progress, setProgress] = useState(0);

  useEffect(() => {
    const unlisten = listen<number>('download-progress', (e) => {
      setProgress(e.payload);
    });
    return () => { unlisten.then(fn => fn()); };  // 必须清理
  }, []);

  return progress;
}
```

**子决策：**
| 场景 | 方案 |
|------|------|
| Rust→前端推送 | `app.emit("event", payload)` 前端 `listen` |
| 前端→Rust 通知 | `invoke` Command（不用 event，保证返回值） |
| 前端→前端通信 | `emit` 同窗口事件，跨窗口用 `emitTo("window-label")` |
| 流式数据 | `Channel<T>`（见 `rust-arch`），不用 event 轮询 |

→ 深入：[事件处理](references/event-handling.md)

## 7. 文件对话框

**推荐方案：** `@tauri-apps/plugin-dialog` 的 `open` / `save`，不自己造 UI。
**理由：** 系统 native 对话框有沙盒权限支持，自定义文件对话框在桌面应用中不自然。
**怎么做：**

```typescript
import { open, save } from '@tauri-apps/plugin-dialog';

async function pickFile() {
  const path = await open({
    multiple: false,
    filters: [{ name: 'Images', extensions: ['png', 'jpg', 'webp'] }],
  });
  if (path) return path;  // string | null
}

async function saveFile(defaultName: string) {
  const path = await save({
    defaultPath: defaultName,
    filters: [{ name: 'JSON', extensions: ['json'] }],
  });
  if (path) return path;
}
```

**子决策：**
| 场景 | 方案 |
|------|------|
| 选择文件 | `open({ multiple: false })` |
| 选择目录 | `open({ directory: true })` |
| 保存文件 | `save({ defaultPath })` |
| 移动端 | 用 `@tauri-apps/plugin-fs` + 系统 Picker |

→ 深入：[文件对话框](references/file-dialogs.md)

## 8. 键盘快捷键

**推荐方案：** 全局快捷键用 `@tauri-apps/plugin-global-shortcut`，组件内用原生 `onKeyDown`。
**理由：** 全局快捷键在窗口失焦时仍生效（如截图工具），组件内快捷键作用域有限。
**怎么做：**

```typescript
// 全局快捷键（注册一次，App.tsx 中）
import { register } from '@tauri-apps/plugin-global-shortcut';

useEffect(() => {
  register('CommandOrControl+Shift+S', (event) => {
    if (event.state === 'Pressed') saveAll();
  });
}, []);

// 组件内快捷键
function Editor() {
  const handleKeyDown = (e: KeyboardEvent) => {
    if ((e.metaKey || e.ctrlKey) && e.key === 's') {
      e.preventDefault();
      handleSave();
    }
  };
  // ...
}
```

**子决策：**
| 场景 | 方案 |
|------|------|
| 全局快捷键（任何窗口状态） | `global-shortcut` 插件 |
| 单窗口内快捷键 | `onKeyDown` / `useEffect` + `addEventListener` |
| 菜单快捷键 | Tauri Menu 定义（见 `rust-backend`） |
| 移动端 | 不适用，跳过快捷键注册 |

## 9. 测试

**推荐方案：** Vitest + `vi.mock` + Testing Library。三层覆盖：IPC mock、组件行为、表单校验。
**理由：** Vitest 与 Vite 原生集成，mock 能力覆盖 IPC 调用，Testing Library 测试用户行为而非实现。
**怎么做：**

```typescript
// 1. IPC Mock 模式
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));
import { invoke } from '@tauri-apps/api/core';

it('提交表单时调用 invoke', async () => {
  (invoke as ReturnType<typeof vi.fn>).mockResolvedValueOnce({ id: '1' });
  render(<UserCreateView />);
  fireEvent.change(screen.getByLabelText('姓名'), { target: { value: '张三' } });
  fireEvent.click(screen.getByRole('button', { name: '保存' }));
  await waitFor(() => {
    expect(invoke).toHaveBeenCalledWith('create_user', { name: '张三' });
  });
});

// 2. safeInvoke 测试（直接测试封装函数）
import { safeInvoke, IpcError } from './safe-invoke';

it('重试 transient 错误', async () => {
  (invoke as ReturnType<typeof vi.fn>)
    .mockRejectedValueOnce(new Error('timeout'))
    .mockResolvedValueOnce('ok');
  await expect(safeInvoke('cmd')).resolves.toBe('ok');
});

it('非 transient 错误不重试', async () => {
  (invoke as ReturnType<typeof vi.fn>)
    .mockRejectedValue(new Error('permission denied'));
  await expect(safeInvoke('cmd')).rejects.toThrow(IpcError);
});

// 3. 表单 hook 测试
const { result } = renderHook(() => useForm({
  initialValues: { name: '' },
  validate: (v) => v.name ? {} : { name: '必填' },
  onSubmit: vi.fn(),
}));
act(() => result.current.handleSubmit());
expect(result.current.errors.name).toBe('必填');
```

→ 深入：[测试 Mock](references/testing-mock.md)

## 10. 组件架构

**推荐方案：** 按功能拆分组件，页面组件 + 通用组件 + hook 三层分离。
**理由：** 页面组件处理路由和数据获取，通用组件纯展示可复用，hook 封装逻辑。
**怎么做：**

```
src/
├── views/          # 页面组件（路由 + 数据获取）
├── components/     # 通用组件（纯展示）
├── hooks/          # 自定义 hook（逻辑封装）
├── api/            # IPC 封装（safeInvoke）
└── lib/            # 工具函数
```

### 组件拆分硬约束

| 规则 | 阈值 | 操作 |
|------|------|------|
| 单文件行数 | >400 行 | 必须拆分 |
| useState 数量 | >3 个 | 提取为自定义 hook |
| useEffect 数量 | >3 个 | 提取为自定义 hook |
| Props 数量 | >6 个 | 合并为对象或拆分组件 |
| 嵌套条件渲染 | >3 层 | 提取子组件 |

**ErrorBoundary 规则：**
- 必须使用 class component（React 对函数组件的 error boundary 无官方支持）
- 每个页面级组件至少一个 ErrorBoundary
- fallback UI 显示错误信息 + 重试按钮

**判断信号：**
- 组件文件滚动超过 2 屏 → 太长，拆分
- `// --- Section ---` 注释分割 → 应拆为独立组件
- 同一个 useState 被 >2 个函数修改 → 提取 reducer 或 hook

→ 深入：[组件架构](references/component-architecture.md)
→ 表单模式：[表单模式](references/form-patterns.md)

## Crate Dependencies（前端相关）

```json
{
  "dependencies": {
    "@tauri-apps/api": "^2",
    "@tauri-apps/plugin-dialog": "^2",
    "@tauri-apps/plugin-fs": "^2",
    "@tauri-apps/plugin-store": "^2",
    "@tauri-apps/plugin-global-shortcut": "^2",
    "react": "^19",
    "zustand": "^5"
  },
  "devDependencies": {
    "typescript": "^5.5",
    "vite": "^6",
    "vitest": "^3",
    "@testing-library/react": "^16",
    "@testing-library/react-hooks": "^8",
    "tailwindcss": "^4"
  }
}
```

## References

| Topic | File |
|-------|------|
| 组件架构 | [component-architecture.md](references/component-architecture.md) |
| 表单模式 | [form-patterns.md](references/form-patterns.md) |
| IPC 错误处理 | [ipc-error-handling.md](references/ipc-error-handling.md) |
| 测试 Mock | [testing-mock.md](references/testing-mock.md) |
| 无障碍 | [a11y.md](references/a11y.md) |
| 暗色模式 | [theme.md](references/theme.md) |
| 事件处理 | [event-handling.md](references/event-handling.md) |
| 文件对话框 | [file-dialogs.md](references/file-dialogs.md) |
| Vue/Svelte 适配 | [vue-svelte-migration.md](references/vue-svelte-migration.md) |

## Templates

| 用途 | 文件 |
|------|------|
| IPC 安全调用 | [safe-invoke.ts](templates/safe-invoke.ts) |
| 表单 Hook | [use-form.ts](templates/use-form.ts) |
| 暗色模式 | [theme.ts](templates/theme.ts) |
| 自动更新检查 | [check-for-update.ts](templates/check-for-update.ts) |

## 11. 文档输出规范

**推荐方案：** 每个组件、Hook、IPC 封装函数必须有对应文档，变更时同步更新 README 和变更日志。
**理由：** 文档是接口契约的一部分，缺失文档等于缺失类型检查——别人调用你的组件/Hook/API 时只能猜。
**怎么做：** 按以下五个子项执行。

### 11.1 组件文档

每个 React 组件必须有 Props 说明，用 TypeScript interface 注释或 JSDoc。

```typescript
// BAD: Props 无注释，调用者不知道每个字段含义
interface UserCardProps {
  user: User;
  onDelete: (id: string) => void;
  editable?: boolean;
}

// GOOD: 每个字段有明确说明
interface UserCardProps {
  /** 要展示的用户对象 */
  user: User;
  /** 删除按钮回调，传入用户 ID */
  onDelete: (id: string) => void;
  /** 是否显示编辑入口，默认 false */
  editable?: boolean;
}
```

**例外：** 纯展示组件 props 少于 3 个且字段名自解释时，可省略注释。

### 11.2 Hook 文档

自定义 Hook 必须有入参、返回值、使用示例。

```typescript
// BAD: Hook 无任何说明
export function useDebounce<T>(value: T, delay: number): T { ... }

// GOOD: JSDoc 完整覆盖
/**
 * 防抖 Hook，延迟更新值直到输入停止变化。
 *
 * @param value - 需要防抖的原始值
 * @param delay - 延迟毫秒数，默认 300
 * @returns 防抖后的值
 *
 * @example
 * const debouncedSearch = useDebounce(searchTerm, 500);
 * useEffect(() => { fetchResults(debouncedSearch); }, [debouncedSearch]);
 */
export function useDebounce<T>(value: T, delay = 300): T { ... }
```

**例外：** 内部 Hook（不以 `use` 开头导出或文件名含 `.internal.`）可简化为单行注释。

### 11.3 IPC API 文档

每个 `safeInvoke` 封装函数必须有参数和返回类型说明。

```typescript
// BAD: 无说明，参数类型靠猜
export async function deleteUser(id: string): Promise<void> {
  return safeInvoke('delete_user', { id });
}

// GOOD: 参数、返回值、错误场景清晰
/**
 * 删除指定用户。
 *
 * @param id - 用户唯一标识
 * @throws {IpcError} 用户不存在或权限不足时抛出
 *
 * IPC Command: `delete_user`
 * Request:  { id: string }
 * Response: void
 */
export async function deleteUser(id: string): Promise<void> {
  return safeInvoke('delete_user', { id });
}
```

**例外：** 与 Rust Command 一一对应的 CRUD 函数集，可用表格批量说明而非逐个 JSDoc。

### 11.4 README 片段

新增功能时更新 README 对应区域，保持 README 与代码同步。

```markdown
<!-- BAD: README 只有初始搭建说明，新增的暗色模式功能完全没提 -->

<!-- GOOD: README 按功能区域组织，新增功能有对应条目 -->
## 功能

- 剪贴板历史记录管理
- 暗色模式（系统偏好检测 + 手动切换）
- 全局快捷键呼出（Cmd+Shift+V）
```

**例外：** Bug 修复或内部重构不涉及用户可见行为时，无需更新 README。

### 11.5 变更日志

记录新增、修改、删除的组件和 API，格式统一。

```markdown
<!-- BAD: 提交信息就是唯一的变更记录，无人知道改了什么 -->

<!-- GOOD: CHANGELOG.md 或等效文件中结构化记录 -->
## [0.3.0] - 2025-04-29

### Added
- `UserCard` 组件：支持展示和删除用户
- `useDebounce` Hook：通用防抖

### Changed
- `deleteUser` API：新增权限不足错误码 `PERMISSION_DENIED`

### Removed
- `UserList` 组件：被 `UserCard` + `VirtualList` 替代
```

**例外：** 未发布的开发分支可省略变更日志，合并到 main/release 分支前补齐即可。

**子决策：**
| 场景 | 方案 |
|------|------|
| 新组件 | 11.1 Props 注释 + 11.4 README 条目 |
| 新 Hook | 11.2 JSDoc（入参/返回值/示例） |
| 新 IPC 封装 | 11.3 IPC API 文档 |
| 修改已有接口 | 11.5 变更日志 + 更新对应文档 |
| 删除组件/API | 11.5 变更日志 Removed 段 + README 移除条目 |
| 内部重构不影响接口 | 仅 11.5 变更日志 Changed 段 |
