// Experimental CPU terrain boundary. Geometry generation stays entirely Rust.
// The production client does not create this worker unless explicitly enabled.
import init, { begin_actor_job, install_people_sources } from './crystal-actor-worker.js';
let queue = Promise.resolve();
let wasm = null;
let peakLinearBytes = 0;
function memory() {
  const currentBytes = wasm?.memory?.buffer?.byteLength ?? null;
  if (currentBytes != null) peakLinearBytes = Math.max(peakLinearBytes, currentBytes);
  return { currentBytes, peakBytes: currentBytes == null ? null : peakLinearBytes,
    meaning: 'WebAssembly linear-memory high-water, not total process memory' };
}
const ready = init('./crystal-actor-worker_bg.wasm').then(instance => { wasm = instance; memory(); });
ready.catch(() => {}); // Requests report initialization failures individually.
let active = null;
const cancelled = new Set();
const epoch = () => performance.timeOrigin + performance.now();
globalThis.__crystalActorEpoch = epoch;
function dispose() { if (active) active.job.free(); active = null; }
function pump() {
  while (active) {
    const page = active.job.next_page();
    memory();
    if (page.length === 0) {
      if (active.job.finished()) {
        self.postMessage({ id: active.id, kind: 'done', trace: active.job.trace_json(), cpuStart: active.cpuStart, cpuEnd: active.cpuEnd, traceOriginEpoch: active.traceOriginEpoch, workerMemory: memory(), cache: JSON.parse(active.job.cache_json()) });
        dispose();
      }
      return;
    }
    self.postMessage({ id: active.id, kind: 'page', page: page.buffer }, [page.buffer]);
  }
}
self.onmessage = ({ data }) => {
  if (data.kind === 'cancel') {
    if (active?.id === data.id) dispose(); else cancelled.add(data.id);
    return;
  }
  if (data.kind === 'ack') {
    if (active?.id !== data.id) return;
    try { active.job.acknowledge(data.buffer, data.offset); pump(); }
    catch (error) { self.postMessage({ id: data.id, error: String(error?.message ?? error) }); dispose(); }
    return;
  }
  queue = queue.then(async () => {
    try {
      await ready;
      if (cancelled.delete(data.id)) return;
      if (data.kind === 'initialize') {
        self.postMessage({ id: data.id, kind: 'ready', workerMemory: memory() });
        return;
      }
      if (data.kind === 'scenery') {
        install_people_sources(data.json);
        self.postMessage({ id: data.id, kind: 'ready', workerMemory: memory() });
        return;
      }
      if (data.kind === 'actor-pages') {
        dispose();
        const cpuStart = epoch();
        const job = begin_actor_job(new Uint8Array(data.request), Boolean(data.trace));
        active = { id: data.id, job, cpuStart, cpuEnd: epoch(), traceOriginEpoch: job.trace_origin_epoch() };
        const header = job.header();
        self.postMessage({ id: data.id, kind: 'header', header: header.buffer, rawBytes: job.raw_bytes(), cpuStart, cpuEnd: active.cpuEnd, traceOriginEpoch: active.traceOriginEpoch, workerMemory: memory(), cache: JSON.parse(active.job.cache_json()) }, [header.buffer]);
        pump();
        return;
      }
      throw new Error('Unknown actor worker request');
    } catch (error) {
      self.postMessage({ id: data.id, requestKind: data.kind, error: String(error?.message ?? error) });
    }
  });
};
