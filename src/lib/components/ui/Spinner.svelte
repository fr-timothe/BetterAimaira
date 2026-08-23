<script lang="ts">
  import { LoaderCircle } from 'lucide-svelte';

  type Props = {
    size?: number;
    /** Screen-reader name. Omit only when an ancestor already announces the wait. */
    label?: string;
  };

  const { size = 18, label }: Props = $props();
</script>

<LoaderCircle
  {size}
  class="ui-spinner"
  aria-hidden={label ? undefined : 'true'}
  aria-label={label}
  role={label ? 'img' : undefined}
/>

<style>
  :global(.ui-spinner) {
    flex: 0 0 auto;
    animation: spin var(--duration-spin) linear infinite;
  }
</style>
