import { invoke, isTauri } from '$lib/invoke';

/**
 * The events this interface may report. The Rust side keeps the same list and
 * refuses anything outside it, so a name added here without being added there
 * fails loudly instead of shipping a silent no-op.
 */
export type AnalyticsEvent = 'app_launched' | 'login_succeeded' | 'login_failed';

export type AnalyticsStatus = {
  /** Whether this build has a reporting destination at all. */
  available: boolean;
  /** Whether the reader has answered once. Never asked is not the same as no. */
  decided: boolean;
  enabled: boolean;
};

const UNAVAILABLE: AnalyticsStatus = { available: false, decided: true, enabled: false };

/** The browser preview has no Rust side, so it has nothing to report with. */
export async function analyticsStatus(): Promise<AnalyticsStatus> {
  if (!isTauri()) return UNAVAILABLE;
  return invoke<AnalyticsStatus>('analytics_status');
}

export async function setAnalyticsConsent(enabled: boolean): Promise<AnalyticsStatus> {
  if (!isTauri()) return UNAVAILABLE;
  return invoke<AnalyticsStatus>('set_analytics_consent', { enabled });
}

/**
 * Reports `event`, unless the reader declined or this build cannot report.
 *
 * The Rust side owns that decision, so callers never guard on consent — and the
 * call is deliberately fire-and-forget: a usage counter has no business failing
 * a screen, so a rejected capture is swallowed here rather than surfaced.
 *
 * `variant` is the only payload allowed, and only as a short lowercase token
 * such as a stable error code. Never pass anything read from the portal.
 */
export function captureEvent(event: AnalyticsEvent, variant?: string): void {
  if (!isTauri()) return;
  void invoke<void>('capture_analytics_event', { event, variant: variant ?? null }).catch(
    (error) => {
      // A malformed call is a bug in this file, not a runtime condition, so it
      // stays visible in the console instead of vanishing.
      if (import.meta.env.DEV) console.warn('analytics: capture rejected', error);
    }
  );
}
