import test from 'node:test';
import assert from 'node:assert/strict';
import { fetchAsset, loadWasm } from './asset-progress.js';

test('Wasm compiles from the original response before the download finishes', async (t) => {
  const original = globalThis.fetch;
  t.after(() => { globalThis.fetch = original; });
  let release, started = false, pulls = 0;
  const finalChunk = new Promise(resolve => { release = resolve; });
  const response = new Response(new ReadableStream({
    async pull(controller) {
      if (++pulls === 1) controller.enqueue(new Uint8Array([0,97,115,109]));
      else {
        await finalChunk;
        assert.equal(started, true);
        controller.enqueue(new Uint8Array([1,0,0,0]));
        controller.close();
      }
    },
  }), { headers: { 'Content-Type':'application/wasm', 'Content-Length':'8' } });
  globalThis.fetch = async () => response;
  const events = [];
  const runtime = await loadWasm(async ({ module_or_path }) => {
    assert.equal(module_or_path, response);
    started = true;
    release();
    return WebAssembly.instantiateStreaming(module_or_path);
  }, '/game.wasm', 'Game engine', event => events.push(event));
  assert.ok(runtime.instance instanceof WebAssembly.Instance);
  assert.deepEqual(events.at(-1), {label:'Game engine',loaded:8,total:8,phase:'ready'});
});

test('progress-only transport retains byte limits without creating a module buffer', async () => {
  const response = new Response(new Uint8Array([1,2,3]), {headers:{'Content-Length':'3'}});
  assert.equal(await fetchAsset('/module', {response,retainBytes:false}), 3);
  await assert.rejects(fetchAsset('/module', {response:new Response(new Uint8Array(5),
    {headers:{'Content-Length':'4'}}),retainBytes:false}), /unexpected download size/);
});

test('failed Wasm initialization aborts its fetch and reports a startup error', async (t) => {
  const original = globalThis.fetch;
  t.after(() => { globalThis.fetch = original; });
  let signal;
  globalThis.fetch = async (_, options) => {
    signal = options.signal;
    return new Response(new Uint8Array([0,97,115,109,1,0,0,0]),
      {headers:{'Content-Type':'application/wasm','Content-Length':'8'}});
  };
  const events = [];
  await assert.rejects(loadWasm(async () => { throw Error('invalid module'); },
    '/game.wasm', 'Game engine', event => events.push(event)), /invalid module/);
  assert.equal(signal.aborted, true);
  assert.equal(events.at(-1).phase, 'error');
});

test('asset download bounds and stream cancellation', async (t) => {
  const original = globalThis.fetch;
  t.after(() => { globalThis.fetch = original; });
  let cancelled = false;
  globalThis.fetch = async () => new Response(new ReadableStream({
    pull(controller) { controller.enqueue(new Uint8Array(5)); },
    cancel() { cancelled = true; },
  }));
  await assert.rejects(fetchAsset('/asset', { maxBytes: 4 }), /size limit/);
  assert.equal(cancelled, true);
  cancelled = false;
  await assert.rejects(fetchAsset('/asset', { expectedBytes: 4 }), /unexpected download size/);
  assert.equal(cancelled, true);
  globalThis.fetch = async () => new Response(new Uint8Array(4), { headers: { 'Content-Length': '999999999999' } });
  await assert.rejects(fetchAsset('/asset'), /size limit/);
  await assert.rejects(fetchAsset('/asset', { expectedBytes: -1 }), /invalid download size/);
  globalThis.fetch = async () => new Response(new Uint8Array([1, 2, 3]));
  assert.deepEqual(await fetchAsset('/asset', { expectedBytes: 3 }), new Uint8Array([1, 2, 3]));
  await assert.rejects(fetchAsset('/asset', { expectedBytes: 4 }), /incomplete download/);
});
