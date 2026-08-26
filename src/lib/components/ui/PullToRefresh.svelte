<script lang="ts">
  import { onDestroy, onMount, type Snippet } from 'svelte';
  import { RefreshCw } from 'lucide-svelte';
  import * as m from '$lib/paraglide/messages.js';
  import { cn } from '$lib/utils';

  type Props = {
    onRefresh: () => Promise<void>;
    disabled?: boolean;
    threshold?: number;
    maxPull?: number;
    scrollElement?: HTMLElement | null;
    children: Snippet;
  };

  let {
    onRefresh,
    disabled = false,
    threshold = 64,
    maxPull = 120,
    scrollElement = null,
    children,
  }: Props = $props();

  let containerRef = $state<HTMLDivElement | null>(null);
  let isPulling = $state(false);
  let pullDistance = $state(0);
  let isRefreshing = $state(false);
  let isThresholdReached = $state(false);

  let touchStartY = 0;
  let touchStartX = 0;
  let isAtTopAtStart = false;
  let gestureDirection: 'undecided' | 'vertical' | 'horizontal' = 'undecided';
  let hasVibrated = false;

  const REFRESH_REST_HEIGHT = 52;

  // A sheet is a DOM descendant of the scroll container it covers, so its touch
  // events still reach our listener. Pulling inside a dialog must never refresh
  // the page underneath it.
  const OVERLAY_SELECTOR = '.ui-sheet-root, [role="dialog"], [aria-modal="true"]';

  function isOverlayOpen(): boolean {
    if (typeof document === 'undefined') return false;
    return document.querySelector(OVERLAY_SELECTOR) !== null;
  }

  function getEffectiveScrollElement(): HTMLElement | null {
    if (scrollElement) return scrollElement;
    if (!containerRef) return null;
    let parent = containerRef.parentElement;
    while (parent) {
      const overflowY = window.getComputedStyle(parent).overflowY;
      if (overflowY === 'auto' || overflowY === 'scroll') {
        return parent;
      }
      parent = parent.parentElement;
    }
    return null;
  }

  /**
   * A view can nest its own vertical scroller — the calendar's time grid is one.
   * A drag that starts inside one belongs to that scroller: the viewport behind
   * it stays at the top the whole time, so without this test every swipe down
   * over the grid refreshed the page instead of scrolling the hours.
   */
  function startsInsideNestedScroller(target: EventTarget | null): boolean {
    if (typeof window === 'undefined') return false;

    const outer = getEffectiveScrollElement();
    let node: Element | null = target instanceof Element ? target : null;

    while (node && node !== outer && node !== document.body) {
      const overflowY = window.getComputedStyle(node).overflowY;
      if (
        (overflowY === 'auto' || overflowY === 'scroll') &&
        node.scrollHeight > node.clientHeight + 1
      ) {
        return true;
      }
      node = node.parentElement;
    }

    return false;
  }

  function getScrollTop(elem: HTMLElement | null): number {
    if (elem) return elem.scrollTop;
    if (typeof window !== 'undefined') return window.scrollY || document.documentElement.scrollTop || 0;
    return 0;
  }

  function handleTouchStart(e: TouchEvent) {
    if (disabled || isRefreshing || e.touches.length !== 1) return;

    if (isOverlayOpen()) {
      isAtTopAtStart = false;
      return;
    }

    if (startsInsideNestedScroller(e.target)) {
      isAtTopAtStart = false;
      return;
    }

    const targetElem = getEffectiveScrollElement();
    const currentScrollTop = getScrollTop(targetElem);

    // Only enable pull to refresh when scrolled at the very top
    isAtTopAtStart = currentScrollTop <= 1;
    if (!isAtTopAtStart) return;

    touchStartY = e.touches[0].clientY;
    touchStartX = e.touches[0].clientX;
    gestureDirection = 'undecided';
    isPulling = false;
    pullDistance = 0;
    isThresholdReached = false;
    hasVibrated = false;
  }

  function handleTouchMove(e: TouchEvent) {
    if (disabled || isRefreshing || !isAtTopAtStart || e.touches.length !== 1) return;

    // A dialog can also open mid-gesture, so the pull has to be abandoned here
    // too and not only at the start of the touch.
    if (isOverlayOpen()) {
      isAtTopAtStart = false;
      isPulling = false;
      pullDistance = 0;
      isThresholdReached = false;
      hasVibrated = false;
      return;
    }

    const touch = e.touches[0];
    const diffY = touch.clientY - touchStartY;
    const diffX = touch.clientX - touchStartX;

    // Detect horizontal swipes (tabs, carousels) and allow them without pulling
    if (gestureDirection === 'undecided') {
      if (Math.abs(diffX) > Math.abs(diffY) && Math.abs(diffX) > 8) {
        gestureDirection = 'horizontal';
        return;
      }
      if (diffY > 8) {
        gestureDirection = 'vertical';
        isPulling = true;
      } else if (diffY < -8) {
        gestureDirection = 'vertical';
        isPulling = false;
        return;
      }
    }

    if (gestureDirection !== 'vertical' || !isPulling) return;

    const targetElem = getEffectiveScrollElement();
    const currentScrollTop = getScrollTop(targetElem);

    if (currentScrollTop > 1) {
      isPulling = false;
      pullDistance = 0;
      return;
    }

    if (diffY > 0) {
      // Elastic rubber-band resistance curve
      const damped = Math.pow(diffY, 0.82) * 1.55;
      pullDistance = Math.min(maxPull, damped);
      isThresholdReached = pullDistance >= threshold;

      if (isThresholdReached && !hasVibrated) {
        hasVibrated = true;
        if (typeof navigator !== 'undefined' && 'vibrate' in navigator) {
          try {
            navigator.vibrate(12);
          } catch {
            // Ignore if vibrations blocked
          }
        }
      } else if (!isThresholdReached && hasVibrated) {
        hasVibrated = false;
      }

      if (e.cancelable) {
        e.preventDefault();
      }
    } else {
      pullDistance = 0;
      isPulling = false;
      isThresholdReached = false;
    }
  }

  async function handleTouchEnd() {
    if (!isPulling || isRefreshing) return;

    if (isThresholdReached) {
      isRefreshing = true;
      isPulling = false;
      pullDistance = REFRESH_REST_HEIGHT;

      try {
        await onRefresh();
      } catch (err) {
        console.error('Pull to refresh failed:', err);
      } finally {
        isRefreshing = false;
        pullDistance = 0;
        isThresholdReached = false;
        hasVibrated = false;
      }
    } else {
      isPulling = false;
      pullDistance = 0;
      isThresholdReached = false;
      hasVibrated = false;
    }
  }

  function handleTouchCancel() {
    if (!isRefreshing) {
      isPulling = false;
      pullDistance = 0;
      isThresholdReached = false;
      hasVibrated = false;
    }
  }

  onMount(() => {
    const target = getEffectiveScrollElement() || containerRef || window;

    target.addEventListener('touchstart', handleTouchStart as EventListener, { passive: true });
    target.addEventListener('touchmove', handleTouchMove as EventListener, { passive: false });
    target.addEventListener('touchend', handleTouchEnd as EventListener, { passive: true });
    target.addEventListener('touchcancel', handleTouchCancel as EventListener, { passive: true });

    return () => {
      target.removeEventListener('touchstart', handleTouchStart as EventListener);
      target.removeEventListener('touchmove', handleTouchMove as EventListener);
      target.removeEventListener('touchend', handleTouchEnd as EventListener);
      target.removeEventListener('touchcancel', handleTouchCancel as EventListener);
    };
  });

  const pullProgress = $derived(
    threshold > 0 ? Math.min(1, Math.max(0, pullDistance / threshold)) : 0
  );

  const indicatorOffset = $derived.by(() => {
    if (isRefreshing) return 16;
    if (pullDistance <= 0) return -60;
    return Math.min(30, pullDistance - 40);
  });

  const indicatorOpacity = $derived.by(() => {
    if (isRefreshing) return 1;
    if (pullDistance <= 10) return 0;
    return Math.min(1, (pullDistance - 10) / (threshold * 0.6));
  });

  const statusLabel = $derived.by(() => {
    if (isRefreshing) return m.sync_refreshing();
    if (isThresholdReached) return m.pull_to_refresh_release();
    return m.pull_to_refresh_pull();
  });
</script>

<div
  class="ui-pull-to-refresh-container relative flex min-h-full w-full flex-1 flex-col"
  bind:this={containerRef}
>
  <!-- Floating pull indicator pill. The transform and opacity are written per
       frame from the gesture, so the transition here is what they ride on. -->
  <div
    class="pointer-events-none absolute top-0 left-1/2 z-sticky transition-[transform,opacity]
           duration-fast ease-out will-change-[transform,opacity]"
    style:transform={`translate3d(-50%, ${indicatorOffset}px, 0)`}
    style:opacity={indicatorOpacity}
    aria-hidden={!isRefreshing}
    role={isRefreshing ? 'status' : undefined}
    aria-live={isRefreshing ? 'polite' : undefined}
  >
    <div
      class={cn(
        'inline-flex items-center gap-2 rounded-pill border px-[0.85rem] py-[0.4rem]',
        'bg-card-veil backdrop-blur-lg text-xs font-bold tracking-[-0.01em] select-none',
        isThresholdReached
          ? 'border-border text-primary-deep shadow-lg'
          : isRefreshing
            ? 'border-border text-primary-deep shadow-md'
            : 'border-border-subtle text-muted-foreground shadow-md'
      )}
    >
      <div
        class={cn(
          'grid shrink-0 place-items-center transition-colors duration-fast ease-out',
          isRefreshing && 'animate-spin'
        )}
        style:transform={isRefreshing ? undefined : `rotate(${pullProgress * 320}deg) scale(${0.85 + pullProgress * 0.15})`}
      >
        <RefreshCw size={15} strokeWidth={2.4} aria-hidden="true" />
      </div>
      <span class="whitespace-nowrap">{statusLabel}</span>
    </div>
  </div>

  <!-- Content with elastic push down. `transition` is written inline because it
       is switched off mid-gesture, so it must stay the only source of truth. -->
  <div
    class="flex min-h-full w-full flex-1 flex-col will-change-transform"
    style:transform={`translate3d(0, ${isPulling ? pullDistance * 0.42 : isRefreshing ? REFRESH_REST_HEIGHT * 0.42 : 0}px, 0)`}
    style:transition={isPulling ? 'none' : 'transform var(--duration-normal) var(--ease-out)'}
  >
    {@render children()}
  </div>
</div>

