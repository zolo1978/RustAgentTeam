# 状态管理 — Zustand 桌面端模式

## 概述

Tauri 桌面端状态管理使用 Zustand，轻量、TypeScript 友好、无 Provider 嵌套。

## Store 拆分策略

按职责拆分为 3 个独立 Store：

### 1. uiStore — UI 状态

```typescript
interface UIStore {
  theme: 'dark' | 'light';
  sidebarOpen: boolean;
  toasts: Toast[];
  toggleTheme: () => void;
  toggleSidebar: () => void;
  addToast: (toast: Toast) => void;
  removeToast: (id: string) => void;
}
```

### 2. clipStore — 剪贴板数据

```typescript
interface ClipStore {
  clips: ClipSummary[];
  selectedId: string | null;
  query: string;
  loading: boolean;
  error: string | null;
  setClips: (clips: ClipSummary[]) => void;
  select: (id: string | null) => void;
  search: (query: string) => void;
  setError: (error: string | null) => void;
  setLoading: (loading: boolean) => void;
}
```

### 3. prefStore — 用户偏好

```typescript
interface PrefStore {
  shortcut: string;
  maxHistory: number;
  autoStart: boolean;
  setShortcut: (key: string) => void;
  setMaxHistory: (count: number) => void;
}
```

## IPC 数据流

```
Rust 后端 → IPC invoke → Store action → React re-render

用户操作 → React event → IPC invoke → Rust 处理
                                        ↓
                                   IPC event → Store update → React re-render
```

### BAD — 组件内直接 IPC

```tsx
function ClipList() {
  const [clips, setClips] = useState([]);
  useEffect(() => {
    listClips({ limit: 50, offset: 0 }).then(setClips); // 每个组件独立请求
  }, []);
}
```

### GOOD — Store 统一管理

```tsx
// store.ts — 统一数据源
export const useClipStore = create<ClipStore>((set) => ({
  clips: [],
  loading: false,
  error: null,
  setClips: (clips) => set({ clips }),
  loadClips: async () => {
    set({ loading: true, error: null });
    try {
      const clips = await listClips({ limit: 50, offset: 0 });
      set({ clips, loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },
}));

// 组件消费 Store
function ClipList() {
  const clips = useClipStore((s) => s.clips);
  const loading = useClipStore((s) => s.loading);
  // 单一数据源，不重复请求
}
```

## 事件监听模式

```typescript
// 监听 Rust 侧事件，更新 Store
import { listen } from '@tauri-apps/api/event';

export function setupEventListeners() {
  listen<ClipSummary>('clip-created', () => {
    useClipStore.getState().loadClips();
  });
  listen<string>('clip-deleted', () => {
    useClipStore.getState().loadClips();
  });
}
```

## 持久化

```typescript
import { create } from 'zustand';
import { persist } from 'zustand/middleware';

export const usePrefStore = create<PrefStore>()(
  persist(
    (set) => ({
      shortcut: 'CommandOrControl+Shift+V',
      maxHistory: 1000,
      setShortcut: (shortcut) => set({ shortcut }),
    }),
    { name: 'clipvault-prefs' } // localStorage key
  )
);
```
