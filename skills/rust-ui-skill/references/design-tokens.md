# Design Token — ClipVault 桌面端

## 颜色系统

### 亮色主题

| Token | 值 | 用途 |
|-------|---|------|
| bg-primary | white | 主背景 |
| bg-secondary | gray-50 | 次要背景 |
| text-primary | gray-900 | 主文字 |
| text-secondary | gray-500 | 辅助文字 |
| border | gray-200 | 边框 |
| accent | blue-500 | 交互高亮 |
| danger | red-500 | 删除、危险 |
| success | green-500 | 确认 |

### 暗色主题

| Token | 值 | 用途 |
|-------|---|------|
| bg-primary | gray-900 | 主背景 |
| bg-secondary | gray-800 | 次要背景 |
| text-primary | gray-100 | 主文字 |
| text-secondary | gray-400 | 辅助文字 |
| border | gray-700 | 边框 |
| accent | blue-400 | 交互高亮 |

### Tailwind 用法

```tsx
// 自动适配亮暗色
<div className="bg-white dark:bg-gray-900 text-gray-900 dark:text-gray-100">
  <p className="text-gray-500 dark:text-gray-400">辅助信息</p>
  <button className="bg-blue-500 hover:bg-blue-600 dark:bg-blue-400">
    操作
  </button>
</div>
```

## 间距系统

| Token | 值 | Tailwind | 用途 |
|-------|---|---------|------|
| space-1 | 4px | p-1 m-1 | 图标内边距 |
| space-2 | 8px | p-2 m-2 | 按钮内边距 |
| space-3 | 12px | p-3 m-3 | 输入框内边距 |
| space-4 | 16px | p-4 m-4 | 页面边距 |
| space-6 | 24px | p-6 m-6 | 区块间距 |
| space-8 | 32px | p-8 m-8 | 大区块间距 |

**规则：** 所有间距必须是 4 的倍数。

## 字体系统

| Token | 大小 | 行高 | Tailwind | 用途 |
|-------|------|------|---------|------|
| text-xs | 12px | 16px | text-xs | 时间戳、标签 |
| text-sm | 14px | 20px | text-sm | 正文、列表项 |
| text-base | 16px | 24px | text-base | 默认 |
| text-lg | 18px | 28px | text-lg | 标题 |
| text-xl | 20px | 28px | text-xl | 页面标题 |

**系统字体栈（Tailwind 默认）：**
```css
font-family: ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
```

## 圆角

| Token | 值 | Tailwind | 用途 |
|-------|---|---------|------|
| radius-sm | 4px | rounded-sm | 小元素 |
| radius-md | 6px | rounded-md | 按钮、输入框 |
| radius-lg | 8px | rounded-lg | 卡片、对话框 |

## 阴影

| Token | 值 | 用途 |
|-------|---|------|
| shadow-sm | `0 1px 2px rgba(0,0,0,0.05)` | 列表项 |
| shadow-md | `0 4px 6px rgba(0,0,0,0.1)` | 浮层、下拉 |
| shadow-lg | `0 10px 15px rgba(0,0,0,0.1)` | 对话框 |

## 主题切换实现

```typescript
// lib/theme.ts
export async function setTheme(theme: 'dark' | 'light') {
  const root = document.documentElement;
  root.classList.toggle('dark', theme === 'dark');
  localStorage.setItem('theme', theme);
}

export function getInitialTheme(): 'dark' | 'light' {
  const stored = localStorage.getItem('theme');
  if (stored) return stored as 'dark' | 'light';
  return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
}
```
