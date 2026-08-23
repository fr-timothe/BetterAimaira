import { connectivity } from '$lib/state/connectivity.svelte';
import {
  checkForUpdate,
  defaultUpdateChannel,
  installUpdate,
  isUpdateChannel,
  onDownloadFinished,
  onDownloadProgress,
  onInstallStatus,
  parseUpdateError,
  updatesSupported,
  type DownloadProgress,
  type UpdateChannel,
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
const CHANNEL_KEY = 'betteraimaira.update.channel';

function readLastCheck(): number {
  if (typeof localStorage === 'undefined') return 0;
  const stored = Number.parseInt(localStorage.getItem(LAST_CHECK_KEY) ?? '', 10);
  return Number.isFinite(stored) ? stored : 0;
}

function writeLastCheck(at: number): void {
  if (typeof localStorage === 'undefined') return;
  localStorage.setItem(LAST_CHECK_KEY, String(at));
}

function clearLastCheck(): void {
  if (typeof localStorage === 'undefined') return;
  localStorage.removeItem(LAST_CHECK_KEY);
}

function readStoredChannel(): UpdateChannel | null {
  if (typeof localStorage === 'undefined') return null;
  const stored = localStorage.getItem(CHANNEL_KEY);
  return isUpdateChannel(stored) ? stored : null;
}

function writeStoredChannel(channel: UpdateChannel): void {
  if (typeof localStorage === 'undefined') return;
  localStorage.setItem(CHANNEL_KEY, channel);
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
  /** `null` until the stored choice or the build's own channel is resolved. */
  channel = $state<UpdateChannel | null>(null);
  /** The system installer's own wording, when it failed and gave one. */
  installMessage = $state<string | null>(null);

  #inFlight: Promise<void> | null = null;
  #channelReady: Promise<UpdateChannel> | null = null;
  #listening = false;

  get available(): boolean {
    return this.info?.available === true;
  }

  /**
   * The user's choice if there is one, otherwise the channel this build came
   * from: a version carrying a prerelease suffix has to watch beta, since the
   * newest stable is by definition older than the beta already installed.
   */
  async resolveChannel(): Promise<UpdateChannel> {
    if (this.channel) return this.channel;
    if (!this.#channelReady) {
      this.#channelReady = (async () => {
        const stored = readStoredChannel();
        // Falling back to beta rather than stable: only a broken Rust side can
        // reach this, and beta is the superset that never strands a build.
        const resolved =
          stored ?? (await defaultUpdateChannel().catch((): UpdateChannel => 'beta'));
        this.channel = resolved;
        return resolved;
      })();
    }
    return this.#channelReady;
  }

  /** Switching channel invalidates the previous answer and its throttle. */
  async setChannel(channel: UpdateChannel): Promise<void> {
    if (this.channel === channel) return;
    // Set first, so the control moves under the finger instead of waiting on a
    // request.
    this.channel = channel;
    this.#channelReady = Promise.resolve(channel);
    writeStoredChannel(channel);

    // `check` hands back the in-flight promise rather than starting a second
    // request, so a switch during the startup check would be answered by the
    // call already running and the card would never move. Drain it before
    // clearing, or its late write lands on top of the cleared state.
    await this.#inFlight?.catch(() => {});

    this.info = null;
    this.progress = null;
    this.installMessage = null;
    this.errorCode = null;
    this.status = 'idle';
    this.lastCheckedAt = null;
    clearLastCheck();

    await this.check();
  }

  /** Runs at app start, skips a recent check, and never surfaces an error. */
  async checkOnStart(): Promise<void> {
    if (!updatesSupported()) return;
    await this.resolveChannel();
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
      const info = await checkForUpdate(await this.resolveChannel());
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
    this.installMessage = null;
    await this.#listenToInstall();

    try {
      const outcome = await installUpdate(await this.resolveChannel());
      this.status = outcome.permissionRequired ? 'permissionRequired' : 'handedOff';
    } catch (error) {
      this.errorCode = parseUpdateError(error);
      this.status = 'error';
    }
  }

  async #listenToInstall(): Promise<void> {
    if (this.#listening) return;
    this.#listening = true;
    await onDownloadProgress((progress) => {
      this.progress = progress;
    });
    await onDownloadFinished(() => {
      this.progress = null;
    });
    // Android only: the package installer refuses or fails long after the
    // command returned, and until this listener existed that verdict was lost
    // and the card kept claiming the system had taken over.
    await onInstallStatus((status) => {
      this.progress = null;
      if (status.succeeded) return;
      this.installMessage = status.message;
      this.errorCode = 'update_install_failed';
      this.status = 'error';
    });
  }
}

export const updates = new Updates();
