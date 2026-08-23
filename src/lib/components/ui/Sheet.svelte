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

  // The safe-area insets are the reason these margins are spelled out: a sheet
  // that clears the home indicator on one platform must not float on another.
  const roots = {
    center: 'items-end justify-items-center md:items-center',
    end: 'items-start justify-items-end'
  } as const satisfies Record<NonNullable<Props['placement']>, string>;

  const panels = {
    center:
      'w-[min(100%,34rem)] mt-0 mx-3 mb-[max(var(--space-3),env(safe-area-inset-bottom))] md:m-6 animate-slide-up-in',
    end: 'w-[min(100%,24rem)] mt-[max(var(--space-3),env(safe-area-inset-top))] mx-[max(var(--space-3),env(safe-area-inset-right))] mb-3 animate-slide-up-drawer'
  } as const satisfies Record<NonNullable<Props['placement']>, string>;

  let root: HTMLDivElement | undefined = $state();
  let panel: HTMLDivElement | undefined = $state();

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
  >
    {@render children()}
  </div>
</div>

