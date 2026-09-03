import type { Snippet } from 'svelte';

/**
 * The dock's control tier: one slot the active view fills, rendered by the
 * shell inside the dock itself.
 *
 * The alternative was for a view to render its own bar above the dock, and
 * that is what this exists to prevent. Two floating surfaces of the same
 * material stacked on each other read as a mistake — two borders, two shadows,
 * two radii, and a clearance the view has to guess. Handing the shell a
 * snippet makes the controls a *row of the dock*: one border, one shadow, one
 * radius, and the clearance is measured by the thing that owns it.
 *
 * Compact windows only. On an expanded window the dock is hidden and the view
 * keeps its controls in its own header, where there is width for them.
 */
class ViewControls {
  /** Rendered by the shell. Null whenever the active view offers no controls. */
  content = $state<Snippet | null>(null);

  /**
   * Claim the slot for as long as the caller lives. Returns the teardown, so
   * the usual shape is `$effect(() => viewControls.claim(controls))` — a view
   * that unmounts without releasing would leave the shell rendering a snippet
   * whose owner is gone.
   */
  claim = (snippet: Snippet) => {
    this.content = snippet;
    return () => {
      if (this.content === snippet) this.content = null;
    };
  };
}

export const viewControls = new ViewControls();
