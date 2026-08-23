<script lang="ts">
  import type { Snippet } from 'svelte';

  type Props = {
    children: Snippet;
    /**
     * Status tones carry meaning, so each pairs a tinted surface with a text
     * tone that clears 4.5:1 on it. Never let the tone be the only signal —
     * the label has to say it too.
     */
    tone?: 'neutral' | 'accent' | 'success' | 'warning' | 'danger' | 'live';
    /** Small leading dot for live/attention states. */
    dot?: boolean;
  };

  const { children, tone = 'neutral', dot = false }: Props = $props();
</script>

<span class="ui-badge {tone}">
  {#if dot}<span class="dot" aria-hidden="true"></span>{/if}
  {@render children()}
</span>

<style>
  .ui-badge {
    display: inline-flex;
    align-items: center;
    gap: var(--space-1);
    padding: 0.2rem var(--space-2);
    border-radius: var(--radius-pill);
    font-size: var(--text-2xs);
    font-weight: var(--weight-bold);
    line-height: 1.5;
    white-space: nowrap;
  }

  .dot {
    width: 0.4rem;
    height: 0.4rem;
    border-radius: 50%;
    background: currentColor;
  }

  .neutral {
    color: var(--category-other-text);
    background: var(--category-other-surface);
  }

  .accent {
    color: var(--primary-deep);
    background: var(--muted);
  }

  .success {
    color: var(--success-strong);
    background: var(--success-surface);
  }

  .warning {
    color: var(--warning-strong);
    background: var(--warning-surface);
  }

  .danger {
    color: var(--danger-strong);
    background: var(--danger-surface);
  }

  .live {
    color: var(--card);
    background: var(--primary-deep);
  }

  .live .dot {
    animation: pulse-beacon 1.6s var(--ease-in-out) infinite;
  }
</style>
