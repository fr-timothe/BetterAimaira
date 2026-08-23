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

<article class="document-card">
  <span class="document-icon" aria-hidden="true"><FileText size={22} /></span>

  <div class="document-meta">
    <h3>{doc.label}</h3>
    <p>{documentKindLabel(doc.kind)}</p>
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

<style>
  /* Structure comes from the sunken field and its line — no shadow on top. */
  .document-card {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    padding: var(--space-3) var(--space-4);
    background: var(--surface-sunken);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-lg);
  }

  .document-icon {
    display: grid;
    width: 2.5rem;
    height: 2.5rem;
    flex-shrink: 0;
    place-items: center;
    color: var(--primary-deep);
    background: var(--card);
    border-radius: var(--radius-md);
  }

  .document-meta {
    min-width: 0;
    flex: 1;
  }

  .document-meta h3 {
    margin: 0;
    overflow: hidden;
    color: var(--foreground);
    font-size: var(--text-base);
    font-weight: var(--weight-bold);
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .document-meta p {
    margin: 0.15rem 0 0;
    color: var(--muted-foreground);
    font-size: var(--text-xs);
  }
</style>
