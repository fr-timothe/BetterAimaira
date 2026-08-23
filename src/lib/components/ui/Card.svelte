<script lang="ts">
  import type { Snippet } from 'svelte';

  type Props = {
    children: Snippet;
    /**
     * `plain` is the default panel. `sunken` recedes for nested content — the
     * only nesting a card is allowed, since a card inside a card is always wrong.
     * `ink` is the one emphatic surface, reserved for the current course.
     */
    tone?: 'plain' | 'sunken' | 'ink';
    /** Interactive cards get hover feedback and a pointer. */
    interactive?: boolean;
    padding?: 'none' | 'sm' | 'md' | 'lg';
  };

  const { children, tone = 'plain', interactive = false, padding = 'md' }: Props = $props();
</script>

<div class="ui-card {tone} pad-{padding}" class:interactive>
  {@render children()}
</div>

<style>
  /* Elevation is declared once. A plain card is a border; only the ink surface
     and interactive hover reach for a shadow. */
  .ui-card {
    min-width: 0;
    border-radius: var(--radius-xl);
  }

  .plain {
    background: var(--card);
    border: 1px solid var(--border-subtle);
  }

  .sunken {
    background: var(--surface-sunken);
  }

  .ink {
    color: var(--secondary-foreground);
    background: var(--secondary);
    box-shadow: var(--shadow-lg);
  }

  .pad-none { padding: 0; }
  .pad-sm { padding: var(--space-3); }
  .pad-md { padding: var(--space-4); }
  .pad-lg { padding: var(--space-5); }

  .interactive {
    cursor: pointer;
    transition:
      border-color var(--duration-fast) var(--ease-out),
      box-shadow var(--duration-fast) var(--ease-out),
      transform var(--duration-instant) var(--ease-out);
  }

  .interactive:hover {
    border-color: var(--primary-deep);
    box-shadow: var(--shadow-sm);
  }

  .interactive:active {
    transform: scale(0.995);
  }
</style>
