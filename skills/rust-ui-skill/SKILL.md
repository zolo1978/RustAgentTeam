---
name: rust-ui
description: 'Tauri v2 桌面应用 UI 设计和前端开发 — shadcn/ui 组件、Design Token、状态管理、桌面端交互'
type: reference
---

# Tauri v2 桌面应用 UI 设计

## Quick Start

遇到 UI/前端设计问题时，从匹配的 Section 开始。每个 Section 都是独立的决策树。

## 适用范围

**适用于：** Tauri v2 桌面应用的前端 UI 设计、组件选型、状态管理、桌面端交互。
**不适用于：** 后端 Rust 代码（见 `rust-backend`）、架构设计（见 `rust-arch`）。
**目标：** 输出可执行的设计规格，不是直接写代码。

## 1. 组件选型

**触发信号：** 新增 UI 组件、页面重构

**决策树：**

```
需要 UI 组件
  ├─ 基础交互组件
  │   ├─ 按钮 → Button (shadcn)
  │   ├─ 输入框 → Input (shadcn)
  │   ├─ 下拉选择 → Select (shadcn)
  │   └─ 对话框 → Dialog (shadcn)
  ├─ 数据展示组件
  │   ├─ 列表 → VirtualList (自定义，大数据量)
  │   ├─ 卡片 → Card (shadcn)
  │   └─ 表格 → 不推荐（桌面小窗口场景）
  ├─ 反馈组件
  │   ├─ Toast → Toast (shadcn)
  │   ├─ 加载 → Skeleton (shadcn)
  │   └─ 空状态 → 自定义插画 + 文案
  └─ 桌面端特殊组件
      ├─ 全局搜索 → 自定义 Command Palette
      ├─ 上下文菜单 → ContextMenu (shadcn)
      └─ 拖拽排序 → dnd-kit
```

**组件无障碍要求：**
- 所有交互元素有 `aria-label`
- 支持键盘导航（Tab/Enter/Escape/Arrow）
- 焦点可见（`focus:ring-2 focus:ring-blue-500`）
- 屏幕阅读器兼容（`role`、`aria-selected`）

## 2. Design Token 体系

**触发信号：** 新增样式、主题切换

**决策树：**

```
需要样式
  ├─ 颜色
  │   ├─ 主色 → blue-500 / blue-600（交互）
  │   ├─ 危险色 → red-500 / red-600（删除、警告）
  │   ├─ 成功色 → green-500（确认）
  │   ├─ 中性色 → gray-50~900（背景、文字、边框）
  │   └─ 主题切换 → dark: 前缀（Tailwind 内置）
  ├─ 间距（4px 基数）
  │   ├─ 1 → 4px（紧凑）
  │   ├─ 2 → 8px（默认）
  │   ├─ 3 → 12px（中等）
  │   ├─ 4 → 16px（宽松）
  │   ├─ 6 → 24px（大间距）
  │   └─ 8 → 32px（分区）
  └─ 字体
      ├─ xs → 12px（辅助信息）
      ├─ sm → 14px（正文）
      ├─ base → 16px（默认）
      └─ lg → 18px（标题）
```

**BAD vs GOOD：**

```tsx
// BAD — emoji 当按钮
<button>☀️</button>

// GOOD — icon library + aria-label
import { Sun, Moon } from 'lucide-react';
<button aria-label="切换到亮色模式">
  <Sun className="h-4 w-4" />
</button>
```

## 3. 状态管理

**触发信号：** 新增全局状态、跨组件共享

**决策树：**

```
数据需要跨组件共享？
  ├─ 否 → 组件内 useState
  ├─ 是 → 共享范围
      ├─ 父子 → Props 传递
      ├─ 兄弟 → 状态提升 + Props
      └─ 全局 → Zustand Store
          ├─ UI 状态 → uiStore（主题、侧边栏、模态框）
          ├─ 剪贴板数据 → clipStore（列表、选中、搜索）
          └─ 用户偏好 → prefStore（快捷键、过滤规则）

IPC 数据流（单向）：
  Rust → IPC → Zustand Store → React Component
  用户操作 → React Event → IPC → Rust → IPC → Store 更新
```

**Zustand Store 设计：**

```typescript
import { create } from 'zustand';

interface ClipStore {
  clips: ClipSummary[];
  selectedId: string | null;
  query: string;
  setClips: (clips: ClipSummary[]) => void;
  select: (id: string | null) => void;
  search: (query: string) => void;
}

export const useClipStore = create<ClipStore>((set) => ({
  clips: [],
  selectedId: null,
  query: '',
  setClips: (clips) => set({ clips }),
  select: (id) => set({ selectedId: id }),
  search: (query) => set({ query }),
}));
```

## 4. 桌面端交互

**触发信号：** 涉及平台特定交互

**决策树：**

```
桌面端交互需求
  ├─ 窗口管理
  │   ├─ 无边框 → decorations: false + 自定义拖拽区域 data-tauri-drag-region
  │   ├─ 置顶 → alwaysOnTop: true
  │   └─ 显示/隐藏 → Window.show() / Window.hide()
  ├─ 全局快捷键
  │   ├─ 注册 → register('CommandOrControl+Shift+V', handler)
  │   ├─ 冲突 → 检测已占用 + 用户自定义
  │   └─ 响应 → 切换窗口显示/隐藏
  ├─ 系统托盘
  │   ├─ 菜单 → 显示窗口 / 偏好设置 / 退出
  │   └─ 点击 → macOS: show, Windows: menu
  └─ 原生操作
      ├─ 复制 → 通过 Rust arboard
      ├─ 粘贴 → 通过 Rust enigo
      └─ 文件选择 → tauri dialog（如需要）
```

## 5. 响应式与自适应

**触发信号：** 布局调整、窗口大小变更

**决策树：**

```
窗口尺寸（400×600 默认）
  ├─ 紧凑模式（< 400px 宽）
  │   ├─ 隐藏侧边栏
  │   ├─ 图标替代文字
  │   └─ 单列列表
  ├─ 标准模式（400-600px）
  │   ├─ 搜索栏 + 列表
  │   └─ 底部操作栏
  └─ 展开模式（> 600px）
      ├─ 搜索栏 + 列表 + 详情面板
      └─ 三栏布局
```

## 6. 动效与反馈

**触发信号：** 用户体验优化

**决策树：**

```
需要反馈
  ├─ 加载状态
  │   ├─ 首次加载 → Skeleton 骨架屏
  │   ├─ 操作中 → Spinner (16px)
  │   └─ 后台刷新 → 顶部进度条
  ├─ 操作反馈
  │   ├─ 成功 → Toast (2s 自动消失)
  │   ├─ 失败 → Toast (红色，需手动关闭)
  │   └─ 危险操作 → 确认对话框
  └─ 过渡动画
      ├─ 列表项增删 → fade + slide (150ms)
      ├─ 页面切换 → fade (200ms)
      └─ 模态框 → scale + fade (150ms)
```

## 参考资料

| 文件 | 内容 |
|------|------|
| `references/shadcn-components.md` | shadcn/ui 组件库桌面端使用指南 |
| `references/design-tokens.md` | Design Token 具体值和 Tailwind 映射 |
| `references/state-management.md` | Zustand store 设计模式 |
| `references/desktop-interactions.md` | 桌面端交互实现细节 |

## 模板

| 文件 | 用途 |
|------|------|
| `templates/ui-spec-template.md` | UI 设计规格输出模板（组件/Token/状态/交互四节） |
