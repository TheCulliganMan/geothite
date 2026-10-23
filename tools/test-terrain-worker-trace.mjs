import assert from 'node:assert/strict';
import { mergeTerrainWorkerTrace, mergeActorWorkerTrace } from './terrain-worker-trace.mjs';
const main = { traceEvents: [{ name: 'worker.callback_header', ph: 'X', ts: 2_000_000, dur: 500,
  args: { job: 3 } }], droppedEvents: 0 };
const job = { id: 3, cpuStart: 1000, traceOriginEpoch: 1001, cpuEnd: 1100, headerCallbackEpoch: 1200,
  trace: JSON.stringify({ traceEvents: [{ name: 'region.build_source', ph: 'X', ts: 1000, dur: 50000,
    pid: 1, tid: 1, args: { map: 'NewBarkTown' } }], droppedEvents: 0 }) };
const result = mergeTerrainWorkerTrace(main, [job]);
assert.equal(result.trace.traceEvents.find(e => e.name === 'worker.cpu_job').ts, 1_800_000);
assert.equal(result.trace.traceEvents.find(e => e.name === 'region.build_source').ts, 1_802_000);
assert.equal(result.trace.traceEvents.at(-1).tid, 2);
assert.equal(main.traceEvents.length, 1);
assert.throws(() => mergeTerrainWorkerTrace(main, [{ ...job, id: 4 }]), /header callback/);
assert.throws(() => mergeTerrainWorkerTrace(main, [{ ...job, cpuEnd: 1300 }]), /precede/);
assert.throws(() => mergeTerrainWorkerTrace(main, [{ ...job, trace: '{"traceEvents":[],"droppedEvents":1}' }]), /dropped/);
console.log('terrain worker trace: epoch alignment, separate lane and invalid captures PASS');

const actorMain = { ...main, traceEvents: [...main.traceEvents,
  { ...main.traceEvents[0], name: 'actor_worker.callback_header' }] };
const actorMerged = mergeActorWorkerTrace(result.trace.traceEvents.length ?
  { ...result.trace, traceEvents: [...result.trace.traceEvents, actorMain.traceEvents.at(-1)] } : actorMain, [job]);
assert.equal(actorMerged.trace.traceEvents.find(e => e.name === 'actor_worker.cpu_job').tid, 3);
assert.equal(actorMerged.trace.traceEvents.at(-1).args.actorWorkerJob, 3);
assert.ok(actorMerged.trace.traceEvents.some(e => e.tid === 2));
console.log('actor worker trace: independent lane and equal job IDs cannot collide with terrain PASS');

const parallelMain = { ...main, traceEvents: [...main.traceEvents,
  { ...main.traceEvents[0], args: { job: 4 } }] };
const parallel = mergeTerrainWorkerTrace(parallelMain, [job, { ...job, id: 4, worker: 1 }]);
assert.deepEqual(parallel.trace.traceEvents.filter(e => e.name === 'worker.cpu_job').map(e => e.tid), [2, 4]);
assert.throws(() => mergeTerrainWorkerTrace(main, [{ ...job, worker: -1 }]), /worker slot/);
console.log('parallel terrain workers: independent overlapping lanes preserve actor lane 3 PASS');
