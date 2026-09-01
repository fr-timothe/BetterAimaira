<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke, isTauri } from '$lib/invoke';
  import {
    AlertCircle,
    ArrowRight,
    Check,
    Eye,
    EyeOff,
    Globe2,
    LoaderCircle,
    LockKeyhole,
  } from 'lucide-svelte';
  import Logo from '$lib/assets/Logo.svelte';
  import * as m from '$lib/paraglide/messages.js';
  import type { Locale } from '$lib/paraglide/runtime.js';
  import { captureEvent } from '$lib/features/analytics/analytics-service';
  import { clearPortalResourceCache } from '$lib/features/schedule/portal-cache';
  import SessionRestoreScreen from '$lib/features/auth/SessionRestoreScreen.svelte';
  import SchoolPicker from '$lib/features/auth/SchoolPicker.svelte';
  import OnboardingScreen from '$lib/features/onboarding/OnboardingScreen.svelte';
  import { onboardingSeen } from '$lib/features/onboarding/onboarding.svelte';
  import type { SavedIdentity } from '$lib/features/auth/session';
  import type { School } from '$lib/data/schools';
  import { connectivity } from '$lib/state/connectivity.svelte';
  import { appLocale } from '$lib/state/locale.svelte';
  import { backGesture } from '$lib/state/back-gesture';
  import { sessionRecovery } from '$lib/state/session-recovery.svelte';
  import { updates } from '$lib/features/updates/updates.svelte';
  import { cn } from '$lib/utils';

  type ScheduleAppComponent = (typeof import('$lib/features/schedule/ScheduleApp.svelte'))['default'];

  type PortalInfo = { portalUrl: string };
  type LoginResult = {
    portalUrl: string;
    username: string;
    credentialsSaved: boolean;
    sundaysVisible: boolean;
  };
  type RestoreResult = {
    status: 'restored' | 'no_credentials' | 'credentials_rejected';
    session: LoginResult | null;
    identity: SavedIdentity | null;
  };
  type ErrorCode =
    | 'invalid_portal_url'
    | 'insecure_portal_url'
    | 'portal_unreachable'
    | 'portal_not_aimaira'
    | 'invalid_credentials'
    | 'missing_credentials'
    | 'saved_credentials_rejected'
    | 'credential_store'
    | 'internal_error'
    | 'desktop_required';

  /**
   * `checking` reads the credential store, `restoring` logs the saved account
   * back in, `failed` holds a recoverable startup error, `school` is the picker
   * that fills the portal address, `manual` is the login form and `ready` the
   * authenticated app. Startup begins on `checking` so a returning account never
   * sees the form flash by.
   */
  type BootPhase = 'checking' | 'restoring' | 'failed' | 'school' | 'manual' | 'ready';

  /** How long a restore may run before the screen admits it is slow. */
  const SLOW_RESTORE_DELAY = 12_000;
  /** Retries spent by the network coming back, so a flapping link cannot loop. */
  const MAX_AUTO_RETRIES = 3;

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
  let phase = $state<BootPhase>('checking');
  /**
   * The login steps behind the one on screen, oldest first.
   *
   * Kept so the platform's back gesture has somewhere to go: the picker and the
   * form are one route, so nothing in the history says which of them the reader
   * came through, and the picker can be reached from either side.
   */
  let loginTrail = $state<Extract<BootPhase, 'school' | 'manual'>[]>([]);
  let errorCode = $state<ErrorCode | null>(null);
  let restoreErrorCode = $state<ErrorCode | null>(null);
  let restoreOffline = $state(false);
  let restoreSlow = $state(false);
  let savedIdentity = $state<SavedIdentity | null>(null);
  // The school behind the address in the field, when it came from the picker.
  // Kept so the form can name it, and so a school whose portal address is not
  // known can explain itself instead of leaving an empty field with no reason.
  let selectedSchool = $state<School | null>(null);
  // Deliberately not reactive: the counter only guards the retry effect, and a
  // reactive read there would re-run the effect on its own write.
  let autoRetries = 0;
  let loginResult = $state<LoginResult | null>(null);
  let ScheduleApp = $state<ScheduleAppComponent | null>(null);
  /**
   * The shell is open on stored snapshots, with no live portal session behind
   * it. Kept so the network coming back can quietly sign the account in rather
   * than stranding the reader on data that can never refresh.
   */
  let offlineMode = $state(false);
  /**
   * Bumped only when an offline start is upgraded to a real session, so the
   * authenticated shell remounts once and refills from the portal. Nothing else
   * changes it, so ordinary use never remounts.
   */
  let sessionEpoch = $state(0);
  const locale = $derived(appLocale.current);
  let now = $state(new Date());
  // Read once: the introduction is dismissed inside this session, and re-reading
  // storage on every render would put it back for the frame after the write.
  let introductionPending = $state(!onboardingSeen());

  let slowTimer: ReturnType<typeof setTimeout> | null = null;

  const restoring = $derived(phase === 'checking' || phase === 'restoring');
  /** Both steps a reader signs in through, picker and form. */
  const atLogin = $derived(phase === 'school' || phase === 'manual');
  /** First start only: a device with a saved account has been through this. */
  const introductionVisible = $derived(introductionPending && atLogin && !savedIdentity);

  const signalTime = $derived(
    new Intl.DateTimeFormat(locale, { hour: '2-digit', minute: '2-digit' }).format(now)
  );

  const copy = $derived.by(() => {
    locale;
    return {
      appName: m.app_name(),
      heading: m.login_heading(),
      description: m.login_description(),
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
      schoolChange: m.school_change(),
      schoolChoose: m.school_choose(),
    };
  });

  /**
   * Who the form is signing in to. The school when the picker named one, and
   * the portal's host when the address was hand-typed for a school nobody has
   * a name for — the only identity there is in that case, and shorter than the
   * address the picker keeps.
   */
  const signedInSchool = $derived(selectedSchool?.name ?? (portalUrl ? portalHost(portalUrl) : null));

  const schoolReminder = $derived.by(() => {
    locale;
    const school = signedInSchool;
    return school ? m.login_school_reminder({ school }) : m.login_school_none();
  });

  function portalHost(url: string): string {
    try {
      return new URL(url).host;
    } catch {
      return url.replace(/^https?:\/\//, '').replace(/\/.*$/, '');
    }
  }

  /**
   * What the form has to say about the school the picker handed it: nothing at
   * all when its portal answers the ordinary sign-in form, and a warning when
   * the portal signs in elsewhere. A school whose address nobody knows never
   * reaches the form without one — the picker's sheet asks for it there.
   */
  const schoolNotice = $derived.by(() => {
    locale;
    const school = selectedSchool;
    if (!school || school.portalLogin !== 'sso') return null;
    return { tone: 'warning' as const, text: m.school_login_sso_hint({ school: school.name }) };
  });

  /** Shared by the form's own error and by the picker's address sheet. */
  function messageForError(code: ErrorCode | null): string {
    switch (code) {
      case 'invalid_portal_url': return m.error_invalid_portal_url();
      case 'insecure_portal_url': return m.error_insecure_portal_url();
      case 'portal_unreachable': return m.error_portal_unreachable();
      case 'portal_not_aimaira': return m.error_portal_not_aimaira();
      case 'invalid_credentials': return m.error_invalid_credentials();
      case 'missing_credentials': return m.error_missing_credentials();
      case 'saved_credentials_rejected': return m.error_saved_credentials_rejected();
      case 'credential_store': return m.error_credential_store();
      case 'internal_error': return m.error_internal_error();
      case 'desktop_required': return m.error_desktop_required();
      default: return '';
    }
  }

  const errorMessage = $derived.by(() => {
    locale;
    return messageForError(errorCode);
  });

  const restoreErrorMessage = $derived.by(() => {
    locale;
    switch (restoreErrorCode) {
      case 'portal_unreachable': return m.error_portal_unreachable();
      case 'portal_not_aimaira': return m.error_portal_not_aimaira();
      case 'insecure_portal_url': return m.error_insecure_portal_url();
      case 'invalid_portal_url': return m.error_invalid_portal_url();
      case 'credential_store': return m.error_credential_store();
      default: return m.error_internal_error();
    }
  });

  onMount(() => {
    const clock = setInterval(() => (now = new Date()), 30_000);
    // Asked for at boot, in parallel with the sign-in it does not depend on:
    // by the time the schedule is on screen the answer is already there. It
    // stays silent until the signed-in shell opens the notice surface, so
    // nothing about a release lands on the login or onboarding screens.
    void updates.checkOnStart();
    // Reported at boot rather than after the introduction: a reader who agreed
    // on a previous run is counted the same way on every start. The Rust side
    // drops it outright when nobody agreed.
    captureEvent('app_launched');
    if (isTauri()) {
      void startSession();
    } else {
      // No credential store to read outside Tauri, so skip straight to the
      // first step a reader sees — the picker, same as a fresh install.
      openLoginForm();
    }
    return () => {
      clearInterval(clock);
      stopSlowWatchdog();
    };
  });

  // The network coming back is the one event that turns an unreachable portal
  // into a reachable one without the reader doing anything, so spend it on a
  // retry — a bounded number of times, in case the link keeps flapping.
  $effect(() => {
    const online = connectivity.online;
    if (!online || phase !== 'failed') return;
    if (restoreErrorCode !== 'portal_unreachable' && !restoreOffline) return;
    if (autoRetries >= MAX_AUTO_RETRIES) return;
    autoRetries += 1;
    void startSession();
  });

  // The other half of that idea: the app opened on snapshots alone, and the
  // network has just come back. Sign the saved account in so the views can
  // actually refresh. Unbounded on purpose — unlike the restore retry above,
  // this costs nothing while it fails and the reader keeps a usable screen.
  $effect(() => {
    if (!connectivity.online || !offlineMode) return;
    void upgradeOfflineSession();
  });

  function startSlowWatchdog() {
    stopSlowWatchdog();
    restoreSlow = false;
    slowTimer = setTimeout(() => (restoreSlow = true), SLOW_RESTORE_DELAY);
  }

  function stopSlowWatchdog() {
    if (slowTimer === null) return;
    clearTimeout(slowTimer);
    slowTimer = null;
  }

  function failRestore(code: ErrorCode, offline = false) {
    stopSlowWatchdog();
    restoreSlow = false;
    restoreErrorCode = code;
    restoreOffline = offline;
    phase = 'failed';
  }

  /**
   * Sends the reader to the login form, with the saved account already filled.
   *
   * A first-time reader with no address to start from goes to the picker
   * instead: the form's first field would otherwise be an empty box asking for
   * a URL they have never been told.
   */
  function openLoginForm(code: ErrorCode | null = null) {
    stopSlowWatchdog();
    restoreSlow = false;
    restoreErrorCode = null;
    restoreOffline = false;
    if (savedIdentity) {
      portalUrl ||= savedIdentity.portalUrl;
      username ||= savedIdentity.username;
    }
    errorCode = code;
    enterLoginStep(code === null && !portalUrl ? 'school' : 'manual', { fresh: true });
  }

  /**
   * Moves to a login step, remembering the one it came from. A step entered
   * again — the picker reopened from the form — is a step forward like any
   * other, so the trail only ever grows on a move the reader made.
   */
  function enterLoginStep(step: 'school' | 'manual', { fresh = false } = {}) {
    loginTrail = fresh ? [step] : [...loginTrail, step];
    phase = step;
  }

  /** Returns to the step behind the current one, for the back gesture. */
  function leaveLoginStep() {
    if (loginTrail.length < 2) return;
    const trail = loginTrail.slice(0, -1);
    loginTrail = trail;
    phase = trail[trail.length - 1]!;
  }

  function chooseSchool(school: School) {
    selectedSchool = school;
    portalUrl = school.portalUrl ?? '';
    errorCode = null;
    enterLoginStep('manual');
  }

  /**
   * A hand-typed address, cleaned before the form opens on it. The cleaning
   * used to run on the form's own field; the field is gone, so a refusal has to
   * be answered where the address was typed — the picker's sheet keeps it and
   * shows what came back.
   */
  async function submitManualPortal(rawUrl: string, school: School | null): Promise<string | null> {
    if (!isTauri()) {
      // No backend to clean the address outside Tauri, and the form will refuse
      // the sign-in anyway. Take it as typed rather than blocking the step.
      openManualPortal(rawUrl.trim(), school);
      return null;
    }
    try {
      const result = await invoke<PortalInfo>('normalize_portal_url', { portalUrl: rawUrl });
      openManualPortal(result.portalUrl, school);
      return null;
    } catch (error) {
      return messageForError(extractErrorCode(error));
    }
  }

  function openManualPortal(cleanUrl: string, school: School | null) {
    selectedSchool = school;
    portalUrl = cleanUrl;
    errorCode = null;
    enterLoginStep('manual');
  }

  function extractErrorCode(error: unknown): ErrorCode {
    if (typeof error === 'object' && error !== null && 'code' in error) {
      const code = (error as { code: string }).code;
      return code as ErrorCode;
    }
    return 'internal_error';
  }

  async function submit(event: SubmitEvent) {
    event.preventDefault();
    errorCode = null;
    loginResult = null;

    if (!portalUrl.trim()) {
      errorCode = 'invalid_portal_url';
      return;
    }

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
      captureEvent('login_succeeded');
    } catch (error) {
      errorCode = extractErrorCode(error);
      // The stable error code, and nothing else: it names which of the portal's
      // refusals happened, never who was refused or by which portal.
      captureEvent('login_failed', errorCode);
    } finally {
      submitting = false;
    }
  }

  /**
   * The startup path: find out whether an account is saved, then sign it back
   * in. Every outcome lands somewhere explicit — the app, the login form with a
   * reason, or the restore screen with a retry.
   */
  async function startSession() {
    phase = 'checking';
    restoreErrorCode = null;
    restoreOffline = false;
    startSlowWatchdog();

    let identity: SavedIdentity | null;
    try {
      identity = await invoke<SavedIdentity | null>('saved_identity');
    } catch (error) {
      // Without a credential store there is nothing to restore, and a retry
      // would fail the same way: the password in hand is the way out.
      openLoginForm(extractErrorCode(error));
      return;
    }

    savedIdentity = identity;
    if (!identity) {
      openLoginForm();
      return;
    }

    if (!connectivity.online) {
      // Snapshots on disk are enough to open the app. Signing in is impossible
      // without the portal, but the schedule, grades and absences last read are
      // exactly what a reader between two classes came to check, and every
      // surface states how old they are.
      if (identity.hasSnapshots) {
        await openAuthenticatedApp(
          {
            portalUrl: identity.portalUrl,
            username: identity.username,
            credentialsSaved: true,
            sundaysVisible: false,
          },
          { offline: true }
        );
        portalUrl = identity.portalUrl;
        return;
      }
      failRestore('portal_unreachable', true);
      return;
    }

    phase = 'restoring';
    try {
      const result = await invoke<RestoreResult>('restore_session');

      if (result.status === 'restored' && result.session) {
        stopSlowWatchdog();
        await openAuthenticatedApp(result.session);
        portalUrl = result.session.portalUrl;
        return;
      }

      savedIdentity = result.identity ?? savedIdentity;
      openLoginForm(
        result.status === 'credentials_rejected' ? 'saved_credentials_rejected' : null
      );
    } catch (error) {
      const code = extractErrorCode(error);
      if (code === 'credential_store') {
        openLoginForm(code);
        return;
      }
      failRestore(code, !connectivity.online);
    }
  }

  async function openAuthenticatedApp(result: LoginResult, { offline = false } = {}) {
    ScheduleApp ??= (await import('$lib/features/schedule/ScheduleApp.svelte')).default;
    loginResult = result;
    offlineMode = offline;
    autoRetries = 0;
    // A real sign-in settles every question a past recovery answered, including
    // the ones it gave up on, so the module starts this session from scratch.
    sessionRecovery.reset();
    phase = 'ready';
  }

  /**
   * Turns an offline start into a real session once the network is back.
   *
   * The shell remounts on success rather than being patched in place: every
   * view is holding snapshot data that only a live session can replace, and one
   * remount at the moment connectivity returns is both simpler and more honest
   * than leaving stale surfaces behind a session that is now live.
   */
  async function upgradeOfflineSession() {
    // Both re-authentication paths now share the recovery module's own
    // single-flight lock, so the connectivity effect firing twice is already
    // handled there and no local guard is left to keep.
    const restored = await sessionRecovery.recover();
    // Still unreachable, or the portal refused. The shell keeps showing what it
    // has; the next time connectivity flips, this runs again.
    if (!restored) return;
    clearPortalResourceCache();
    offlineMode = false;
    sessionEpoch += 1;
  }

  // The back button and the edge swipe walk the login steps the reader came
  // through. The introduction, which is painted over them, claims the gesture
  // for its own panels while it is on screen.
  $effect(() => {
    const canLeave = atLogin && !introductionVisible && loginTrail.length > 1;
    backGesture.claim('login', canLeave ? leaveLoginStep : null);
  });

  const changeLocale = (nextLocale: Locale) => appLocale.set(nextLocale);

  async function logout() {
    try {
      await invoke('logout');
    } catch (error) {
      // The session is dropped either way; a keyring that refuses to forget is
      // worth reporting on the form the reader is about to land on.
      errorCode = extractErrorCode(error);
    }
    clearPortalResourceCache();
    // The account is gone from the keyring, so every outcome the module is
    // holding is about a session that no longer has a password behind it.
    sessionRecovery.reset();
    loginResult = null;
    ScheduleApp = null;
    savedIdentity = null;
    offlineMode = false;
    autoRetries = 0;
    password = '';
    enterLoginStep('manual', { fresh: true });
  }
</script>

<svelte:head>
  <title>{copy.appName}</title>
</svelte:head>

{#if phase === 'ready' && loginResult && ScheduleApp}
  <!-- One composite key, two reasons to rebuild the shell. An offline start
       that reaches a real session has to refill every view instead of leaving
       snapshot data behind a live session; and a language change has to re-run
       every `m.*()` the shell renders, which Paraglide cannot do on its own
       because its locale is a plain variable, not a signal. The pre-sign-in
       screens are deliberately outside this: the language switch also sits on
       the login form, and remounting there would wipe an address, a username
       and a password the reader had already typed. -->
  {#key `${sessionEpoch}:${locale}`}
    <ScheduleApp
      username={loginResult.username}
      portalUrl={loginResult.portalUrl}
      {locale}
      credentialsWarning={remember && !loginResult.credentialsSaved}
      sundaysVisible={loginResult.sundaysVisible}
      onLocaleChange={changeLocale}
      onLogout={logout}
    />
  {/key}
{:else if introductionVisible}
  <OnboardingScreen {locale} onDone={() => (introductionPending = false)} />
{:else if phase === 'checking' || phase === 'restoring' || phase === 'failed'}
  <SessionRestoreScreen
    stage={phase}
    identity={savedIdentity}
    errorMessage={restoreErrorMessage}
    offline={restoreOffline && !connectivity.online}
    slow={restoreSlow}
    {locale}
    onRetry={() => {
      autoRetries = 0;
      void startSession();
    }}
    onManualLogin={() => openLoginForm()}
  />
{:else if phase === 'school'}
  <SchoolPicker {locale} onSelect={chooseSchool} onSubmitAddress={submitManualPortal} />
{:else}
<main
  class="login-shell grid min-h-full grow overflow-y-auto mb-(--keyboard-inset)
         grid-cols-[minmax(360px,0.84fr)_minmax(460px,1.16fr)] lte-820:block"
>
  <!-- The signal panel is abstract geometry plus the real clock. It states no
       course, room or name it does not have. -->
  <section
    class="relative flex min-h-full flex-col overflow-hidden bg-secondary text-secondary-foreground
           pl-safe-12 pr-12 py-safe-8
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
           px-screen pt-safe-20 pb-safe-12
           lte-820:min-h-dvh lte-820:items-start
           lte-820:pt-safe-26 lte-820:pb-safe-8
           short-desktop:pt-safe-18 short-desktop:pb-safe-6"
  >
    <div
      class="hidden items-center gap-3 text-lg font-bold
             lte-820:absolute lte-820:top-safe-5 lte-820:left-safe-5 lte-820:flex"
    >
      <Logo size={28} variant="icon" />
      <span class="lte-600:hidden">{copy.appName}</span>
    </div>

    <div
      class="absolute top-safe-6 right-safe-6 flex min-h-(--tap-min) items-center
             gap-[0.2rem] text-muted-foreground
             lte-820:top-safe-4 lte-820:right-safe-4"
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
        <!-- The address belongs to the picker, sheet included, and stays there.
             The form only names who the sign-in is for, so a reader who opened
             the wrong school sees it without a portal field back on the form. -->
        <!-- `min-w-0` twice over, and not for tidiness: the reminder never
             wraps, so a grid or flex track left at its automatic minimum takes
             the width of the longest school name in the list and pushes the
             whole form past the window. -->
        <div class="grid min-w-0 gap-2">
          <div class="flex min-w-0 items-center justify-between gap-3">
            <p class="min-w-0 truncate text-sm leading-[1.45] text-muted-foreground">
              {schoolReminder}
            </p>
            <button
              class="min-h-(--tap-min) flex-none rounded-sm bg-transparent text-xs font-bold text-primary-deep
                     underline underline-offset-2 transition-control hover:text-secondary
                     active:scale-(--press-scale)"
              type="button"
              onclick={() => enterLoginStep('school')}
            >
              {signedInSchool ? copy.schoolChange : copy.schoolChoose}
            </button>
          </div>

          {#if schoolNotice}
            <p
              class={cn(
                'rounded-sm px-[0.7rem] py-[0.55rem] text-xs leading-[1.45]',
                schoolNotice.tone === 'warning'
                  ? 'bg-danger-surface text-danger-strong'
                  : 'bg-muted text-muted-foreground'
              )}
            >
              {schoolNotice.text}
            </p>
          {/if}
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
