<script lang="ts">
  import { AlertCircle } from 'lucide-svelte';
  import Button from './Button.svelte';
  import StateCard from './StateCard.svelte';
  import * as m from '$lib/paraglide/messages.js';
  import { sessionRecovery } from '$lib/state/session-recovery.svelte';

  type Props = {
    /** Re-runs the read that discovered the expiry, once a session is open again. */
    onRetry: () => void;
    /**
     * Drops to the sign-in form AND wipes the keyring. Omitted where the surface
     * must not offer a destructive action.
     */
    onLogout?: () => Promise<void>;
    locale: string;
    class?: string;
  };

  const { onRetry, onLogout, locale, class: className }: Props = $props();

  // Reaching this card means the request path already replayed the saved
  // password and did not get a session back. It reports which way that failed,
  // so the reader is not told to sign in again when the app can still do it.
  const status = $derived(sessionRecovery.status);
  const reconnecting = $derived(status === 'running');
  /** Only these are worth another replay; the rest need a password typed. */
  const canReconnect = $derived(
    status === 'idle' || status === 'unreachable' || status === 'exhausted'
  );

  async function reconnect() {
    // The one caller that is a person: a tap may restart the loop budget where
    // an automatic read may not.
    if (await sessionRecovery.recover({ prompted: true })) onRetry();
  }

  const title = $derived.by(() => {
    return m.account_disconnected();
  });

  const description = $derived.by(() => {
    switch (status) {
      case 'running':
        return m.restore_signing_in();
      case 'rejected':
        return m.error_saved_credentials_rejected();
      case 'unavailable':
        return m.session_expired_no_password();
      default:
        return m.session_reconnect_failed();
    }
  });

  const backToLoginLabel = $derived.by(() => {
    return m.back_to_login();
  });

  const actionLabel = $derived.by(() => {
    if (reconnecting) return m.session_reconnecting();
    if (canReconnect) return m.session_reconnect();
    // Nothing left to replay: signing out is the only move, and where the
    // surface refuses to offer it the card simply states the situation.
    return onLogout ? backToLoginLabel : undefined;
  });

  const action = $derived.by(() => {
    // Present but inert while a replay runs: the button is reporting progress,
    // not offering a second attempt.
    if (reconnecting) return () => {};
    if (canReconnect) return () => void reconnect();
    return onLogout ? () => void onLogout() : undefined;
  });

  // The destructive way out stays available while a replay is still worth
  // trying, but demoted below it.
  const showsLogoutFooter = $derived(canReconnect && onLogout !== undefined);
</script>

{#snippet logoutFooter()}
  <Button variant="ghost" size="sm" onclick={() => void onLogout?.()}>
    {backToLoginLabel}
  </Button>
{/snippet}

<StateCard
  kind="expired"
  icon={AlertCircle}
  {title}
  {description}
  {actionLabel}
  onAction={action}
  actionLoading={reconnecting}
  actionDisabled={reconnecting}
  footer={showsLogoutFooter ? logoutFooter : undefined}
  class={className}
/>
