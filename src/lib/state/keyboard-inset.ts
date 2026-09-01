/**
 * The strip the on-screen keyboard takes from the window, as a custom property.
 *
 * Neither mobile webview shrinks the layout viewport for the keyboard on its
 * own: WKWebView never does, and an Android webview only does when the page
 * asks for it with `interactive-widget=resizes-content`, which older webviews
 * ignore. `100dvh` therefore stays the whole window while the keyboard is up,
 * and anything pinned to the bottom edge — a sheet, above all — is painted
 * underneath it.
 *
 * `visualViewport` is what both report honestly: the part of the window still
 * visible. The gap against the layout viewport is the keyboard, and it lands on
 * `--keyboard-inset` for whatever sits on that edge to pay. On a webview that
 * does resize the layout the gap is zero, so the two mechanisms add up to one
 * correction rather than two.
 */

/** Below this a reported gap is rounding noise, not a keyboard. */
const NOISE_PX = 1;

function readInset(viewport: VisualViewport): number {
  // Pinch-zoom shrinks the visual viewport exactly like a keyboard does. A
  // zoomed page is being read, not typed into, so it pays nothing.
  if (viewport.scale > 1.01) return 0;
  const hidden = window.innerHeight - viewport.height - viewport.offsetTop;
  return hidden > NOISE_PX ? Math.round(hidden) : 0;
}

/** Publishes `--keyboard-inset` until the returned teardown is called. */
export function initKeyboardInset(): () => void {
  if (typeof window === 'undefined') return () => {};

  const viewport = window.visualViewport;
  const root = document.documentElement;
  if (!viewport) return () => {};

  const update = () => root.style.setProperty('--keyboard-inset', `${readInset(viewport)}px`);

  // `scroll` as well as `resize`: iOS answers a focus near the bottom of the
  // page by offsetting the visual viewport rather than resizing it again.
  viewport.addEventListener('resize', update);
  viewport.addEventListener('scroll', update);
  update();

  return () => {
    viewport.removeEventListener('resize', update);
    viewport.removeEventListener('scroll', update);
    root.style.removeProperty('--keyboard-inset');
  };
}
