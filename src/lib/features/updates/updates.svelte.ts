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
/** The version the last check found waiting, if it found one. */
const PENDING_VERSION_KEY = "betteraimaira.update.pendingVersion";
/** The version whose notice was closed by hand; it never raises one again. */
const NOTICE_DISMISSED_KEY = 'betteraimaira.update.noticeDismissed';

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

function readDismissedNotice(): string | null {
  if (typeof localStorage === 'undefined') return null;
  return localStorage.getItem(NOTICE_DISMISSED_KEY);
}

function writeDismissedNotice(version: string): void {
  if (typeof localStorage === 'undefined') return;
  localStorage.setItem(NOTICE_DISMISSED_KEY, version);
}

function readPendingVersion(): string | null {
  if (typeof localStorage === "undefined") return null;
  return localStorage.getItem(PENDING_VERSION_KEY);
}

function writePendingVersion(version: string | null): void {
  if (typeof localStorage === "undefined") return;
  if (version) localStorage.setItem(PENDING_VERSION_KEY, version);
  else localStorage.removeItem(PENDING_VERSION_KEY);
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
  /**
   * The in-app notice raised when a check finds a new version. It is what the
   * user sees at launch: the badge on More is a mark, not an announcement.
   */
  noticeVisible = $state(false);
  /**
   * Set when the notice is tapped. The update card watches it and scrolls
   * itself into view, so the shell does not have to wait for a lazily loaded
   * view to mount before it can find the card.
   */
  revealRequested = $state(false);

  #dismissedNoticeVersion: string | null = readDismissedNotice();
  /** Announced once per version per run: a re-check is not a second event. */
  #announcedVersion: string | null = null;
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
    this.noticeVisible = false;
    this.#announcedVersion = null;
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
    // An update already known to be waiting is re-checked on every launch: the
    // throttle exists to spare pointless requests, and this one is not
    // pointless. Skipping it would leave the notice unraised and the card empty
    // for six hours after the update landed.
    const pending = readPendingVersion();
    if (previous && Date.now() - previous < AUTO_CHECK_INTERVAL_MS && !pending) {
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
      this.#refreshNotice(info);
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
   * A version already turned down stays turned down; anything newer speaks up
   * again. An update with no version name still raises the notice: not knowing
   * which version it is says nothing about whether it matters.
   */
  #refreshNotice(info: UpdateInfo): void {
    const version = info.available ? (info.latestVersion ?? "unknown") : null;
    writePendingVersion(version);
    if (!version) {
      this.noticeVisible = false;
      return;
    }
    if (version === this.#dismissedNoticeVersion) return;
    // Opening the update card runs a check of its own, and without this the
    // notice the user just tapped would raise itself again behind the card.
    if (version === this.#announcedVersion) return;
    this.#announcedVersion = version;
    this.noticeVisible = true;
  }

  /** Tapped: hand the user to the card that installs it. */
  revealFromNotice(): void {
    this.noticeVisible = false;
    this.revealRequested = true;
  }

  /** Consumed by the card once it has scrolled itself into view. */
  clearReveal(): void {
    this.revealRequested = false;
  }

  /** Closed by hand: this version never announces itself again. */
  dismissNotice(): void {
    this.noticeVisible = false;
    const version = this.info?.latestVersion;
    if (!version) return;
    this.#dismissedNoticeVersion = version;
    writeDismissedNotice(version);
  }

  /**
   * Hidden without a verdict, when the notice times out on its own. The next
   * launch raises it again: an announcement nobody saw was not turned down.
   */
  hideNotice(): void {
    this.noticeVisible = false;
  }

  /**
   * Starts the install. Desktop never returns from this: the process restarts
   * into the new version. Android returns once the system installer has the
   * APK, iOS once AltStore is open.
   */
  async install(): Promise<void> {
    if (!updatesSupported() || !this.available) return;

    this.status = 'installing';
    this.noticeVisible = false;
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
