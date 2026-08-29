<script lang="ts">
  import { onMount } from 'svelte';
  import {
    AlertCircle,
    ArrowLeft,
    ArrowRight,
    CalendarDays,
    Check,
    CircleAlert,
    DownloadCloud,
    ExternalLink,
    GraduationCap,
    RefreshCw
  } from 'lucide-svelte';
  import Logo from '$lib/assets/Logo.svelte';
  import Button from '$lib/components/ui/Button.svelte';
  import * as m from '$lib/paraglide/messages.js';
  import type { Locale } from '$lib/paraglide/runtime.js';
  import { cn } from '$lib/utils';
  import AnalyticsConsentPanel from './AnalyticsConsentPanel.svelte';
  import { onboarding } from './onboarding.svelte';
  import type { PermissionKind } from './permissions-service';

  type Props = {
    /** Paraglide reads the locale at call time, so re-derive the copy on change. */
    locale: Locale;
    /** Ends the introduction and hands the reader to the login form. */
    onDone: () => void;
  };

  const { locale, onDone }: Props = $props();

  /** Set when the reporting answer could not be written, so the panel stays. */
  let consentFailed = $state(false);

  const copy = $derived.by(() => {
    locale;
    return {
      appName: m.app_name(),
      welcomeTitle: m.onboarding_welcome_title(),
      welcomeDescription: m.onboarding_welcome_description(),
      scheduleTitle: m.onboarding_feature_schedule_title(),
      scheduleDescription: m.onboarding_feature_schedule_description(),
      gradesTitle: m.onboarding_feature_grades_title(),
      gradesDescription: m.onboarding_feature_grades_description(),
      updatesTitle: m.onboarding_feature_updates_title(),
      updatesDescription: m.onboarding_feature_updates_description(),
      permissionsTitle: m.onboarding_permissions_title(),
      permissionsDescription: m.onboarding_permissions_description(),
      installTitle: m.onboarding_permission_install_title(),
      installDescription: m.onboarding_permission_install_description(),
      granted: m.onboarding_permission_granted(),
      missing: m.onboarding_permission_missing(),
      grant: m.onboarding_permission_grant(),
      recheck: m.onboarding_permission_recheck(),
      pending: m.onboarding_permission_pending(),
      required: m.onboarding_permissions_required(),
      errorCheck: m.onboarding_permission_error_check(),
      errorRequest: m.onboarding_permission_error_request(),
      errorUnavailable: m.onboarding_permission_error_unavailable(),
      continueLabel: m.onboarding_continue(),
      back: m.onboarding_back(),
      start: m.onboarding_start()
    };
  });

  const features = $derived([
    { icon: CalendarDays, title: copy.scheduleTitle, description: copy.scheduleDescription },
    { icon: GraduationCap, title: copy.gradesTitle, description: copy.gradesDescription },
    { icon: DownloadCloud, title: copy.updatesTitle, description: copy.updatesDescription }
  ]);

  const permissionCopy = $derived<Record<PermissionKind, { title: string; description: string }>>({
    installPackages: { title: copy.installTitle, description: copy.installDescription }
  });

  const errorMessage = $derived.by(() => {
    switch (onboarding.errorCode) {
      case 'permission_screen_unavailable':
        return copy.errorUnavailable;
      case 'permission_request_failed':
        return copy.errorRequest;
      case 'permission_check_failed':
      case 'unknown':
        return copy.errorCheck;
      default:
        return '';
    }
  });

  const onPermissions = $derived(onboarding.step === 'permissions');
  const onAnalytics = $derived(onboarding.step === 'analytics');

  onMount(() => {
    void onboarding.load();

    // Granting happens in the settings app, so the answer only ever arrives when
    // this window comes back: that is the moment to re-read the rights.
    const refresh = () => {
      if (document.visibilityState === 'visible') void onboarding.refresh();
    };
    document.addEventListener('visibilitychange', refresh);
    return () => document.removeEventListener('visibilitychange', refresh);
  });

  function advance() {
    if (onboarding.onLastStep) {
      finish();
      return;
    }
    onboarding.next();
  }

  // The state refuses to end while a right is still missing, so the reader is
  // only handed to the login form once it says the introduction is over.
  function finish() {
    if (onboarding.finish()) onDone();
  }

  // Either answer ends the introduction; only a failure to record one keeps the
  // reader here, since carrying on would leave the question silently unanswered.
  async function chooseAnalytics(enabled: boolean) {
    consentFailed = false;
    if (await onboarding.chooseAnalytics(enabled)) {
      finish();
      return;
    }
    consentFailed = true;
  }
</script>

<!-- The introduction owns the whole frame, exactly like the restore screen: the
     login form is never painted underneath a reader who has not started yet. -->
<main
  class="grid min-h-full grow place-items-center overflow-y-auto bg-background
         px-[max(1.5rem,7vw)] pt-[max(2rem,var(--safe-top))]
         pb-[max(2rem,var(--safe-bottom))]"
>
  <div class="grid w-[min(100%,30rem)] gap-6">
    <div class="flex items-center gap-3 text-lg font-bold text-foreground">
      <Logo size={34} variant="icon" />
      <span>{copy.appName}</span>
    </div>

    {#if onboarding.steps.length > 1}
      {@const reached = onboarding.steps.indexOf(onboarding.step)}
      <ol class="flex items-center gap-2" aria-hidden="true">
        <!-- Every bar behind the reader stays lit: it reads as progress made,
             not as the panel currently open. -->
        {#each onboarding.steps as step, index (step)}
          <li
            class={cn(
              'h-1 flex-1 rounded-full transition-control',
              index <= reached ? 'bg-primary-deep' : 'bg-muted'
            )}
          ></li>
        {/each}
      </ol>
    {/if}

    {#if onAnalytics}
      <AnalyticsConsentPanel
        {locale}
        saving={onboarding.savingConsent}
        failed={consentFailed}
        onChoose={chooseAnalytics}
      />
    {:else if onPermissions}
      <header class="grid gap-2">
        <h1 class="text-2xl leading-[1.2] font-extrabold tracking-[-0.01em] text-foreground">
          {copy.permissionsTitle}
        </h1>
        <p class="text-base leading-[1.55] text-pretty text-muted-foreground">
          {copy.permissionsDescription}
        </p>
      </header>

      <ul class="grid gap-3">
        {#each onboarding.permissions as permission (permission.kind)}
          <li
            class="grid gap-3 rounded-xl border border-border-subtle bg-card p-4"
          >
            <div class="flex items-start gap-3">
              <span
                class={cn(
                  'grid size-10 flex-none place-items-center rounded-md',
                  permission.granted
                    ? 'bg-success-surface text-success-strong'
                    : 'bg-muted text-primary-deep'
                )}
              >
                {#if permission.granted}
                  <Check size={20} aria-hidden="true" />
                {:else}
                  <CircleAlert size={20} aria-hidden="true" />
                {/if}
              </span>
              <div class="grid min-w-0 gap-1">
                <span class="text-base font-bold text-foreground">
                  {permissionCopy[permission.kind].title}
                </span>
                <span class="text-sm leading-[1.5] text-muted-foreground">
                  {permissionCopy[permission.kind].description}
                </span>
                <span
                  class={cn(
                    'mt-1 w-fit rounded-full px-2 py-[0.15rem] text-xs font-bold',
                    permission.granted
                      ? 'bg-success-surface text-success-strong'
                      : 'bg-muted text-muted-foreground'
                  )}
                >
                  {permission.granted ? copy.granted : copy.missing}
                </span>
              </div>
            </div>

            {#if !permission.granted && permission.requestable}
              <div class="grid gap-2">
                <Button variant="outline" block onclick={() => onboarding.request(permission.kind)}>
                  <ExternalLink size={17} aria-hidden="true" />
                  {copy.grant}
                </Button>
                {#if onboarding.pending === permission.kind}
                  <p class="text-sm text-muted-foreground">{copy.pending}</p>
                  <Button variant="ghost" block onclick={() => onboarding.refresh()}>
                    <RefreshCw size={16} aria-hidden="true" />
                    {copy.recheck}
                  </Button>
                {/if}
              </div>
            {/if}
          </li>
        {/each}
      </ul>

      {#if errorMessage}
        <div
          class="flex items-start gap-[0.65rem] rounded-sm bg-danger-surface px-[0.95rem]
                 py-[0.85rem] text-base leading-[1.45] text-danger-strong"
          role="alert"
        >
          <AlertCircle class="mt-[0.05rem] flex-none" size={19} aria-hidden="true" />
          <span>{errorMessage}</span>
        </div>
      {/if}

      <div class="grid gap-2">
        <p class="text-sm leading-[1.5] text-muted-foreground">{copy.required}</p>
        <Button variant="ink" size="lg" block disabled={!onboarding.allGranted} onclick={advance}>
          {onboarding.onLastStep ? copy.start : copy.continueLabel}
          <ArrowRight size={18} aria-hidden="true" />
        </Button>
        <Button variant="ghost" block onclick={() => onboarding.back()}>
          <ArrowLeft size={16} aria-hidden="true" />
          {copy.back}
        </Button>
      </div>
    {:else}
      <header class="grid gap-2">
        <h1
          class="text-[clamp(var(--text-2xl),6vw,var(--text-3xl))] leading-[1.15] font-extrabold
                 tracking-[-0.02em] text-balance text-foreground"
        >
          {copy.welcomeTitle}
        </h1>
        <p class="text-base leading-[1.55] text-pretty text-muted-foreground">
          {copy.welcomeDescription}
        </p>
      </header>

      <ul class="grid gap-4">
        {#each features as feature (feature.title)}
          {@const Icon = feature.icon}
          <li class="flex items-start gap-3">
            <span class="grid size-10 flex-none place-items-center rounded-md bg-muted text-primary-deep">
              <Icon size={20} aria-hidden="true" />
            </span>
            <div class="grid min-w-0 gap-1">
              <span class="text-base font-bold text-foreground">{feature.title}</span>
              <span class="text-sm leading-[1.5] text-muted-foreground">{feature.description}</span>
            </div>
          </li>
        {/each}
      </ul>

      <Button variant="ink" size="lg" block onclick={advance} loading={onboarding.loading}>
        {onboarding.onLastStep ? copy.start : copy.continueLabel}
        <ArrowRight size={18} aria-hidden="true" />
      </Button>
    {/if}
  </div>
</main>
