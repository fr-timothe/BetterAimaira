<script lang="ts">
  import { schoolInitials, schoolLogoUrl, type School } from '$lib/data/schools';
  import { cn } from '$lib/utils';

  type Props = {
    school: School;
    class?: string;
  };

  const { school, class: className }: Props = $props();

  // The logos are served by the site, not bundled, so every one of them is
  // allowed to fail: no network, a blocked host, a file that has not been
  // published yet. The initials are the drawing until one arrives, and stay if
  // none does — a card with an empty hole in it is worse than no logo at all.
  let failed = $state(false);

  const source = $derived(schoolLogoUrl(school));

  $effect(() => {
    source;
    failed = false;
  });
</script>

<span
  class={cn(
    'grid h-14 w-full place-items-center overflow-hidden rounded-md bg-white px-2',
    className
  )}
>
  {#if failed}
    <span class="text-md font-extrabold tracking-[0.02em] text-secondary">
      {schoolInitials(school)}
    </span>
  {:else}
    <img
      class="max-h-12 w-auto max-w-full object-contain"
      src={source}
      alt=""
      loading="lazy"
      decoding="async"
      onerror={() => (failed = true)}
    />
  {/if}
</span>
