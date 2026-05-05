# 暗色/浅色模式（CSS 变量 + 系统检测 + 手动切换 + 持久化）

> 代码模板：[templates/theme.ts](../templates/theme.ts)

## 核心模式

- **initTheme**：应用启动时从 Store 读取用户偏好，监听系统 `prefers-color-scheme` 变化。
- **setTheme**：切换主题并持久化到 Tauri Store。
- **CSS 变量**：`:root` 定义浅色变量，`:root.dark` 覆盖为深色值。

## 关键设计

1. **三种模式**：`light` / `dark` / `system`，system 自动跟随 OS 偏好。
2. **持久化**：通过 `@tauri-apps/plugin-store` 存储到 `preferences.json`。
3. **CSS 变量方案**：所有颜色通过 `var(--xxx)` 引用，切换主题只需 toggle `dark` class。
4. **系统监听**：`matchMedia('prefers-color-scheme: dark')` 的 `change` 事件自动跟随。

```typescript
// lib/theme.ts — 完整主题管理
import { Store } from '@tauri-apps/plugin-store';

type Theme = 'light' | 'dark' | 'system';

let _store: Store | null = null;
async function getStore() { if (!_store) _store = await Store.load('preferences.json'); return _store; }

function applyTheme(theme: Theme) {
  const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
  const isDark = theme === 'dark' || (theme === 'system' && prefersDark);
  document.documentElement.classList.toggle('dark', isDark);
}

export async function initTheme() {
  const store = await getStore();
  const saved = await store.get<Theme>('theme') ?? 'system';
  applyTheme(saved);
  window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', () => {
    applyTheme('system');    // 系统切换时自动跟随
  });
}

export async function setTheme(theme: Theme) {
  const store = await getStore();
  await store.set('theme', theme);
  applyTheme(theme);
}
```

```css
/* styles/theme.css — CSS 变量方案 */
:root {
  --bg-primary: #ffffff;
  --text-primary: #111827;
  --border-color: #e5e7eb;
}
:root.dark {
  --bg-primary: #111827;
  --text-primary: #f9fafb;
  --border-color: #374151;
}
body { background: var(--bg-primary); color: var(--text-primary); }
```
