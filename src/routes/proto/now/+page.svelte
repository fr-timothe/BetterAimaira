<script lang="ts">
  /**
   * DEV-ONLY PROTOTYPE ROUTE. The dynamic import is what keeps the whole
   * `proto` tree out of a production bundle: `import.meta.env.DEV` is replaced
   * by `false` at build time, so the branch and everything it reaches are
   * dropped.
   */
  import { onMount } from 'svelte';

  let View = $state<any>(null);

  onMount(async () => {
    if (!import.meta.env.DEV) return;
    View = (await import('../NowView.svelte')).default;
  });
</script>

<svelte:head><title>Prototype — Maintenant d’abord</title></svelte:head>

{#if View}
  <View />
{/if}
