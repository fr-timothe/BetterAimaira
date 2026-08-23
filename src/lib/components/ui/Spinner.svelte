<script lang="ts">
  import { LoaderCircle } from 'lucide-svelte';
  import { cn } from '$lib/utils';

  type Props = {
    size?: number;
    /** Screen-reader name. Omit only when an ancestor already announces the wait. */
    label?: string;
    class?: string;
  };

  const { size = 18, label, class: className }: Props = $props();
</script>

<LoaderCircle
  {size}
  class={cn('ui-spinner flex-none animate-spin', className)}
  aria-hidden={label ? undefined : 'true'}
  aria-label={label}
  role={label ? 'img' : undefined}
/>
