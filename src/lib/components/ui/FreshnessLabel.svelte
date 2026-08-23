<script lang="ts">
  import { CloudOff, RefreshCw, TriangleAlert } from 'lucide-svelte';
  import * as m from '$lib/paraglide/messages.js';
  import { connectivity } from '$lib/state/connectivity.svelte';

  type Props = {
    /** Epoch ms of the last successful fetch, or null if nothing has landed yet. */
    fetchedAt: number | null;
    locale: string;
    /** A refresh is in flight. */
    refreshing?: boolean;
    /** The last refresh failed while data was already on screen. */
    failed?: boolean;
  };

  const { fetchedAt, locale, refreshing = false, failed = false }: Props = $props();

  const timeLabel = $derived(
    fetchedAt === null
      ? null
      : new Intl.DateTimeFormat(locale, { hour: '2-digit', minute: '2-digit' }).format(
          new Date(fetchedAt)
        )
  );

  /** Data older than the cache TTL is worth flagging, not just timestamping. */
  const STALE_AFTER_MS = 5 * 60 * 1000;
  const stale = $derived(fetchedAt !== null && Date.now() - fetchedAt > STALE_AFTER_MS);

  type Tone = 'offline' | 'failed' | 'refreshing' | 'stale' | 'fresh' | 'never';

  const tone = $derived.by<Tone>(() => {
    if (!connectivity.online) return 'offline';
    if (failed) return 'failed';
    if (refreshing) return 'refreshing';
    if (timeLabel === null) return 'never';
    return stale ? 'stale' : 'fresh';
  });

  const text = $derived.by(() => {
    locale;
    switch (tone) {
      case 'offline':
        return m.sync_offline();
      case 'failed':
        return timeLabel ? m.sync_failed({ time: timeLabel }) : m.sync_offline();
      case 'refreshing':
        return m.sync_refreshing();
      case 'never':
        return m.sync_never();
      case 'stale':
        return m.sync_stale({ time: timeLabel ?? '' });
      default:
        return m.sync_fresh({ time: timeLabel ?? '' });
    }
  });
</script>

<!-- Freshness is a promise this product makes, so it is stated in words and not
     signalled by colour alone. -->
<p class="ui-freshness {tone}" aria-label={m.freshness_label()}>
  {#if tone === 'offline'}
    <CloudOff size={14} aria-hidden="true" />
  {:else if tone === 'failed'}
    <TriangleAlert size={14} aria-hidden="true" />
  {:else if tone === 'refreshing'}
    <RefreshCw size={14} class="freshness-spin" aria-hidden="true" />
  {:else}
    <span class="freshness-dot" aria-hidden="true"></span>
  {/if}
  <span>{text}</span>
</p>

<style>
  .ui-freshness {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2);
    margin: 0;
    font-size: var(--text-xs);
    font-weight: var(--weight-medium);
    font-variant-numeric: tabular-nums;
  }

  .freshness-dot {
    width: 0.4rem;
    height: 0.4rem;
    border-radius: 50%;
    background: currentColor;
  }

  :global(.freshness-spin) {
    animation: spin var(--duration-spin) linear infinite;
  }

  .fresh {
    color: var(--success-strong);
  }

  .stale,
  .refreshing,
  .never {
    color: var(--muted-foreground);
  }

  .failed,
  .offline {
    color: var(--danger-strong);
  }
</style>
