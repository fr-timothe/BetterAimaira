<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke, isTauri } from '@tauri-apps/api/core';
  import {
    AlertCircle,
    ArrowRight,
    Check,
    Eye,
    EyeOff,
    Globe2,
    Link2,
    LoaderCircle,
    LockKeyhole,
  } from 'lucide-svelte';
  import Logo from '$lib/assets/Logo.svelte';
  import * as m from '$lib/paraglide/messages.js';
  import { getLocale, setLocale, type Locale } from '$lib/paraglide/runtime.js';
  import { clearPortalResourceCache } from '$lib/features/schedule/portal-cache';
  import { cn } from '$lib/utils';

  type ScheduleAppComponent = (typeof import('$lib/features/schedule/ScheduleApp.svelte'))['default'];

  type PortalInfo = { portalUrl: string };
  type LoginResult = {
    portalUrl: string;
    username: string;
    credentialsSaved: boolean;
    sundaysVisible: boolean;
  };
  type ErrorCode =
    | 'invalid_portal_url'
    | 'insecure_portal_url'
    | 'portal_unreachable'
    | 'portal_not_aimaira'
    | 'invalid_credentials'
    | 'missing_credentials'
    | 'credential_store'
    | 'internal_error'
    | 'desktop_required';

  // The field frame is shared by the bare input and the two icon wrappers, so
  // the focus ring lands on the same box in all three.
  const fieldFrame =
    'w-full min-h-[3.1rem] rounded-md border border-border bg-input text-foreground' +
    ' transition-[border-color,box-shadow] duration-fast ease-out';
  const fieldFocus =
    'focus:border-ring focus:shadow-[0_0_0_2px_var(--ring)] focus:outline-0' +
    ' focus-within:border-ring focus-within:shadow-[0_0_0_2px_var(--ring)]';

  const locales: Locale[] = ['fr', 'en'];

  const signalRows = [
    { time: '09', width: 'w-[88%]', fill: 'bg-primary' },
    { time: '11', width: 'w-[63%]', fill: 'bg-course-soft' },
    { time: '14', width: 'w-[74%]', fill: 'bg-course-accent' },
    { time: '16', width: 'w-[42%]', fill: '' }
  ];

  let portalUrl = $state('');
  let username = $state('');
  let password = $state('');
  let remember = $state(true);
  let passwordVisible = $state(false);
  let submitting = $state(false);
  let restoring = $state(false);
  let errorCode = $state<ErrorCode | null>(null);
  let loginResult = $state<LoginResult | null>(null);
  let ScheduleApp = $state<ScheduleAppComponent | null>(null);
  let locale = $state<Locale>(getLocale());
  let now = $state(new Date());

  const signalTime = $derived(
    new Intl.DateTimeFormat(locale, { hour: '2-digit', minute: '2-digit' }).format(now)
  );

  const copy = $derived.by(() => {
    locale;
    return {
      appName: m.app_name(),
      heading: m.login_heading(),
      description: m.login_description(),
      portalLabel: m.portal_label(),
      portalPlaceholder: m.portal_placeholder(),
      portalHint: m.portal_hint(),
      emailLabel: m.email_label(),
      emailPlaceholder: m.email_placeholder(),
      passwordLabel: m.password_label(),
      passwordPlaceholder: m.password_placeholder(),
      showPassword: m.show_password(),
      hidePassword: m.hide_password(),
      rememberLabel: m.remember_label(),
      submit: m.submit(),
      submitting: m.submitting(),
      languageLabel: m.language_label(),
      previewToday: m.preview_today(),
      previewSchedule: m.preview_schedule(),
      successHeading: m.success_heading(),
      successDescription: m.success_description(),
      credentialsNotSaved: m.credentials_not_saved(),
    };
  });

  const errorMessage = $derived.by(() => {
    locale;
    switch (errorCode) {
      case 'invalid_portal_url': return m.error_invalid_portal_url();
      case 'insecure_portal_url': return m.error_insecure_portal_url();
      case 'portal_unreachable': return m.error_portal_unreachable();
      case 'portal_not_aimaira': return m.error_portal_not_aimaira();
      case 'invalid_credentials': return m.error_invalid_credentials();
      case 'missing_credentials': return m.error_missing_credentials();
      case 'credential_store': return m.error_credential_store();
      case 'internal_error': return m.error_internal_error();
      case 'desktop_required': return m.error_desktop_required();
      default: return '';
    }
  });

  onMount(() => {
    const clock = setInterval(() => (now = new Date()), 30_000);
    if (isTauri()) {
      restoring = true;
      void restoreSession();
    }
    return () => clearInterval(clock);
  });

  function extractErrorCode(error: unknown): ErrorCode {
    if (typeof error === 'object' && error !== null && 'code' in error) {
      const code = (error as { code: string }).code;
      return code as ErrorCode;
    }
    return 'internal_error';
  }

  async function cleanPortalUrl() {
    if (!portalUrl.trim() || !isTauri()) return;

    try {
      const result = await invoke<PortalInfo>('normalize_portal_url', { portalUrl });
      portalUrl = result.portalUrl;
      errorCode = null;
    } catch (error) {
      errorCode = extractErrorCode(error);
    }
  }

  async function submit(event: SubmitEvent) {
    event.preventDefault();
    errorCode = null;
    loginResult = null;

    if (!isTauri()) {
      errorCode = 'desktop_required';
      return;
    }

    submitting = true;
    try {
      const result = await invoke<LoginResult>('login', {
        request: { portalUrl, username, password, remember },
      });
      await openAuthenticatedApp(result);
      portalUrl = result.portalUrl;
      password = '';
    } catch (error) {
      errorCode = extractErrorCode(error);
    } finally {
      submitting = false;
    }
  }

  async function restoreSession() {
    try {
      const result = await invoke<LoginResult | null>('restore_session');
      if (result) {
        await openAuthenticatedApp(result);
        portalUrl = result.portalUrl;
      }
    } catch (error) {
      errorCode = extractErrorCode(error);
    } finally {
      restoring = false;
    }
  }

  async function openAuthenticatedApp(result: LoginResult) {
    ScheduleApp ??= (await import('$lib/features/schedule/ScheduleApp.svelte')).default;
    loginResult = result;
  }

  async function changeLocale(nextLocale: Locale) {
    await setLocale(nextLocale, { reload: false });
    locale = nextLocale;
    document.documentElement.lang = nextLocale;
  }

  async function logout() {
    await invoke('logout');
    clearPortalResourceCache();
    loginResult = null;
    ScheduleApp = null;
    password = '';
  }
</script>

<svelte:head>
  <title>{copy.appName}</title>
</svelte:head>

{#if loginResult && ScheduleApp}
  <ScheduleApp
    username={loginResult.username}
    portalUrl={loginResult.portalUrl}
    {locale}
    credentialsWarning={remember && !loginResult.credentialsSaved}
    sundaysVisible={loginResult.sundaysVisible}
    onLocaleChange={changeLocale}
    onLogout={logout}
  />
{:else}
<main
  class="login-shell grid min-h-full grow grid-cols-[minmax(360px,0.84fr)_minmax(460px,1.16fr)] lte-820:block"
>
  <!-- The signal panel is abstract geometry plus the real clock. It states no
       course, room or name it does not have. -->
  <section
    class="relative flex min-h-full flex-col overflow-hidden bg-secondary text-secondary-foreground
           px-12 pt-[max(2rem,env(safe-area-inset-top))] pb-[max(2rem,env(safe-area-inset-bottom))]
           after:absolute after:right-[-7rem] after:bottom-20 after:size-72 after:rounded-full
           after:border after:border-brand-ring after:content-['']
           lte-820:hidden short-desktop:py-6"
    aria-label={copy.previewSchedule}
  >
    <div class="flex items-center gap-3 text-lg font-bold">
      <Logo size={32} variant="icon" />
      <span>{copy.appName}</span>
    </div>

    <div
      class="relative z-raised my-auto w-[min(100%,31rem)] py-8 short-desktop:py-4"
      aria-hidden="true"
    >
      <div class="flex items-start justify-between border-b border-signal-rule pb-[1.4rem]">
        <div class="flex flex-col gap-1">
          <span class="text-sm font-semibold text-signal-label">{copy.previewToday}</span>
          <strong class="max-w-[14ch] text-2xl leading-[1.15] font-bold">
            {copy.previewSchedule}
          </strong>
        </div>
        <!-- The clock reads the real time \u2014 the rest of the panel is geometry, so
             this is the one place the panel states a fact. -->
        <span
          class="flex items-center gap-2 text-md font-semibold tabular-nums
                 before:size-2 before:rounded-full before:bg-primary before:content-['']"
          >{signalTime}</span
        >
      </div>
      <div class="grid grid-cols-[2.3rem_1fr] gap-x-4 gap-y-[0.65rem] py-6 short-desktop:py-4">
        {#each signalRows as row (row.time)}
          <span class="pt-[0.2rem] text-xs font-bold tabular-nums text-signal-time">{row.time}</span>
          <span
            class={cn(
              'block h-[2.65rem] rounded-sm bg-course-idle short-desktop:h-8',
              row.width,
              row.fill
            )}
          ></span>
        {/each}
      </div>
    </div>
  </section>

  <section
    class="relative grid min-w-0 place-items-center bg-background
           px-[max(2rem,7vw)] pt-[max(5rem,env(safe-area-inset-top))]
           pb-[max(3rem,env(safe-area-inset-bottom))]
           lte-820:min-h-dvh lte-820:items-start
           lte-820:pt-[max(6.5rem,calc(env(safe-area-inset-top)+5rem))]
           lte-820:pr-[max(1.25rem,env(safe-area-inset-right))]
           lte-820:pb-[max(2rem,env(safe-area-inset-bottom))]
           lte-820:pl-[max(1.25rem,env(safe-area-inset-left))]
           lte-600:pr-[max(1rem,env(safe-area-inset-right))]
           lte-600:pl-[max(1rem,env(safe-area-inset-left))]
           short-desktop:pt-[4.5rem] short-desktop:pb-6"
  >
    <div
      class="hidden items-center gap-3 text-lg font-bold
             lte-820:absolute lte-820:top-[max(1.25rem,env(safe-area-inset-top))]
             lte-820:left-[max(1.25rem,env(safe-area-inset-left))] lte-820:flex"
    >
      <Logo size={28} variant="icon" />
      <span class="lte-600:hidden">{copy.appName}</span>
    </div>

    <div
      class="absolute top-[max(1.5rem,env(safe-area-inset-top))]
             right-[max(1.5rem,env(safe-area-inset-right))] flex min-h-(--tap-min) items-center
             gap-[0.2rem] text-muted-foreground
             lte-820:top-[max(1.1rem,env(safe-area-inset-top))]
             lte-820:right-[max(1rem,env(safe-area-inset-right))]"
      aria-label={copy.languageLabel}
    >
      <Globe2 size={17} class="mr-[0.3rem] lte-600:hidden" aria-hidden="true" />
      {#each locales as option (option)}
        <button
          class="min-h-(--tap-min) min-w-(--tap-min) rounded-sm bg-transparent text-xs font-bold
                 text-muted-foreground transition-control hover:bg-muted hover:text-foreground
                 active:scale-(--press-scale) aria-pressed:bg-muted aria-pressed:text-foreground"
          type="button"
          aria-pressed={locale === option}
          onclick={() => changeLocale(option)}>{option.toUpperCase()}</button
        >
      {/each}
    </div>

    <div class="w-[min(100%,29rem)]">
      <header class="mb-8 short-desktop:mb-5">
        <h1
          class="mb-[0.65rem] text-[clamp(var(--text-3xl),3.2vw,var(--text-4xl))] leading-[1.12]
                 font-extrabold tracking-[-0.02em] text-balance text-foreground
                 lte-600:text-3xl"
        >
          {copy.heading}
        </h1>
        <p class="max-w-[58ch] text-md leading-[1.6] text-muted-foreground">{copy.description}</p>
      </header>

      <form
        class="grid gap-5 short-desktop:gap-[0.85rem]"
        onsubmit={submit}
        method="post"
        action="/login"
        novalidate
      >
        <div class="grid gap-2">
          <label class="text-sm font-bold" for="portal">{copy.portalLabel}</label>
          <div
            class={cn(fieldFrame, 'flex items-center pl-[0.85rem] text-muted-foreground', fieldFocus)}
          >
            <Link2 size={19} aria-hidden="true" />
            <input
              class="min-w-0 flex-1 self-stretch bg-transparent px-[0.8rem] py-3 text-foreground
                     outline-0 placeholder:text-muted-foreground"
              id="portal"
              name="portal"
              type="url"
              placeholder={copy.portalPlaceholder}
              autocomplete="off"
              enterkeyhint="next"
              bind:value={portalUrl}
              onblur={cleanPortalUrl}
              aria-describedby="portal-hint"
              required
            />
          </div>
          <p id="portal-hint" class="text-xs leading-[1.45] text-muted-foreground">
            {copy.portalHint}
          </p>
        </div>

        <div class="grid gap-2">
          <label class="text-sm font-bold" for="email">{copy.emailLabel}</label>
          <input
            class={cn(fieldFrame, 'px-4 py-3 placeholder:text-muted-foreground', fieldFocus)}
            id="email"
            name="username"
            type="email"
            placeholder={copy.emailPlaceholder}
            autocomplete="username"
            inputmode="email"
            enterkeyhint="next"
            bind:value={username}
            required
          />
        </div>

        <div class="grid gap-2">
          <label class="text-sm font-bold" for="password">{copy.passwordLabel}</label>
          <div
            class={cn(fieldFrame, 'flex items-center pl-[0.85rem] text-muted-foreground', fieldFocus)}
          >
            <LockKeyhole size={19} aria-hidden="true" />
            <input
              class="min-w-0 flex-1 self-stretch bg-transparent px-[0.8rem] py-3 text-foreground
                     outline-0 placeholder:text-muted-foreground"
              id="password"
              name="password"
              type={passwordVisible ? 'text' : 'password'}
              placeholder={copy.passwordPlaceholder}
              autocomplete="current-password"
              enterkeyhint="done"
              bind:value={password}
              required
            />
            <button
              class="mr-[0.15rem] grid size-(--tap-min) flex-none place-items-center rounded-sm
                     bg-transparent text-muted-foreground transition-control hover:bg-muted
                     hover:text-foreground active:scale-(--press-scale)"
              type="button"
              aria-label={passwordVisible ? copy.hidePassword : copy.showPassword}
              title={passwordVisible ? copy.hidePassword : copy.showPassword}
              onclick={() => (passwordVisible = !passwordVisible)}
            >
              {#if passwordVisible}<EyeOff size={19} />{:else}<Eye size={19} />{/if}
            </button>
          </div>
        </div>

        <label
          class="group flex min-h-(--tap-min) w-fit cursor-pointer items-center gap-[0.65rem]
                 text-sm font-semibold"
        >
          <input
            class="peer absolute size-px overflow-hidden opacity-0"
            type="checkbox"
            bind:checked={remember}
          />
          <span
            class="grid size-[1.15rem] place-items-center rounded-xs border border-border bg-card
                   text-transparent transition-control group-active:scale-(--press-scale)
                   peer-checked:border-primary peer-checked:bg-primary
                   peer-checked:text-primary-foreground peer-focus-visible:outline-3
                   peer-focus-visible:outline-offset-2 peer-focus-visible:outline-ring"
            ><Check size={14} strokeWidth={3} /></span
          >
          <span>{copy.rememberLabel}</span>
        </label>

        {#if errorCode}
          <div
            class="flex items-start gap-[0.65rem] rounded-sm bg-danger-surface px-[0.95rem]
                   py-[0.85rem] text-base leading-[1.45] text-danger-strong animate-message-in"
            role="alert"
          >
            <AlertCircle class="mt-[0.05rem] flex-none" size={19} aria-hidden="true" />
            <span>{errorMessage}</span>
          </div>
        {/if}

        <button
          class="flex min-h-[3.1rem] w-full items-center justify-center gap-[0.65rem] rounded-md
                 bg-secondary px-5 py-3 font-bold text-secondary-foreground transition-control
                 enabled:hover:bg-secondary-hover enabled:active:scale-[0.985] disabled:opacity-70"
          type="submit"
          disabled={submitting || restoring}
        >
          {#if submitting}
            <LoaderCircle class="animate-spin" size={19} aria-hidden="true" />
            <span>{copy.submitting}</span>
          {:else}
            <span>{copy.submit}</span>
            <ArrowRight size={19} aria-hidden="true" />
          {/if}
        </button>
      </form>
    </div>
  </section>
</main>
{/if}
