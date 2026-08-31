import { tick } from 'svelte';
import { pushState } from '$app/navigation';

/** What one back press does on the screen that claimed the gesture. */
type BackAction = () => void;

/**
 * The platform's own back gesture — Android's hardware button and the edge
 * swipe that replaced it — routed to whatever step the screen on top can
 * return to.
 *
 * A back press on Android walks the webview's history, so a step that can be
 * returned to is held open by exactly one extra entry: the guard. Popping it
 * runs the deepest claim and the guard goes straight back up while a further
 * step remains behind it, so a three-panel introduction walks backwards one
 * press at a time and the press on its first panel falls through to the system
 * and leaves the app — which is what an Android reader expects. Nothing is
 * pushed while no screen claims the gesture, so the app never grows a history
 * entry that swallows a press and appears to do nothing.
 *
 * The entry goes through SvelteKit's shallow router rather than `history`
 * directly: the router owns the history state on this page, and an entry it did
 * not stamp is popped as a real navigation, which would re-run the page and
 * discard everything the reader had typed.
 */
class BackGesture {
  /**
   * One claim per screen, in the order the screens claimed. Insertion order is
   * the depth order — a sheet claims over the screen under it — and `Map.set`
   * keeps an existing key in place, so a claim that only changes its action
   * never reorders itself above a sheet opened after it.
   */
  #actions = new Map<string, BackAction>();
  #guarded = false;
  /** True while the guard entry is being taken out by us, not by the reader. */
  #dropping = false;
  #syncQueued = false;

  constructor() {
    if (typeof window === 'undefined') return;
    window.addEventListener('popstate', this.#handlePop);
  }

  /**
   * States what the back gesture does on one screen, or clears the claim with
   * `null` when that screen has nowhere left to go.
   *
   * Meant to be called from an effect: it is safe to call on every change, and
   * the last screen to claim is the one the gesture reaches.
   */
  claim(id: string, action: BackAction | null): void {
    if (action) {
      this.#actions.set(id, action);
    } else if (!this.#actions.delete(id)) {
      return;
    }
    this.#scheduleSync();
  }

  #handlePop = () => {
    if (this.#dropping) {
      this.#dropping = false;
      return;
    }
    // Somebody else's entry, or the base one: the press belongs to the system.
    if (!this.#guarded) return;
    this.#guarded = false;

    const action = this.#deepestAction();
    if (!action) return;
    action();
    this.#scheduleSync();
  };

  #deepestAction(): BackAction | null {
    let deepest: BackAction | null = null;
    for (const action of this.#actions.values()) deepest = action;
    return deepest;
  }

  /**
   * Coalesced through `tick`, for two reasons: a claim released and re-taken in
   * the same flush must not spend a history entry, and a claim answering a press
   * has to be read after the state it just changed has settled.
   */
  #scheduleSync(): void {
    if (this.#syncQueued) return;
    this.#syncQueued = true;
    void tick().then(() => {
      this.#syncQueued = false;
      this.#sync();
    });
  }

  #sync(): void {
    if (typeof window === 'undefined') return;
    const wanted = this.#actions.size > 0;
    if (wanted === this.#guarded) return;

    if (wanted) {
      this.#guarded = true;
      // The current address, unchanged: the app has one route, and the entry
      // exists to be popped, not to be linked to.
      pushState('', {});
      return;
    }

    // The screen ran out of steps while the guard was up. Left in place, the
    // next press would land on it and visibly do nothing.
    this.#guarded = false;
    this.#dropping = true;
    history.back();
  }
}

export const backGesture = new BackGesture();
