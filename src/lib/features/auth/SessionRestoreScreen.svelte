<script lang="ts">
  import { AlertCircle, KeyRound, RefreshCw, WifiOff } from 'lucide-svelte';
  import Logo from '$lib/assets/Logo.svelte';
  import Button from '$lib/components/ui/Button.svelte';
  import ScreenShell from '$lib/components/ui/ScreenShell.svelte';
  import Spinner from '$lib/components/ui/Spinner.svelte';
  import * as m from '$lib/paraglide/messages.js';
  import type { Locale } from '$lib/paraglide/runtime.js';
  import { cn } from '$lib/utils';
  import type { SavedIdentity } from './session';

  type Props = {
    /**
     * `checking` is the credential-store read, `restoring` the portal login,
     * `failed` the dead end that needs a decision from the reader.
     */
    stage: 'checking' | 'restoring' | 'failed';
    identity: SavedIdentity | null;
    /** Already translated. Only read in the `failed` stage. */
    errorMessage?: string;
    /** The device has no network path, so a retry cannot succeed yet. */
    offline?: boolean;
    /** The wait has outlived the watchdog: say so instead of spinning silently. */
    slow?: boolean;
    /** Paraglide reads the locale at call time, so re-derive the copy on change. */
    locale: Locale;
    onRetry: () => void;
    onManualLogin: () => void;
  };

  const {
    stage,
    identity,
    errorMessage,
    offline = false,
    slow = false,
    locale,
    onRetry,
    onManualLogin
  }: Props = $props();

  const copy = $derived.by(() => {
    locale;
    return {
      appName: m.app_name(),
      checking: m.restore_checking(),
      restoring: m.restore_signing_in(),
      slow: m.restore_slow(),
      accountLabel: m.restore_account_label(),
      failedHeading: m.restore_failed_heading(),
      offlineHeading: m.restore_offline_heading(),
      offlineDescription: m.restore_offline_description(),
      offlineWaiting: m.restore_offline_waiting(),
      retry: m.restore_retry(),
      manualLogin: m.restore_manual_login()
    };
  });

  const portalHost = $derived.by(() => {
    if (!identity?.portalUrl) return '';
    try {
      return new URL(identity.portalUrl).host;
    } catch {
      return identity.portalUrl;
    }
  });

  const heading = $derived(
    stage === 'failed'
      ? offline
        ? copy.offlineHeading
        : copy.failedHeading
      : stage === 'checking'
        ? copy.checking
        : copy.restoring
  );

  const description = $derived(
    stage === 'failed'
      ? offline
        ? copy.offlineDescription
        : (errorMessage ?? '')
      : slow
        ? copy.slow
        : ''
  );
</script>

<!-- Startup owns the whole frame: the login form is never painted for an account
     that is about to be restored, and a failed restore keeps the reader here
     with both ways out rather than dropping them on an empty form. -->
<ScreenShell>
  <div
    class="grid w-[min(100%,26rem)] justify-items-center gap-4 text-center"
    role={stage === 'failed' ? 'alert' : 'status'}
    aria-live={stage === 'failed' ? 'assertive' : 'polite'}
    aria-busy={stage === 'failed' ? undefined : 'true'}
  >
    <div class="mb-2 flex items-center gap-3 text-lg font-bold text-foreground">
      <Logo size={34} variant="icon" />
      <span>{copy.appName}</span>
    </div>

    <span
      class={cn(
        'grid size-12 place-items-center rounded-md',
        stage === 'failed' ? 'bg-danger-surface text-danger-strong' : 'bg-muted text-primary-deep'
      )}
    >
      {#if stage === 'failed'}
        {#if offline}
          <WifiOff size={22} aria-hidden="true" />
        {:else}
          <AlertCircle size={22} aria-hidden="true" />
        {/if}
      {:else}
        <Spinner size={22} />
      {/if}
    </span>

    <h1 class="max-w-[30ch] text-xl leading-[1.25] font-bold text-balance text-foreground">
      {heading}
    </h1>

    {#if description}
      <p class="max-w-[42ch] text-base leading-[1.55] text-pretty text-muted-foreground">
        {description}
      </p>
    {/if}

    {#if identity}
      <p class="grid gap-[0.15rem] text-sm text-muted-foreground">
        <span class="font-semibold text-foreground">{identity.username}</span>
        <span>{copy.accountLabel}{portalHost ? ` · ${portalHost}` : ''}</span>
      </p>
    {/if}

    {#if stage === 'failed'}
      <div class="mt-2 grid w-full gap-2">
        {#if offline}
          <p class="text-sm text-muted-foreground">{copy.offlineWaiting}</p>
        {/if}
        <Button variant="ink" size="lg" block onclick={onRetry} disabled={offline}>
          <RefreshCw size={18} aria-hidden="true" />
          {copy.retry}
        </Button>
        <Button variant="ghost" onclick={onManualLogin}>
          <KeyRound size={17} aria-hidden="true" />
          {copy.manualLogin}
        </Button>
      </div>
    {/if}
  </div>
</ScreenShell>
