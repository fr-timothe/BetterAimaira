import { invoke, isTauri } from '$lib/invoke';

/**
 * A right the reader grants by hand. Android is the only platform that has any;
 * everywhere else the list comes back empty and the step that renders it is
 * skipped.
 */
export type PermissionKind = 'installPackages';

export type PermissionState = {
  kind: PermissionKind;
  granted: boolean;
  /** Whether the app can open the screen that grants it. */
  requestable: boolean;
};

export type PermissionErrorCode =
  | 'permission_check_failed'
  | 'permission_request_failed'
  | 'permission_screen_unavailable'
  | 'unknown';

const KNOWN_CODES = new Set<PermissionErrorCode>([
  'permission_check_failed',
  'permission_request_failed',
  'permission_screen_unavailable',
]);

export function parsePermissionError(error: unknown): PermissionErrorCode {
  const code = (error as { code?: string } | null)?.code;
  return code && KNOWN_CODES.has(code as PermissionErrorCode)
    ? (code as PermissionErrorCode)
    : 'unknown';
}

/** The browser preview has no Rust side, so it has no rights to report. */
export async function permissionStates(): Promise<PermissionState[]> {
  if (!isTauri()) return [];
  return invoke<PermissionState[]>('permission_states');
}

/**
 * Opens the system screen for `kind`. It returns as soon as that screen is up,
 * long before the reader has decided anything, so the caller re-reads the state
 * when the app comes back to the foreground.
 */
export function requestPermission(kind: PermissionKind): Promise<void> {
  return invoke<void>('request_permission', { kind });
}
