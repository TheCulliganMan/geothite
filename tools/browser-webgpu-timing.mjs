/** Opt-in QA timestamps. No readback or per-frame trace serialization during walking. */
export function installWebGpuTiming() {
  const encoders = new WeakMap();
  const buffers = new WeakMap();
  let device, queries, resolve, active = false, limit = 30, slots = 0;
  let frames = [], pending = [];
  const maxPasses = 1024, stride = 256;
  const state = window.gpuFrameTiming = { supported: false, reason: 'No timestamp-query device',
    start(count) {
      if (!state.supported) return;
      limit = count; slots = 0; frames = []; pending = [];
      queries = device.createQuerySet({ type: 'timestamp', count: maxPasses * 2 });
      resolve = device.createBuffer({ size: maxPasses * stride,
        usage: GPUBufferUsage.QUERY_RESOLVE | GPUBufferUsage.COPY_SRC });
      active = true;
    },
    async finish() {
      active = false;
      if (!state.supported || !resolve) return { supported: false, reason: state.reason };
      const read = device.createBuffer({ size: maxPasses * stride,
        usage: GPUBufferUsage.COPY_DST | GPUBufferUsage.MAP_READ });
      const encoder = device.createCommandEncoder({ label: 'QA timestamp readback after walking' });
      encoder.copyBufferToBuffer(resolve, 0, read, 0, maxPasses * stride);
      device.queue.submit([encoder.finish()]);
      await read.mapAsync(GPUMapMode.READ);
      const timestamps = new BigUint64Array(read.getMappedRange());
      const result = frames.map((frame, index) => ({ index,
        submitCpuMs: frame.cpu, passes: frame.passes.map(pass => ({
          label: pass.label, kind: pass.kind,
          startNs: timestamps[pass.slot * stride / 8].toString(),
          endNs: timestamps[pass.slot * stride / 8 + 1].toString(),
        })) }));
      read.unmap(); read.destroy(); resolve.destroy(); queries.destroy();
      return { supported: true, frameCount: result.length, frames: result,
        meaning: 'GPU pass timestamps; readback after walking; instrumentation adds query/resolve work' };
    }
  };
  if (typeof GPUAdapter === 'undefined') return;
  const request = GPUAdapter.prototype.requestDevice;
  GPUAdapter.prototype.requestDevice = async function (descriptor = {}) {
    const supported = this.features.has('timestamp-query');
    const requiredFeatures = [...new Set([...(descriptor.requiredFeatures ?? []),
      ...(supported ? ['timestamp-query'] : [])])];
    const result = await request.call(this, { ...descriptor, requiredFeatures });
    if (supported) { device = result; state.supported = true; state.reason = null; }
    return result;
  };
  for (const [method, kind] of [['beginRenderPass', 'render'], ['beginComputePass', 'compute']]) {
    const begin = GPUCommandEncoder.prototype[method];
    GPUCommandEncoder.prototype[method] = function (descriptor = {}) {
      if (!active || slots >= maxPasses || descriptor.timestampWrites) return begin.call(this, descriptor);
      const slot = slots++;
      const passes = encoders.get(this) ?? [];
      passes.push({ slot, label: descriptor.label ?? `${kind} pass`, kind });
      encoders.set(this, passes);
      return begin.call(this, { ...descriptor, timestampWrites: { querySet: queries,
        beginningOfPassWriteIndex: slot * 2, endOfPassWriteIndex: slot * 2 + 1 } });
    };
  }
  const finish = GPUCommandEncoder.prototype.finish;
  GPUCommandEncoder.prototype.finish = function (...args) {
    const passes = encoders.get(this) ?? [];
    for (const pass of passes) this.resolveQuerySet(queries, pass.slot * 2, 2, resolve, pass.slot * stride);
    const buffer = finish.apply(this, args);
    if (passes.length) buffers.set(buffer, passes);
    return buffer;
  };
  const submit = GPUQueue.prototype.submit;
  GPUQueue.prototype.submit = function (commandBuffers) {
    const commands = [...commandBuffers];
    const passes = commands.flatMap(buffer => buffers.get(buffer) ?? []);
    if (passes.length) {
      pending.push(...passes);
      if (passes.some(pass => pass.kind === 'render')) {
        frames.push({ cpu: performance.now(), passes: pending }); pending = [];
        if (frames.length >= limit) active = false;
      }
    }
    return submit.call(this, commands);
  };
}
