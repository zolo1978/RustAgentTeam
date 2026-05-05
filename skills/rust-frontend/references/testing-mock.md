# Tauri invoke 测试 Mock（Vitest）

## 核心模式

使用 `vi.mock` 替换 `@tauri-apps/api/core` 的 `invoke`，在测试中完全控制 IPC 返回值。

- **成功用例**：`mockResolvedValueOnce` 返回预期数据，验证 invoke 被正确调用。
- **失败用例**：`mockRejectedValueOnce` 模拟错误，验证错误 UI 正确展示。

## 关键设计

1. **vi.mock 必须在 import 之前**：确保模块被替换后再导入组件。
2. **成功 + 失败双用例**：覆盖正常路径和异常路径。
3. **waitFor**：异步状态更新后用 `waitFor` 断言，避免时序问题。
4. **用户友好错误**：验证 UI 展示的是中文提示，非原始英文错误。

```typescript
// tests/components/UserForm.test.tsx
import { describe, it, expect, vi } from 'vitest';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

import { invoke } from '@tauri-apps/api/core';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { UserCreateView } from '../../src/views/UserCreateView';

describe('UserCreateView', () => {
  it('提交表单时调用 invoke 并显示成功', async () => {
    (invoke as ReturnType<typeof vi.fn>).mockResolvedValueOnce({ id: '1' });
    render(<UserCreateView />);
    fireEvent.change(screen.getByLabelText('姓名'), { target: { value: '张三' } });
    fireEvent.change(screen.getByLabelText('邮箱'), { target: { value: 'a@b.c' } });
    fireEvent.click(screen.getByRole('button', { name: '保存' }));
    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith('create_user', {
        name: '张三', email: 'a@b.c',
      });
    });
  });

  it('invoke 失败时显示用户友好错误', async () => {
    (invoke as ReturnType<typeof vi.fn>).mockRejectedValueOnce(new Error('permission denied'));
    render(<UserCreateView />);
    fireEvent.click(screen.getByRole('button', { name: '保存' }));
    await waitFor(() => {
      expect(screen.getByText(/权限不足/)).toBeInTheDocument();
    });
  });
});
```
