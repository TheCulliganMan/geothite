// The matching Rust renderer feature gates this bridge. Dormant 2D sessions
// create no worker; the first authoritative 3D request activates it.
import { createTerrainWorkerClient } from './terrain-worker-client.js';
import { createCpuWorkerLifecycle } from './cpu-worker-lifecycle.js';
export function installActorWorkerBridge(wasm, { trace = false, scope = globalThis, sharedBudget = null, eager = true } = {}) {
  if (typeof wasm.crystal_actor_worker_header !== 'function') return null;
  const traces = [];
  const metrics = { cacheHistory: [], cacheHistoryEvicted: 0, traceEvicted: 0, initializations: 0, initFailures: 0, modelsInstalled: 0, fallback: 0, status: 'dormant', successfulJobIds: [], submitted: 0, completed: 0, cancelled: 0, failed: 0, stages: [0, 1, 2].map(() => ({ count: 0, maxMs: 0, totalMs: 0 })) };
  const headerEpochs = new Map();
  scope.__crystalActorWorkerTraces = traces;
  scope.__crystalActorWorkerMetrics = metrics;
  const timed = (id, stage, bytes, operation) => {
    const start = performance.now();
    try { return operation(); }
    finally {
      const elapsed = performance.now() - start;
      const metric = metrics.stages[stage];
      metric.count++; metric.totalMs += elapsed; metric.maxMs = Math.max(metric.maxMs, elapsed);
      if (stage === 0) headerEpochs.set(id, performance.timeOrigin + start);
      wasm.crystal_actor_worker_callback_timing(id, stage, elapsed, bytes);
    }
  };
  let lifecycle;
  const callbacks = {
    onWorkerState: data => { if (data.workerMemory) metrics.workerMemory = data.workerMemory; },
    onHeader: (id, bytes) => timed(id, 0, bytes.byteLength, () => wasm.crystal_actor_worker_header(id, bytes)),
    onPage: (id, bytes) => timed(id, 1, bytes.byteLength, () => wasm.crystal_actor_worker_page(id, bytes)),
    onAdvance: (id, budget) => timed(id, 2, 0, () => wasm.crystal_actor_worker_advance(id, budget)),
    onComplete: (id, result, timing) => {
      wasm.crystal_actor_worker_complete(id);
      metrics.completed++;
      metrics.successfulJobIds.push(id);
      if (metrics.successfulJobIds.length > 16) metrics.successfulJobIds.shift();
      metrics.lastRawBytes = timing?.rawBytes;
      metrics.workerMemory = result.workerMemory ?? timing?.workerMemory ?? null;
      metrics.cache = result.cache ?? null;
      metrics.cacheHistory.push({ id, cache: metrics.cache, memory: metrics.workerMemory });
      if (metrics.cacheHistory.length > 64) { metrics.cacheHistory.shift(); metrics.cacheHistoryEvicted++; }
      if (trace) {
        // Worker epoch start aligns its relative Chrome events with the main
        // lane when the capture tool merges this bounded diagnostic history.
        traces.push({ id, ...timing, cpuStart: result.cpuStart, cpuEnd: result.cpuEnd, traceOriginEpoch: result.traceOriginEpoch, headerCallbackEpoch: headerEpochs.get(id), trace: result.trace });
        if (traces.length > 8) { traces.shift(); metrics.traceEvicted++; }
      }
      headerEpochs.delete(id);
    },
    onCancel: id => { metrics.cancelled++; headerEpochs.delete(id); },
    onError: (id, error) => {
      lifecycle.fail(); metrics.failed++;
      headerEpochs.delete(id);
      if (id != null) wasm.crystal_actor_worker_error(id, error);
    },
  };
  lifecycle = createCpuWorkerLifecycle(() => createTerrainWorkerClient(callbacks, { workerUrl: new URL('./actor-worker.js', import.meta.url), jobKind: 'actor-pages', sharedBudget }), metrics, { eager });
  scope.__crystalActorWorkerReady = activate => lifecycle.ready(activate);
  scope.__crystalActorWorkerSubmit = (id, request, capture) => {
    lifecycle.submit(id, request, trace || capture); metrics.submitted++;
  };
  scope.__crystalActorWorkerCancel = () => lifecycle.cancel();
  scope.__crystalActorWorkerFallback = error => {
    metrics.fallback++; metrics.status = 'fallback'; metrics.lastFallback = String(error).slice(0, 2048);
  };
  scope.__crystalActorWorkerPeople = json => lifecycle.sources(json);
  return { dispose() { lifecycle.dispose(); } };
}
