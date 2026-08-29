/**
 * The one polite live region the application owns.
 *
 * A screen reader only speaks an `aria-live="polite"` change it observes while
 * the region is already in the accessibility tree. Almost every region in this
 * app is mounted together with the text it wants announced — a loading skeleton
 * appears with its `aria-live` wrapper, and a tab change destroys the wrapper
 * again — so those regions are silent by construction. Holding the message here
 * lets `+layout.svelte` render it from a region that outlives every view,
 * including the `{#key sessionEpoch}` remount, and therefore actually speaks.
 *
 * `role="alert"` banners are deliberately not routed through this: an assertive
 * region inserted into the DOM is announced on its own, and duplicating it here
 * would say everything twice.
 */

/**
 * Long enough for Svelte to flush the blanked region to the DOM before the text
 * comes back, short enough that the repeat still reads as one utterance.
 */
const REPEAT_DELAY_MS = 120;

class Announcer {
  message = $state('');

  /** Pending restore of a repeated message; see `announce`. */
  #repeat: ReturnType<typeof setTimeout> | undefined;

  announce = (message: string) => {
    const text = message.trim();
    if (!text) return;

    if (this.#repeat !== undefined) {
      clearTimeout(this.#repeat);
      this.#repeat = undefined;
    }

    if (text !== this.message) {
      this.message = text;
      return;
    }

    // Writing the string the region already holds changes nothing, so most
    // screen readers stay quiet — and two refreshes that fail the same way in a
    // row have to be heard twice. Blanking the region and restoring the text on
    // a later task is a real change, which is what gets announced.
    this.message = '';
    this.#repeat = setTimeout(() => {
      this.message = text;
      this.#repeat = undefined;
    }, REPEAT_DELAY_MS);
  };
}

export const announcer = new Announcer();
