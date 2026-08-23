<script lang="ts">
  import { onDestroy, onMount, type Snippet } from 'svelte';
  import { RefreshCw } from 'lucide-svelte';
  import * as m from '$lib/paraglide/messages.js';

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

  function getScrollTop(elem: HTMLElement | null): number {
    if (elem) return elem.scrollTop;
    if (typeof window !== 'undefined') return window.scrollY || document.documentElement.scrollTop || 0;
    return 0;
  }

  function handleTouchStart(e: TouchEvent) {
    if (disabled || isRefreshing || e.touches.length !== 1) return;

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

<div class="ui-pull-to-refresh-container" bind:this={containerRef}>
  <!-- Floating pull indicator pill -->
  <div
    class="pull-indicator-track"
    style:transform={`translate3d(-50%, ${indicatorOffset}px, 0)`}
    style:opacity={indicatorOpacity}
    class:is-active={isPulling || isRefreshing}
    class:is-refreshing={isRefreshing}
    class:is-ready={isThresholdReached}
    aria-hidden={!isRefreshing}
    role={isRefreshing ? 'status' : undefined}
    aria-live={isRefreshing ? 'polite' : undefined}
  >
    <div class="pull-indicator-bubble">
      <div
        class="pull-icon-spinner"
        class:is-spinning={isRefreshing}
        style:transform={isRefreshing ? undefined : `rotate(${pullProgress * 320}deg) scale(${0.85 + pullProgress * 0.15})`}
      >
        <RefreshCw size={15} strokeWidth={2.4} aria-hidden="true" />
      </div>
      <span class="pull-status-text">{statusLabel}</span>
    </div>
  </div>

  <!-- Content with elastic push down -->
  <div
    class="pull-content-wrap"
    style:transform={`translate3d(0, ${isPulling ? pullDistance * 0.42 : isRefreshing ? REFRESH_REST_HEIGHT * 0.42 : 0}px, 0)`}
    style:transition={isPulling ? 'none' : 'transform var(--duration-normal) var(--ease-out)'}
  >
    {@render children()}
  </div>
</div>

<style>
  .ui-pull-to-refresh-container {
    position: relative;
    width: 100%;
    min-height: 100%;
    display: flex;
    flex-direction: column;
    flex: 1 1 0%;
  }

  .pull-indicator-track {
    position: absolute;
    top: 0;
    left: 50%;
    z-index: var(--z-sticky);
    pointer-events: none;
    transition:
      transform var(--duration-fast) var(--ease-out),
      opacity var(--duration-fast) var(--ease-out);
    will-change: transform, opacity;
  }

  .pull-indicator-bubble {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2);
    padding: 0.4rem 0.85rem;
    background: color-mix(in oklch, var(--card) 92%, transparent);
    color: var(--muted-foreground);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-pill);
    box-shadow: var(--shadow-md);
    backdrop-filter: blur(16px);
    -webkit-backdrop-filter: blur(16px);
    font-size: var(--text-xs);
    font-weight: var(--weight-bold);
    letter-spacing: -0.01em;
    user-select: none;
    -webkit-user-select: none;
  }

  .pull-indicator-track.is-ready .pull-indicator-bubble {
    color: var(--primary-deep);
    border-color: var(--border);
    box-shadow: var(--shadow-lg);
  }

  .pull-indicator-track.is-refreshing .pull-indicator-bubble {
    color: var(--primary-deep);
    border-color: var(--border);
    box-shadow: var(--shadow-md);
  }

  .pull-icon-spinner {
    display: grid;
    place-items: center;
    flex-shrink: 0;
    transition: color var(--duration-fast) var(--ease-out);
  }

  .pull-icon-spinner.is-spinning {
    animation: spin var(--duration-spin) linear infinite;
  }

  .pull-status-text {
    white-space: nowrap;
  }

  .pull-content-wrap {
    width: 100%;
    display: flex;
    flex-direction: column;
    flex: 1 1 0%;
    min-height: 100%;
    will-change: transform;
  }
</style>
