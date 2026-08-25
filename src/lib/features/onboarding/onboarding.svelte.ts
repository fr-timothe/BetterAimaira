import {
  parsePermissionError,
  permissionStates,
  requestPermission,
  type PermissionErrorCode,
  type PermissionKind,
  type PermissionState,
} from './permissions-service';

/** Which panel the reader is on. `permissions` is skipped where none exist. */
export type OnboardingStep = 'welcome' | 'permissions';

const SEEN_KEY = 'betteraimaira.onboarding.seen';
const STEP_KEY = 'betteraimaira.onboarding.step';

/**
 * Whether the introduction has already been read on this device.
 *
 * A missing `localStorage` — the browser preview, a locked-down webview — reads
 * as "already seen": an introduction that cannot record being dismissed would
 * otherwise reappear on every start.
 */
export function onboardingSeen(): boolean {
  if (typeof localStorage === 'undefined') return true;
  return localStorage.getItem(SEEN_KEY) === 'true';
}

function markSeen(): void {
  if (typeof localStorage === 'undefined') return;
  localStorage.setItem(SEEN_KEY, 'true');
  localStorage.removeItem(STEP_KEY);
}

/**
 * The panel to open on start.
 *
 * Granting "install unknown apps" restarts the process on Android, so the
 * introduction comes back from the dead in the middle of itself: without this
 * the reader is dropped back on the welcome panel and has to walk forward again
 * to see whether the right they just granted took.
 */
function storedStep(): OnboardingStep {
  if (typeof localStorage === 'undefined') return 'welcome';
  return localStorage.getItem(STEP_KEY) === 'permissions' ? 'permissions' : 'welcome';
}

function rememberStep(step: OnboardingStep): void {
  if (typeof localStorage === 'undefined') return;
  localStorage.setItem(STEP_KEY, step);
}

/**
 * The state behind the introduction: the panel being shown and the rights the
 * platform still wants.
 *
 * The permission list is read from the Rust side rather than assumed, so a
 * platform that needs nothing collapses the whole step instead of showing an
 * empty page.
 */
class Onboarding {
  step = $state<OnboardingStep>(storedStep());
  permissions = $state<PermissionState[]>([]);
  /** True until the first read answers, so the step never flashes empty. */
  loading = $state(true);
  errorCode = $state<PermissionErrorCode | null>(null);
  /** The right whose settings screen was opened last, awaiting the reader. */
  pending = $state<PermissionKind | null>(null);

  get hasPermissions(): boolean {
    return this.permissions.length > 0;
  }

  /**
   * Whether the introduction may be left behind.
   *
   * A right the app cannot open a screen for is not counted: the reader has no
   * way to grant it from here, and blocking on it would lock them out of the
   * app entirely.
   */
  get allGranted(): boolean {
    return this.permissions.every(
      (permission) => permission.granted || !permission.requestable
    );
  }

  async load(): Promise<void> {
    try {
      this.permissions = await permissionStates();
      // A step restored from a previous run must not outlive the list it was
      // showing: an empty list has no second panel to open.
      if (!this.hasPermissions) this.step = 'welcome';
      this.errorCode = null;
    } catch (error) {
      // A list that cannot be read is reported as empty: the introduction still
      // has to end somewhere, and the update card asks for the right again.
      this.permissions = [];
      this.errorCode = parsePermissionError(error);
    } finally {
      this.loading = false;
    }
  }

  /** Re-reads the rights, for when the app comes back from the settings app. */
  async refresh(): Promise<void> {
    try {
      this.permissions = await permissionStates();
      if (this.pending) {
        const granted = this.permissions.find(
          (permission) => permission.kind === this.pending
        )?.granted;
        if (granted) this.pending = null;
      }
      this.errorCode = null;
    } catch (error) {
      this.errorCode = parsePermissionError(error);
    }
  }

  async request(kind: PermissionKind): Promise<void> {
    this.errorCode = null;
    try {
      await requestPermission(kind);
      this.pending = kind;
    } catch (error) {
      this.pending = null;
      this.errorCode = parsePermissionError(error);
    }
  }

  next(): void {
    this.step = this.hasPermissions ? 'permissions' : 'welcome';
    rememberStep(this.step);
  }

  back(): void {
    this.step = 'welcome';
    rememberStep(this.step);
  }

  /**
   * Ends the introduction, and only then: every right the reader can grant is
   * a condition of entry, so a missing one leaves them on this step.
   *
   * Returns whether the introduction actually ended, so the caller does not
   * hand a reader to the login form who never got past the permission panel.
   */
  finish(): boolean {
    if (!this.allGranted) return false;
    markSeen();
    return true;
  }
}

export const onboarding = new Onboarding();
