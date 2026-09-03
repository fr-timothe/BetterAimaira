<script lang="ts">
  /**
   * DEV-ONLY PROTOTYPE — not part of the shipped app.
   *
   * The phone chrome each prototype is measured inside. The dock is the same
   * floating object `ScheduleApp` renders on compact windows — except that a
   * view's own controls, when it has any, are a **tier of that object** rather
   * than a second bar stacked under it. Two floating pills of different
   * materials sitting on top of each other is what made the bottom of the
   * first pass unreadable: one border, one radius, one shadow, one blur, two
   * rows.
   *
   * For the build this means the shell owns a control slot above the dock and
   * the view fills it, instead of the view rendering a bar of its own and
   * hoping the clearance matches.
   */
  import type { Snippet } from 'svelte';
  import {
    BookOpenCheck,
    CalendarCheck,
    CalendarDays,
    ChevronLeft,
    ChevronRight,
    ClipboardCheck,
    Home,
    UserRound,
  } from 'lucide-svelte';
  import IconButton from '$lib/components/ui/IconButton.svelte';
  import SegmentedControl from '$lib/components/ui/SegmentedControl.svelte';
  import { viewControls } from '$lib/state/view-controls.svelte';
  import type { CalendarScope } from '$lib/features/schedule/types';
  import { cn } from '$lib/utils';

  type Props = {
    children: Snippet;
    /** Omitted by a structure that carries its own navigation, like the ink panel. */
    scope?: CalendarScope;
    offToday?: boolean;
    onScope?: (scope: CalendarScope) => void;
    onMove?: (direction: -1 | 1) => void;
    onToday?: () => void;
  };

  const { children, scope, offToday = false, onScope, onMove, onToday }: Props = $props();

  const hasOwnControls = $derived(
    scope !== undefined && onScope !== undefined && onMove !== undefined
  );
  /**
   * The shipped view fills the shell's control slot instead of taking props,
   * so this harness renders it the same way `ScheduleApp` does — otherwise
   * `/proto/current` would show the real view without its real bottom bar.
   */
  const hasControls = $derived(hasOwnControls || viewControls.content !== null);

  /**
   * The clearance the scroller owes the bar, measured rather than declared:
   * the bar is one row taller in the views that have controls, and a constant
   * would be wrong in one of the two cases by construction.
   */
  let barHeight = $state(0);

  const items = [
    { label: 'Aujourd’hui', icon: Home, active: false },
    { label: 'Planning', icon: CalendarDays, active: true },
    { label: 'Notes', icon: BookOpenCheck, active: false },
    { label: 'Absences', icon: ClipboardCheck, active: false },
  ];

  const scopeOptions = [
    { value: 'day', label: 'Jour' },
    { value: 'week', label: 'Semaine' },
    { value: 'month', label: 'Mois' },
  ];

  const dockPill =
    'flex min-h-11 flex-col items-center justify-center gap-[0.15rem] rounded-lg' +
    ' bg-transparent px-[0.15rem] py-1 transition-control active:scale-(--press-scale)';
  const dockLabel = 'text-2xs leading-[1.1] font-bold tracking-[-0.01em]';
</script>

<div
  class="relative flex h-dvh max-h-dvh w-full flex-col overflow-hidden bg-background text-foreground"
>
  <div
    class="flex min-h-0 w-full flex-1 flex-col"
    style:padding-bottom={`${barHeight}px`}
  >
    {@render children()}
  </div>

  <nav
    class="bottom-nav pointer-events-none fixed inset-x-0 bottom-0 z-nav justify-center px-3
           pt-1 pb-safe-2"
    aria-label="Navigation principale"
    bind:clientHeight={barHeight}
  >
    <div
      class="pointer-events-auto relative w-[min(100%,32rem)] rounded-xl border border-dock-edge
             bg-dock-veil shadow-lg backdrop-blur-[20px]"
    >
      <!-- The one control that is only worth its width some of the time, and
           the reason it is positioned out of the flow rather than stacked in
           the bar: appearing and disappearing inside it would change the bar's
           height by 44px on every period change, which resizes the content
           above — and one of these structures claims its grid always fits the
           height it was given. It floats over the content instead, which is
           what a transient affordance should do. -->
      {#if hasOwnControls && offToday}
        <button
          type="button"
          class="absolute -top-12 left-1/2 inline-flex min-h-9 -translate-x-1/2 items-center
                 gap-1.5 rounded-pill bg-secondary px-3.5 text-xs font-bold whitespace-nowrap
                 text-secondary-foreground shadow-md transition-control animate-slide-up-in
                 active:scale-(--press-scale)"
          onclick={onToday}
        >
          <CalendarCheck size={14} aria-hidden="true" />
          Revenir à aujourd’hui
        </button>
      {/if}

      {#if viewControls.content}
        <div class="flex items-center gap-1 px-1 pt-1">
          {@render viewControls.content()}
        </div>
        <div class="mx-2 mt-1 h-px bg-border-subtle" aria-hidden="true"></div>
      {:else if hasOwnControls}
        <!-- Tier one: what this view does. The track drops its border and
             takes `--muted` instead of `--surface-sunken`: the veil resolves to
             about L 0.996, so the control's white active pill was invisible on
             it and the selected scope was carried by text colour alone. On a
             pale cyan track the same white pill reads by fill, elevation and
             weight, and it echoes the dock's own active tint one row below. -->
        <div class="flex items-center gap-1 px-1 pt-1">
          <IconButton label="Période précédente" variant="ghost" onclick={() => onMove?.(-1)}>
            <ChevronLeft size={19} strokeWidth={2.2} aria-hidden="true" />
          </IconButton>

          <SegmentedControl
            options={scopeOptions}
            value={scope!}
            label="Portée du planning"
            onChange={(value) => onScope?.(value as CalendarScope)}
            class="min-w-0 flex-1 border-transparent bg-muted"
          />

          <IconButton label="Période suivante" variant="ghost" onclick={() => onMove?.(1)}>
            <ChevronRight size={19} strokeWidth={2.2} aria-hidden="true" />
          </IconButton>
        </div>

        <div class="mx-2 mt-1 h-px bg-border-subtle" aria-hidden="true"></div>
      {/if}

      <!-- Tier two: where you go next. -->
      <div class="grid grid-cols-5 gap-1 px-1 py-[0.3rem]">
        {#each items as item (item.label)}
          {@const Icon = item.icon}
          <button
            type="button"
            class={cn(dockPill, item.active ? 'bg-muted text-primary-deep' : 'text-muted-foreground')}
            aria-current={item.active ? 'page' : undefined}
          >
            <Icon size={20} strokeWidth={item.active ? 2.4 : 1.9} aria-hidden="true" />
            <span class={cn(dockLabel, item.active && 'font-extrabold')}>{item.label}</span>
          </button>
        {/each}
        <button type="button" class={cn(dockPill, 'text-muted-foreground')}>
          <UserRound size={20} strokeWidth={1.9} aria-hidden="true" />
          <span class={dockLabel}>Compte</span>
        </button>
      </div>
    </div>
  </nav>
</div>

<style>
  /* `.bottom-nav` in `ScheduleApp` is a scoped rule, so the class alone carries
     nothing here. Mirroring it means the harness hides the dock at exactly the
     width the app does, instead of showing a phone bar on a desktop check. */
  .bottom-nav {
    display: flex;
  }

  @media (min-width: 48rem) {
    .bottom-nav {
      display: none;
    }
  }
</style>
