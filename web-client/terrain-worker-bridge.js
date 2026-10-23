// The matching Rust renderer feature gates this bridge. Dormant 2D sessions
// create no worker; the first authoritative 3D request activates it.
import { createCpuWorkerBudget, createTerrainWorkerClient } from './terrain-worker-client.js';
import { createCpuWorkerPool } from './cpu-worker-pool.js';
export function installTerrainWorkerBridge(wasm, { trace = false, scope = globalThis, sharedBudget = null, eager = true,
  workerCount = (scope.navigator?.hardwareConcurrency ?? 2) >= 4 ? 2 : 1 } = {}) {
  if (typeof wasm.crystal_terrain_worker_header !== 'function') return null;
  const traces = [];
  const restoreBudget = sharedBudget ?? createCpuWorkerBudget();
  const metrics = { initializations: 0, initFailures: 0, modelsInstalled: 0, fallback: 0, status: 'dormant', successfulJobIds: [], submitted: 0, completed: 0, cancelled: 0, failed: 0, stages: [0, 1, 2].map(() => ({ count: 0, maxMs: 0, totalMs: 0 })) };
  const headerEpochs = new Map();
  const tracedJobs = new Set();
  scope.__crystalTerrainWorkerTraces = traces;
  scope.__crystalTerrainWorkerMetrics = metrics;
  const timed = (id, stage, bytes, operation) => {
    const start = performance.now();
    try { return operation(); }
    finally {
      const elapsed = performance.now() - start;
      const metric = metrics.stages[stage];
      metric.count++; metric.totalMs += elapsed; metric.maxMs = Math.max(metric.maxMs, elapsed);
      if (stage === 0) headerEpochs.set(id, performance.timeOrigin + start);
      wasm.crystal_terrain_worker_callback_timing(id, stage, elapsed, bytes);
    }
  };
  let pool;
  const updateMemory = (index, memory) => {
    if (memory) metrics.workers[index].workerMemory = memory;
    const observed = metrics.workers.map(worker => worker.workerMemory).filter(Boolean);
    metrics.workerMemory = {
      currentBytes: observed.reduce((sum, memory) => sum + memory.currentBytes, 0),
      peakBytes: observed.reduce((sum, memory) => sum + memory.peakBytes, 0),
      workersObserved: observed.length,
      meaning: 'Sum of reported worker WebAssembly linear memory; excludes renderer memory; not RSS',
    };
  };
  const callbacks = index => ({
    onWorkerState: data => {
      if (data.cancelAcknowledgements != null) {
        metrics.workers[index].cancelAcknowledgements = data.cancelAcknowledgements;
      }
      updateMemory(index, data.workerMemory);
    },
    onHeader: (id, bytes) => timed(id, 0, bytes.byteLength, () => wasm.crystal_terrain_worker_header(id, bytes)),
    onPage: (id, bytes) => timed(id, 1, bytes.byteLength, () => wasm.crystal_terrain_worker_page(id, bytes)),
    onAdvance: (id, budget) => timed(id, 2, 0, () => wasm.crystal_terrain_worker_advance(id, budget)),
    onComplete: (id, result, timing) => {
      wasm.crystal_terrain_worker_complete(id);
      pool.complete(id);
      metrics.completed++;
      metrics.successfulJobIds.push(id);
      if (metrics.successfulJobIds.length > 16) metrics.successfulJobIds.shift();
      metrics.lastRawBytes = timing?.rawBytes;
      updateMemory(index, result.workerMemory ?? timing?.workerMemory);
      if (tracedJobs.delete(id)) {
        // Worker epoch start aligns its relative Chrome events with the main
        // lane when the capture tool merges this bounded diagnostic history.
        traces.push({ id, worker: index, ...timing, cpuStart: result.cpuStart, cpuEnd: result.cpuEnd, traceOriginEpoch: result.traceOriginEpoch, headerCallbackEpoch: headerEpochs.get(id), trace: result.trace });
        if (traces.length > 8) traces.shift();
      }
      headerEpochs.delete(id);
    },
    onCancel: id => { pool.complete(id); tracedJobs.delete(id); metrics.cancelled++; headerEpochs.delete(id); },
    onError: (id, error) => {
      pool.fail(index); metrics.failed++;
      tracedJobs.delete(id);
      headerEpochs.delete(id);
      if (id != null) wasm.crystal_terrain_worker_error(id, error);
    },
  });
  pool = createCpuWorkerPool(index => createTerrainWorkerClient(callbacks(index), { sharedBudget: restoreBudget, cancelAcknowledgements: true }), metrics,
    { count: workerCount, eager: false });
  if (eager) pool.ready(true);
  scope.__crystalTerrainWorkerReady = activate => pool.ready(activate);
  scope.__crystalTerrainWorkerCapacity = () => pool.capacity();
  scope.__crystalTerrainWorkerSubmit = (id, request, capture) => {
    const collect = trace || capture;
    if (collect) tracedJobs.add(id);
    try { pool.submit(id, request, collect); metrics.submitted++; }
    catch (error) { tracedJobs.delete(id); throw error; }
  };
  scope.__crystalTerrainWorkerCancel = id => pool.cancel(id);
  scope.__crystalTerrainWorkerFallback = error => {
    metrics.fallback++; metrics.status = 'fallback'; metrics.lastFallback = String(error).slice(0, 2048);
  };
  scope.__crystalTerrainWorkerScenery = json => pool.sources(json);
  return { dispose() { pool.dispose(); } };
}
