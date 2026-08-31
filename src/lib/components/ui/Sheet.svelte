<script lang="ts">
  import type { Snippet } from 'svelte';
  import { cn } from '$lib/utils';

  type Props = {
    children: Snippet;
    /** Accessible name for the dialog. */
    title: string;
    /** `center` rises from the bottom on small screens; `end` slides in from the inline end. */
    placement?: 'center' | 'end';
    /** Label for the backdrop's own dismiss control. */
    closeLabel: string;
    onClose: () => void;
    class?: string;
  };

  const {
    children,
    title,
    placement = 'center',
    closeLabel,
    onClose,
    class: className
  }: Props = $props();

  // A sheet is the one surface that sits directly on a system edge, so its
  // margins are spelled out: `*-safe-*` adds the gesture pill, the status bar or
  // a landscape cutout to the gap, and the panel clears both instead of resting
  // on one of them.
  const roots = {
    center: 'items-end justify-items-center md:items-center',
    end: 'items-start justify-items-end'
  } as const satisfies Record<NonNullable<Props['placement']>, string>;

  const panels = {
    center:
      'w-[min(100%,34rem)] mt-0 mx-safe-3 mb-safe-3 md:m-6 animate-slide-up-in',
    end: 'w-[min(100%,24rem)] mt-safe-3 mx-safe-3 mb-3 animate-slide-up-drawer'
  } as const satisfies Record<NonNullable<Props['placement']>, string>;

  let root: HTMLDivElement | undefined = $state();
  let panel: HTMLDivElement | undefined = $state();

  // Drag to dismiss. A bottom sheet is expected to follow the finger down and
  // close, which is also what stops the gesture from reading as a pull to
  // refresh on the page it covers.
  const DISMISS_DISTANCE = 96;
  const DRAG_MAX = 320;

  let dragOffset = $state(0);
  let isDragging = $state(false);
  let dragStartY = 0;
  let dragStartX = 0;
  let dragAxis: 'undecided' | 'vertical' | 'horizontal' = 'undecided';

  const isDismissable = $derived(placement === 'center');

  function handlePanelTouchStart(event: TouchEvent) {
    if (!isDismissable || event.touches.length !== 1) return;
    if ((panel?.scrollTop ?? 0) > 0) return;

    dragStartY = event.touches[0].clientY;
    dragStartX = event.touches[0].clientX;
    dragAxis = 'undecided';
    isDragging = false;
    dragOffset = 0;
  }

  function handlePanelTouchMove(event: TouchEvent) {
    if (!isDismissable || event.touches.length !== 1) return;

    const diffY = event.touches[0].clientY - dragStartY;
    const diffX = event.touches[0].clientX - dragStartX;

    if (dragAxis === 'undecided') {
      if (Math.abs(diffX) > Math.abs(diffY) && Math.abs(diffX) > 8) {
        dragAxis = 'horizontal';
        return;
      }
      if (diffY > 8) {
        dragAxis = 'vertical';
        isDragging = true;
      } else if (diffY < -8) {
        dragAxis = 'vertical';
        return;
      }
    }

    if (dragAxis !== 'vertical' || !isDragging) return;

    // Scrolling the panel back up mid-drag hands the gesture back to the list.
    if ((panel?.scrollTop ?? 0) > 0 || diffY <= 0) {
      dragOffset = 0;
      isDragging = false;
      return;
    }

    dragOffset = Math.min(DRAG_MAX, Math.pow(diffY, 0.9));
    if (event.cancelable) event.preventDefault();
  }

  function handlePanelTouchEnd() {
    if (!isDragging) return;

    const shouldClose = dragOffset >= DISMISS_DISTANCE;
    isDragging = false;
    dragOffset = 0;
    dragAxis = 'undecided';

    if (shouldClose) onClose();
  }

  function handlePanelTouchCancel() {
    isDragging = false;
    dragOffset = 0;
    dragAxis = 'undecided';
  }

  // Bound by hand rather than with `ontouchmove`: the drag has to call
  // `preventDefault`, which a passive listener cannot do.
  $effect(() => {
    const node = panel;
    if (!node || !isDismissable) return;

    node.addEventListener('touchstart', handlePanelTouchStart, { passive: true });
    node.addEventListener('touchmove', handlePanelTouchMove, { passive: false });
    node.addEventListener('touchend', handlePanelTouchEnd, { passive: true });
    node.addEventListener('touchcancel', handlePanelTouchCancel, { passive: true });

    return () => {
      node.removeEventListener('touchstart', handlePanelTouchStart);
      node.removeEventListener('touchmove', handlePanelTouchMove);
      node.removeEventListener('touchend', handlePanelTouchEnd);
      node.removeEventListener('touchcancel', handlePanelTouchCancel);
    };
  });

  const FOCUSABLE =
    'a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])';

  /**
   * A dialog owes the keyboard three things none of the previous sheets did:
   * take focus on open, keep Tab inside while open, and hand focus back to
   * whatever opened it on close.
   */
  $effect(() => {
    const opener = document.activeElement as HTMLElement | null;
    const { body } = document;
    const scrollParent = findScrollableParent(root);
    const lockedElements = [document.documentElement, body, scrollParent].filter(
      (element, index, all): element is HTMLElement => element !== null && all.indexOf(element) === index
    );
    const previousOverflow = lockedElements.map((element) => [element, element.style.overflow] as const);

    for (const element of lockedElements) element.style.overflow = 'hidden';
    panel?.focus();

    return () => {
      for (const [element, overflow] of previousOverflow) element.style.overflow = overflow;
      opener?.focus?.();
    };
  });

  function findScrollableParent(element: HTMLElement | undefined): HTMLElement | null {
    let parent = element?.parentElement ?? null;
    while (parent && parent !== document.body) {
      const styles = getComputedStyle(parent);
      if (/(auto|scroll|overlay)/.test(styles.overflowY) && parent.scrollHeight > parent.clientHeight) {
        return parent;
      }
      parent = parent.parentElement;
    }
    return null;
  }

  function handleKeyDown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      event.preventDefault();
      onClose();
      return;
    }

    if (event.key !== 'Tab' || !panel) return;

    const focusable = [...panel.querySelectorAll<HTMLElement>(FOCUSABLE)].filter(
      (node) => node.offsetParent !== null
    );
    if (focusable.length === 0) {
      event.preventDefault();
      return;
    }

    const first = focusable[0];
    const last = focusable[focusable.length - 1];
    const active = document.activeElement;

    if (event.shiftKey && (active === first || active === panel)) {
      event.preventDefault();
      last.focus();
    } else if (!event.shiftKey && active === last) {
      event.preventDefault();
      first.focus();
    }
  }
</script>

<div
  bind:this={root}
  class={cn('ui-sheet-root fixed inset-0 z-modal grid overscroll-contain', roots[placement], className)}
  onkeydown={handleKeyDown}
  role="presentation"
>
  <button
    type="button"
    class="absolute inset-0 cursor-default bg-surface-overlay backdrop-blur-sm animate-fade-in-fast"
    aria-label={closeLabel}
    onclick={onClose}
  ></button>

  <div
    bind:this={panel}
    class={cn(
      'relative z-[1] flex max-h-[calc(100dvh-2*var(--space-6))] flex-col overflow-y-auto',
      'overscroll-contain rounded-xl bg-card shadow-xl',
      'focus-visible:outline-3 focus-visible:-outline-offset-3 focus-visible:outline-ring',
      panels[placement]
    )}
    role="dialog"
    aria-modal="true"
    aria-label={title}
    tabindex="-1"
    style:transform={`translate3d(0, ${dragOffset}px, 0)`}
    style:transition={isDragging ? 'none' : 'transform var(--duration-normal) var(--ease-out)'}
  >
    {@render children()}
  </div>
</div>

