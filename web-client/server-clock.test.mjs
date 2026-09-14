import test from 'node:test';
import assert from 'node:assert/strict';
import { startServerClock } from './server-clock.js';

test('server time advances independently of device wall time and resynchronizes', async () => {
  let now = 100;
  let server = Date.UTC(2026, 8, 6, 23, 59, 59);
  let refresh;
  const clock = await startServerClock({
    monotonicNow: () => now,
    schedule: callback => { refresh = callback; },
    fetch: async (url, options) => {
      assert.equal(url, '/v1/clock');
      assert.equal(options.cache, 'no-store');
      now += 20;
      return { ok: true, json: async () => ({ unixMillis: server, timeZone: 'UTC' }) };
    },
  });
  assert.equal(clock.sample(), server + 10);
  now += 1500;
  assert.equal(new Date(clock.sample()).toISOString(), '2026-09-07T00:00:00.510Z');
  server += 5000;
  await refresh();
  assert.equal(clock.sample(), server + 10);
  now += 120001;
  assert.equal(clock.sample(), null);
  await clock.synchronize();
  assert.equal(clock.sample(), server + 10);
});

test('startup fails closed without a valid server clock', async () => {
  for (const response of [{ ok: false }, { ok: true, json: async () => ({ unixMillis: 123, timeZone: 'local' }) }]) {
    await assert.rejects(startServerClock({ fetch: async () => response, schedule: () => {} }));
  }
});

test('failed refresh never replaces the last server anchor with device time', async () => {
  let now = 0;
  let available = true;
  let refresh;
  let failures = 0;
  const clock = await startServerClock({ monotonicNow: () => now,
    schedule: callback => { refresh = callback; }, onError: () => failures++,
    fetch: async () => ({ ok: available, json: async () => ({ unixMillis: 1000, timeZone: 'UTC' }) }),
  });
  available = false;
  now = 30000;
  await refresh();
  assert.equal(failures, 1);
  assert.equal(clock.sample(), 31000);
  now = 120001;
  assert.equal(clock.sample(), null);
});

test('resynchronization jitter cannot roll the game day backwards', async () => {
  let server = Date.UTC(2026, 8, 7);
  const clock = await startServerClock({ monotonicNow: () => 0, schedule: () => {},
    fetch: async () => ({ ok: true, json: async () => ({ unixMillis: server, timeZone: 'UTC' }) }),
  });
  const midnight = clock.sample();
  server -= 20;
  await clock.synchronize();
  assert.equal(clock.sample(), midnight);
});

test('players with different device timer origins receive the same server time', async () => {
  const sample = { unixMillis: Date.UTC(2026, 8, 6, 10, 30), timeZone: 'UTC' };
  const create = origin => startServerClock({ monotonicNow: () => origin,
    schedule: () => {}, fetch: async () => ({ ok: true, json: async () => sample }),
  });
  const first = await create(5);
  const second = await create(987654321);
  assert.equal(first.sample(), second.sample());
  assert.equal(first.sample(), sample.unixMillis);
});
