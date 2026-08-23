/**
 * Window insets from the platform shell.
 *
 * An Android webview only reports `env(safe-area-inset-*)` for display cutouts,
 * never for the status bar or the gesture pill, so an edge-to-edge window lays
 * the page out underneath the system HUD. `MainActivity` measures the real
 * insets and hands them over two ways: the page pulls the current values
 * through `__nativeInsets` on its first paint, and every later change —
 * rotation, a bar that hides, a foldable that unfolds — is pushed to
 * `__applyNativeInsets`.
 *
 * Both write the same four custom properties, which `--safe-*` in `app.css`
 * reads once the `native-insets` class is on the root element. Platforms with
 * no bridge, desktop and iOS included, keep the `env()` fallback.
 */

type NativeInsets = {
  top: number;
  right: number;
  bottom: number;
  left: number;
};

declare global {
  interface Window {
    /** Kotlin-side bridge; absent on every platform but Android. */
    __nativeInsets?: { get(): string };
    /** Push target the activity calls, installed by `initNativeInsets`. */
    __applyNativeInsets?: (insets: NativeInsets) => void;
  }
}

const EDGES = ['top', 'right', 'bottom', 'left'] as const;

function applyNativeInsets(insets: NativeInsets) {
  const root = document.documentElement;

  for (const edge of EDGES) {
    // A malformed payload must not blank the layout: an unusable number falls
    // back to no inset rather than to `NaNpx`, which the browser drops.
    const value = Number(insets?.[edge]);
    root.style.setProperty(`--native-inset-${edge}`, `${Number.isFinite(value) ? value : 0}px`);
  }

  root.classList.add('native-insets');
}

/** Installs the push target and reads the insets the shell already knows. */
export function initNativeInsets() {
  if (typeof window === 'undefined') return;

  const bridge = window.__nativeInsets;
  if (!bridge) return;

  window.__applyNativeInsets = applyNativeInsets;

  try {
    applyNativeInsets(JSON.parse(bridge.get()) as NativeInsets);
  } catch (error) {
    // A bridge that answers nothing leaves the `env()` fallback in force, so
    // the layout stays usable and the cause stays visible in the log.
    console.warn('Native window insets could not be read', error);
  }
}
