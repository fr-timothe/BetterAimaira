<script lang="ts">
  import { onMount } from 'svelte';
  import { AlertCircle, ArrowDownToLine, CheckCircle2, RefreshCw, Store } from 'lucide-svelte';
  import Badge from '$lib/components/ui/Badge.svelte';
  import Button from '$lib/components/ui/Button.svelte';
  import Card from '$lib/components/ui/Card.svelte';
  import SectionHeader from '$lib/components/ui/SectionHeader.svelte';
  import * as m from '$lib/paraglide/messages.js';
  import { connectivity } from '$lib/state/connectivity.svelte';
  import { updatesSupported } from './update-service';
  import { updates } from './updates.svelte';
  import { cn } from '$lib/utils';

  type Props = {
    locale: string;
  };

  const { locale }: Props = $props();

  const supported = updatesSupported();

  onMount(() => {
    if (!supported) return;
    // A card that is on screen is worth one fresh look; the store throttles the
    // rest.
    void updates.checkOnStart();
  });

  const info = $derived(updates.info);
  const status = $derived(updates.status);
  const busy = $derived(status === 'checking' || status === 'installing');

  const percent = $derived.by(() => {
    const progress = updates.progress;
    if (!progress?.total) return null;
    return Math.min(100, Math.round((progress.downloaded / progress.total) * 100));
  });

  const lastChecked = $derived.by(() => {
    locale;
    if (!updates.lastCheckedAt) return null;
    return new Intl.DateTimeFormat(locale, { hour: '2-digit', minute: '2-digit' }).format(
      new Date(updates.lastCheckedAt)
    );
  });

  /** iOS never installs from inside the app: AltStore owns that step. */
  const installLabel = $derived.by(() => {
    locale;
    switch (info?.delivery) {
      case 'androidPackage':
        return m.update_install_android();
      case 'altStore':
        return m.update_open_altstore();
      default:
        return m.update_install_restart();
    }
  });

  const errorMessage = $derived.by(() => {
    locale;
    switch (updates.errorCode) {
      case 'update_download_failed':
        return m.update_error_download();
      case 'update_install_failed':
        return m.update_error_install();
      case 'update_store_unavailable':
        return m.update_error_store();
      case 'update_not_available':
        return m.update_up_to_date();
      default:
        return m.update_error_check();
    }
  });

  const handedOffMessage = $derived.by(() => {
    locale;
    return info?.delivery === 'altStore' ? m.update_handed_off_store() : m.update_handed_off_system();
  });
</script>

{#if supported}
  <Card>
    <div class="flex min-w-0 flex-col gap-3">
      <SectionHeader
        icon={ArrowDownToLine}
        title={m.update_section_title()}
        subtitle={info ? m.update_current_version({ version: info.currentVersion }) : undefined}
        level={3}
      />

      <div class="flex flex-wrap items-center gap-2">
        {#if status === 'available' && info}
          <Badge tone="accent" dot>
            {m.update_available({ version: info.latestVersion ?? '' })}
          </Badge>
        {:else if status === 'upToDate'}
          <Badge tone="success">{m.update_up_to_date()}</Badge>
        {:else if status === 'checking'}
          <Badge tone="neutral">{m.update_checking()}</Badge>
        {:else if status === 'handedOff'}
          <Badge tone="success">{handedOffMessage}</Badge>
        {:else if status === 'permissionRequired'}
          <Badge tone="warning">{m.update_permission_required()}</Badge>
        {:else if status === 'error'}
          <Badge tone="danger">{errorMessage}</Badge>
        {:else if !connectivity.online}
          <Badge tone="warning">{m.update_offline()}</Badge>
        {/if}

        {#if lastChecked}
          <span class="text-sm text-muted-foreground">{m.update_last_checked({ time: lastChecked })}</span>
        {/if}
      </div>

      {#if status === 'installing'}
        <div
          class="flex flex-col gap-2 text-sm tabular-nums text-muted-foreground"
          role="status"
          aria-live="polite"
        >
          <span>
            {percent === null
              ? m.update_downloading()
              : m.update_downloading_percent({ percent: String(percent) })}
          </span>
          <div class="h-1.5 overflow-hidden rounded-pill bg-surface-sunken" aria-hidden="true">
            <div
              class={cn(
                'h-full w-0 rounded-[inherit] bg-primary-deep',
                'transition-[inline-size] duration-fast ease-linear',
                percent === null && 'w-[35%] animate-update-sweep'
              )}
              style={percent === null ? undefined : `inline-size: ${percent}%`}
            ></div>
          </div>
        </div>
      {/if}

      {#if status === 'available' && info?.notes}
        <details class="text-sm">
          <summary>{m.update_notes_label()}</summary>
          <p>{info.notes}</p>
        </details>
      {/if}

      {#if info?.delivery === 'altStore'}
        <p class="text-sm leading-[1.45] text-muted-foreground">{m.update_altstore_hint()}</p>
      {/if}

      <div class="flex flex-wrap items-center gap-2">
        <!-- A failed install, or a refused install permission, still leaves an
             update to install: the action stays until it succeeded. -->
        {#if updates.available && status !== 'handedOff'}
          <Button variant="primary" loading={busy} onclick={() => void updates.install()}>
            {#if info?.delivery === 'altStore'}
              <Store size={18} aria-hidden="true" />
            {:else}
              <ArrowDownToLine size={18} aria-hidden="true" />
            {/if}
            <span>{installLabel}</span>
          </Button>
        {/if}

        <Button
          variant="outline"
          loading={status === 'checking'}
          disabled={busy || !connectivity.online}
          onclick={() => void updates.check()}
        >
          {#if status === 'upToDate'}
            <CheckCircle2 size={18} aria-hidden="true" />
          {:else if status === 'error'}
            <AlertCircle size={18} aria-hidden="true" />
          {:else}
            <RefreshCw size={18} aria-hidden="true" />
          {/if}
          <span>{m.update_check_action()}</span>
        </Button>
      </div>
    </div>
  </Card>
{/if}

