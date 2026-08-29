import { invoke, isTauri } from '$lib/invoke';
import { invalidatePortalResourceCache } from '$lib/features/schedule/portal-cache';
import { connectivity } from '$lib/state/connectivity.svelte';

/** The answer `restore_session` gives; only its verdict matters here. */
type RestoreResult = {
  status: 'restored' | 'no_credentials' | 'credentials_rejected';
};

export type RecoveryStatus =
  | 'idle'
  | 'running'
  | 'unreachable'
  | 'rejected'
  | 'unavailable'
  | 'exhausted';

/**
 * A session that has just been recovered and expires again within this window
 * is not an expiry, it is a loop: a portal that caps concurrent sessions hands
 * back a session it invalidates on the next read.
 */
const LOOP_WINDOW_MS = 30_000;
/** Two refusals in a row are enough to conclude replaying will not help. */
const MAX_LOOP_REFUSALS = 2;

function errorCode(error: unknown): string | null {
  if (typeof error === 'object' && error !== null && 'code' in error) {
    const code = (error as { code: unknown }).code;
    if (typeof code === 'string') return code;
  }
  return null;
}

/**
 * Replays the saved password when the portal drops the session, so an expiry
 * costs a round trip instead of the account.
 *
 * Wiping the keyring used to be the only offered way out of `session_expired`,
 * which asked the reader to retype a password the app already had. The request
 * path now intercepts the expiry, replays the credentials and retries the read;
 * a card only appears once that replay has failed.
 */
class SessionRecovery {
  status = $state<RecoveryStatus>('idle');
  /** Bumped on each successful replay, so surfaces can react to a live session. */
  recoveries = $state(0);

  /**
   * Not reactive on purpose: an effect that read these would re-run itself on
   * its own write. They are bookkeeping for the loop guard, never rendered.
   */
  #inFlight: Promise<boolean> | null = null;
  #lastSuccessAt = 0;
  #loopRefusals = 0;

  get busy(): boolean {
    return this.status === 'running';
  }

  /** Whether offering the reader a retry is honest. */
  get retryable(): boolean {
    return this.status === 'idle' || this.status === 'unreachable' || this.status === 'exhausted';
  }

  /**
   * Resolves true when a session is open again.
   *
   * Concurrent callers share one replay and all get the real answer: each of
   * them has a read of its own to retry, so answering false to everyone but the
   * first would strand four of the five resources the shell loads at once.
   *
   * `prompted` marks the one caller that is a person rather than a request path
   * — the button on the expired card. Only that caller may restart the loop
   * budget once the app has given up, which is what keeps `exhausted` meaning
   * "stops trying on its own" rather than "waits for the next tab change".
   */
  recover({ prompted = false } = {}): Promise<boolean> {
    if (this.#inFlight) return this.#inFlight;

    if (!isTauri()) {
      // No backend to replay against, and no amount of waiting adds one.
      this.status = 'unavailable';
      return Promise.resolve(false);
    }

    if (!connectivity.online) {
      // An attempt made with no network path says nothing about the saved
      // password, so it must not spend the loop budget either.
      this.status = 'unreachable';
      return Promise.resolve(false);
    }

    if (this.status === 'exhausted' && !prompted) {
      // Given up already, and nothing new has happened. Reads keep arriving
      // whenever the reader changes tab, and each one would otherwise restart
      // the budget below and cost the portal another sign-in — the exact
      // hammering the guard exists to stop.
      return Promise.resolve(false);
    }

    if (this.status === 'exhausted') {
      // A deliberate tap on the card. A tap is not a loop: the reader has seen
      // the app give up and asked for one more go, so the budget starts over.
      this.#loopRefusals = 0;
      this.#lastSuccessAt = 0;
    } else if (this.#lastSuccessAt !== 0 && Date.now() - this.#lastSuccessAt < LOOP_WINDOW_MS) {
      this.#loopRefusals += 1;
      this.status = this.#loopRefusals >= MAX_LOOP_REFUSALS ? 'exhausted' : 'unreachable';
      return Promise.resolve(false);
    }

    // An attempt that actually reaches the portal proves the previous refusals
    // were not a loop.
    this.#loopRefusals = 0;

    this.#inFlight = this.#replay().finally(() => {
      this.#inFlight = null;
    });
    return this.#inFlight;
  }

  async #replay(): Promise<boolean> {
    this.status = 'running';

    try {
      const result = await invoke<RestoreResult>('restore_session');

      if (result.status === 'restored') {
        this.#lastSuccessAt = Date.now();
        // Everything in the memory tier was read through the dead session.
        invalidatePortalResourceCache();
        this.recoveries += 1;
        this.status = 'idle';
        return true;
      }

      // `credentials_rejected` is the portal refusing the saved password;
      // `no_credentials` means there was never anything to replay. Neither is
      // worth retrying without the reader typing something.
      this.status = result.status === 'credentials_rejected' ? 'rejected' : 'unavailable';
      return false;
    } catch (error) {
      // A keyring that cannot be read is terminal; anything else is the portal
      // or the network, which the next attempt may well get past.
      this.status = errorCode(error) === 'credential_store' ? 'unavailable' : 'unreachable';
      return false;
    }
  }

  /** A real sign-in happened: every earlier verdict and budget is now stale. */
  reset(): void {
    this.status = 'idle';
    this.recoveries = 0;
    this.#lastSuccessAt = 0;
    this.#loopRefusals = 0;
  }
}

export const sessionRecovery = new SessionRecovery();
