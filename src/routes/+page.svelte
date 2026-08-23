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
<main class="login-shell">
  <section class="brand-field" aria-label={copy.previewSchedule}>
    <div class="brand-lockup">
      <Logo size={32} variant="icon" />
      <span>{copy.appName}</span>
    </div>

    <div class="schedule-signal" aria-hidden="true">
      <div class="signal-heading">
        <div>
          <span>{copy.previewToday}</span>
          <strong>{copy.previewSchedule}</strong>
        </div>
        <span class="signal-status">{signalTime}</span>
      </div>
      <div class="signal-grid">
        <span class="time">09</span><span class="course course-primary"></span>
        <span class="time">11</span><span class="course course-soft"></span>
        <span class="time">14</span><span class="course course-accent"></span>
        <span class="time">16</span><span class="course course-short"></span>
      </div>
    </div>
  </section>

  <section class="form-field">
    <div class="mobile-brand">
      <Logo size={28} variant="icon" />
      <span>{copy.appName}</span>
    </div>

    <div class="language-control" aria-label={copy.languageLabel}>
      <Globe2 size={17} aria-hidden="true" />
      <button class:active={locale === 'fr'} type="button" aria-pressed={locale === 'fr'} onclick={() => changeLocale('fr')}>FR</button>
      <button class:active={locale === 'en'} type="button" aria-pressed={locale === 'en'} onclick={() => changeLocale('en')}>EN</button>
    </div>

    <div class="form-wrap">
        <header>
          <h1>{copy.heading}</h1>
          <p>{copy.description}</p>
        </header>

        <form onsubmit={submit} novalidate>
          <div class="field-group">
            <label for="portal">{copy.portalLabel}</label>
            <div class="input-wrap">
              <Link2 size={19} aria-hidden="true" />
              <input
                id="portal"
                name="portal"
                type="url"
                placeholder={copy.portalPlaceholder}
                autocomplete="url"
                bind:value={portalUrl}
                onblur={cleanPortalUrl}
                aria-describedby="portal-hint"
                required
              />
            </div>
            <p id="portal-hint" class="field-hint">{copy.portalHint}</p>
          </div>

          <div class="field-group">
            <label for="email">{copy.emailLabel}</label>
            <input
              id="email"
              name="email"
              type="email"
              placeholder={copy.emailPlaceholder}
              autocomplete="username"
              bind:value={username}
              required
            />
          </div>

          <div class="field-group">
            <label for="password">{copy.passwordLabel}</label>
            <div class="input-wrap password-wrap">
              <LockKeyhole size={19} aria-hidden="true" />
              <input
                id="password"
                name="password"
                type={passwordVisible ? 'text' : 'password'}
                placeholder={copy.passwordPlaceholder}
                autocomplete="current-password"
                bind:value={password}
                required
              />
              <button
                class="icon-button"
                type="button"
                aria-label={passwordVisible ? copy.hidePassword : copy.showPassword}
                title={passwordVisible ? copy.hidePassword : copy.showPassword}
                onclick={() => passwordVisible = !passwordVisible}
              >
                {#if passwordVisible}<EyeOff size={19} />{:else}<Eye size={19} />{/if}
              </button>
            </div>
          </div>

          <label class="remember-control">
            <input type="checkbox" bind:checked={remember} />
            <span class="check-control"><Check size={14} strokeWidth={3} /></span>
            <span>{copy.rememberLabel}</span>
          </label>

          {#if errorCode}
            <div class="error-message" role="alert">
              <AlertCircle size={19} aria-hidden="true" />
              <span>{errorMessage}</span>
            </div>
          {/if}

          <button class="submit-button" type="submit" disabled={submitting || restoring}>
            {#if submitting}
              <LoaderCircle class="spinner" size={19} aria-hidden="true" />
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
