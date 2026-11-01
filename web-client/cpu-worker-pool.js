// Independent transports share one frame-thread restore budget. Geometry,
// parsing, and source validation remain in Rust on each CPU worker.
import { createCpuWorkerLifecycle } from './cpu-worker-lifecycle.js';

export function createCpuWorkerPool(createClient, metrics, { count = 1, eager = true } = {}) {
  if (!Number.isInteger(count) || count < 1) throw Error('Invalid CPU worker count');
  const owners = new Map();
  let disposed = false;
  const slots = Array.from({ length: count }, (_, index) => {
    const workerMetrics = { initializations: 0, initFailures: 0, modelsInstalled: 0, status: 'dormant' };
    return { metrics: workerMetrics, job: null,
      lifecycle: createCpuWorkerLifecycle(() => createClient(index), workerMetrics, { eager: false }) };
  });
  metrics.workers = slots.map(slot => slot.metrics);
  const refresh = () => {
    for (const key of ['initializations', 'initFailures', 'modelsInstalled', 'retainedSourceCharacters']) {
      metrics[key] = slots.reduce((sum, slot) => sum + (slot.metrics[key] ?? 0), 0);
    }
    if (disposed) { metrics.capacity = 0; metrics.status = 'disposed'; return 0; }
    const states = slots.map(slot => slot.lifecycle.ready());
    metrics.capacity = states.filter(state => state === 2).length;
    metrics.status = metrics.capacity ? 'ready' : states.some(state => state === 1)
      ? metrics.initializations ? 'initializing' : 'dormant' : 'failed';
    return metrics.capacity ? 2 : states.some(state => state === 1) ? 1 : 0;
  };
  const complete = id => {
    const slot = owners.get(id);
    if (slot) { slot.job = null; owners.delete(id); }
  };
  const pool = {
    ready(activate = false) {
      if (activate) for (const slot of slots) slot.lifecycle.ready(true);
      return refresh();
    },
    capacity() { refresh(); return metrics.capacity; },
    sources(json) { for (const slot of slots) slot.lifecycle.sources(json); refresh(); },
    submit(id, ...args) {
      const slot = slots.find(slot => slot.job === null && slot.lifecycle.ready() === 2);
      if (!slot) throw Error('All terrain workers are busy');
      slot.job = id; owners.set(id, slot);
      try { slot.lifecycle.submit(id, ...args); }
      catch (error) { complete(id); throw error; }
    },
    complete,
    cancel(id) { const slot = owners.get(id); slot?.lifecycle.cancel(); complete(id); },
    fail(index) { const slot = slots[index]; slot.lifecycle.fail(); complete(slot.job); refresh(); },
    dispose() {
      disposed = true;
      for (const slot of slots) slot.lifecycle.dispose();
      owners.clear(); refresh();
    },
  };
  if (eager) pool.ready(true);
  return pool;
}
