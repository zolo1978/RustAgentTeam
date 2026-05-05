// lib/ipc.ts — 类型安全 invoke 封装，内置重试和超时
import { invoke } from '@tauri-apps/api/core';

export class IpcError extends Error {
  constructor(
    public readonly code: string,
    message: string,
    public readonly retryable: boolean,
  ) { super(message); this.name = 'IpcError'; }
}

export async function safeInvoke<T>(
  cmd: string,
  args: Record<string, unknown> = {},
  opts: { retries?: number; timeoutMs?: number } = {},
): Promise<T> {
  const { retries = 2, timeoutMs = 10000 } = opts;
  let lastError: Error | null = null;

  for (let attempt = 0; attempt <= retries; attempt++) {
    try {
      // Tauri invoke 不支持 AbortSignal，用 Promise.race 实现超时
      // Use try-finally to guarantee timer cleanup in both resolve and reject paths.
      let timer!: ReturnType<typeof setTimeout>;
      try {
        const result = await Promise.race([
          invoke<T>(cmd, args),
          new Promise<never>((_, reject) => {
            timer = setTimeout(() => reject(new Error(`IPC ${cmd} timed out after ${timeoutMs}ms`)), timeoutMs);
          }),
        ]);
        return result;
      } finally {
        clearTimeout(timer);
      }
    } catch (err) {
      lastError = err instanceof Error ? err : new Error(String(err));
      if (attempt < retries && isTransient(lastError)) {
        await delay(Math.pow(2, attempt) * 500);    // 指数退避
        continue;
      }
    }
  }
  throw new IpcError(
    'IPC_FAILED',
    toUserMessage(lastError!),
    isTransient(lastError!),
  );
}

function isTransient(err: Error): boolean {
  return /timeout|network|busy/i.test(err.message);
}

function toUserMessage(err: Error): string {
  if (/permission denied/i.test(err.message)) return '权限不足，请检查设置';
  if (/not found/i.test(err.message)) return '请求的数据不存在';
  return '操作失败，请稍后重试';
}

function delay(ms: number) { return new Promise(r => setTimeout(r, ms)); }
