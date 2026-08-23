import { connectivity } from '$lib/state/connectivity.svelte';
import {
  checkForUpdate,
  installUpdate,
  onDownloadFinished,
  onDownloadProgress,
  parseUpdateError,
  updatesSupported,
  type DownloadProgress,
  type UpdateErrorCode,
  type UpdateInfo,
} from './update-service';

export type UpdateStatus =
  | 'idle'
  | 'checking'
  | 'upToDate'
  | 'available'
  | 'installing'
  | 'handedOff'
  | 'permissionRequired'
  | 'error';

/** One automatic check per window: more would be noise, not safety. */
const AUTO_CHECK_INTERVAL_MS = 6 * 60 * 60 * 1000;
const LAST_CHECK_KEY = 'betteraimaira.update.lastCheck';

function readLastCheck(): number {
  if (typeof localStorage === 'undefined') return 0;
  const stored = Number.parseInt(localStorage.getItem(LAST_CHECK_KEY) ?? '', 10);
  return Number.isFinite(stored) ? stored : 0;
}

function writeLastCheck(at: number): void {
  if (typeof localStorage === 'undefined') return;
  localStorage.setItem(LAST_CHECK_KEY, String(at));
}

/**
 * Update state, shared so the automatic startup check and the card in More read
 * the same thing.
 *
 * A failed check is deliberately quiet when it was not asked for: an app that
 * cannot reach GitHub is still a working app, and a startup error banner about
 * updates would bury the day's schedule.
 */
class Updates {
  status = $state<UpdateStatus>('idle');
  info = $state<UpdateInfo | null>(null);
  errorCode = $state<UpdateErrorCode | null>(null);
  progress = $state<DownloadProgress | null>(null);
  lastCheckedAt = $state<number | null>(null);

  #inFlight: Promise<void> | null = null;
  #listening = false;

  get available(): boolean {
    return this.info?.available === true;
  }

  /** Runs at app start, skips a recent check, and never surfaces an error. */
  async checkOnStart(): Promise<void> {
    if (!updatesSupported()) return;
    const previous = readLastCheck();
    if (previous && Date.now() - previous < AUTO_CHECK_INTERVAL_MS) {
      this.lastCheckedAt = previous;
      return;
    }
    await this.check({ silent: true });
  }

  async check({ silent = false }: { silent?: boolean } = {}): Promise<void> {
    if (!updatesSupported()) return;
    if (this.#inFlight) return this.#inFlight;
    if (!connectivity.online) {
      if (!silent) {
        this.status = 'error';
        this.errorCode = 'update_check_failed';
      }
      return;
    }

    this.#inFlight = this.#runCheck(silent).finally(() => {
      this.#inFlight = null;
    });
    return this.#inFlight;
  }

  async #runCheck(silent: boolean): Promise<void> {
    if (!silent) this.status = 'checking';
    this.errorCode = null;

    try {
      const info = await checkForUpdate();
      this.info = info;
      this.status = info.available ? 'available' : 'upToDate';
      const now = Date.now();
      this.lastCheckedAt = now;
      writeLastCheck(now);
    } catch (error) {
      const code = parseUpdateError(error);
      if (silent) {
        // Keep the previous state: a background check that failed says nothing
        // about the installed version.
        this.status = this.info?.available ? 'available' : 'idle';
        return;
      }
      this.errorCode = code;
      this.status = 'error';
    }
  }

  /**
   * Starts the install. Desktop never returns from this: the process restarts
   * into the new version. Android returns once the system installer has the
   * APK, iOS once AltStore is open.
   */
  async install(): Promise<void> {
    if (!updatesSupported() || !this.available) return;

    this.status = 'installing';
    this.errorCode = null;
    this.progress = null;
    await this.#listenToDownload();

    try {
      const outcome = await installUpdate();
      this.status = outcome.permissionRequired ? 'permissionRequired' : 'handedOff';
    } catch (error) {
      this.errorCode = parseUpdateError(error);
      this.status = 'error';
    }
  }

  async #listenToDownload(): Promise<void> {
    if (this.#listening) return;
    this.#listening = true;
    await onDownloadProgress((progress) => {
      this.progress = progress;
    });
    await onDownloadFinished(() => {
      this.progress = null;
    });
  }
}

export const updates = new Updates();
