# UI 设计规格模板

> UI Designer Agent 输出此模板。每个功能/页面一份。

## 1. 组件清单

### [页面名称]

| 组件 | 用途 | 关键 Props | 交互说明 |
|------|------|-----------|---------|
| SearchBar | 搜索输入 | value, onChange, placeholder | 300ms debounce → IPC search |
| ClipList | 剪贴板列表 | clips[], onSelect, onFav, onDelete | 虚拟滚动，键盘导航 |
| ClipItem | 单条记录 | clip, selected, onFav, onDelete | 右键菜单，双击粘贴 |
| ThemeToggle | 主题切换 | dark, onToggle | 切换亮暗色 |
| RefreshBtn | 刷新 | onClick, loading | loading 时显示 spinner |

### 组件层级

```
ClipVaultView
  ├── Header (data-tauri-drag-region)
  │   ├── Title
  │   ├── SearchBar
  │   ├── RefreshBtn
  │   └── ThemeToggle
  ├── ErrorAlert (条件渲染)
  ├── LoadingSkeleton (条件渲染)
  └── ClipList
      └── ClipItem × N
          ├── FavButton
          ├── ContentPreview
          ├── MetaInfo
          └── DeleteButton
```

## 2. Design Token

### 颜色

| 元素 | 亮色 | 暗色 |
|------|------|------|
| 背景 | white | gray-900 |
| 卡片 | white | gray-800 |
| 文字 | gray-900 | gray-100 |
| 辅助文字 | gray-500 | gray-400 |
| 边框 | gray-200 | gray-700 |
| 高亮项 | blue-50 | gray-800 |
| 按钮悬停 | gray-100 | gray-800 |
| 危险按钮 | red-100/red-600 | red-900/red-200 |

### 间距

| 区域 | 值 |
|------|---|
| 页面内边距 | 16px (p-4) |
| 列表项内边距 | 12px (py-3 px-4) |
| 搜索框内边距 | 6px 12px (py-1.5 px-3) |
| 列表项间距 | 1px (border-b) |

### 字体

| 元素 | 大小 | 粗细 |
|------|------|------|
| 标题 | text-lg | font-semibold |
| 正文 | text-sm | font-normal |
| 时间戳 | text-xs | font-normal |
| 按钮 | text-sm | font-medium |

## 3. 状态设计

### Store Slice: clipStore

```
State:
  clips: ClipSummary[]
  selectedId: string | null
  query: string
  loading: boolean
  error: string | null

Actions:
  loadClips() → IPC list_clips → set clips
  search(query) → debounce 300ms → IPC search_clips → set clips
  toggleFav(id) → IPC toggle_favorite → update clip in list
  remove(id) → IPC delete_clip → filter from list
  select(id) → set selectedId

Events:
  clip-created → loadClips() (自动刷新)
```

### 状态流转

```
初始 → loadClips() → [loading=true]
                     → IPC 返回 → [loading=false, clips=data]
                     → IPC 失败 → [loading=false, error=msg]

搜索 → search(query) → [loading=true]
                      → IPC 返回 → [loading=false, clips=filtered]
```

## 4. 交互规范

### 键盘操作

| 按键 | 上下文 | 动作 |
|------|--------|------|
| ArrowDown | 列表 | 选中下一项 |
| ArrowUp | 列表 | 选中上一项 |
| Enter | 有选中项 | 粘贴选中项到活动应用 |
| Delete / Backspace | 有选中项 | 删除选中项（需确认） |
| Escape | 搜索有内容 | 清空搜索 |
| Escape | 搜索为空 | 隐藏窗口 |
| Cmd/Ctrl+K | 全局 | 聚焦搜索框 |

### 鼠标操作

| 操作 | 目标 | 动作 |
|------|------|------|
| 单击 | 列表项 | 选中该项 |
| 双击 | 列表项 | 粘贴该项 |
| 右键 | 列表项 | 上下文菜单（复制/删除/收藏） |
| 点击 | 收藏星号 | 切换收藏状态 |
| 点击 | 删除按钮 | 确认后删除 |

### 全局快捷键

| 快捷键 | 动作 |
|--------|------|
| Cmd+Shift+V | 显示/隐藏窗口 |
