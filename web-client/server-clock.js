// The device wall clock and timezone never participate in the hosted game clock.
export async function startServerClock({ fetch, monotonicNow = () => performance.now(),
  schedule = setInterval, onError = () => {} }) {
  let anchor;
  let pending = false;
  let lastSample = 0;
  async function synchronize() {
    if (pending) return;
    pending = true;
    try {
      const started = monotonicNow();
      const response = await fetch('/v1/clock', { cache: 'no-store', signal: AbortSignal.timeout(10000) });
      if (!response.ok) throw new Error('Server clock is unavailable.');
      const sample = await response.json();
      const received = monotonicNow();
      if (!Number.isSafeInteger(sample.unixMillis) || sample.unixMillis < 0 || sample.timeZone !== 'UTC') {
        throw new Error('Invalid server clock response.');
      }
      anchor = { millis: sample.unixMillis + (received - started) / 2, received };
    } finally {
      pending = false;
    }
  }
  await synchronize();
  schedule(() => synchronize().catch(onError), 30000);
  return {
    synchronize,
    sample() {
      const elapsed = monotonicNow() - anchor.received;
      // Suspend gameplay if the server cannot be reached for two minutes.
      if (elapsed < 0 || elapsed > 120000) return null;
      // Network jitter must not cross midnight backwards and repeat daily resets.
      lastSample = Math.max(lastSample, anchor.millis + elapsed);
      return lastSample;
    },
  };
}
