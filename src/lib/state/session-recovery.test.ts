import { beforeEach, describe, expect, it, mock } from 'bun:test';

/**
 * The command log every test reads its verdict from. Nothing here asserts on a
 * network call: the guarantee is which replays are attempted at all.
 */
let invoked: string[] = [];
let restoreStatus: 'restored' | 'no_credentials' | 'credentials_rejected' = 'restored';
let restoreFailure: unknown = null;
let runningInTauri = true;

function stubInvoke() {
  mock.module('$lib/invoke', () => ({
    isTauri: () => runningInTauri,
    invoke: async (command: string) => {
      invoked.push(command);
      if (restoreFailure) throw restoreFailure;
      return { status: restoreStatus };
    },
  }));
}

stubInvoke();

const { sessionRecovery } = await import('$lib/state/session-recovery.svelte');
const { connectivity } = await import('$lib/state/connectivity.svelte');

beforeEach(() => {
  stubInvoke();
  invoked = [];
  restoreStatus = 'restored';
  restoreFailure = null;
  runningInTauri = true;
  connectivity.online = true;
  sessionRecovery.reset();
});

describe('session recovery replays the saved password once per expiry', () => {
  it('answers every caller of one expiry with a single replay', async () => {
    // Five resources notice the same dead session at the same moment. Signing
    // in five times would be five sessions the portal has to accept.
    const verdicts = await Promise.all([
      sessionRecovery.recover(),
      sessionRecovery.recover(),
      sessionRecovery.recover(),
    ]);

    expect(invoked).toEqual(['restore_session']);
    // All of them get the real answer: each has a read of its own to retry, so
    // answering false to the losers would strand four views out of five.
    expect(verdicts).toEqual([true, true, true]);
    expect(sessionRecovery.recoveries).toBe(1);
    expect(sessionRecovery.status).toBe('idle');
  });

  it('is busy only while the replay is in flight', async () => {
    const pending = sessionRecovery.recover();
    expect(sessionRecovery.busy).toBe(true);

    await pending;
    expect(sessionRecovery.busy).toBe(false);
  });

  it('lets an expiry after a real sign-in replay again', async () => {
    expect(await sessionRecovery.recover()).toBe(true);
    // A sign-in the reader performed themselves makes every earlier verdict
    // stale, budget included; the next expiry is a first suspicion again.
    sessionRecovery.reset();

    expect(await sessionRecovery.recover()).toBe(true);
    expect(invoked).toEqual(['restore_session', 'restore_session']);
  });
});

describe('session recovery refuses to sign in on a loop', () => {
  it('gives up after two expiries that follow a replay straight away', async () => {
    expect(await sessionRecovery.recover()).toBe(true);

    // A portal that caps concurrent sessions hands back a session it kills on
    // the next read. Replaying that is a sign-in loop, not a recovery.
    expect(await sessionRecovery.recover()).toBe(false);
    expect(sessionRecovery.status).toBe('unreachable');

    expect(await sessionRecovery.recover()).toBe(false);
    expect(sessionRecovery.status).toBe('exhausted');

    // Exactly one sign-in reached the portal in the whole sequence.
    expect(invoked).toEqual(['restore_session']);
  });

  it('keeps refusing the request path once it has given up', async () => {
    await exhaust();

    // Reads keep arriving every time the reader changes tab. Each one would
    // otherwise restart the budget and cost the portal another sign-in.
    expect(await sessionRecovery.recover()).toBe(false);
    expect(await sessionRecovery.recover()).toBe(false);
    expect(invoked).toEqual([]);
    expect(sessionRecovery.status).toBe('exhausted');
  });

  it('still lets the reader ask for one more go from the card', async () => {
    await exhaust();

    // The button on the expired card is a person, not a loop: they have seen
    // the app give up and asked anyway.
    expect(await sessionRecovery.recover({ prompted: true })).toBe(true);
    expect(invoked).toEqual(['restore_session']);
    expect(sessionRecovery.status).toBe('idle');
  });

  it('starts the budget over after a prompted replay, rather than giving up at once', async () => {
    await exhaust();
    invoked = [];

    expect(await sessionRecovery.recover({ prompted: true })).toBe(true);
    // The tap bought a full budget back, so the next expiry is refused as a
    // first suspicion instead of landing straight back on `exhausted`.
    expect(await sessionRecovery.recover()).toBe(false);
    expect(sessionRecovery.status).toBe('unreachable');
  });
});

describe('session recovery tells the reader which failures are worth retrying', () => {
  it('does not spend the loop budget on an attempt with no network', async () => {
    connectivity.online = false;

    expect(await sessionRecovery.recover()).toBe(false);
    expect(invoked).toEqual([]);
    expect(sessionRecovery.status).toBe('unreachable');
    // Offline says nothing about the saved password, so the retry is honest.
    expect(sessionRecovery.retryable).toBe(true);

    connectivity.online = true;
    expect(await sessionRecovery.recover()).toBe(true);
  });

  it('stops offering a retry when the portal refused the saved password', async () => {
    restoreStatus = 'credentials_rejected';

    expect(await sessionRecovery.recover()).toBe(false);
    expect(sessionRecovery.status).toBe('rejected');
    // Nothing the app can do alone fixes a password the portal rejects.
    expect(sessionRecovery.retryable).toBe(false);
  });

  it('stops offering a retry when there was never a password to replay', async () => {
    restoreStatus = 'no_credentials';

    expect(await sessionRecovery.recover()).toBe(false);
    expect(sessionRecovery.status).toBe('unavailable');
    expect(sessionRecovery.retryable).toBe(false);
  });

  it('treats an unreadable keyring as terminal and a portal error as worth retrying', async () => {
    restoreFailure = { code: 'credential_store' };
    expect(await sessionRecovery.recover()).toBe(false);
    expect(sessionRecovery.status).toBe('unavailable');

    sessionRecovery.reset();
    restoreFailure = { code: 'portal_unreachable' };
    expect(await sessionRecovery.recover()).toBe(false);
    expect(sessionRecovery.status).toBe('unreachable');
    expect(sessionRecovery.retryable).toBe(true);
  });

  it('never reaches for a backend the browser preview does not have', async () => {
    runningInTauri = false;

    expect(await sessionRecovery.recover()).toBe(false);
    expect(invoked).toEqual([]);
    expect(sessionRecovery.status).toBe('unavailable');
    // No amount of waiting adds a Rust side, so no retry is offered.
    expect(sessionRecovery.retryable).toBe(false);
  });
});

/** Two expiries on the heels of a replay: the state the loop guard gives up in. */
async function exhaust() {
  await sessionRecovery.recover();
  await sessionRecovery.recover();
  await sessionRecovery.recover();
  expect(sessionRecovery.status).toBe('exhausted');
  invoked = [];
}
