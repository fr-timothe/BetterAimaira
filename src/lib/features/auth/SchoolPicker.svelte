<script lang="ts">
  import { Link2, Search, WifiOff, X } from 'lucide-svelte';
  import Logo from '$lib/assets/Logo.svelte';
  import Button from '$lib/components/ui/Button.svelte';
  import SchoolLogo from './SchoolLogo.svelte';
  import { searchSchools, type School } from '$lib/data/schools';
  import * as m from '$lib/paraglide/messages.js';
  import type { Locale } from '$lib/paraglide/runtime.js';
  import { connectivity } from '$lib/state/connectivity.svelte';
  import { cn } from '$lib/utils';

  type Props = {
    /** Paraglide reads the locale at call time, so re-derive the copy on change. */
    locale: Locale;
    /** The school the reader picked, portal address known or not. */
    onSelect: (school: School) => void;
    /** Skips the list entirely and opens the form on the address field. */
    onManual: () => void;
  };

  const { locale, onSelect, onManual }: Props = $props();

  let query = $state('');
  let field = $state<HTMLInputElement | null>(null);

  const results = $derived(searchSchools(query));

  const copy = $derived.by(() => {
    locale;
    return {
      appName: m.app_name(),
      title: m.school_title(),
      description: m.school_description(),
      searchLabel: m.school_search_label(),
      searchPlaceholder: m.school_search_placeholder(),
      clear: m.school_clear(),
      noPortal: m.school_no_portal(),
      sso: m.school_login_sso(),
      emptyTitle: m.school_empty_title(),
      emptyDescription: m.school_empty_description(),
      manual: m.school_manual(),
      offlineTitle: m.school_offline_title(),
      offlineDescription: m.school_offline_description()
    };
  });

  const countLabel = $derived.by(() => {
    locale;
    const count = results.length;
    return count === 1 ? m.school_count_one({ count }) : m.school_count_other({ count });
  });

  /**
   * Only what changes what the reader has to do next. The group a school
   * inherits its address from is true but idle here — it belongs on the site's
   * compatibility page, not on a card being scanned for a name.
   */
  function badge(school: School): string | null {
    if (!school.portalUrl) return copy.noPortal;
    if (school.portalLogin === 'sso') return copy.sso;
    return null;
  }
</script>

<!-- The picker owns the whole frame like the introduction before it: the login
     form is never painted under a reader who has not named their school yet. -->
<main
  class="flex min-h-full grow flex-col bg-background
         px-[max(1.25rem,5vw)] pt-[max(2rem,var(--safe-top))]
         pb-[max(2rem,var(--safe-bottom))]"
>
  <div class="mx-auto flex w-[min(100%,54rem)] min-h-0 grow flex-col gap-5">
    <div class="flex items-center gap-3 text-lg font-bold text-foreground">
      <Logo size={34} variant="icon" />
      <span>{copy.appName}</span>
    </div>

    <header class="grid gap-2">
      <h1
        class="text-[clamp(var(--text-2xl),5vw,var(--text-3xl))] leading-[1.15] font-extrabold
               tracking-[-0.02em] text-balance text-foreground"
      >
        {copy.title}
      </h1>
      <p class="max-w-[62ch] text-base leading-[1.55] text-pretty text-muted-foreground">
        {copy.description}
      </p>
    </header>

    {#if !connectivity.online}
      <!-- Worth saying here rather than on the form: without a network the
           logos below are blank squares and the sign-in that follows cannot
           succeed either, so the blank grid is not the reader's mistake. -->
      <div
        class="flex items-start gap-[0.65rem] rounded-md bg-danger-surface px-[0.95rem] py-[0.85rem]
               text-base leading-[1.45] text-danger-strong"
        role="status"
      >
        <WifiOff class="mt-[0.15rem] flex-none" size={19} aria-hidden="true" />
        <span class="grid gap-1">
          <strong class="font-bold">{copy.offlineTitle}</strong>
          <span>{copy.offlineDescription}</span>
        </span>
      </div>
    {/if}

    <div class="grid gap-2">
      <label class="text-sm font-bold" for="school-search">{copy.searchLabel}</label>
      <div
        class="flex min-h-[3.1rem] items-center rounded-md border border-border bg-input pl-[0.85rem]
               text-muted-foreground transition-[border-color,box-shadow] duration-fast ease-out
               focus-within:border-ring focus-within:shadow-[0_0_0_2px_var(--ring)]"
      >
        <Search size={19} aria-hidden="true" />
        <input
          class="min-w-0 flex-1 self-stretch bg-transparent px-[0.8rem] py-3 text-foreground
                 outline-0 placeholder:text-muted-foreground"
          id="school-search"
          type="search"
          autocomplete="off"
          autocapitalize="none"
          spellcheck="false"
          enterkeyhint="search"
          placeholder={copy.searchPlaceholder}
          bind:this={field}
          bind:value={query}
        />
        {#if query}
          <button
            class="mr-[0.15rem] grid size-(--tap-min) flex-none place-items-center rounded-sm
                   bg-transparent text-muted-foreground transition-control hover:bg-muted
                   hover:text-foreground active:scale-(--press-scale)"
            type="button"
            aria-label={copy.clear}
            title={copy.clear}
            onclick={() => {
              query = '';
              field?.focus();
            }}
          >
            <X size={18} />
          </button>
        {/if}
      </div>
      <p class="text-xs text-muted-foreground" aria-live="polite">{countLabel}</p>
    </div>

    {#if results.length === 0}
      <div
        class="grid content-center justify-items-center gap-3 rounded-xl border border-border-subtle
               bg-card px-5 py-10 text-center"
      >
        <h2 class="max-w-[32ch] text-lg leading-[1.25] font-bold text-balance">
          {copy.emptyTitle}
        </h2>
        <p class="max-w-[46ch] text-base leading-[1.55] text-pretty text-muted-foreground">
          {copy.emptyDescription}
        </p>
      </div>
    {:else}
      <ul
        class="grid min-h-0 grow auto-rows-min gap-3 overflow-y-auto
               [grid-template-columns:repeat(auto-fill,minmax(9.5rem,1fr))]"
      >
        {#each results as school (school.id)}
          <li>
            <button
              class={cn(
                'flex h-full w-full flex-col items-start gap-2 rounded-xl border border-border-subtle',
                'bg-card p-3 text-left transition-control',
                'hover:border-primary-deep active:scale-(--press-scale)',
                'focus-visible:outline-3 focus-visible:outline-offset-2 focus-visible:outline-ring'
              )}
              type="button"
              onclick={() => onSelect(school)}
            >
              <SchoolLogo {school} />
              <span class="text-sm leading-[1.35] font-bold text-pretty text-foreground">
                {school.name}
              </span>
              {#if badge(school)}
                {@const label = badge(school)}
                <span
                  class={cn(
                    'mt-auto w-fit rounded-full px-2 py-[0.15rem] text-xs font-bold',
                    school.portalLogin === 'sso'
                      ? 'bg-danger-surface text-danger-strong'
                      : 'bg-surface-sunken text-muted-foreground'
                  )}
                >
                  {label}
                </span>
              {/if}
            </button>
          </li>
        {/each}
      </ul>
    {/if}

    <Button variant="outline" size="lg" block onclick={onManual}>
      <Link2 size={18} aria-hidden="true" />
      {copy.manual}
    </Button>
  </div>
</main>
