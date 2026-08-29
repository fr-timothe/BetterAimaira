import { getLocale, setLocale, type Locale } from '$lib/paraglide/runtime.js';

/**
 * The interface language, as a signal.
 *
 * Paraglide resolves every `m.*()` against a plain module variable, so a message
 * read in markup compiles to an effect with no reactive dependency and never
 * re-runs on a language change. The fix is not to make each read reactive by
 * hand — that is a dependency list nothing in the toolchain checks — but to
 * remount the subtree that renders them, which needs the language itself to be
 * observable. Hence this rune: the shell and the title bar live under different
 * roots and both key on it.
 *
 * `setLocale` is called with `reload: false` on purpose; a reload would throw
 * away the session the reader is signed into.
 */
class AppLocale {
  current = $state<Locale>(getLocale());

  async set(next: Locale) {
    if (next === this.current) return;
    await setLocale(next, { reload: false });
    this.current = next;
  }
}

export const appLocale = new AppLocale();
