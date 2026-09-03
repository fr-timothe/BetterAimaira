/**
 * DEV-ONLY PROTOTYPE — not part of the shipped app.
 *
 * Horizontal swipe, as a Svelte action so the three structures share one
 * gesture instead of three copies of it. The rules are the ones the current
 * view already worked out and documented: a mouse never swipes, a mostly
 * vertical drag is not a swipe, and a recognised swipe eats the synthetic
 * click the browser fires on whatever the finger landed on — otherwise
 * releasing over a course block opens the detail of a course from the period
 * the swipe just left.
 */

const SWIPE_DISTANCE = 64;

const PULL_DISTANCE = 48;

export type SwipeOptions = {
  onSwipe: (direction: -1 | 1) => void;
  /**
   * A downward drag on the same element, for a surface that opens something
   * by being pulled. It never fires on a gesture the horizontal test claimed.
   */
  onPullDown?: () => void;
  /** Set false where another surface owns the horizontal axis. */
  enabled?: boolean;
};

export function swipe(node: HTMLElement, options: SwipeOptions) {
  let current = options;
  let startX = 0;
  let startY = 0;
  let tracking = false;
  let consumedClick = false;

  function onPointerDown(event: PointerEvent) {
    tracking = event.pointerType !== 'mouse' && current.enabled !== false;
    consumedClick = false;
    startX = event.clientX;
    startY = event.clientY;
  }

  function onPointerUp(event: PointerEvent) {
    if (!tracking) return;
    tracking = false;

    const deltaX = event.clientX - startX;
    const deltaY = event.clientY - startY;

    if (Math.abs(deltaX) < SWIPE_DISTANCE || Math.abs(deltaX) < Math.abs(deltaY) * 1.5) {
      if (current.onPullDown && deltaY > PULL_DISTANCE && deltaY > Math.abs(deltaX) * 1.5) {
        consumedClick = true;
        current.onPullDown();
      }
      return;
    }

    consumedClick = true;
    current.onSwipe(deltaX < 0 ? 1 : -1);
  }

  function onPointerCancel() {
    tracking = false;
  }

  function onClickCapture(event: MouseEvent) {
    if (!consumedClick) return;
    consumedClick = false;
    event.stopPropagation();
    event.preventDefault();
  }

  node.addEventListener('pointerdown', onPointerDown);
  node.addEventListener('pointerup', onPointerUp);
  node.addEventListener('pointercancel', onPointerCancel);
  node.addEventListener('click', onClickCapture, true);

  return {
    update(next: SwipeOptions) {
      current = next;
    },
    destroy() {
      node.removeEventListener('pointerdown', onPointerDown);
      node.removeEventListener('pointerup', onPointerUp);
      node.removeEventListener('pointercancel', onPointerCancel);
      node.removeEventListener('click', onClickCapture, true);
    },
  };
}
