import test from 'node:test';
import assert from 'node:assert/strict';
import { fetchAsset } from './asset-progress.js';

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
