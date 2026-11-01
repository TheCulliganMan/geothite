/** Offscreen production walking and door transitions, with raw timing receipts. */
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import os from 'node:os';
import { chromium, webkit } from 'playwright';
import { installWebGpuTiming } from './browser-webgpu-timing.mjs';
import { mergeTerrainWorkerTrace, mergeActorWorkerTrace } from './terrain-worker-trace.mjs';

const [url, output, scenario = 'long', engine = 'chromium'] = process.argv.slice(2);
const tracing = process.env.STREAM_TRACE !== '0';
assert.ok(url && output && ['long', 'outdoor', 'vertical', 'house'].includes(scenario));
assert.ok(['chromium', 'webkit'].includes(engine));
const parsed = new URL(url);
assert.ok(['127.0.0.1', 'localhost'].includes(parsed.hostname), 'use a disposable local game');
assert.equal(parsed.searchParams.get('multiplayer'), 'off');
assert.ok(tracing || !parsed.searchParams.has('loading-trace'), 'Profiling-off runs must not enable startup tracing');
const preview = { long: 'world-stream', outdoor: 'route29-border',
  vertical: 'cherrygrove-border', house: 'house-door' }[scenario];
assert.equal(parsed.searchParams.get('preview'), preview);
await fs.mkdir(output, { recursive: true });
const report = { url, scenario, engine, headless: true, success: false, errors: [],
  profiling: tracing ? 'detailed' : 'off',
  host: { cpu: os.cpus()[0]?.model, cores: os.cpus().length, load: os.loadavg() },
  meaning: 'Instrumented offscreen browser presentation intervals; no physical display latency.',
  steps: [] };
let browser, page, cpuSession;
let cpuTraceActive = false;
const stopCpuTrace = async () => {
  if (!cpuTraceActive) return;
  cpuTraceActive = false;
  const completed = new Promise(resolve => cpuSession.once('Tracing.tracingComplete', resolve));
  await cpuSession.send('Tracing.end');
  const { stream } = await completed;
  const outputFile = await fs.open(`${output}/browser-cpu-trace.json`, 'w');
  try {
    for (;;) {
      const chunk = await cpuSession.send('IO.read', { handle: stream });
      await outputFile.writeFile(chunk.base64Encoded ? Buffer.from(chunk.data, 'base64') : chunk.data);
      if (chunk.eof) break;
    }
  } finally {
    await outputFile.close();
    await cpuSession.send('IO.close', { handle: stream });
  }
};
const deadline = setTimeout(() => {
  report.failure = 'Host deadline exceeded';
  void fs.writeFile(`${output}/timeout.json`, JSON.stringify(report)).finally(() => browser?.close());
}, 360000);
try {
  browser = await ({ chromium, webkit })[engine].launch({ headless: true,
    ...(engine === 'chromium' ? { args: ['--mute-audio', '--enable-gpu'] } : {}) });
  report.browser = browser.version();
  const size = /^(\d+)x(\d+)$/.exec(process.env.STREAM_VIEWPORT ?? '1280x800');
  const deviceScaleFactor = Number(process.env.STREAM_DEVICE_SCALE ?? 1);
  assert.ok(size && Number(size[1]) > 0 && Number(size[2]) > 0, 'Invalid STREAM_VIEWPORT');
  assert.ok(Number.isFinite(deviceScaleFactor) && deviceScaleFactor > 0, 'Invalid STREAM_DEVICE_SCALE');
  const context = await browser.newContext({ viewport: { width: Number(size[1]), height: Number(size[2]) },
    deviceScaleFactor });
  if (process.env.GPU_WATERFALL === '1' || process.env.GPU_WATERFALL_COMPLETIONS === '1') {
    await context.addInitScript(installWebGpuTiming, { timestamps: process.env.GPU_WATERFALL === '1' });
  }
  if (process.env.STREAM_GPU_API_TRACE === '1') await context.addInitScript(() => {
    window.streamGpuApi = { stats: {}, slowCalls: [] };
    for (const [type, methods] of [
      ['GPUDevice', ['createRenderPipeline', 'createComputePipeline', 'createShaderModule',
        'createBuffer', 'createTexture', 'createBindGroup']],
      ['GPUQueue', ['submit', 'writeBuffer', 'writeTexture']],
      ['GPUBuffer', ['destroy']], ['GPUTexture', ['destroy']],
    ]) {
      const prototype = globalThis[type]?.prototype;
      if (!prototype) continue;
      for (const method of methods) {
        const original = prototype[method];
        if (typeof original !== 'function') continue;
        prototype[method] = function (...args) {
          const start = performance.now();
          const phase = window.streamTiming?.active ? 'measured'
            : window.streamTiming?.started ? 'after' : 'startup';
          const api = `${type}.${method}`;
          try { return Reflect.apply(original, this, args); }
          finally {
            const duration = performance.now() - start;
            const key = `${phase}:${api}`;
            const stat = streamGpuApi.stats[key] ??= { count: 0, totalMs: 0, maxMs: 0 };
            stat.count++; stat.totalMs += duration; stat.maxMs = Math.max(stat.maxMs, duration);
            if (duration >= 1) streamGpuApi.slowCalls.push({ api, phase, start, duration,
              label: args[0]?.label ?? this.label ?? null });
          }
        };
      }
    }
  });
  if (process.env.STREAM_AUDIO_TRACE === '1') await context.addInitScript(() => {
    window.streamAudioCalls = [];
    const Original = window.AudioContext;
    if (!Original) return;
    window.AudioContext = new Proxy(Original, {
      construct(target, args, receiver) {
        const start = performance.now();
        try { return Reflect.construct(target, args, receiver); }
        finally { streamAudioCalls.push({ operation: 'construct', start, wallMs: performance.now() - start }); }
      },
    });
    const resume = Original.prototype.resume;
    Original.prototype.resume = function (...args) {
      const start = performance.now();
      const call = { operation: 'resume', start };
      streamAudioCalls.push(call);
      try {
        const result = Reflect.apply(resume, this, args);
        result.then(() => { call.settledMs = performance.now() - start; },
          error => { call.error = String(error); });
        return result;
      } finally { call.wallMs = performance.now() - start; }
    };
  });
  await context.addInitScript(() => {
    localStorage.setItem('crystal.audio.preference', 'muted');
    localStorage.setItem('crystal.display.voxel', 'true');
    window.streamTiming = { active: false, frames: [], longTasks: [], gpuErrors: [], lost: [], wasmPeakBytes: 0 };
    const getContext = HTMLCanvasElement.prototype.getContext;
    HTMLCanvasElement.prototype.getContext = function (...args) {
      const context = getContext.apply(this, args);
      if (context && ['webgl', 'webgl2'].includes(args[0])) {
        const debug = context.getExtension('WEBGL_debug_renderer_info');
        streamTiming.glRenderer = debug ? context.getParameter(debug.UNMASKED_RENDERER_WEBGL) : null;
      }
      return context;
    };
    let previous;
    const frame = time => {
      if (streamTiming.active && previous != null) streamTiming.frames.push({
        start: previous, interval: time - previous });
      if (streamTiming.active && window.streamMemory) streamTiming.wasmPeakBytes = Math.max(
        streamTiming.wasmPeakBytes, streamMemory.buffer.byteLength);
      previous = streamTiming.active ? time : null;
      requestAnimationFrame(frame);
    };
    requestAnimationFrame(frame);
    if (typeof PerformanceObserver !== 'undefined'
      && PerformanceObserver.supportedEntryTypes?.includes('longtask')) {
      new PerformanceObserver(list => {
        if (streamTiming.active) streamTiming.longTasks.push(...list.getEntries()
          .map(entry => ({ start: entry.startTime, duration: entry.duration })));
      }).observe({ type: 'longtask' });
    }
    if (typeof GPUAdapter !== 'undefined') {
      const request = GPUAdapter.prototype.requestDevice;
      GPUAdapter.prototype.requestDevice = async function (...args) {
        const device = await request.apply(this, args);
        device.lost.then(info => streamTiming.lost.push({ reason: info.reason, message: info.message }));
        device.addEventListener('uncapturederror', event => streamTiming.gpuErrors.push(String(event.error.message)));
        return device;
      };
    }
    document.addEventListener('webglcontextlost', event => {
      if (event.target === document.querySelector('#crystal-canvas')) streamTiming.lost.push({ reason: 'webglcontextlost' });
    }, true);
  });
  page = await context.newPage();
  page.on('pageerror', error => report.errors.push(String(error)));
  page.on('crash', () => report.errors.push('Browser page crashed'));
  page.on('console', message => {
    if (message.type() === 'error') report.errors.push(message.text());
  });
  const startup = Date.now();
  await page.goto(url);
  await page.waitForFunction(() => document.querySelector('#loading').hidden
    || document.querySelector('#startup-error').open, null, { timeout: 180000 });
  assert.equal(await page.locator('#startup-error').evaluate(node => node.open ? node.textContent : null), null);
  await page.evaluate(async () => {
    window.streamWasm = await import(window.__crystalRendererBundle.module);
    const exports = await streamWasm.default();
    window.streamMemory = Object.values(exports).find(value => value instanceof WebAssembly.Memory);
    const { createGameBridge } = await import('./webmcp.js');
    window.streamBridge = createGameBridge(streamWasm, { timeoutMs: 30000 });
    // Production movement is camera-relative. Use the same north-facing
    // presentation as the native benchmark for these authored cardinal routes.
    streamWasm.crystal_set_voxel_camera_pose(0, 0, 45, false);
  });
  report.camera = { zoom: 0, rotation: 0, pitch: 45, firstPerson: false };
  const observe = () => page.evaluate(() => streamBridge.execute({ kind: 'observe' }));
  const ready = async () => {
    const stop = Date.now() + 120000;
    for (;;) {
      const state = await observe();
      if (state.presentation?.voxel?.active && !state.presentation.voxel.profiles_pending
        && !state.presentation.voxel.terrain_pending && state.presentation.voxel.active_frames >= 3) return state;
      assert.ok(Date.now() < stop, 'Production terrain did not become ready');
      await page.waitForTimeout(100);
    }
  };
  report.before = await ready();
  report.startupReadyMs = Date.now() - startup;
  report.renderer = await page.evaluate(() => window.__crystalRendererBundle);
  report.viewport = await page.evaluate(() => {
    const canvas = document.querySelector('#crystal-canvas');
    const bounds = canvas.getBoundingClientRect();
    return { browserCssWidth: innerWidth, browserCssHeight: innerHeight,
      devicePixelRatio, canvasPixelWidth: canvas.width, canvasPixelHeight: canvas.height,
      canvasCssWidth: bounds.width, canvasCssHeight: bounds.height };
  });
  report.adapter = await page.evaluate(async () => {
    const adapter = await navigator.gpu?.requestAdapter();
    return adapter ? { vendor: adapter.info.vendor, architecture: adapter.info.architecture,
      device: adapter.info.device, description: adapter.info.description,
      fallback: adapter.isFallbackAdapter ?? adapter.info.isFallbackAdapter ?? null } : null;
  });
  assert.ok(parsed.searchParams.get('renderer') !== 'webgpu'
    || /webgpu/.test(report.renderer.binary), 'Requested WebGPU bundle was not selected');
  // Focus remains inside the invisible page; no browser window is opened.
  await page.locator('#crystal-canvas').focus();
  if (process.env.STREAM_SCREENSHOTS === '1') await page.screenshot({ path: `${output}/before.png` });
  report.workersBefore = await page.evaluate(() => ({ terrain: __crystalTerrainWorkerMetrics,
    actors: __crystalActorWorkerMetrics }));
  if (tracing && parsed.searchParams.has('loading-trace')) {
    const startupTrace = await page.evaluate(() => streamWasm.crystal_loading_trace_read());
    await fs.writeFile(`${output}/startup-trace.json`, startupTrace);
  }
  if (process.env.STREAM_CPU_TRACE === '1') {
    assert.equal(engine, 'chromium', 'CPU timeline capture requires Chromium');
    cpuSession = await context.newCDPSession(page);
    await cpuSession.send('Tracing.start', { transferMode: 'ReturnAsStream',
      categories: 'devtools.timeline,v8,blink.user_timing,disabled-by-default-v8.gc' });
    cpuTraceActive = true;
    report.cpuProfiler = { tool: 'Chromium CPU timeline and V8 GC events',
      file: 'browser-cpu-trace.json', scope: 'Measured journey; diagnostic overhead included' };
  }
  report.traceStart = await page.evaluate(tracing => {
    if (tracing) streamWasm.crystal_loading_trace_start();
    performance.mark('geothite_stream_measure_begin');
    streamTiming.started = true;
    streamTiming.active = true;
    if (window.gpuFrameTiming?.supported) window.gpuFrameTiming.start();
    return performance.now();
  }, tracing);
  const hold = async (key, predicate) => {
    const started = Date.now();
    const before = await observe();
    let state = before;
    await page.keyboard.down(key);
    try {
      do {
        await page.waitForTimeout(50);
        state = await observe();
        report.lastState = state;
        assert.equal(state.status.screen, 'overworld', 'Journey entered another gameplay screen');
        assert.ok(Date.now() - started < 45000, `Held ${key} never reached its expected destination: ${JSON.stringify(state.map_info)}`);
      } while (!predicate(state.map_info));
    } finally {
      await page.keyboard.up(key);
    }
    report.steps.push({ key, wallMs: Date.now() - started, before: before.map_info, after: state.map_info });
    console.log(JSON.stringify(report.steps.at(-1)));
  };
  if (scenario === 'long') {
    await hold('ArrowLeft', map => map.name === 'SilverCaveOutside' && map.player.x <= 20);
    await hold('ArrowRight', map => map.name === 'Route28' && map.player.x >= 31);
  } else if (scenario === 'outdoor') {
    await hold('ArrowRight', map => map.name === 'NewBarkTown' && map.player.x >= 2);
    await hold('ArrowLeft', map => map.name === 'Route29' && map.player.x <= 58);
  } else if (scenario === 'vertical') {
    await hold('ArrowUp', map => map.name === 'Route30' && map.player.y <= 51);
    await hold('ArrowDown', map => map.name === 'CherrygroveCity' && map.player.y >= 2);
  } else {
    for (let cycle = 0; cycle < 3; cycle++) {
      await hold('ArrowDown', map => map.name === 'NewBarkTown');
      const arrivalY = report.steps.at(-1).after.player.y;
      await hold('ArrowDown', map => map.name === 'NewBarkTown' && map.player.y > arrivalY);
      await hold('ArrowUp', map => map.name === 'PlayersHouse1F');
    }
  }
  await page.waitForTimeout(500);
  const captured = await page.evaluate(tracing => {
    streamTiming.active = false;
    window.gpuFrameTiming?.stop?.();
    performance.mark('geothite_stream_measure_end');
    return { stoppedAt: performance.now(), trace: tracing ? JSON.parse(streamWasm.crystal_loading_trace_read()) : { traceEvents: [], droppedEvents: 0 }, timing: streamTiming, audioCalls: window.streamAudioCalls, gpuApi: window.streamGpuApi,
      workers: { terrain: __crystalTerrainWorkerMetrics, actors: __crystalActorWorkerMetrics },
      workerTraces: { terrain: globalThis.__crystalTerrainWorkerTraces ?? [],
        actors: globalThis.__crystalActorWorkerTraces ?? [] } };
  }, tracing);
  report.traceEnd = captured.stoppedAt;
  await stopCpuTrace();
  await fs.writeFile(`${output}/trace.json`, JSON.stringify(captured.trace));
  await fs.writeFile(`${output}/worker-traces.json`, JSON.stringify(captured.workerTraces));
  let merged = captured.trace;
  report.workerTraceAlignment = {};
  for (const [lane, merge, header] of [['terrain', mergeTerrainWorkerTrace, 'worker.callback_header'],
    ['actors', mergeActorWorkerTrace, 'actor_worker.callback_header']]) {
    const jobs = captured.workerTraces[lane];
    const included = jobs.filter(job => captured.trace.traceEvents.some(event =>
      event.name === header && !event.inProgress && event.args?.job === job.id));
    const result = merge(merged, included);
    merged = result.trace;
    report.workerTraceAlignment[lane] = { alignments: result.alignments,
      outsideCapture: jobs.filter(job => !included.includes(job)).map(job => job.id) };
  }
  await fs.writeFile(`${output}/cpu-trace.json`, JSON.stringify(merged));
  await fs.writeFile(`${output}/timing.json`, JSON.stringify(captured.timing));
  report.workers = captured.workers;
  if (captured.audioCalls) report.audioCalls = captured.audioCalls;
  if (captured.gpuApi) report.gpuApi = captured.gpuApi;
  report.wasmPeakBytes = captured.timing.wasmPeakBytes;
  report.glRenderer = captured.timing.glRenderer;
  report.traceDropped = captured.trace.droppedEvents;
  report.gpu = process.env.GPU_WATERFALL === '1' || process.env.GPU_WATERFALL_COMPLETIONS === '1'
    ? await page.evaluate(() => window.gpuFrameTiming?.finish()) : null;
  report.after = await ready();
  const sorted = captured.timing.frames.map(frame => frame.interval).sort((a, b) => a - b);
  const quantile = probability => {
    const position = (sorted.length - 1) * probability;
    const low = Math.floor(position), high = Math.ceil(position);
    return sorted[low] + (sorted[high] - sorted[low]) * (position - low);
  };
  report.intervals = { count: sorted.length, mean: sorted.reduce((a, b) => a + b, 0) / sorted.length,
    median: quantile(.5), p95: quantile(.95), p99: quantile(.99), maximum: sorted.at(-1) };
  report.presentationFrames = captured.trace.traceEvents.filter(event => event.name === 'presentation.world_motion'
    && !event.inProgress).length;
  if (process.env.STREAM_SCREENSHOTS === '1') {
    await page.screenshot({ path: `${output}/after.png` });
    report.pairBefore = await page.evaluate(() => streamBridge.execute({ kind: 'press', button: 'start', frames: 8 }));
    assert.ok(report.pairBefore.observe.menus.length > 0, 'Production Start menu must open');
    report.pairThreeDAvailable = report.pairBefore.presentation?.voxel?.active === true;
    await page.screenshot({ path: `${output}/${report.pairThreeDAvailable ? 'pair-3d' : 'pair-unavailable'}.png` });
    const toggle = page.locator('#view-toggle');
    const options = page.locator('#player-options');
    if (!await toggle.isVisible()) await options.locator('summary').click();
    await toggle.click();
    if (await options.getAttribute('open') !== null) await options.locator('summary').click();
    const stop = Date.now() + 10000;
    do {
      report.pairAfter = await observe();
      assert.ok(Date.now() < stop, 'Classic view did not activate');
      if (!report.pairAfter.presentation?.voxel?.active) break;
      await page.waitForTimeout(50);
    } while (true);
    const state = observation => ({ map: observation.map_info.name,
      player: observation.map_info.player, status: observation.status,
      text: observation.observe.text, menus: observation.observe.menus });
    assert.deepEqual(state(report.pairAfter), state(report.pairBefore), 'View switch changed authoritative gameplay');
    report.pairStatePreserved = true;
    await page.screenshot({ path: `${output}/pair-2d.png` });
  }
  assert.equal(report.traceDropped, 0, 'A dropped trace is incomplete evidence');
  assert.ok((!tracing || report.presentationFrames > 0) && sorted.length > 0, 'No presentation observations recorded');
  assert.deepEqual(captured.timing.gpuErrors, []);
  assert.deepEqual(captured.timing.lost, []);
  assert.deepEqual(report.errors, []);
  // Worker fallback and unavailable frames are measured outcomes, not reasons
  // to discard a slower baseline and bias the comparison toward success.
  report.success = true;
} catch (error) {
  report.failure = String(error);
  if (page && !page.isClosed()) {
    try { await page.screenshot({ path: `${output}/failure.png`, timeout: 5000 }); } catch {}
  }
} finally {
  clearTimeout(deadline);
  if (cpuTraceActive) {
    try { await stopCpuTrace(); } catch (error) { report.errors.push(`CPU trace: ${error}`); }
  }
  await fs.writeFile(`${output}/report.json`, JSON.stringify(report, null, 2));
  await browser?.close();
}
console.log(JSON.stringify({ success: report.success, intervals: report.intervals, failure: report.failure }));
assert.ok(report.success, `Inspect ${output}/report.json`);
