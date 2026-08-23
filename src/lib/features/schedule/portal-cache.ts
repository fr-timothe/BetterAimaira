import { invoke } from '@tauri-apps/api/core';
import type { PortalPage, PortalResource } from './types';

const CACHE_TTL_MS = 5 * 60_000;

type CacheEntry = {
  page: PortalPage;
  expiresAt: number;
};

const cache = new Map<PortalResource, CacheEntry>();
const pending = new Map<PortalResource, { generation: number; request: Promise<PortalPage> }>();
let generation = 0;

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
  const request = invoke<PortalPage>('get_portal_resource', { resource, force })
    .then((page) => {
      if (requestGeneration === generation) {
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

export function clearPortalResourceCache() {
  generation += 1;
  cache.clear();
  pending.clear();
}
