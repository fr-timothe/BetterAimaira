<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { IconComponent } from './icon';

  type Props = {
    title: string;
    subtitle?: string;
    /** Lucide icon, shown in a tinted plate. */
    icon?: IconComponent;
    /** Heading level. Pick the one the document outline needs, not a look. */
    level?: 2 | 3;
    /** Right-aligned controls: a refresh button, a count, a link. */
    actions?: Snippet;
  };

  const { title, subtitle, icon, level = 2, actions }: Props = $props();

  const Icon = $derived(icon);
</script>

<div class="ui-section-header">
  {#if Icon}
    <span class="section-plate" aria-hidden="true"><Icon size={18} /></span>
  {/if}

  <div class="section-text">
    {#if level === 2}
      <h2>{title}</h2>
    {:else}
      <h3>{title}</h3>
    {/if}
    {#if subtitle}<p>{subtitle}</p>{/if}
  </div>

  {#if actions}
    <div class="section-actions">{@render actions()}</div>
  {/if}
</div>

<style>
  .ui-section-header {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    min-width: 0;
  }

  .section-plate {
    display: grid;
    width: 2.25rem;
    height: 2.25rem;
    flex: 0 0 2.25rem;
    place-items: center;
    color: var(--primary-deep);
    background: var(--muted);
    border-radius: var(--radius-sm);
  }

  .section-text {
    min-width: 0;
    flex: 1;
  }

  h2,
  h3 {
    margin: 0;
    font-size: var(--text-lg);
    font-weight: var(--weight-bold);
    line-height: 1.25;
    overflow-wrap: anywhere;
  }

  h3 {
    font-size: var(--text-md);
  }

  p {
    margin: 0.1rem 0 0;
    color: var(--muted-foreground);
    font-size: var(--text-sm);
    line-height: 1.45;
  }

  .section-actions {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    flex: 0 0 auto;
  }
</style>
