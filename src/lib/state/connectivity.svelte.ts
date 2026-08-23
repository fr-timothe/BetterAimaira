/**
 * Whether the machine currently has a network path. Every portal failure used to
 * surface as "the portal is unavailable", which is the wrong message — and the
 * wrong recovery — when it is the device that is offline.
 *
 * `navigator.onLine` only proves the interface is up, not that the portal is
 * reachable, so treat a false value as authoritative and a true value as "worth
 * trying".
 */
class Connectivity {
  online = $state(true);

  constructor() {
    if (typeof window === 'undefined') return;

    this.online = navigator.onLine;
    window.addEventListener('online', this.#handleOnline);
    window.addEventListener('offline', this.#handleOffline);
  }

  #handleOnline = () => {
    this.online = true;
  };

  #handleOffline = () => {
    this.online = false;
  };
}

export const connectivity = new Connectivity();
