<script lang="ts">
  import { Download, FileText } from 'lucide-svelte';
  import Button from '$lib/components/ui/Button.svelte';
  import { documentKindLabel } from './portal-utils';
  import type { PortalDocument } from './types';

  type Props = {
    doc: PortalDocument;
    /** True only while this document's own request is in flight. */
    downloading?: boolean;
    downloadLabel: string;
    downloadingLabel: string;
    onDownload: (doc: PortalDocument) => void;
  };

  const { doc, downloading = false, downloadLabel, downloadingLabel, onDownload }: Props =
    $props();

  const actionLabel = $derived(downloading ? downloadingLabel : downloadLabel);
</script>

<!-- Structure comes from the sunken field and its line — no shadow on top. -->
<article
  class="flex items-center gap-3 rounded-lg border border-border-subtle bg-surface-sunken px-4 py-3"
>
  <span
    class="grid size-10 shrink-0 place-items-center rounded-md bg-card text-primary-deep"
    aria-hidden="true"><FileText size={22} /></span
  >

  <div class="min-w-0 flex-1">
    <h3 class="truncate text-base font-bold text-foreground">{doc.label}</h3>
    <p class="mt-[0.15rem] text-xs text-muted-foreground">{documentKindLabel(doc.kind)}</p>
  </div>

  <Button
    size="sm"
    loading={downloading}
    title={actionLabel}
    ariaLabel={`${actionLabel}: ${doc.label}`}
    onclick={() => onDownload(doc)}
  >
    {#if !downloading}<Download size={16} aria-hidden="true" />{/if}
    <span>{actionLabel}</span>
  </Button>
</article>
