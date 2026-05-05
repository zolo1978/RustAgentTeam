# 表单完整模式（状态 + 校验 + 提交 + Tauri invoke）

> 泛型代码模板：[templates/use-form.ts](../templates/use-form.ts)

## 核心模式

泛型 `useForm<T>` hook 封装表单状态：`useState` 管理值、校验错误、提交状态和服务端错误。

- **validate**：调用方提供的纯函数，返回 `Partial<Record<keyof T, string>>`，前端校验先于后端。
- **handleChange**：更新对应字段值，同时清除已有错误。
- **handleSubmit**：先校验，通过后调用调用方提供的 `onSubmit`（底层通常是 `safeInvoke`），失败时回填 `serverError`。
- **serverError**：展示后端返回的用户友好消息，不暴露技术细节。

## 关键原则

1. 组件（如 `UserForm`）只负责 UI 渲染，不直接调用 `invoke`。
2. Hook 负责状态管理和副作用，返回稳定的接口。
3. API 层（如 `api/users.ts`）封装 `safeInvoke`，类型安全 + 统一错误处理。
4. 校验在前端完成后才提交，减少无谓 IPC 调用。

## 使用示例

```typescript
// 基于 useForm<T> 构建具体表单 hook
import { useForm } from '../hooks/useForm';
import { createUser } from '../api/users';

interface UserInput { name: string; email: string }

function useUserCreate(onSuccess: () => void) {
  return useForm<UserInput>({
    initialValues: { name: '', email: '' },
    validate: (v) => {
      const e: Partial<Record<keyof UserInput, string>> = {};
      if (!v.name.trim()) e.name = '姓名必填';
      if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(v.email)) e.email = '邮箱格式错误';
      return e;
    },
    onSubmit: async (values) => {
      await createUser(values);  // 底层调用 Tauri invoke
      onSuccess();
    },
  });
}
```
