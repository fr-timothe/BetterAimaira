import * as m from '$lib/paraglide/messages.js';
import type { Locale } from '$lib/paraglide/runtime.js';
import { announcer } from '$lib/state/announcements.svelte';
import { connectivity } from '$lib/state/connectivity.svelte';
import { loadPortalResource } from './portal-cache';
import { parseResourceError, resourceErrorMessage } from './portal-utils';
import type {
  PortalPage,
  PortalResource,
  PortalResourceErrorCode,
  PortalResourceState,
} from './types';

export type PortalResourceOptions = {
  resource: PortalResource;
  /** The code to report when the failure carries none of its own. */
  fallbackErrorCode: PortalResourceErrorCode;
  /**
   * The section the live region names before every sentence. A function, not a
   * string, because Paraglide messages are not signals: the caller re-reads it
   * through its own locale-touched `$derived`.
   */
  heading: () => string;
  locale: () => Locale;
  /**
   * Whether an expired session replaces data already on screen instead of being
   * flagged on top of it. It is a real difference and not a preference: a
   * surface that offers no way in has to hand the reader the sign-in card, while
   * one reachable from a signed-in shell can keep the stale page and say so.
   */
  expiredReplacesData?: boolean;
};

export type PortalResourceHandle = {
  readonly state: PortalResourceState;
  /** The page currently on screen, or null while loading or in error. */
  readonly page: PortalPage | null;
  /** A refresh is in flight over data that is already showing. */
  readonly refreshing: boolean;
  /** The last refresh failed while data was already on screen. */
  readonly refreshFailed: boolean;
  /** Epoch ms of the last successful read, or null if nothing has landed yet. */
  readonly fetchedAt: number | null;
  load: (force?: boolean) => Promise<void>;
};

/**
 * One portal read, with the three things every view around it kept re-deriving:
 * the request-sequence guard, the `hasData ? refreshFailed : error` policy, and
 * the sentences the shared live region owes a screen reader.
 *
 * Written here once because the copies drifted. A view that re-implemented the
 * catch clause lost the `refreshFailed` flag, and a failed refresh then left
 * stale data on screen wearing a fresh timestamp — a silent lie the reader had
 * no way to catch. The flag cannot go missing from a load that does not spell
 * one out.
 */
export function createPortalResource(options: PortalResourceOptions): PortalResourceHandle {
  let state = $state<PortalResourceState>({ kind: 'loading' });
  let refreshing = $state(false);
  let refreshFailed = $state(false);
  let sequence = 0;

  const page = $derived(state.kind === 'ready' ? state.page : null);
  const fetchedAt = $derived(page?.fetchedAt ?? null);

  /**
   * What a view owes the shared live region: the section it is about, then the
   * same freshness sentence `FreshnessLabel` puts on screen. A sighted reader
   * watches that label change; a screen reader has no way to notice it did. The
   * full stop is the separator every reader turns into a pause instead of
   * spelling out.
   */
  function announce(statement: string) {
    announcer.announce(`${options.heading()}. ${statement}`);
  }

  function syncTime(at: number): string {
    return new Intl.DateTimeFormat(options.locale(), {
      hour: '2-digit',
      minute: '2-digit',
    }).format(at);
  }

  // Only the transition is news. Reading `connectivity.online` in an effect
  // without this guard would announce the current state on mount, over the view
  // the reader has only just opened.
  let wasOnline = connectivity.online;
  $effect(() => {
    const online = connectivity.online;
    if (online === wasOnline) return;
    wasOnline = online;
    if (!online) announce(m.sync_offline_description());
  });

  async function load(force = false) {
    const request = ++sequence;
    const hasData = state.kind === 'ready';
    // Without data on screen the skeleton is the feedback, so a retry from the
    // error card shows it is working instead of looking inert until it lands.
    if (hasData) refreshing = true;
    else state = { kind: 'loading' };

    try {
      const fresh = await loadPortalResource(options.resource, force);
      if (request !== sequence) return;
      state = { kind: 'ready', page: fresh };
      refreshFailed = false;
      announce(m.sync_fresh({ time: syncTime(fresh.fetchedAt) }));
    } catch (error) {
      if (request !== sequence) return;
      const code = parseResourceError(error, options.fallbackErrorCode);
      const keepsData =
        hasData && !(options.expiredReplacesData === true && code === 'session_expired');
      // Keeping stale data on screen is fine; keeping it unmarked is not. Flag
      // the failure so the freshness label can say what the user is looking at.
      if (keepsData) {
        refreshFailed = true;
        announce(
          fetchedAt === null ? m.sync_offline() : m.sync_failed({ time: syncTime(fetchedAt) })
        );
      } else {
        state = { kind: 'error', code };
        announce(
          connectivity.online ? resourceErrorMessage(code) : m.sync_offline_description()
        );
      }
    } finally {
      if (request === sequence) refreshing = false;
    }
  }

  return {
    get state() {
      return state;
    },
    get page() {
      return page;
    },
    get refreshing() {
      return refreshing;
    },
    get refreshFailed() {
      return refreshFailed;
    },
    get fetchedAt() {
      return fetchedAt;
    },
    load,
  };
}
