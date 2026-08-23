<script lang="ts">
  import type { Snippet } from 'svelte';
  import { cn } from '$lib/utils';

  type Props = {
    children: Snippet;
    class?: string;
  };

  const { children, class: className }: Props = $props();
</script>

<!-- The root padding recipe every view shares. Declared once here so the five
     copies cannot drift apart, and so the bottom gap always clears the mobile
     dock and the home indicator. -->
<div
  class={cn(
    'ui-page-shell flex w-full min-w-0 flex-col gap-4 px-4 pt-4 pb-6',
    'md:gap-5 md:px-6 md:pt-5',
    className
  )}
>
  {@render children()}
</div>
