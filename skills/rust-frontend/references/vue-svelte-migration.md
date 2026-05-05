# Vue / Svelte 适配指南

React 的所有模式都可以用 Vue Composition API 或 Svelte 等价实现。核心差异仅在响应式系统。

## IPC 调用（框架无关）

`safeInvoke` 直接复用，无需修改：

```typescript
// 三个框架完全相同
import { safeInvoke } from './safe-invoke';
const data = await safeInvoke<User[]>('get_users');
```

## 表单 hook → Vue composable

```typescript
// composables/useForm.ts
import { ref, reactive, computed } from 'vue';

export function useForm<T extends Record<string, unknown>>(options: {
  initialValues: T;
  validate: (values: T) => Partial<Record<keyof T, string>>;
  onSubmit: (values: T) => Promise<void>;
}) {
  const { initialValues, validate, onSubmit } = options;
  const values = reactive({ ...initialValues }) as T;
  const errors = ref<Partial<Record<keyof T, string>>>({});
  const isSubmitting = ref(false);
  const serverError = ref<string | null>(null);

  const handleChange = (field: keyof T, value: T[keyof T]) => {
    (values as any)[field] = value;
    const { [field]: _, ...rest } = errors.value;
    errors.value = rest as typeof errors.value;
  };

  const handleSubmit = async (e?: Event) => {
    e?.preventDefault();
    const errs = validate(values);
    if (Object.keys(errs).length > 0) { errors.value = errs; return; }
    isSubmitting.value = true;
    serverError.value = null;
    try { await onSubmit(values); }
    catch (err) {
      isSubmitting.value = false;
      serverError.value = err instanceof Error ? err.message : '操作失败';
    }
  };

  return { values, errors, isSubmitting, serverError, handleChange, handleSubmit };
}
```

## 表单 hook → Svelte

```typescript
// lib/useForm.svelte.ts
import { writable, derived } from 'svelte/store';

export function useForm<T extends Record<string, unknown>>(options: {
  initialValues: T;
  validate: (values: T) => Partial<Record<keyof T, string>>;
  onSubmit: (values: T) => Promise<void>;
}) {
  const { initialValues, validate, onSubmit } = options;
  const values = writable({ ...initialValues });
  const errors = writable<Partial<Record<keyof T, string>>>({});
  const isSubmitting = writable(false);
  const serverError = writable<string | null>(null);

  const handleChange = (field: keyof T, value: T[keyof T]) => {
    values.update(v => ({ ...v, [field]: value }));
    errors.update(e => {
      const { [field]: _, ...rest } = e;
      return rest as typeof e;
    });
  };

  const handleSubmit = async (e?: Event) => {
    e?.preventDefault();
    let currentValues: T;
    values.subscribe(v => currentValues = v)();
    const errs = validate(currentValues!);
    if (Object.keys(errs).length > 0) { errors.set(errs); return; }
    isSubmitting.set(true);
    serverError.set(null);
    try { await onSubmit(currentValues!); }
    catch (err) {
      isSubmitting.set(false);
      serverError.set(err instanceof Error ? err.message : '操作失败');
    }
  };

  return { values, errors, isSubmitting, serverError, handleChange, handleSubmit };
}
```

## 事件监听

```typescript
// Vue
import { onUnmounted } from 'vue';
import { listen } from '@tauri-apps/api/event';

export function useTauriEvent<T>(event: string, handler: (payload: T) => void) {
  let unlisten: (() => void) | null = null;
  listen<T>(event, (e) => handler(e.payload)).then(fn => unlisten = fn);
  onUnmounted(() => unlisten?.());
}

// Svelte
import { onMount } from 'svelte';
import { listen } from '@tauri-apps/api/event';

export function useTauriEvent<T>(event: string, handler: (payload: T) => void) {
  let unlisten: (() => void) | null = null;
  onMount(() => {
    listen<T>(event, (e) => handler(e.payload)).then(fn => unlisten = fn);
    return () => unlisten?.();
  });
}
```

## 状态管理对照表

| React | Vue | Svelte |
|-------|-----|--------|
| `useState` | `ref` | `writable` |
| `useReducer` | `reactive` + method | `writable` + derived |
| Zustand | Pinia | Svelte store |
| `useEffect(fn, [])` | `onMounted` | `onMount` |
| `useEffect` cleanup | `onUnmounted` | `onMount` return |
| Context | Provide/Inject | `setContext`/`getContext` |
