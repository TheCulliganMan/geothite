/** Opt-in QA timestamps. No readback or per-frame trace serialization during walking. */
export function installWebGpuTiming({ timestamps: collectTimestamps = true } = {}) {
  const encoders = new WeakMap();
  const buffers = new WeakMap();
  let device, active = false, limit = null, slots = 0;
  let frames = [], pending = [], pages = [], completions = [];
  // Each query set stays within WebGPU's device limit. New pages extend the
  // capture through the whole journey, rather than truncating it at startup.
  const passesPerPage = 1024, stride = 256;
  const page = () => {
    if (!pages.length || pages.at(-1).slots === passesPerPage) {
      pages.push({ index: pages.length, slots: 0,
        queries: device.createQuerySet({ type: 'timestamp', count: passesPerPage * 2 }),
        resolve: device.createBuffer({ size: passesPerPage * stride,
          usage: GPUBufferUsage.QUERY_RESOLVE | GPUBufferUsage.COPY_SRC }) });
    }
    return pages.at(-1);
  };
  const state = window.gpuFrameTiming = { supported: false, reason: 'No timestamp-query device',
    start(count = null) {
      if (!state.supported) return;
      if (count !== null && (!Number.isInteger(count) || count < 1)) throw Error('Invalid GPU capture length');
      for (const page of pages) { page.resolve.destroy(); page.queries.destroy(); }
      limit = count; slots = 0; frames = []; pending = []; pages = []; completions = [];
      active = true;
    },
    stop() { active = false; },
    async finish() {
      active = false;
      if (!state.supported) return { supported: false, reason: state.reason };
      const size = pages.length * passesPerPage * stride;
      let timestamps;
      let read;
      try {
        if (size) {
          read = device.createBuffer({ size, usage: GPUBufferUsage.COPY_DST | GPUBufferUsage.MAP_READ });
          const encoder = device.createCommandEncoder({ label: 'QA timestamp readback after walking' });
          for (const page of pages) {
            encoder.copyBufferToBuffer(page.resolve, 0, read, page.index * passesPerPage * stride,
              passesPerPage * stride);
          }
          device.queue.submit([encoder.finish()]);
          await read.mapAsync(GPUMapMode.READ);
          timestamps = new BigUint64Array(read.getMappedRange());
        }
        await Promise.all(completions);
        const result = frames.map((frame, index) => ({ index, complete: true,
          submitCpuMs: frame.cpu, completionCallbackCpuMs: frame.completed ?? null,
          completionCallbackLatencyMs: frame.completed == null ? null : frame.completed - frame.cpu,
          completionError: frame.completionError ?? null, passes: collectTimestamps ? frame.passes.map(pass => {
            const offset = (pass.page.index * passesPerPage + pass.slot) * stride / 8;
            return { label: pass.label, kind: pass.kind,
              startNs: timestamps[offset].toString(), endNs: timestamps[offset + 1].toString() };
          }) : [] }));
        return { supported: true, timestamps: collectTimestamps, frameCount: result.length, requestedFrames: limit,
          passCapacity: pages.length * passesPerPage, recordedPasses: collectTimestamps ? slots : 0, observedPasses: slots, passCapacityReached: false,
          passPageCapacity: passesPerPage, queryPages: pages.length, unframedPasses: pending.length,
          frames: result,
          meaning: 'GPU pass timestamps for render submissions; completion callback latency includes GPU queue and event-loop delivery; no physical display timing; readback after walking; instrumentation adds query/resolve and callback work' };
      } finally {
        if (read) { read.unmap(); read.destroy(); }
        for (const page of pages) { page.resolve.destroy(); page.queries.destroy(); }
        pages = [];
      }
    }
  };
  if (typeof GPUAdapter === 'undefined') return;
  const request = GPUAdapter.prototype.requestDevice;
  GPUAdapter.prototype.requestDevice = async function (descriptor = {}) {
    const supported = !collectTimestamps || this.features.has('timestamp-query');
    const requiredFeatures = [...new Set([...(descriptor.requiredFeatures ?? []),
      ...(collectTimestamps && supported ? ['timestamp-query'] : [])])];
    const result = await request.call(this, { ...descriptor, requiredFeatures });
    if (supported) { device = result; state.supported = true; state.reason = null; }
    return result;
  };
  for (const [method, kind] of [['beginRenderPass', 'render'], ['beginComputePass', 'compute']]) {
    const begin = GPUCommandEncoder.prototype[method];
    GPUCommandEncoder.prototype[method] = function (descriptor = {}) {
      if (!active || descriptor.timestampWrites) return begin.call(this, descriptor);
      const passes = encoders.get(this) ?? [];
      slots++;
      if (!collectTimestamps) {
        passes.push({ label: descriptor.label ?? `${kind} pass`, kind });
        encoders.set(this, passes);
        return begin.call(this, descriptor);
      }
      const current = page();
      const slot = current.slots++;
      passes.push({ page: current, slot, label: descriptor.label ?? `${kind} pass`, kind });
      encoders.set(this, passes);
      return begin.call(this, { ...descriptor, timestampWrites: { querySet: current.queries,
        beginningOfPassWriteIndex: slot * 2, endOfPassWriteIndex: slot * 2 + 1 } });
    };
  }
  const finish = GPUCommandEncoder.prototype.finish;
  GPUCommandEncoder.prototype.finish = function (...args) {
    const passes = encoders.get(this) ?? [];
    for (const pass of passes) {
      if (!pass.page) continue;
      this.resolveQuerySet(pass.page.queries, pass.slot * 2, 2, pass.page.resolve, pass.slot * stride);
    }
    const buffer = finish.apply(this, args);
    if (passes.length) buffers.set(buffer, passes);
    return buffer;
  };
  const submit = GPUQueue.prototype.submit;
  GPUQueue.prototype.submit = function (commandBuffers) {
    const commands = [...commandBuffers];
    const passes = commands.flatMap(buffer => buffers.get(buffer) ?? []);
    let frame;
    if (passes.length) {
      pending.push(...passes);
      if (passes.some(pass => pass.kind === 'render')) {
        frame = { cpu: performance.now(), passes: pending };
        frames.push(frame); pending = [];
        if (limit !== null && frames.length >= limit) active = false;
      }
    }
    const result = submit.call(this, commands);
    if (frame && typeof this.onSubmittedWorkDone === 'function') {
      completions.push(this.onSubmittedWorkDone().then(
        () => { frame.completed = performance.now(); },
        error => { frame.completionError = String(error); }));
    }
    return result;
  };
}
