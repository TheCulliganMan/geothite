// Transport regression: initialization must settle before jobs, and page ACKs
// follow successful Rust-side consumption. No browser or generated Wasm needed.
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
const pending = [];
const workers = [];
globalThis.Worker = class {
  constructor() { this.sent = []; workers.push(this); }
  postMessage(message) { this.sent.push(message); }
  emit(data) { this.onmessage({ data }); }
  terminate() { this.terminated = true; }
};
globalThis.MessageChannel = class {
  constructor() {
    this.port1 = { close() {} };
    this.port2 = { close() {}, postMessage: () => pending.push(() => this.port1.onmessage()) };
  }
};
globalThis.requestAnimationFrame = callback => pending.push(callback);
const moduleUrl = new URL('../web-client/terrain-worker-client.js', import.meta.url);
const source = (await readFile(moduleUrl, 'utf8')).replaceAll('import.meta.url', JSON.stringify(moduleUrl.href));
const { createTerrainWorkerClient } = await import(`data:text/javascript;base64,${Buffer.from(source).toString('base64')}`);
function drain() { let turns = 0; while (pending.length) { assert.ok(++turns < 100, 'pump stalled'); pending.shift()(); } }
const calls = [];
const client = createTerrainWorkerClient({
  onHeader: id => calls.push(['header', id]),
  onPage: id => calls.push(['page', id]),
  onAdvance: id => { calls.push(['advance', id]); return true; },
  onComplete: id => calls.push(['complete', id]),
  onCancel: id => calls.push(['cancel', id]),
  onError: (id, error) => calls.push(['error', id, error]),
});
const worker = workers.at(-1);
const boot = client.initialize('boot');
assert.throws(() => client.submit(0, new Uint8Array(1)), /pending/);
worker.emit({ id: 'boot', kind: 'ready' });
await boot;
const ready = client.installScenery('initial-models', '{}');
assert.throws(() => client.submit(1, new Uint8Array(1)), /pending/);
worker.emit({ id: 'initial-models', kind: 'ready' });
await ready;
client.submit(1, new Uint8Array(1));
worker.emit({ id: 1, kind: 'header', header: new ArrayBuffer(1) });
const page = new ArrayBuffer(40);
const view = new DataView(page);
view.setUint32(16, 7, true); view.setUint32(20, 256, true);
worker.emit({ id: 1, kind: 'page', page });
worker.emit({ id: 1, kind: 'done' });
drain();
assert.deepEqual(calls, [['header', 1], ['page', 1], ['advance', 1], ['complete', 1]]);
assert.deepEqual(worker.sent.at(-1), { kind: 'ack', id: 1, buffer: 7, offset: 256 });
client.submit(2, new Uint8Array(1)); client.cancel();
worker.emit({ id: 2, kind: 'header', header: new ArrayBuffer(1) }); drain();
assert.equal(calls.at(-1)[0], 'cancel');
const failed = client.installScenery('replacement-models', '{}');
worker.emit({ id: 'replacement-models', requestKind: 'scenery', error: 'invalid source' });
await assert.rejects(failed, /invalid source/);
assert.throws(() => client.submit(3, new Uint8Array(1)), /invalid source/);
assert.equal(calls.filter(call => call[0] === 'error').length, 0);
const disposed = client.installScenery('pending-models', '{}');
client.dispose();
await assert.rejects(disposed, /disposed/);
assert.ok(worker.terminated);
console.log('terrain worker transport: bootstrap, exact page ACK, stale cancellation and failure gating PASS');

// Exercise the actual worker transport with a fake Rust boundary. The Rust
// paging quota is separate from this check; here ACKs unblock its next_page.
const { runInNewContext } = await import('node:vm');
const workerSource = (await readFile(new URL('../web-client/terrain-worker.js', import.meta.url), 'utf8'))
  .replace(/^import .* from .*;$/m, '');
const responses = [];
const jobs = [];
const self = { postMessage(message) { responses.push(message); } };
runInNewContext(workerSource, {
  self, performance, Uint8Array, Set, Promise,
  init: () => Promise.resolve(),
  install_scenery_sources: json => { if (json === 'bad') throw new Error('invalid models'); },
  build_terrain: () => new Uint8Array([1]),
  begin_terrain_job: () => {
    const job = {
      sent: 0, inFlight: 0, freed: false,
      header: () => new Uint8Array([1]), raw_bytes: () => 24, trace_origin_epoch: () => 4,
      next_page() { if (this.inFlight === 2 || this.sent === 3) return new Uint8Array(); this.sent++; this.inFlight++; return new Uint8Array(40); },
      acknowledge() { this.inFlight--; },
      finished() { return this.sent === 3 && this.inFlight === 0; },
      trace_json: () => '{}', free() { this.freed = true; },
    };
    jobs.push(job); return job;
  },
});
const flush = () => new Promise(resolve => setImmediate(resolve));
self.onmessage({ data: { kind: 'initialize', id: 4 } });
await flush();
assert.equal(responses.at(-1).kind, 'ready');
self.onmessage({ data: { kind: 'scenery', id: 5, json: 'bad' } });
await flush();
assert.equal(responses.at(-1).requestKind, 'scenery');
assert.match(responses.at(-1).error, /invalid models/);
self.onmessage({ data: { kind: 'terrain-pages', id: 6, request: new ArrayBuffer(1) } });
await flush();
assert.equal(responses.filter(message => message.kind === 'page').length, 2);
self.onmessage({ data: { kind: 'ack', id: 6, buffer: 0, offset: 0 } });
assert.equal(responses.filter(message => message.kind === 'page').length, 3);
self.onmessage({ data: { kind: 'ack', id: 6, buffer: 0, offset: 0 } });
self.onmessage({ data: { kind: 'ack', id: 6, buffer: 0, offset: 0 } });
assert.equal(responses.at(-1).kind, 'done');
assert.ok(jobs[0].freed);
self.onmessage({ data: { kind: 'terrain-pages', id: 7, request: new ArrayBuffer(1) } });
await flush();
self.onmessage({ data: { kind: 'cancel', id: 7 } });
assert.ok(jobs[1].freed);
const responseCount = responses.length;
self.onmessage({ data: { kind: 'ack', id: 7, buffer: 0, offset: 0 } });
assert.equal(responses.length, responseCount);
console.log('terrain worker producer: initialization error, two-page ACK backpressure, completion and cancellation PASS');

// The QA bridge waits for both Wasm boot and the exact main-installed scenery
// payload; it never starts a second artwork fetch or a job from the 2D path.
const bridgeSource = (await readFile(new URL('../web-client/terrain-worker-bridge.js', import.meta.url), 'utf8'))
  .replace(/^import .* from .*;$/m, 'const createTerrainWorkerClient = globalThis.__terrainTestClientFactory;')
  .replace("'./cpu-worker-lifecycle.js'", JSON.stringify(new URL('../web-client/cpu-worker-lifecycle.js', import.meta.url).href));
let resolveBoot;
let resolveModels;
const bridgeCalls = [];
let bridgeCallbacks;
globalThis.__terrainTestClientFactory = callbacks => {
  bridgeCallbacks = callbacks;
  return {
    initialize: () => new Promise(resolve => { resolveBoot = resolve; }),
    installScenery: (id, json) => { bridgeCalls.push(['models', id, json]); return new Promise(resolve => { resolveModels = resolve; }); },
    submit: (id, request, trace) => bridgeCalls.push(['submit', id, request.length, trace]),
    cancel: () => bridgeCalls.push(['cancel']), dispose: () => bridgeCalls.push(['dispose']),
  };
};
const { installTerrainWorkerBridge } = await import(`data:text/javascript;base64,${Buffer.from(bridgeSource).toString('base64')}`);
assert.equal(installTerrainWorkerBridge({}), null);
const scope = {};
const bridge = installTerrainWorkerBridge({
  crystal_terrain_worker_header: (id, bytes) => bridgeCalls.push(['header', id, bytes.length]),
  crystal_terrain_worker_page: (id, bytes) => bridgeCalls.push(['page', id, bytes.length]),
  crystal_terrain_worker_advance: id => { bridgeCalls.push(['advance', id]); return true; },
  crystal_terrain_worker_callback_timing: (id, stage, milliseconds, bytes) => bridgeCalls.push(['timing', id, stage, milliseconds >= 0, bytes]),
  crystal_terrain_worker_complete: id => bridgeCalls.push(['complete', id]),
  crystal_terrain_worker_error: (id, error) => bridgeCalls.push(['error', id, error]),
}, { scope, trace: true });
assert.equal(scope.__crystalTerrainWorkerReady(), 1);
scope.__crystalTerrainWorkerScenery('exact-authoritative-sources');
resolveBoot(); await flush();
assert.deepEqual(bridgeCalls[0], ['models', 'models', 'exact-authoritative-sources']);
assert.equal(scope.__crystalTerrainWorkerReady(), 1);
resolveModels(); await flush();
assert.equal(scope.__crystalTerrainWorkerReady(), 2);
scope.__crystalTerrainWorkerSubmit(20, new Uint8Array(4), false);
assert.deepEqual(bridgeCalls.at(-1), ['submit', 20, 4, true]);
bridgeCallbacks.onHeader(20, new Uint8Array(12));
assert.deepEqual(bridgeCalls.at(-1), ['timing', 20, 0, true, 12]);
bridgeCallbacks.onPage(20, new Uint8Array(100));
assert.deepEqual(bridgeCalls.at(-1), ['timing', 20, 1, true, 100]);
assert.equal(bridgeCallbacks.onAdvance(20, 4), true);
assert.deepEqual(bridgeCalls.at(-1), ['timing', 20, 2, true, 0]);
bridgeCallbacks.onComplete(20, { trace: '{}', cpuStart: 4, cpuEnd: 8 }, { submit: 1 });
assert.equal(scope.__crystalTerrainWorkerTraces.length, 1);
bridgeCallbacks.onError(21, 'worker failed');
assert.equal(scope.__crystalTerrainWorkerReady(), 0);
assert.deepEqual(bridgeCalls.at(-1), ['error', 21, 'worker failed']);
bridge.dispose();
delete globalThis.__terrainTestClientFactory;
console.log('terrain QA bridge: feature gate, exact scenery readiness, trace collection and fail-closed fallback PASS');
