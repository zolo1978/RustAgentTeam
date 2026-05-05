# 组件架构参考

## 目录约定

```
src/
  components/       # 可复用 UI 原子组件（Button, Modal, Input, Toast）
  views/            # 页面级组件，与路由一一对应
  hooks/            # 自定义 hook（useForm, useIpcQuery）
  stores/           # Zustand store 定义
  api/              # safeInvoke 封装层，按领域分文件
```

## 组件 API 设计原则

1. **Props 只传数据**，事件用 `onXxx` 回调命名（`onSubmit`, `onChange`, `onCancel`）。
2. **禁止在组件内部直接调用 `invoke`**——所有 IPC 调用必须通过 `api/` 层或 hook 间接调用。
3. 组件只负责 UI 渲染和用户交互，状态管理和副作用由 hook 驱动。

## 拆分时机

以下任一条件满足时必须拆分：
- 组件超过 **150 行**
- 包含 **3 个以上独立关注点**（如表单逻辑 + 列表渲染 + 弹窗控制）
- 有可复用的子逻辑（提取为独立组件或 hook）

## 示例：结构清晰的 UserForm 组件

```typescript
// views/UserForm.tsx
import { useForm } from '../hooks/useForm';
import { createUser } from '../api/users';
import { InputField, Button } from '../components';

interface User {
  name: string;
  email: string;
}

interface UserFormProps {
  initialData?: User;
  onSubmit: (data: User) => Promise<void>;
  onCancel: () => void;
}

export function UserForm({ initialData, onSubmit, onCancel }: UserFormProps) {
  const { values, errors, handleChange, handleSubmit, isSubmitting } =
    useForm<User>({
      initialValues: initialData ?? { name: '', email: '' },
      validate: userSchema,
      onSubmit,
    });

  return (
    <form onSubmit={handleSubmit(onSubmit)} aria-label="用户表单">
      <InputField label="姓名" name="name" value={values.name}
        error={errors.name} onChange={handleChange} />
      <InputField label="邮箱" name="email" value={values.email}
        error={errors.email} onChange={handleChange} />
      <div className="flex gap-4 mt-4">
        <Button type="submit" loading={isSubmitting}>保存</Button>
        <Button variant="ghost" onClick={onCancel}>取消</Button>
      </div>
    </form>
  );
}
```

## 状态管理：Zustand 模式

```typescript
// stores/userStore.ts
import { create } from 'zustand';

interface UserState {
  user: User | null;
  setUser: (user: User) => void;
  clear: () => void;
}

export const useUserStore = create<UserState>((set) => ({
  user: null,
  setUser: (user) => set({ user }),
  clear: () => set({ user: null }),
}));

// 消费方：用 selector 按需订阅，避免不必要的重渲染
const name = useUserStore((s) => s.user?.name);
```

**关键原则：** 用 `useStore(s => s.field)` selector 精确订阅，杜绝 prop drilling。

## 错误边界：useIpcQuery 模式

```typescript
// hooks/useIpcQuery.ts
function useIpcQuery<T>(command: string, args: unknown[]) {
  const [state, setState] = useState<{ data: T | null; error: IpcError | null }>({
    data: null, error: null,
  });
  // Serialize args to stabilize the useCallback dependency and prevent infinite re-fetches.
  // Without this, spreading `...args` creates a new dependency array on every render.
  const argsKey = JSON.stringify(args);
  const fetch = useCallback(async () => {
    setState({ data: null, error: null });
    try {
      const data = await safeInvoke<T>(command, args);
      setState({ data, error: null });
    } catch (err) {
      setState({ data: null, error: err instanceof IpcError ? err : new IpcError('UNKNOWN', String(err)) });
    }
  }, [command, argsKey]);

  useEffect(() => { fetch(); }, [fetch]);

  return { ...state, retry: fetch, loading: state.data === null && state.error === null };
}

// 组件中的使用
const { data, error, retry, loading } = useIpcQuery('get_files', []);
if (loading) return <Spinner />;
if (error) return <ErrorPanel message={error.toUserMessage()} onRetry={retry} />;
return <FileList files={data!} />;
```
