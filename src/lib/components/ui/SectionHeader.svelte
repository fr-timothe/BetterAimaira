<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { IconComponent } from './icon';
  import { cn } from '$lib/utils';

  type Props = {
    title: string;
    subtitle?: string;
    /** Lucide icon, shown in a tinted plate. */
    icon?: IconComponent;
    /** Heading level. Pick the one the document outline needs, not a look. */
    level?: 2 | 3;
    /** Right-aligned controls: a refresh button, a count, a link. */
    actions?: Snippet;
    class?: string;
  };

  const { title, subtitle, icon, level = 2, actions, class: className }: Props = $props();

  const Icon = $derived(icon);

  // `leading-*` has to follow the size: tailwind-merge treats `text-lg` as a
  // size-and-leading shorthand, so a later `text-lg` drops an earlier leading.
  const heading = 'font-bold wrap-anywhere';
</script>

<div class={cn('ui-section-header flex min-w-0 items-center gap-3', className)}>
  {#if Icon}
    <span
      class="grid size-9 flex-none place-items-center rounded-sm bg-muted text-primary-deep"
      aria-hidden="true"><Icon size={18} /></span
    >
  {/if}

  <div class="min-w-0 flex-1">
    {#if level === 2}
      <h2 class={cn(heading, 'text-lg', 'leading-[1.25]')}>{title}</h2>
    {:else}
      <h3 class={cn(heading, 'text-md', 'leading-[1.25]')}>{title}</h3>
    {/if}
    {#if subtitle}
      <p class="mt-[0.1rem] text-sm leading-[1.45] text-muted-foreground">{subtitle}</p>
    {/if}
  </div>

  {#if actions}
    <div class="flex flex-none items-center gap-2">{@render actions()}</div>
  {/if}
</div>
