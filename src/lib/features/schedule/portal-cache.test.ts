import { beforeEach, describe, expect, it, mock } from 'bun:test';
import type { PortalPage, PortalResource } from './types';

/** Every command the cache sent, in order, so a stray sign-in is visible. */
let invoked: string[] = [];
/** One entry per portal read still waiting for the portal to answer. */
let reads: { resolve: (page: PortalPage) => void; reject: (error: unknown) => void }[] = [];

function stubInvoke() {
  mock.module('$lib/invoke', () => ({
    isTauri: () => true,
    invoke: (command: string) => {
      invoked.push(command);
      if (command === 'restore_session') return Promise.resolve({ status: 'restored' });
      return new Promise<PortalPage>((resolve, reject) => {
        reads.push({ resolve, reject });
      });
    },
  }));
}

stubInvoke();

const { clearPortalResourceCache, invalidatePortalResourceCache, loadPortalResource } =
  await import('./portal-cache');
const { sessionRecovery } = await import('$lib/state/session-recovery.svelte');
const { connectivity } = await import('$lib/state/connectivity.svelte');

function page(overrides: Partial<PortalPage> = {}): PortalPage {
  return {
    resource: 'grades',
    fetchedAt: 1_756_000_000,
    title: 'Notes',
    headings: [],
    tables: [],
    fields: [],
    documents: [],
    gradePeriods: [],
    absencePeriods: [],
    questionnaires: [],
    markupRecognized: true,
    stale: false,
    ...overrides,
  };
}

/** Lets the promise chain inside the cache run to its next pause. */
function settle(): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, 0));
}

const GRADES: PortalResource = 'grades';

beforeEach(() => {
  stubInvoke();
  invoked = [];
  reads = [];
  connectivity.online = true;
  sessionRecovery.reset();
  clearPortalResourceCache();
});

describe('portal cache spends one request per page', () => {
  it('answers two views that ask for the same page at once with one read', async () => {
    const first = loadPortalResource(GRADES);
    const second = loadPortalResource(GRADES);
    expect(reads).toHaveLength(1);

    reads[0].resolve(page());
    // The same page object, not two: the shell opens on several views and none
    // of them should cost the portal a request of its own.
    expect(await first).toBe(await second);
    expect(invoked).toEqual(['get_portal_resource']);
  });

  it('serves a fresh page from memory instead of asking again', async () => {
    const fresh = page();
    loadPortalResource(GRADES);
    reads[0].resolve(fresh);
    await settle();

    expect(await loadPortalResource(GRADES)).toBe(fresh);
    expect(invoked).toEqual(['get_portal_resource']);
  });

  it('never memoises a page the portal did not answer', async () => {
    loadPortalResource(GRADES);
    // `stale` is the Rust side replaying its local snapshot because the portal
    // was unreachable. Holding it would keep serving it after the network came
    // back, for the whole five minutes of the TTL.
    reads[0].resolve(page({ stale: true }));
    await settle();

    loadPortalResource(GRADES);
    expect(invoked).toEqual(['get_portal_resource', 'get_portal_resource']);
  });

  it('lets a deliberate refresh past the memory tier', async () => {
    loadPortalResource(GRADES);
    reads[0].resolve(page());
    await settle();

    loadPortalResource(GRADES, true);
    expect(invoked).toEqual(['get_portal_resource', 'get_portal_resource']);
  });

  it('keeps one resource from being answered with another', async () => {
    loadPortalResource(GRADES);
    loadPortalResource('absences');
    expect(reads).toHaveLength(2);
  });
});

describe('portal cache keeps one session out of the next', () => {
  it('drops a read that was in flight when the session was torn down', async () => {
    const inFlight = loadPortalResource(GRADES);

    // A sign-out, halfway through. Whoever signs in next must not be handed
    // the page the previous account asked for.
    clearPortalResourceCache();
    reads[0].resolve(page());
    await inFlight;

    loadPortalResource(GRADES);
    expect(invoked).toEqual(['get_portal_resource', 'get_portal_resource']);
  });

  it('lets the read that noticed an expiry finish after the session is replayed', async () => {
    const fresh = page();
    const inFlight = loadPortalResource(GRADES);

    // A replay clears the pages read through the dead session, but must leave
    // the request that spotted the expiry alone — it is the one the recovery
    // exists to let through.
    invalidatePortalResourceCache();
    reads[0].resolve(fresh);
    await inFlight;

    expect(await loadPortalResource(GRADES)).toBe(fresh);
    expect(invoked).toEqual(['get_portal_resource']);
  });

  it('still forgets the pages a replay made stale', async () => {
    loadPortalResource(GRADES);
    reads[0].resolve(page());
    await settle();

    invalidatePortalResourceCache();
    loadPortalResource(GRADES);
    expect(invoked).toEqual(['get_portal_resource', 'get_portal_resource']);
  });
});

describe('portal cache replays an expired session exactly once', () => {
  it('signs back in and retries the read that failed', async () => {
    const fresh = page();
    const read = loadPortalResource(GRADES);

    reads[0].reject({ code: 'session_expired' });
    await settle();

    expect(invoked).toEqual(['get_portal_resource', 'restore_session', 'get_portal_resource']);
    reads[1].resolve(fresh);
    // The reader sees their grades, not an expiry card: the whole point of the
    // replay is that the expiry costs a round trip instead of the account.
    expect(await read).toBe(fresh);
  });

  it('gives the reader the error rather than a second sign-in', async () => {
    const read = loadPortalResource(GRADES);
    reads[0].reject({ code: 'session_expired' });
    await settle();

    // A session that dies again on the very next request is not one another
    // replay will fix, and a spinner that never ends is worse than the error.
    reads[1].reject({ code: 'session_expired' });
    await expect(read).rejects.toMatchObject({ code: 'session_expired' });
    expect(invoked).toEqual(['get_portal_resource', 'restore_session', 'get_portal_resource']);
  });

  it('never answers an ordinary portal failure with a sign-in', async () => {
    const read = loadPortalResource(GRADES);
    reads[0].reject({ code: 'grades_unavailable' });

    await expect(read).rejects.toMatchObject({ code: 'grades_unavailable' });
    expect(invoked).toEqual(['get_portal_resource']);
  });

  it('surfaces the expiry when the saved password can no longer be replayed', async () => {
    // The recovery has already given up, so the request path gets nothing.
    sessionRecovery.status = 'exhausted';

    const read = loadPortalResource(GRADES);
    reads[0].reject({ code: 'session_expired' });

    await expect(read).rejects.toMatchObject({ code: 'session_expired' });
    expect(invoked).toEqual(['get_portal_resource']);
  });
});
