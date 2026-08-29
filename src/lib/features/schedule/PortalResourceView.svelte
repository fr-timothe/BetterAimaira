<script lang="ts">
  import type { Snippet } from 'svelte';
  import { RefreshCw } from 'lucide-svelte';
  import FreshnessLabel from '$lib/components/ui/FreshnessLabel.svelte';
  import IconButton from '$lib/components/ui/IconButton.svelte';
  import StateCard from '$lib/components/ui/StateCard.svelte';
  import type { IconComponent } from '$lib/components/ui/icon';
  import * as m from '$lib/paraglide/messages.js';
  import type { Locale } from '$lib/paraglide/runtime.js';
  import PortalResourceError from './PortalResourceError.svelte';
  import type { PortalResourceHandle } from './portal-resource.svelte';

  type Props = {
    resource: PortalResourceHandle;
    locale: Locale;
    onLogout: () => Promise<void>;
    /** The read landed but holds nothing to show. */
    empty: boolean;
    emptyIcon: IconComponent;
    /** Shown while the first read is in flight; each view has its own shape. */
    skeleton: Snippet;
    /**
     * The data. Receives the toolbar as an argument rather than having it
     * rendered around it: the freshness label sits in the middle of the page,
     * below the hero and above the list, and only the view knows where.
     */
    ready: Snippet<[toolbar: Snippet]>;
    /** Extra controls the toolbar hosts, e.g. a cards/table switch. */
    controls?: Snippet;
  };

  const { resource, locale, onLogout, empty, emptyIcon, skeleton, ready, controls }: Props =
    $props();

  const copy = $derived.by(() => {
    return {
      refresh: m.resource_refresh(),
      emptyHeading: m.resource_empty_heading(),
      emptyDescription: m.resource_empty_description(),
    };
  });
</script>

{#snippet toolbar()}
  <div class="flex flex-wrap items-center justify-end gap-3">
    <!-- The failed refresh is stated here rather than swallowed: the data stays
         on screen, and the label says when it was actually read. -->
    <FreshnessLabel
      fetchedAt={resource.fetchedAt}
      {locale}
      refreshing={resource.refreshing}
      failed={resource.refreshFailed}
    />

    <div class="flex min-w-0 items-center gap-2">
      {@render controls?.()}

      <div class="desktop-only">
        <IconButton
          label={copy.refresh}
          variant="ghost"
          size="sm"
          loading={resource.refreshing}
          onclick={() => resource.load(true)}
        >
          <RefreshCw size={14} aria-hidden="true" />
        </IconButton>
      </div>
    </div>
  </div>
{/snippet}

{#if resource.state.kind === 'loading'}
  {@render skeleton()}
{:else if resource.state.kind === 'error'}
  <PortalResourceError
    code={resource.state.code}
    onRetry={() => resource.load(true)}
    {onLogout}
    {locale}
  />
{:else if empty}
  <StateCard
    kind="empty"
    icon={emptyIcon}
    title={copy.emptyHeading}
    description={copy.emptyDescription}
    actionLabel={copy.refresh}
    onAction={() => resource.load(true)}
  />
{:else}
  {@render ready(toolbar)}
{/if}
