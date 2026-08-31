<script lang="ts">
  import type { Snippet } from 'svelte';
  import { cn } from '$lib/utils';

  type Props = {
    children: Snippet;
    /** `center` places one card in the middle of the frame, `stack` fills it top-down. */
    layout?: 'center' | 'stack';
    class?: string;
  };

  const { children, layout = 'center', class: className }: Props = $props();

  const layouts = {
    center: 'grid place-items-center',
    stack: 'flex flex-col'
  } as const satisfies Record<NonNullable<Props['layout']>, string>;
</script>

<!-- The frame a full-frame screen shares — startup, the school picker, the
     introduction. Two things are declared once here rather than in each screen:
     it scrolls on its own, because the window shell above it does not, and it
     pays the system insets on all four edges, so a status bar, a gesture pill
     or a landscape cutout never lands on the content. -->
<main
  class={cn(
    'ui-screen-shell min-h-full grow overflow-y-auto bg-background px-screen py-safe-8',
    layouts[layout],
    className
  )}
>
  {@render children()}
</main>
