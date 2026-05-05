// lib/theme.ts — 完整主题管理
import { Store } from '@tauri-apps/plugin-store';

type Theme = 'light' | 'dark' | 'system';

let _store: Store | null = null;
async function getStore() {
  if (!_store) _store = await Store.load('preferences.json');
  return _store;
}

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
    const current = store.getSync<Theme>('theme') ?? 'system';
    if (current === 'system') applyTheme('system');  // 仅 system 模式跟随系统
  });
}

export async function setTheme(theme: Theme) {
  const store = await getStore();
  await store.set('theme', theme);
  applyTheme(theme);
}
