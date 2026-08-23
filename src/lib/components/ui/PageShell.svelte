<script lang="ts">
  import type { Snippet } from 'svelte';

  type Props = {
    children: Snippet;
  };

  const { children }: Props = $props();
</script>

<div class="ui-page-shell">
  {@render children()}
</div>

<style>
  /* The root padding recipe every view shares. Declared once here so the five
     copies cannot drift apart, and so the bottom gap always clears the mobile
     dock and the home indicator. */
  .ui-page-shell {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
    width: 100%;
    min-width: 0;
    padding: var(--space-4) var(--space-4) var(--space-6);
    box-sizing: border-box;
  }

  @media (min-width: 48rem) {
    .ui-page-shell {
      gap: var(--space-5);
      padding: var(--space-5) var(--space-6) var(--space-6);
    }
  }
</style>
