<script module lang="ts">
  /**
   * Goes on the label span each cell carries for the small-screen layout. The
   * stylesheet below hides it again once the real table assembles.
   */
  export const cellLabel = 'cell-label text-xs font-bold text-muted-foreground';
</script>

<script lang="ts">
  import type { Snippet } from 'svelte';

  type Props = {
    /** `<thead>` and `<tbody>`, written by the view that owns the columns. */
    children: Snippet;
  };

  const { children }: Props = $props();
</script>

<section class="overflow-hidden rounded-xl border border-border-subtle bg-card">
  <div class="overflow-x-auto [-webkit-overflow-scrolling:touch]">
    <table class="data-table">
      {@render children()}
    </table>
  </div>
</section>

<style>
  /* The base layout is the small-screen card fallback: every cell carries its
     own header, and the real table only assembles once there is room for the
     columns. Switching a table's own display model means rewriting table,
     thead, tbody, tr, td and th together, so it stays one block of CSS rather
     than a class on every cell — and `.cell-label` is part of the same
     mechanism.

     The rows come from the caller's snippet, which Svelte scopes to the
     caller, so everything below the table itself is reached through
     `:global`. The `.data-table` prefix keeps it from leaking further. */
  .data-table {
    display: block;
    width: 100%;
    padding: var(--space-3);
    border-collapse: collapse;
    font-size: var(--text-base);
    text-align: left;
  }

  .data-table :global(thead) {
    display: none;
  }

  .data-table :global(tbody),
  .data-table :global(tr),
  .data-table :global(td) {
    display: block;
  }

  .data-table :global(tr + tr) {
    margin-top: var(--space-2);
  }

  .data-table :global(tr) {
    padding: var(--space-3);
    background: var(--surface-sunken);
    border-radius: var(--radius-md);
  }

  .data-table :global(td) {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: var(--space-2);
    padding: var(--space-1) 0;
  }

  @media (min-width: 48rem) {
    .data-table {
      display: table;
      padding: 0;
    }

    .data-table :global(thead) {
      display: table-header-group;
    }

    .data-table :global(tbody) {
      display: table-row-group;
    }

    .data-table :global(tr),
    .data-table :global(tr + tr) {
      display: table-row;
      margin: 0;
      padding: 0;
      background: transparent;
      border-radius: 0;
    }

    .data-table :global(td) {
      display: table-cell;
      padding: var(--space-3) var(--space-4);
      border-bottom: 1px solid var(--border-subtle);
      vertical-align: middle;
    }

    .data-table :global(th) {
      padding: var(--space-3) var(--space-4);
      color: var(--muted-foreground);
      background: var(--surface-sunken);
      border-bottom: 1px solid var(--border-subtle);
      font-size: var(--text-xs);
      font-weight: var(--weight-bold);
      white-space: nowrap;
    }

    .data-table :global(tbody tr:last-child td) {
      border-bottom: 0;
    }

    /* A child row is indented in its first column only. */
    .data-table :global(tr.is-child td:first-child) {
      padding-left: var(--space-5);
    }

    .data-table :global(.cell-label) {
      display: none;
    }
  }

  @media (hover: hover) {
    .data-table :global(tbody tr:hover td) {
      background: var(--surface-sunken);
    }
  }
</style>
