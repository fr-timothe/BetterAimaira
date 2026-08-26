<script lang="ts">
  import { AlertCircle, BarChart3, EyeOff, ShieldCheck, UserX } from 'lucide-svelte';
  import Button from '$lib/components/ui/Button.svelte';
  import * as m from '$lib/paraglide/messages.js';
  import type { Locale } from '$lib/paraglide/runtime.js';

  type Props = {
    /** Paraglide reads the locale at call time, so re-derive the copy on change. */
    locale: Locale;
    /** True while the answer is being written, and both buttons wait for it. */
    saving: boolean;
    /** Set when the answer could not be recorded, so the panel stays put. */
    failed: boolean;
    onChoose: (enabled: boolean) => void;
  };

  const { locale, saving, failed, onChoose }: Props = $props();

  const copy = $derived.by(() => {
    locale;
    return {
      title: m.onboarding_analytics_title(),
      description: m.onboarding_analytics_description(),
      anonymousTitle: m.onboarding_analytics_anonymous_title(),
      anonymousDescription: m.onboarding_analytics_anonymous_description(),
      scopeTitle: m.onboarding_analytics_scope_title(),
      scopeDescription: m.onboarding_analytics_scope_description(),
      neverTitle: m.onboarding_analytics_never_title(),
      neverDescription: m.onboarding_analytics_never_description(),
      accept: m.onboarding_analytics_accept(),
      decline: m.onboarding_analytics_decline(),
      footnote: m.onboarding_analytics_footnote(),
      error: m.onboarding_analytics_error()
    };
  });

  const points = $derived([
    { icon: UserX, title: copy.anonymousTitle, description: copy.anonymousDescription },
    { icon: BarChart3, title: copy.scopeTitle, description: copy.scopeDescription },
    { icon: EyeOff, title: copy.neverTitle, description: copy.neverDescription }
  ]);
</script>

<header class="grid gap-2">
  <span class="flex size-10 items-center justify-center rounded-md bg-muted text-primary-deep">
    <ShieldCheck size={20} aria-hidden="true" />
  </span>
  <h1 class="mt-1 text-2xl leading-[1.2] font-extrabold tracking-[-0.01em] text-foreground">
    {copy.title}
  </h1>
  <p class="text-base leading-[1.55] text-pretty text-muted-foreground">{copy.description}</p>
</header>

<ul class="grid gap-4">
  {#each points as point (point.title)}
    {@const Icon = point.icon}
    <li class="flex items-start gap-3">
      <span class="grid size-10 flex-none place-items-center rounded-md bg-muted text-primary-deep">
        <Icon size={20} aria-hidden="true" />
      </span>
      <div class="grid min-w-0 gap-1">
        <span class="text-base font-bold text-foreground">{point.title}</span>
        <span class="text-sm leading-[1.5] text-muted-foreground">{point.description}</span>
      </div>
    </li>
  {/each}
</ul>

{#if failed}
  <div
    class="flex items-start gap-[0.65rem] rounded-sm bg-danger-surface px-[0.95rem]
           py-[0.85rem] text-base leading-[1.45] text-danger-strong"
    role="alert"
  >
    <AlertCircle class="mt-[0.05rem] flex-none" size={19} aria-hidden="true" />
    <span>{copy.error}</span>
  </div>
{/if}

<div class="grid gap-2">
  <!-- Refusing is a button of its own, never a link or a skip: the reader has to
       be able to say no in one tap, without reading which of two labels agrees. -->
  <Button variant="ink" size="lg" block loading={saving} onclick={() => onChoose(true)}>
    {copy.accept}
  </Button>
  <Button variant="outline" size="lg" block disabled={saving} onclick={() => onChoose(false)}>
    {copy.decline}
  </Button>
  <p class="text-sm leading-[1.5] text-muted-foreground">{copy.footnote}</p>
</div>
