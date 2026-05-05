# Tauri 文件对话框

## 基本用法

```typescript
import { open, save } from '@tauri-apps/plugin-dialog';

// 选择单个文件
const filePath = await open({
  multiple: false,
  filters: [{ name: 'Images', extensions: ['png', 'jpg', 'webp'] }],
});
// filePath: string | null（取消返回 null）

// 选择多个文件
const paths = await open({
  multiple: true,
  filters: [{ name: 'Documents', extensions: ['pdf', 'docx'] }],
});
// paths: string[] | null

// 选择目录
const dirPath = await open({ directory: true });

// 保存文件
const savePath = await save({
  defaultPath: 'untitled.json',
  filters: [{ name: 'JSON', extensions: ['json'] }],
});
```

## 与文件系统配合

```typescript
import { open } from '@tauri-apps/plugin-dialog';
import { readTextFile, writeTextFile } from '@tauri-apps/plugin-fs';

async function openFile() {
  const path = await open({
    filters: [{ name: 'Text', extensions: ['txt', 'md'] }],
  });
  if (!path) return null;
  const content = await readTextFile(path);
  return { path, content };
}

async function saveFile(content: string, defaultName: string) {
  const path = await save({
    defaultPath: defaultName,
    filters: [{ name: 'Text', extensions: ['txt'] }],
  });
  if (!path) return false;
  await writeTextFile(path, content);
  return true;
}
```

## Capabilities 配置

对话框需要在 `capabilities/*.json` 中声明权限：

```json
{
  "permissions": [
    "dialog:allow-open",
    "dialog:allow-save"
  ]
}
```

## 移动端差异

| 功能 | 桌面 | 移动端 |
|------|------|--------|
| 文件选择 | `open()` 系统对话框 | 系统 Picker（沙盒） |
| 目录选择 | `open({ directory: true })` | 不适用 |
| 保存 | `save()` 系统对话框 | 用 `writeFile` + 系统 Picker |
| 文件过滤 | `filters` 支持 | 有限支持 |

## 常见模式

### 最近文件列表

```typescript
import { Store } from '@tauri-apps/plugin-store';

const store = await Store.load('preferences.json');

async function openRecentFile() {
  const recent: string[] = await store.get('recentFiles') ?? [];
  // 展示最近文件列表给用户选择，或直接打开第一个
  if (recent.length > 0) {
    const content = await readTextFile(recent[0]);
    return content;
  }
}

async function addToRecent(path: string) {
  const recent: string[] = await store.get('recentFiles') ?? [];
  const updated = [path, ...recent.filter(p => p !== path)].slice(0, 10);
  await store.set('recentFiles', updated);
}
```
