<script lang="ts">
  import { CloudOff, RefreshCw, TriangleAlert } from 'lucide-svelte';
  import * as m from '$lib/paraglide/messages.js';
  import { connectivity } from '$lib/state/connectivity.svelte';
  import { cn } from '$lib/utils';

  type Props = {
    /** Epoch ms of the last successful fetch, or null if nothing has landed yet. */
    fetchedAt: number | null;
    locale: string;
    /** A refresh is in flight. */
    refreshing?: boolean;
    /** The last refresh failed while data was already on screen. */
    failed?: boolean;
    class?: string;
  };

  const {
    fetchedAt,
    locale,
    refreshing = false,
    failed = false,
    class: className
  }: Props = $props();

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

  const tones = {
    fresh: 'text-success-strong',
    stale: 'text-muted-foreground',
    refreshing: 'text-muted-foreground',
    never: 'text-muted-foreground',
    failed: 'text-danger-strong',
    offline: 'text-danger-strong'
  } as const satisfies Record<Tone, string>;

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
<p
  class={cn(
    'ui-freshness inline-flex items-center gap-2 text-xs font-medium tabular-nums',
    tones[tone],
    className
  )}
  aria-label={m.freshness_label()}
>
  {#if tone === 'offline'}
    <CloudOff size={14} aria-hidden="true" />
  {:else if tone === 'failed'}
    <TriangleAlert size={14} aria-hidden="true" />
  {:else if tone === 'refreshing'}
    <RefreshCw size={14} class="animate-spin" aria-hidden="true" />
  {:else}
    <span class="size-[0.4rem] rounded-full bg-current" aria-hidden="true"></span>
  {/if}
  <span>{text}</span>
</p>
