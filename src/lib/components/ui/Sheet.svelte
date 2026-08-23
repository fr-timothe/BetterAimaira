<script lang="ts">
  import type { Snippet } from 'svelte';

  type Props = {
    children: Snippet;
    /** Accessible name for the dialog. */
    title: string;
    /** `center` rises from the bottom on small screens; `end` slides in from the inline end. */
    placement?: 'center' | 'end';
    /** Label for the backdrop's own dismiss control. */
    closeLabel: string;
    onClose: () => void;
  };

  const { children, title, placement = 'center', closeLabel, onClose }: Props = $props();

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

<div bind:this={root} class="ui-sheet-root {placement}" onkeydown={handleKeyDown} role="presentation">
  <button type="button" class="sheet-backdrop" aria-label={closeLabel} onclick={onClose}></button>

  <div
    bind:this={panel}
    class="sheet-panel"
    role="dialog"
    aria-modal="true"
    aria-label={title}
    tabindex="-1"
  >
    {@render children()}
  </div>
</div>

<style>
  .ui-sheet-root {
    position: fixed;
    inset: 0;
    z-index: var(--z-modal);
    display: grid;
    overscroll-behavior: contain;
  }

  .sheet-backdrop {
    position: absolute;
    inset: 0;
    background: var(--surface-overlay);
    backdrop-filter: blur(8px);
    border: 0;
    cursor: default;
    animation: fade-in var(--duration-fast) var(--ease-out);
  }

  .sheet-panel {
    position: relative;
    z-index: 1;
    display: flex;
    flex-direction: column;
    max-height: calc(100dvh - 2 * var(--space-6));
    overflow-y: auto;
    overscroll-behavior: contain;
    background: var(--card);
    border-radius: var(--radius-xl);
    box-shadow: var(--shadow-xl);
  }

  .sheet-panel:focus-visible {
    outline: 3px solid var(--ring);
    outline-offset: -3px;
  }

  .center {
    place-items: end center;
  }

  .center .sheet-panel {
    width: min(100%, 34rem);
    margin: 0 var(--space-3) max(var(--space-3), env(safe-area-inset-bottom));
    animation: slide-up-in var(--duration-normal) var(--ease-out);
  }

  .end {
    place-items: start end;
  }

  .end .sheet-panel {
    width: min(100%, 24rem);
    margin: max(var(--space-3), env(safe-area-inset-top)) max(var(--space-3), env(safe-area-inset-right))
      var(--space-3);
    animation: slide-up-in var(--duration-normal) var(--ease-drawer);
  }

  @media (min-width: 48rem) {
    .center {
      place-items: center;
    }

    .center .sheet-panel {
      margin: var(--space-6);
    }
  }
</style>
