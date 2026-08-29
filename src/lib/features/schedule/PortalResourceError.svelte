<script lang="ts">
  import { AlertCircle, CloudOff } from 'lucide-svelte';
  import SessionExpiredCard from '$lib/components/ui/SessionExpiredCard.svelte';
  import StateCard from '$lib/components/ui/StateCard.svelte';
  import * as m from '$lib/paraglide/messages.js';
  import type { Locale } from '$lib/paraglide/runtime.js';
  import { connectivity } from '$lib/state/connectivity.svelte';
  import { resourceErrorMessage } from './portal-utils';
  import type { PortalResourceErrorCode } from './types';

  type Props = {
    code: PortalResourceErrorCode;
    /** Re-runs the read that failed. */
    onRetry: () => void;
    /** Omitted where the surface must not offer the destructive way out. */
    onLogout?: () => Promise<void>;
    locale: Locale;
  };

  const { code, onRetry, onLogout, locale }: Props = $props();

  const copy = $derived.by(() => {
    return {
      errorHeading: m.resource_error_heading(),
      retry: m.resource_retry(),
      offlineHeading: m.sync_offline(),
      offlineDescription: m.sync_offline_description(),
    };
  });
</script>

<!-- The three ways a portal read fails, told apart once. The offline branch
     comes first on purpose: a device without a network path is not a portal
     outage and must never be reported as one. -->
{#if !connectivity.online}
  <StateCard
    kind="offline"
    icon={CloudOff}
    title={copy.offlineHeading}
    description={copy.offlineDescription}
    actionLabel={copy.retry}
    onAction={onRetry}
  />
{:else if code === 'session_expired'}
  <!-- The one state a retry cannot fix, so it gets the recovery card and not a
       `StateCard` with a retry button. -->
  <SessionExpiredCard {onRetry} {onLogout} {locale} />
{:else}
  <StateCard
    kind="error"
    icon={AlertCircle}
    title={copy.errorHeading}
    description={resourceErrorMessage(code)}
    actionLabel={copy.retry}
    onAction={onRetry}
  />
{/if}
