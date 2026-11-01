// Optional transport only. Rust owns parsing, exact geometry, validation, and
// incremental CPU restoration. The renderer creates GPU assets after completion.
export function createCpuWorkerBudget() {
  return { start: performance.now(), spent: 0, bytes: 0 };
}
export function createTerrainWorkerClient(callbacks, options = {}) {
  const worker = new Worker(options.workerUrl ?? new URL('./terrain-worker.js', import.meta.url), { type: 'module' });
  const budgetMs = options.budgetMs ?? 4;
  const maxPumpBytes = options.maxPumpBytes ?? 8 * 1024 * 1024;
  let windowByteLimit = Math.min(maxPumpBytes, 2 * 1024 * 1024);
  let active = null;
  let remoteId = null;
  let cancelAcknowledgements = false;
  function dispatch() {
    if (!active?.request || (cancelAcknowledgements && remoteId != null)) return;
    const request = active.request;
    active.request = null;
    remoteId = active.id;
    worker.postMessage({ kind: options.jobKind ?? 'terrain-pages', id: active.id,
      request: request.buffer, trace: active.trace }, [request.buffer]);
  }
  const sceneryWaiters = new Map();
  let sceneryFailure = null;
  let scheduled = false;
  let frameScheduled = false;
  const windowBudget = options.sharedBudget ?? createCpuWorkerBudget();
  const turns = new MessageChannel();
  turns.port1.onmessage = pump;
  const epoch = () => performance.timeOrigin + performance.now();
  function schedule(frame = false) {
    if (frame) {
      if (!frameScheduled) { frameScheduled = true; requestAnimationFrame(() => {
        frameScheduled = false; schedule();
      }); }
    } else if (!scheduled) { scheduled = true; turns.port2.postMessage(0); }
  }
  function fail(error) {
    const id = active?.id;
    if (id != null) worker.postMessage({ kind: 'cancel', id });
    active = null;
    callbacks.onError?.(id, String(error?.message ?? error));
  }
  function pump() {
    scheduled = false;
    if (!active) return;
    const started = performance.now();
    if (started - windowBudget.start >= 1000 / 60) {
      if (windowBudget.bytes >= windowByteLimit && windowBudget.spent < budgetMs * 0.5) {
        windowByteLimit = Math.min(maxPumpBytes, windowByteLimit * 2);
      } else if (windowBudget.spent >= budgetMs * 0.9 && windowBudget.bytes < windowByteLimit * 0.5) {
        windowByteLimit = Math.max(Math.min(maxPumpBytes, 256 * 1024), windowByteLimit / 2);
      }
      windowBudget.start = started; windowBudget.spent = 0; windowBudget.bytes = 0;
    }
    if (windowBudget.spent >= budgetMs || windowBudget.bytes >= windowByteLimit) { schedule(true); return; }
    const deadline = started + budgetMs - windowBudget.spent;
    let bytes = 0;
    try {
      if (active.header) {
        callbacks.onHeader(active.id, new Uint8Array(active.header), active.timing);
        active.header = null;
        active.restoring = true;
      }
      while (active.pages.length && performance.now() < deadline && windowBudget.bytes + bytes < windowByteLimit) {
        const page = active.pages.shift();
        const view = new DataView(page);
        callbacks.onPage(active.id, new Uint8Array(page));
        if (view.getUint32(24, true) !== 0) throw new Error('Terrain page offset exceeds supported bounds');
        worker.postMessage({ kind: 'ack', id: active.id, buffer: view.getUint32(16, true), offset: view.getUint32(20, true) });
        bytes += page.byteLength;
      }
      if (active.restoring && performance.now() < deadline) {
        active.restoring = !callbacks.onAdvance(active.id, Math.max(0, deadline - performance.now()));
      }
      if (active.done && !active.restoring && !active.header && active.pages.length === 0) {
        const complete = active;
        active = null;
        remoteId = null;
        callbacks.onComplete(complete.id, complete.done, complete.timing);
      }
    } catch (error) { fail(error); }
    windowBudget.spent += performance.now() - started;
    windowBudget.bytes += bytes;
    if (active && (active.pages.length || active.header || (active.done && active.restoring))) {
      schedule(windowBudget.spent >= budgetMs || windowBudget.bytes >= windowByteLimit);
    }
  }
  worker.onmessage = ({ data }) => {
    if (cancelAcknowledgements && data.kind === 'cancelled') {
      if (data.id === remoteId) { remoteId = null; dispatch(); }
      return;
    }
    if (data.requestKind === 'scenery' || data.requestKind === 'initialize' || data.kind === 'ready') {
      const waiter = sceneryWaiters.get(data.id);
      if (!waiter) return;
      sceneryWaiters.delete(data.id);
      if (data.error) {
        sceneryFailure = new Error(String(data.error));
        waiter.reject(sceneryFailure);
      } else {
        if (data.cancelAcknowledgements === true) {
          cancelAcknowledgements = options.cancelAcknowledgements ?? false;
        }
        callbacks.onWorkerState?.(data);
        waiter.resolve();
      }
      return;
    }
    if (!active || data.id !== active.id) return;
    if (data.error) { fail(data.error); return; }
    if (data.kind === 'header') {
      active.header = data.header;
      active.timing = { submit: active.submit, receive: epoch(), cpuStart: data.cpuStart, cpuEnd: data.cpuEnd, traceOriginEpoch: data.traceOriginEpoch, workerMemory: data.workerMemory, rawBytes: data.rawBytes };
    } else if (data.kind === 'page') {
      if (active.pages.length >= 2) { fail('Terrain worker exceeded page backpressure window'); return; }
      active.pages.push(data.page);
    } else if (data.kind === 'done') active.done = data;
    schedule();
  };
  worker.onerror = error => {
    sceneryFailure = new Error(String(error.message));
    for (const waiter of sceneryWaiters.values()) waiter.reject(sceneryFailure);
    sceneryWaiters.clear();
    fail(error.message);
  };
  return {
    submit(id, request, trace = false) {
      if (sceneryFailure) throw sceneryFailure;
      if (sceneryWaiters.size) throw new Error('Terrain scenery initialization is still pending');
      this.cancel();
      // Own a deferred request before returning the borrowed Wasm view. Keep
      // only the latest replacement until the running CPU job acknowledges
      // cancellation; posting each replacement queues seconds of stale work.
      if (request.byteOffset !== 0 || request.byteLength !== request.buffer.byteLength
        || (cancelAcknowledgements && remoteId != null)) request = request.slice();
      active = { id, submit: epoch(), header: null, pages: [], restoring: false, done: null,
        timing: null, request, trace };
      dispatch();
    },
    cancel() {
      if (active) {
        if (!active.request) worker.postMessage({ kind: 'cancel', id: active.id });
        callbacks.onCancel?.(active.id);
      }
      active = null;
    },
    initialize(id) {
      if (sceneryWaiters.has(id)) return Promise.reject(new Error('Duplicate terrain initialization id'));
      return new Promise((resolve, reject) => {
        sceneryWaiters.set(id, { resolve, reject });
        worker.postMessage({ kind: 'initialize', id });
      });
    },
    installScenery(id, json) {
      if (sceneryWaiters.has(id)) return Promise.reject(new Error('Duplicate terrain scenery initialization id'));
      if (active) return Promise.reject(new Error('Cannot replace scenery during a terrain job'));
      return new Promise((resolve, reject) => {
        sceneryWaiters.set(id, { resolve, reject });
        worker.postMessage({ kind: 'scenery', id, json });
      });
    },
    dispose() {
      this.cancel();
      for (const waiter of sceneryWaiters.values()) waiter.reject(new Error('Terrain worker disposed'));
      sceneryWaiters.clear();
      worker.terminate(); turns.port1.close(); turns.port2.close();
    },
  };
}
