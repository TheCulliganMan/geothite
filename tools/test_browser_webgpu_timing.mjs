import assert from 'node:assert/strict';
import test from 'node:test';
import vm from 'node:vm';
import { installWebGpuTiming } from './browser-webgpu-timing.mjs';

async function fixture(options = {}) {
  let clock = 100n;
  let mappings = 0;
  const resources = [];
  class Buffer {
    constructor({ size }) { this.data = new ArrayBuffer(size); this.destroyed = false; }
    async mapAsync() { mappings++; }
    getMappedRange() { return this.data; }
    unmap() {}
    destroy() { this.destroyed = true; }
  }
  class Encoder {
    beginRenderPass(descriptor = {}) { return this.pass(descriptor); }
    beginComputePass(descriptor = {}) { return this.pass(descriptor); }
    pass({ timestampWrites }) {
      if (timestampWrites) {
        const { querySet, beginningOfPassWriteIndex, endOfPassWriteIndex } = timestampWrites;
        assert.ok(endOfPassWriteIndex < querySet.values.length);
        querySet.values[beginningOfPassWriteIndex] = clock;
        querySet.values[endOfPassWriteIndex] = clock + 17n;
        clock += 25n;
      }
      return { end() {} };
    }
    resolveQuerySet(query, first, count, buffer, offset) {
      new BigUint64Array(buffer.data).set(query.values.subarray(first, first + count), offset / 8);
    }
    copyBufferToBuffer(source, from, destination, to, size) {
      new Uint8Array(destination.data, to, size).set(new Uint8Array(source.data, from, size));
    }
    finish() { return {}; }
  }
  class Queue { submit() {}
    onSubmittedWorkDone() { return Promise.resolve(); } }
  const device = { queue: new Queue(), createCommandEncoder: () => new Encoder(),
    createBuffer: descriptor => { const buffer = new Buffer(descriptor); resources.push(buffer); return buffer; },
    createQuerySet: ({ count }) => {
      const query = { values: new BigUint64Array(count), destroyed: false,
        destroy() { this.destroyed = true; } }; resources.push(query); return query;
    } };
  class Adapter {
    features = new Set(['timestamp-query']);
    async requestDevice() { return device; }
  }
  const scope = { window: {}, options, performance, GPUAdapter: Adapter, GPUCommandEncoder: Encoder, GPUQueue: Queue,
    GPUBufferUsage: { QUERY_RESOLVE: 1, COPY_SRC: 2, COPY_DST: 4, MAP_READ: 8 }, GPUMapMode: { READ: 1 } };
  vm.runInNewContext(`(${installWebGpuTiming.toString()})(options)`, scope);
  await new Adapter().requestDevice();
  const timing = scope.window.gpuFrameTiming;
  const render = (count = 1) => {
    const encoder = device.createCommandEncoder();
    for (let index = 0; index < count; index++) encoder.beginRenderPass({ label: `pass ${index}` }).end();
    device.queue.submit([encoder.finish()]);
  };
  return { timing, render, resources, mappings: () => mappings };
}

test('the full journey spans query pages without a readback during walking', async () => {
  const { timing, render, resources, mappings } = await fixture();
  timing.start();
  for (let index = 0; index < 1100; index++) render();
  assert.equal(mappings(), 0);
  timing.stop();
  render();
  const result = await timing.finish();
  assert.equal(result.frameCount, 1100);
  assert.equal(result.recordedPasses, 1100);
  assert.equal(result.queryPages, 2);
  assert.equal(result.passCapacityReached, false);
  assert.equal(result.requestedFrames, null);
  assert.equal(mappings(), 1);
  assert.ok(result.frames.every(frame => frame.complete));
  assert.ok(result.frames.every(frame => frame.completionCallbackLatencyMs >= 0
    && frame.completionError === null));
  for (const index of [0, 1023, 1024, 1099]) {
    const pass = result.frames[index].passes[0];
    assert.equal(BigInt(pass.startNs), 100n + BigInt(index) * 25n);
    assert.equal(BigInt(pass.endNs) - BigInt(pass.startNs), 17n);
  }
  assert.ok(resources.every(resource => resource.destroyed));
});

test('one command encoder can resolve passes from multiple query sets', async () => {
  const { timing, render } = await fixture();
  timing.start();
  render(1300);
  const result = await timing.finish();
  assert.equal(result.frameCount, 1);
  assert.equal(result.recordedPasses, 1300);
  assert.equal(result.queryPages, 2);
  assert.equal(result.frames[0].passes.length, 1300);
  for (const index of [1023, 1024, 1299]) {
    const pass = result.frames[0].passes[index];
    assert.equal(BigInt(pass.startNs), 100n + BigInt(index) * 25n);
    assert.equal(BigInt(pass.endNs) - BigInt(pass.startNs), 17n);
  }
});

test('an explicit short capture still stops at the requested number of render submissions', async () => {
  const { timing, render } = await fixture();
  timing.start(2);
  for (let index = 0; index < 5; index++) render();
  const result = await timing.finish();
  assert.equal(result.frameCount, 2);
  assert.equal(result.requestedFrames, 2);
  assert.equal(result.recordedPasses, 2);
});

test('an empty capture reports no submissions and allocates no readback', async () => {
  const { timing, mappings, resources } = await fixture();
  timing.start();
  const result = await timing.finish();
  assert.equal(result.frameCount, 0);
  assert.equal(result.recordedPasses, 0);
  assert.equal(mappings(), 0);
  assert.equal(resources.length, 0);
});


test('completion-only diagnostics add no timestamp queries or readback buffers', async () => {
  const { timing, render, mappings, resources } = await fixture({ timestamps: false });
  timing.start();
  for (let index = 0; index < 1100; index++) render();
  const result = await timing.finish();
  assert.equal(result.timestamps, false);
  assert.equal(result.frameCount, 1100);
  assert.equal(result.recordedPasses, 0);
  assert.equal(result.observedPasses, 1100);
  assert.equal(result.queryPages, 0);
  assert.equal(mappings(), 0);
  assert.equal(resources.length, 0);
  assert.ok(result.frames.every(frame => frame.passes.length === 0
    && frame.completionCallbackLatencyMs >= 0));
});
