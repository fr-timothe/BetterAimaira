import { isDemoMode } from '$lib/dev-demo';
import { invoke, isTauri } from '$lib/invoke';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

/**
 * How the pending update reaches the device. The label on the action button has
 * to match: desktop installs and restarts, Android opens the system installer,
 * iOS can only send the user to AltStore.
 */
export type UpdateDelivery = 'inApp' | 'androidPackage' | 'altStore';

/**
 * Which release stream to follow. `stable` carries only releases tagged without
 * a prerelease suffix; `beta` carries everything, because a finished stable
 * release supersedes the betas that led to it.
 */
export type UpdateChannel = 'stable' | 'beta';

export const UPDATE_CHANNELS: readonly UpdateChannel[] = ['stable', 'beta'];

export function isUpdateChannel(value: unknown): value is UpdateChannel {
  return value === 'stable' || value === 'beta';
}

export type UpdateInfo = {
  available: boolean;
  currentVersion: string;
  latestVersion: string | null;
  notes: string | null;
  publishedAt: string | null;
  delivery: UpdateDelivery;
  channel: UpdateChannel;
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

/**
 * Android only: the system package installer answers on a broadcast well after
 * the install command returned, so its verdict arrives as an event.
 */
export type InstallStatus = {
  succeeded: boolean;
  message: string | null;
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
  // Demo mode answers the update commands itself, so the whole update UI can be
  // driven in a plain browser with no Rust side behind it.
  return isTauri() || isDemoMode();
}

/** The install events come from Rust; in demo mode nobody ever emits them. */
function listenIfNative<T>(
  event: string,
  handler: (payload: T) => void,
): Promise<UnlistenFn> {
  if (!isTauri()) return Promise.resolve(() => {});
  return listen<T>(event, (received) => handler(received.payload));
}

export function checkForUpdate(channel: UpdateChannel): Promise<UpdateInfo> {
  return invoke<UpdateInfo>('check_for_update', { channel });
}

export function installUpdate(channel: UpdateChannel): Promise<InstallOutcome> {
  return invoke<InstallOutcome>('install_update', { channel });
}

/** The channel this build belongs to, before the user has chosen one. */
export function defaultUpdateChannel(): Promise<UpdateChannel> {
  return invoke<UpdateChannel>('default_update_channel');
}

export function onDownloadProgress(
  handler: (progress: DownloadProgress) => void,
): Promise<UnlistenFn> {
  return listenIfNative<DownloadProgress>('update://download-progress', handler);
}

export function onDownloadFinished(handler: () => void): Promise<UnlistenFn> {
  return listenIfNative('update://downloaded', () => handler());
}

export function onInstallStatus(handler: (status: InstallStatus) => void): Promise<UnlistenFn> {
  return listenIfNative<InstallStatus>('update://install-status', handler);
}
