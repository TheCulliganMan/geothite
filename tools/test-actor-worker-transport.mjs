// Actual actor-worker JS boundaries with mock Rust, never a model/game copy.
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { runInNewContext } from 'node:vm';
const url = new URL('../web-client/actor-worker.js', import.meta.url);
const source = (await readFile(url, 'utf8')).replace(/^import .* from .*;$/m, '');
const responses = [], jobs = [], people = [];
const self = { postMessage: message => responses.push(message) };
runInNewContext(source, { self, performance, Uint8Array, Set, Promise, JSON,
  init: () => Promise.resolve(),
  install_people_sources: json => { people.push(json); },
  begin_actor_job: () => {
    const job = { sent: 0, outstanding: 0, freed: false,
      header: () => new Uint8Array([1]), raw_bytes: () => 24, trace_origin_epoch: () => 4,
      next_page() { if (this.outstanding === 2 || this.sent === 3) return new Uint8Array(); this.sent++; this.outstanding++; return new Uint8Array(40); },
      acknowledge() { this.outstanding--; }, finished() { return this.sent === 3 && this.outstanding === 0; },
      trace_json: () => '{}', cache_json: () => '{"rigs":[["human-male",true]],"appearance_cache_keys":[]}',
      free() { this.freed = true; },
    }; jobs.push(job); return job;
  },
});
const flush = () => new Promise(resolve => setImmediate(resolve));
self.onmessage({ data: { kind: 'initialize', id: 'boot' } }); await flush();
assert.equal(responses.at(-1).kind, 'ready');
self.onmessage({ data: { kind: 'scenery', id: 'people', json: 'exact-main-installed-people' } }); await flush();
assert.deepEqual(people, ['exact-main-installed-people']);
self.onmessage({ data: { kind: 'actor-pages', id: 1, request: new ArrayBuffer(1) } }); await flush();
assert.equal(responses.filter(r => r.kind === 'page').length, 2, 'only two pages before ACK');
assert.equal(jobs[0].freed, false);
for (let i = 0; i < 3; i++) self.onmessage({ data: { kind: 'ack', id: 1, buffer: i, offset: 0 } });
assert.equal(responses.filter(r => r.kind === 'page').length, 3);
assert.equal(responses.at(-1).kind, 'done');
assert.equal(jobs[0].freed, true);
assert.deepEqual(responses.at(-1).cache.rigs, [['human-male', true]]);
self.onmessage({ data: { kind: 'actor-pages', id: 2, request: new ArrayBuffer(1) } }); await flush();
self.onmessage({ data: { kind: 'cancel', id: 2 } });
assert.equal(jobs[1].freed, true);
const count = responses.length;
self.onmessage({ data: { kind: 'ack', id: 2, buffer: 0, offset: 0 } });
assert.equal(responses.length, count, 'stale ACK cannot revive retired interest');

const bridgeUrl = new URL('../web-client/actor-worker-bridge.js', import.meta.url);
const bridgeSource = (await readFile(bridgeUrl, 'utf8'))
  .replace(/^import .* from .*;$/m, 'const createTerrainWorkerClient = globalThis.__actorTestClientFactory;')
  .replaceAll('import.meta.url', JSON.stringify(bridgeUrl.href))
  .replace("'./cpu-worker-lifecycle.js'", JSON.stringify(new URL('../web-client/cpu-worker-lifecycle.js', import.meta.url).href));
let callbacks, options, bootResolve, peopleResolve;
const calls = [];
globalThis.__actorTestClientFactory = (received, config) => {
  callbacks = received; options = config;
  return { initialize: () => new Promise(resolve => { bootResolve = resolve; }),
    installScenery: (_, json) => { calls.push(json); return new Promise(resolve => { peopleResolve = resolve; }); },
    submit: id => calls.push(['submit', id]), cancel: () => calls.push('cancel'), dispose: () => calls.push('dispose') };
};
const { installActorWorkerBridge } = await import(`data:text/javascript;base64,${Buffer.from(bridgeSource).toString('base64')}`);
assert.equal(installActorWorkerBridge({}), null);
const scope = {}, sharedBudget = {};
const bridge = installActorWorkerBridge({
  crystal_actor_worker_header() {}, crystal_actor_worker_page() {},
  crystal_actor_worker_advance: () => true, crystal_actor_worker_complete() {},
  crystal_actor_worker_error: (_, error) => calls.push(error), crystal_actor_worker_callback_timing() {},
}, { scope, trace: true, sharedBudget });
assert.equal(options.jobKind, 'actor-pages');
assert.ok(options.workerUrl.href.endsWith('/actor-worker.js'));
assert.equal(options.sharedBudget, sharedBudget);
scope.__crystalActorWorkerPeople('exact-main-installed-people');
assert.equal(scope.__crystalActorWorkerReady(), 1);
bootResolve(); await flush(); assert.equal(calls[0], 'exact-main-installed-people');
peopleResolve(); await flush(); assert.equal(scope.__crystalActorWorkerReady(), 2);
scope.__crystalActorWorkerSubmit(3, new Uint8Array(1), true);
callbacks.onHeader(3, new Uint8Array(1));
callbacks.onComplete(3, { trace: '{}', cache: { rigs: [['human-male', true]] }, workerMemory: { peakBytes: 123 } }, {});
assert.deepEqual(scope.__crystalActorWorkerMetrics.successfulJobIds, [3]);
assert.deepEqual(scope.__crystalActorWorkerMetrics.cacheHistory[0].cache.rigs, [['human-male', true]]);
callbacks.onError(4, 'invalid actor');
assert.equal(scope.__crystalActorWorkerReady(), 0);
assert.equal(scope.__crystalActorWorkerMetrics.failed, 1);
bridge.dispose(); delete globalThis.__actorTestClientFactory;
console.log('actor worker: exact people payload, isolated endpoint, shared CPU budget, ACK backpressure, stale cancellation, cache/memory diagnostics and failure gating PASS');
