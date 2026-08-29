import { invoke } from '$lib/invoke';
import { sessionRecovery } from '$lib/state/session-recovery.svelte';
import type { PortalPage, PortalResource } from './types';

const CACHE_TTL_MS = 5 * 60_000;

type CacheEntry = {
  page: PortalPage;
  expiresAt: number;
};

const cache = new Map<PortalResource, CacheEntry>();
const pending = new Map<PortalResource, { generation: number; request: Promise<PortalPage> }>();
let generation = 0;

function isSessionExpired(error: unknown): boolean {
  return (
    typeof error === 'object' &&
    error !== null &&
    'code' in error &&
    (error as { code: unknown }).code === 'session_expired'
  );
}

/**
 * One read, plus one replay of the saved password if the portal says the
 * session is gone.
 *
 * Every resource — grades, absences, profile, documents, questionnaires —
 * comes through here, which is why an expiry can be handled once instead of in
 * each view. The retry is deliberately not a loop: a session that expires again
 * on the very next request is not one a second replay will fix, and the reader
 * is owed the error instead of a spinner that never ends.
 */
async function fetchPortalResource(resource: PortalResource, force: boolean): Promise<PortalPage> {
  try {
    return await invoke<PortalPage>('get_portal_resource', { resource, force });
  } catch (error) {
    if (!isSessionExpired(error)) throw error;
    if (!(await sessionRecovery.recover())) throw error;
    return invoke<PortalPage>('get_portal_resource', { resource, force });
  }
}

export async function loadPortalResource(
  resource: PortalResource,
  force = false
): Promise<PortalPage> {
  const cached = cache.get(resource);
  if (!force && cached && cached.expiresAt > Date.now()) {
    return cached.page;
  }

  const currentRequest = pending.get(resource);
  if (!force && currentRequest?.generation === generation) return currentRequest.request;

  const requestGeneration = generation;
  const request = fetchPortalResource(resource, force)
    .then((page) => {
      // A stale page is the Rust side answering from its local snapshot because
      // the portal was unreachable. Holding it here for the full TTL would keep
      // serving it after the network comes back, so it is deliberately not
      // cached: the next read reaches Rust, which tries the portal again.
      if (requestGeneration === generation && !page.stale) {
        cache.set(resource, { page, expiresAt: Date.now() + CACHE_TTL_MS });
      }
      return page;
    })
    .finally(() => {
      if (pending.get(resource)?.generation === requestGeneration) pending.delete(resource);
    });

  pending.set(resource, { generation: requestGeneration, request });
  return request;
}

/**
 * Drops the memory tier without touching `generation`.
 *
 * Called after a session was replayed: the pages held here were read through
 * the session that just died. Bumping the generation the way
 * `clearPortalResourceCache` does would orphan the in-flight read that noticed
 * the expiry — the very request the recovery exists to let through.
 */
export function invalidatePortalResourceCache(): void {
  cache.clear();
}

export function clearPortalResourceCache() {
  generation += 1;
  cache.clear();
  pending.clear();
}
