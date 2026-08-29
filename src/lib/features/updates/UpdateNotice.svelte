<script lang="ts">
  import { ArrowDownToLine, X } from 'lucide-svelte';
  import * as m from '$lib/paraglide/messages.js';
  import { cn } from '$lib/utils';
  import { updates } from './updates.svelte';

  type Props = {
    locale: string;
    /** Sends the user to the card that installs the update. */
    onOpen: () => void;
  };

  const { locale, onOpen }: Props = $props();

  /**
   * Long enough to be read after the schedule has painted, short enough not to
   * sit over the day. Timing out is not a verdict: the notice comes back on the
   * next launch, and only the close button silences this version for good.
   */
  const AUTO_HIDE_MS = 12_000;

  const visible = $derived(updates.noticeVisible && updates.available);

  const body = $derived.by(() => {
    const version = updates.info?.latestVersion;
    return version ? m.update_notice_body({ version }) : m.update_notice_body_unnamed();
  });

  $effect(() => {
    if (!visible) return;
    const timer = window.setTimeout(() => updates.hideNotice(), AUTO_HIDE_MS);
    return () => window.clearTimeout(timer);
  });
</script>

{#if visible}
  <!-- The shell owns where this sits: it is the only side that knows whether the
       rail is on screen. This component owns what it says. -->
  <div role="status" aria-live="polite">
    <div
      class="pointer-events-auto flex w-[min(100%,26rem)] items-center gap-1 rounded-lg
             border border-dock-edge bg-dock-veil pr-1 shadow-lg backdrop-blur-[20px]
             animate-slide-up-in"
    >
      <button
        class={cn(
          'flex min-h-(--tap-min) min-w-0 flex-1 items-center gap-3 rounded-[inherit] py-2 pl-3',
          'text-left transition-control active:scale-(--press-scale)',
          'fine-hover:bg-muted'
        )}
        type="button"
        onclick={onOpen}
      >
        <span
          class="grid size-9 flex-none place-items-center rounded-sm bg-muted text-primary-deep"
          aria-hidden="true"
        >
          <ArrowDownToLine size={18} />
        </span>
        <span class="flex min-w-0 flex-col">
          <span class="text-base font-bold text-foreground">{m.update_notice_title()}</span>
          <span class="truncate text-sm text-muted-foreground">{body}</span>
        </span>
        <span class="sr-only">{m.update_notice_action()}</span>
      </button>

      <button
        class={cn(
          'grid size-(--tap-min) flex-none place-items-center rounded-md text-muted-foreground',
          'transition-control active:scale-(--press-scale) fine-hover:bg-muted'
        )}
        type="button"
        aria-label={m.update_notice_dismiss()}
        onclick={() => updates.dismissNotice()}
      >
        <X size={18} aria-hidden="true" />
      </button>
    </div>
  </div>
{/if}
