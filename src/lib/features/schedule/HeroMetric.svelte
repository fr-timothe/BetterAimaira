<script lang="ts">
  import type { Snippet } from 'svelte';

  type Props = {
    title: string;
    /** Already formatted with the caller's locale. */
    value: string;
    /** Trails the value: `/20`, `h`. */
    unit?: string;
    /** Status tone. The title always says it too — colour is never the signal. */
    tone?: 'neutral' | 'success' | 'warning' | 'danger';
    /** Small leading icon for the title. */
    icon?: Snippet;
  };

  const { title, value, unit, tone = 'neutral', icon }: Props = $props();
</script>

<div class="hero-metric {tone}">
  <span class="metric-title">
    {#if icon}
      <span class="metric-icon" aria-hidden="true">
        {@render icon()}
      </span>
    {/if}
    <span class="metric-title-text">{title}</span>
  </span>
  <span class="metric-value">
    {value}{#if unit}<small>{unit}</small>{/if}
  </span>
</div>

<style>
  .hero-metric {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    text-align: center;
    gap: 0.25rem;
    padding: var(--space-2-5) var(--space-2);
    background: var(--card);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-lg);
    min-width: 0;
    width: 100%;
    box-shadow: var(--shadow-xs);
    transition:
      border-color var(--duration-fast) var(--ease-out),
      background-color var(--duration-fast) var(--ease-out);
  }

  .metric-title {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-1);
    color: var(--muted-foreground);
    font-size: var(--text-xs);
    font-weight: var(--weight-medium);
    line-height: 1.25;
    text-align: center;
    width: 100%;
    min-width: 0;
  }

  .metric-icon {
    display: inline-flex;
    flex-shrink: 0;
    align-items: center;
    justify-content: center;
    color: inherit;
  }

  .metric-title-text {
    min-width: 0;
    overflow-wrap: break-word;
    word-break: normal;
    text-wrap: balance;
  }

  .metric-value {
    display: inline-flex;
    align-items: baseline;
    justify-content: center;
    gap: 0.1rem;
    color: var(--foreground);
    font-size: var(--text-lg);
    font-variant-numeric: tabular-nums;
    font-weight: var(--weight-bold);
    line-height: 1.2;
  }

  .metric-value small {
    color: var(--muted-foreground);
    font-size: var(--text-xs);
    font-weight: var(--weight-medium);
  }

  /* Tone styling: clean, accessible, elegant with subtle tint and crisp values */
  .hero-metric.success {
    background: color-mix(in oklch, var(--success-surface) 45%, var(--card));
    border-color: color-mix(in oklch, var(--success) 30%, var(--border-subtle));
  }
  .hero-metric.success .metric-icon {
    color: var(--success-strong);
  }
  .hero-metric.success .metric-value {
    color: var(--success-strong);
  }

  .hero-metric.warning {
    background: color-mix(in oklch, var(--warning-surface) 45%, var(--card));
    border-color: color-mix(in oklch, var(--warning) 30%, var(--border-subtle));
  }
  .hero-metric.warning .metric-icon {
    color: var(--warning-strong);
  }
  .hero-metric.warning .metric-value {
    color: var(--warning-strong);
  }

  .hero-metric.danger {
    background: color-mix(in oklch, var(--danger-surface) 45%, var(--card));
    border-color: color-mix(in oklch, var(--danger) 30%, var(--border-subtle));
  }
  .hero-metric.danger .metric-icon {
    color: var(--danger-strong);
  }
  .hero-metric.danger .metric-value {
    color: var(--danger-strong);
  }
</style>
