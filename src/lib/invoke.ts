import { invoke as tauriInvoke, isTauri } from '@tauri-apps/api/core';
import { getDemoResponse, isDemoMode } from '$lib/dev-demo';

export { isTauri };

export async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (isDemoMode()) {
    const demo = getDemoResponse(cmd, args);
    if (demo !== undefined) {
      return demo as T;
    }
  }
  return tauriInvoke<T>(cmd, args);
}
