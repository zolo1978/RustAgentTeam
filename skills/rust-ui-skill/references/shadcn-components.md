# shadcn/ui 组件库 — 桌面端使用

## 概述

shadcn/ui 是可组合的 UI 组件集，基于 Radix UI + Tailwind CSS。适合桌面端使用。

## 桌面端常用组件

### 列表场景（ClipVault 核心）

| 组件 | 用途 | 备注 |
|------|------|------|
| `Command` | 搜索+列表组合 | 核心：快速筛选剪贴板记录 |
| `ScrollArea` | 滚动区域 | 替代原生滚动，自定义滚动条 |
| `ContextMenu` | 右键菜单 | 复制/删除/收藏 |
| `Separator` | 分隔线 | 列表项之间 |
| `Badge` | 标签 | 内容类型标记 |

### 交互场景

| 组件 | 用途 | 备注 |
|------|------|------|
| `Button` | 按钮 | variant: default/destructive/ghost/outline |
| `Input` | 输入框 | 搜索框 |
| `Dialog` | 对话框 | 确认删除、设置 |
| `Toast` | 通知 | 操作反馈 |
| `Tooltip` | 提示 | 图标按钮说明 |

### 数据展示

| 组件 | 用途 | 备注 |
|------|------|------|
| `Skeleton` | 骨架屏 | 加载状态 |
| `Card` | 卡片 | 详情展示 |
| `Toggle` | 切换 | 收藏状态 |

## 安装和定制

```bash
npx shadcn-ui@latest init
npx shadcn-ui@latest add button input dialog toast
```

## BAD vs GOOD

### BAD — emoji 和原生 HTML

```tsx
<button onClick={toggle}>☀️</button>
<ul>{items.map(i => <li>{i}</li>)}</ul>
```

### GOOD — icon + shadcn 组件

```tsx
import { Sun } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { ScrollArea } from '@/components/ui/scroll-area';

<Button variant="ghost" size="icon" onClick={toggle} aria-label="切换主题">
  <Sun className="h-4 w-4" />
</Button>
<ScrollArea className="h-[500px]">
  {items.map(i => <ClipItem key={i.id} clip={i} />)}
</ScrollArea>
```

## 图标库选择

**推荐：lucide-react**
- 800+ 图标
- Tree-shakeable（只打包用到的）
- 24px 网格，4px 描边（与 Tailwind 间距系统对齐）
- 一致的视觉风格

```tsx
import { Search, Star, Trash2, RefreshCw, Sun, Moon, Copy, Settings } from 'lucide-react';
```
