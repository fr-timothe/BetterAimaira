import { invoke, isTauri } from '$lib/invoke';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

/**
 * How the pending update reaches the device. The label on the action button has
 * to match: desktop installs and restarts, Android opens the system installer,
 * iOS can only send the user to AltStore.
 */
export type UpdateDelivery = 'inApp' | 'androidPackage' | 'altStore';

export type UpdateInfo = {
  available: boolean;
  currentVersion: string;
  latestVersion: string | null;
  notes: string | null;
  publishedAt: string | null;
  delivery: UpdateDelivery;
  downloadUrl: string | null;
  storeUrl: string | null;
};

export type InstallOutcome = {
  handedOff: boolean;
  permissionRequired: boolean;
};

export type DownloadProgress = {
  downloaded: number;
  total: number | null;
};

export type UpdateErrorCode =
  | 'update_check_failed'
  | 'update_manifest_invalid'
  | 'update_download_failed'
  | 'update_install_failed'
  | 'update_not_available'
  | 'update_store_unavailable'
  | 'unknown';

const KNOWN_CODES = new Set<UpdateErrorCode>([
  'update_check_failed',
  'update_manifest_invalid',
  'update_download_failed',
  'update_install_failed',
  'update_not_available',
  'update_store_unavailable',
]);

/** Commands answer with `{ code }`; anything else is a bug, not a portal state. */
export function parseUpdateError(error: unknown): UpdateErrorCode {
  const code = (error as { code?: string } | null)?.code;
  return code && KNOWN_CODES.has(code as UpdateErrorCode) ? (code as UpdateErrorCode) : 'unknown';
}

/** The browser preview has no Rust side; it must not pretend to check anything. */
export function updatesSupported(): boolean {
  return isTauri();
}

export function checkForUpdate(): Promise<UpdateInfo> {
  return invoke<UpdateInfo>('check_for_update');
}

export function installUpdate(): Promise<InstallOutcome> {
  return invoke<InstallOutcome>('install_update');
}

export function updateFeedBase(): Promise<string> {
  return invoke<string>('update_feed_base');
}

export function onDownloadProgress(
  handler: (progress: DownloadProgress) => void,
): Promise<UnlistenFn> {
  return listen<DownloadProgress>('update://download-progress', (event) => handler(event.payload));
}

export function onDownloadFinished(handler: () => void): Promise<UnlistenFn> {
  return listen('update://downloaded', () => handler());
}
