/** Real browser input, frame intervals and GPU upload accounting. No gameplay shortcuts. */
import { chromium } from 'playwright';
import { installWebGpuTiming } from './browser-webgpu-timing.mjs';
import { mergeTerrainWorkerTrace, mergeActorWorkerTrace } from './terrain-worker-trace.mjs';
import fs from 'node:fs/promises';
import assert from 'node:assert/strict';
import os from 'node:os';
const [url, storage, mode, output] = process.argv.slice(2);
assert.ok(url && storage && ['2d', '2.5d'].includes(mode) && output,
  'usage: node tools/browser-walking-performance.mjs URL STORAGE_JSON 2d|2.5d OUTPUT_JSON');
const scenario = process.env.SCENARIO ?? 'walking';
assert.ok(['walking', 'stationary', 'lab-exit', 'route-exit'].includes(scenario));
const headless = process.env.BROWSER_HEADLESS === '1';
const browserServer = await chromium.launchServer({ headless, args: ['--mute-audio', ...(headless ? ['--enable-gpu'] : [])],
  ...(process.env.BROWSER_CHANNEL ? { channel: process.env.BROWSER_CHANNEL } : {}) });
const browser = await chromium.connect(browserServer.wsEndpoint());
// Host deadline survives a renderer blocked in a synchronous geometry build.
let hostDeadlineExpired = false;
let report = {};
const errors = [];
const watchdog = setTimeout(() => {
  hostDeadlineExpired = true;
  void fs.writeFile(output, JSON.stringify({ passed: false, url, mode, scenario,
    failure: 'Host deadline exceeded before a playable performance result' }, null, 2))
    .finally(() => browserServer.kill());
}, Number(process.env.PROBE_TIMEOUT_MS ?? 300000));
try {
  const context = await browser.newContext({ viewport: { width: 1280, height: 900 }, storageState: storage });
  const networkRequests = [];
  const networkTasks = [];
  context.on('requestfinished', request => {
    if (!/\.wasm(?:\?|$)|\/(?:actor|terrain)-worker\.js|open-(?:people|models)\.json/.test(request.url())) return;
    networkTasks.push(request.sizes().then(sizes => {
      networkRequests.push({ url: request.url(), resourceType: request.resourceType(), ...sizes });
    }, () => {}));
  });
  if (process.env.GPU_WATERFALL === '1') await context.addInitScript(installWebGpuTiming);
  await context.addInitScript(() => {
    localStorage.setItem('crystal.audio.preference', 'muted');
    window.walkingMetrics = { active: false, last: 0, frames: [], uploads: [], allocations: 0, longTasks: [],
      drawCalls: 0, bindBufferRanges: 0, textureParameterCalls: 0, bufferUploadBytes: 0,
      startupLongTasks: [], startupComplete: false,
      contextLosses: [], deviceLosses: [], gpuErrors: [], shaderErrors: [] };
    walkingMetrics.webgpuCanvasFrames = 0;
    if (typeof GPUCanvasContext !== 'undefined') {
      const acquire = GPUCanvasContext.prototype.getCurrentTexture;
      GPUCanvasContext.prototype.getCurrentTexture = function (...args) {
        const texture = acquire.apply(this, args);
        if (walkingMetrics.active) walkingMetrics.webgpuCanvasFrames++;
        return texture;
      };
    }
    document.addEventListener('webglcontextlost', event => {
      walkingMetrics.contextLosses.push({ time: performance.now(),
        gameCanvas: event.target === document.querySelector('canvas') });
    }, true);
    if (typeof GPUAdapter !== 'undefined') {
      const requestDevice = GPUAdapter.prototype.requestDevice;
      GPUAdapter.prototype.requestDevice = function (...args) {
        return requestDevice.apply(this, args).then(device => {
          device.lost.then(info => walkingMetrics.deviceLosses.push({ reason: info.reason, message: info.message }));
          device.addEventListener('uncapturederror', event => walkingMetrics.gpuErrors.push({
            type: event.error.constructor.name, message: event.error.message }));
          return device;
        });
      };
      const createShaderModule = GPUDevice.prototype.createShaderModule;
      GPUDevice.prototype.createShaderModule = function (...args) {
        const module = createShaderModule.apply(this, args);
        module.getCompilationInfo().then(info => {
          for (const message of info.messages) if (message.type === 'error') {
            walkingMetrics.shaderErrors.push(message.message);
          }
        });
        return module;
      };
    }
    function frame(t) {
      if (walkingMetrics.active && walkingMetrics.last) walkingMetrics.frames.push(t - walkingMetrics.last);
      walkingMetrics.last = walkingMetrics.active ? t : 0;
      requestAnimationFrame(frame);
    }
    requestAnimationFrame(frame);
    new PerformanceObserver(list => {
      for (const entry of list.getEntries()) {
        if (walkingMetrics.active) walkingMetrics.longTasks.push(entry.duration);
        else if (!walkingMetrics.startupComplete) {
          walkingMetrics.startupLongTasks.push({ start: entry.startTime, duration: entry.duration });
        }
      }
    }).observe({ type: 'longtask' });
    const proto = WebGL2RenderingContext.prototype;
    for (const name of ['drawArrays', 'drawElements', 'drawArraysInstanced', 'drawElementsInstanced',
      'bindBufferRange', 'texParameteri']) {
      const original = proto[name];
      const field = name === 'bindBufferRange' ? 'bindBufferRanges' :
        name === 'texParameteri' ? 'textureParameterCalls' : 'drawCalls';
      proto[name] = function (...args) {
        if (walkingMetrics.active) walkingMetrics[field]++;
        return original.apply(this, args);
      };
    }
    const bufferUpload = proto.bufferSubData;
    proto.bufferSubData = function (...args) {
      if (walkingMetrics.active) {
        const unit = args[2]?.BYTES_PER_ELEMENT ?? 1;
        walkingMetrics.bufferUploadBytes += args[4] ? args[4] * unit :
          Math.max(0, (args[2]?.byteLength ?? 0) - (args[3] ?? 0) * unit);
      }
      return bufferUpload.apply(this, args);
    };
    const create = proto.createTexture;
    proto.createTexture = function (...args) {
      if (walkingMetrics.active) walkingMetrics.allocations++;
      return create.apply(this, args);
    };
    const upload = proto.texSubImage2D;
    proto.texSubImage2D = function (...args) {
      if (walkingMetrics.active && args.length >= 9) {
        walkingMetrics.uploads.push({ width: args[4], height: args[5] });
      }
      return upload.apply(this, args);
    };
  });
  const page = await context.newPage();
  page.on('pageerror', e => { errors.push(e.message); console.error(e.message); });
  page.on('console', message => {
    if (/%cERROR%c|Caught rendering error|Quitting the application/.test(message.text())) errors.push(message.text());
  });
  const navigationStartedAt = Date.now();
  report.navigationLoadBefore = os.loadavg();
  await page.goto(url);
  await page.waitForFunction(() => document.querySelector('#loading').hidden
    || document.querySelector('#startup-error').open, null, { timeout: 180000 });
  assert.equal(await page.locator('#startup-error').evaluate(e => e.open ? e.textContent : null), null);
  report.startupReadyMs = Date.now() - navigationStartedAt;
  await page.evaluate(async synchronousAudio => {
    const scripts = [...document.scripts].map(script => script.textContent).join('\n');
    const bundle = window.__crystalRendererBundle?.module ?? scripts.match(/import\('(\.\/crystal-bevy(?:-[a-f0-9]{64})?\.js)'\)/)?.[1];
    if (!bundle) throw new Error('Missing compiled bundle import');
    const wasm = await import(bundle);
    window.walkingWasm = wasm;
    window.walkingRaw = await wasm.default();
    const { createGameBridge } = await import('./webmcp.js');
    window.walkingBridge = createGameBridge(wasm, { timeoutMs: 120000 });
    if (synchronousAudio) {
      // Experimental control only: execute the SAME synthesizer on the UI thread.
      const workerPath = scripts.match(/new URL\('(\.\/audio-worker(?:-[a-f0-9]{64})?\.js)'/)?.[1];
      if (!workerPath) throw new Error('Missing audio worker');
      const workerSource = await (await fetch(workerPath)).text();
      const audioPath = workerSource.match(/from '(\.\/crystal-audio(?:-[a-f0-9]{64})?\.js)'/)?.[1];
      if (!audioPath) throw new Error('Missing Rust audio module');
      const synth = await import(audioPath);
      await synth.default();
      globalThis.__crystalPollMidi = synth.synthesize_crystal_midi;
    }
    window.audioPreparation = { completed: 0, samples: 0 };
    const name = typeof globalThis.__crystalPollMidi === 'function' ? '__crystalPollMidi' : '__crystalSynthesizeMidi';
    const prepare = globalThis[name];
    if (typeof prepare === 'function') globalThis[name] = (...args) => {
      const result = prepare(...args);
      if (result) { audioPreparation.completed++; audioPreparation.samples += result.samples.length; }
      return result;
    };
  }, process.env.AUDIO_PREPARATION === 'sync');
  let lastProgress = 0;
  const observe = async () => {
    const state = await page.evaluate(() => walkingBridge.execute({ kind: 'observe' }));
    report.lastObservation = state;
    if (Date.now() - lastProgress > 10000) {
      lastProgress = Date.now();
      console.log(JSON.stringify({ phase: 'observation', frame: state.frame, voxel: state.presentation?.voxel }));
      if (new URL(url).searchParams.get('loading-trace') === 'on') {
        await fs.writeFile(output.replace(/\.json$/, '-trace.json'),
          await page.evaluate(() => walkingWasm.crystal_loading_trace_read()));
      }
    }
    return state;
  };
  let state = await observe();
  // Current browser builds enter the original intro/title before Continue.
  const press = button => page.evaluate(button => walkingBridge.execute({ kind: 'press', button, frames: 8 }), button);
  if (state.status.screen === 'intro') state = await press('a');
  if (state.status.screen === 'title') {
    if (!state.observe.text.includes('CONTINUE')) state = await press('start');
    assert.ok(state.observe.text.includes('CONTINUE'), 'fixture must have a valid Continue save');
    state = await press('a');
    for (let i = 0; i < 4 && state.status.screen !== 'overworld'; i++) state = await press('a');
  }
  const preview = new URL(url).searchParams.get('preview');
  const expectedMap = process.env.EXPECTED_MAP ?? (preview === 'ilex' ? 'IlexForest' : preview === 'azalea-well' ? 'AzaleaTown' : preview === 'route29' ? 'Route29' : scenario === 'lab-exit' ? 'ElmsLab' : 'NewBarkTown');
  assert.equal(state.map_info.name, expectedMap, 'unexpected fixture map');
  assert.equal(state.status.screen, 'overworld');
  if (process.env.REQUIRE_EMPTY_PARTY === '1') {
    assert.deepEqual(state.status.party, [], 'regression must start with the original empty party');
    report.emptyPartyBefore = state.status.party;
  }
  if (process.env.PROBE_STARTUP_INPUT === '1') {
    const started = Date.now();
    const pendingBeforeInput = state.presentation?.voxel?.terrain_pending;
    const menu = await press('start');
    assert.ok((menu.observe.menus ?? []).length || /POK.MON|PACK|SAVE/.test(JSON.stringify(menu.observe)),
      'production Start menu must respond during startup');
    report.firstControllableMs = Date.now() - navigationStartedAt;
    report.startupInput = { latencyMs: Date.now() - started, pendingBeforeInput,
      voxel: menu.presentation?.voxel };
    state = await press('b');
  }
  const toggle = page.locator('#view-toggle');
  if ((await toggle.getAttribute('aria-pressed') === 'true') !== (mode === '2.5d')) {
    const settings = page.locator('#player-options > summary');
    if (!await toggle.isVisible()) await settings.click();
    await toggle.click();
    if (await settings.isVisible()) await settings.click();
  }
  assert.equal(await toggle.getAttribute('aria-pressed'), String(mode === '2.5d'));
  await page.bringToFront();
  if (process.env.PROBE_STARTUP_INPUT !== '1') {
    const settleMs = Number(process.env.WARMUP_SETTLE_MS ?? 5000);
    assert.ok(Number.isFinite(settleMs) && settleMs >= 0 && settleMs <= 60000);
    report.warmupSettleMs = settleMs;
    await page.waitForTimeout(settleMs);
  }
  await page.locator('canvas').focus();
  state = await observe();
  if (mode === '2.5d') {
    const deadline = Date.now() + 180000;
    let probed = false;
    const probe3dInput = async () => {
      const started = Date.now();
      const menu = await press('start');
      assert.ok((menu.observe.menus ?? []).length || /POK.MON|PACK|SAVE/.test(JSON.stringify(menu.observe)),
        'production Start menu must respond with the 3D camera active');
      report.first3dControllableMs = Date.now() - navigationStartedAt;
      report.startup3dInput = { latencyMs: Date.now() - started,
        pendingBeforeInput: state.presentation.voxel.terrain_pending, voxel: menu.presentation?.voxel };
      state = await press('b');
      probed = true;
    };
    while (!state.presentation?.voxel?.active || state.presentation.voxel.profiles_pending
        || state.presentation.voxel.terrain_pending === true
        || ('terrain_map' in state.presentation.voxel && state.presentation.voxel.terrain_map !== state.map_info.name)
        || state.presentation.voxel.active_frames < 30) {
      assert.ok(Date.now() < deadline, '3D warmup timed out');
      if (!probed && process.env.PROBE_STARTUP_INPUT === '1' && state.presentation?.voxel?.active) {
        await probe3dInput();
      }
      await page.waitForTimeout(250); state = await observe();
    }
    if (!probed && process.env.PROBE_STARTUP_INPUT === '1') await probe3dInput();
    assert.equal(state.presentation.voxel.scenery_full_detail, true, 'preserve full-detail models');
  }
  report.fullTerrainReadyMs = Date.now() - navigationStartedAt;
  report.startupResources = await page.evaluate(() => performance.getEntriesByType('resource')
    .filter(entry => new URL(entry.name).origin === location.origin)
    .map(entry => ({ name: new URL(entry.name).pathname, initiator: entry.initiatorType,
      start: entry.startTime, responseStart: entry.responseStart, responseEnd: entry.responseEnd,
      duration: entry.duration, transferBytes: entry.transferSize,
      encodedBytes: entry.encodedBodySize, decodedBytes: entry.decodedBodySize })));
  await Promise.allSettled(networkTasks);
  report.networkRequests = networkRequests;
  if (process.env.WALK_SETUP) {
    report.walkSetup = [];
    for (const button of process.env.WALK_SETUP.split(',')) {
      assert.ok(['up', 'down', 'left', 'right'].includes(button));
      const result = await press(button);
      report.walkSetup.push({ button, map: result.map_info });
      assert.equal(result.map_info.name, expectedMap, 'walking setup must stay on the measured route');
    }
  }
  const actorSubmissionsBeforeWalk = await page.evaluate(() => globalThis.__crystalActorWorkerMetrics?.submitted ?? 0);
  const workerSubmissionsBeforeWalk = await page.evaluate(() => globalThis.__crystalTerrainWorkerMetrics?.submitted ?? 0);
  const loadBefore = os.loadavg();
  const counterStartedAt = Date.now();
  state = await observe();
  const beginTraceWindow = () => page.evaluate(() => {
    const trace = JSON.parse(walkingWasm.crystal_loading_trace_read());
    const frame = Math.max(0, ...trace.traceEvents
      .filter(e => e.name === 'render.frame' && !e.inProgress)
      .map(e => e.args.frame));
    if (window.gpuFrameTiming?.supported) window.gpuFrameTiming.start(30);
    return frame;
  });
  if (scenario === 'stationary') report.walkTraceStartFrame = await beginTraceWindow();
  await page.evaluate(() => { walkingMetrics.startupComplete = true; walkingMetrics.active = true; });
  if (scenario === 'stationary') await page.waitForTimeout(10000);
  let midpoint = null;
  let leg = 0;
  const legMs = Number(process.env.WALK_LEG_MS ?? 1000);
  assert.ok(Number.isFinite(legMs) && legMs >= 250 && legMs <= 10000);
  report.walkLegMs = legMs;
  const walkingKeys = process.env.WALK_AXIS === 'vertical'
    ? ['ArrowUp', 'ArrowDown', 'ArrowUp', 'ArrowDown', 'ArrowUp', 'ArrowDown']
    : ['ArrowRight', 'ArrowLeft', 'ArrowRight', 'ArrowLeft', 'ArrowRight', 'ArrowLeft'];
  if (process.env.WALK_FIRST === 'negative') walkingKeys.reverse();
  for (const key of (scenario === 'stationary' ? [] : scenario === 'lab-exit' ? ['ArrowDown', 'ArrowDown', 'ArrowDown'] : scenario === 'route-exit' ? Array(3).fill('ArrowLeft') : walkingKeys)) {
    await page.keyboard.down(key);
    if (leg === 0) report.walkTraceStartFrame = await beginTraceWindow();
    await page.waitForTimeout(legMs);
    await page.keyboard.up(key);
    ++leg;
    if (scenario === 'walking') {
      // Every leg is observable. Two endpoints can alias after a round trip.
      // Keep observation overhead outside the walking interval sample.
      await page.evaluate(() => { walkingMetrics.active = false; walkingMetrics.last = 0; });
      const legState = await observe();
      report.walkObservations ??= [];
      const previousPosition = report.walkObservations.at(-1)?.map.player ?? state.map_info.player;
      assert.ok(legState.map_info.player.x !== previousPosition.x
        || legState.map_info.player.y !== previousPosition.y,
        'every measured walking leg must move; do not accept a blocked ledge or NPC as walking FPS');
      report.walkObservations.push({ leg, key, map: legState.map_info, flow: legState.flow_state });
      assert.equal(legState.map_info.name, expectedMap,
        'a route walking comparison must stay on its named map; use route-exit for handoffs');
      if (leg === 3) midpoint = legState;
      await page.evaluate(() => { walkingMetrics.active = true; });
    }
  }
  const metrics = await page.evaluate(() => {
    walkingMetrics.active = false;
    return { ...walkingMetrics, audioPreparation, visible: !document.hidden, focused: document.hasFocus() };
  });
  const after = await observe();
  const counterElapsedSeconds = (Date.now() - counterStartedAt) / 1000;
  report.webgpuCanvasThroughput = metrics.webgpuCanvasFrames > 0
    ? { count: metrics.webgpuCanvasFrames, elapsedSeconds: counterElapsedSeconds,
      framesPerSecond: metrics.webgpuCanvasFrames / counterElapsedSeconds,
      meaning: 'Successful WebGPU canvas texture acquisitions in the active measurement windows over full interaction wall time, including observation overhead; not display-present confirmation.' }
    : { unavailable: 'No WebGPU canvas acquisitions were recorded.' };
  const beforeActive = state.presentation?.voxel;
  const afterActive = after.presentation?.voxel;
  const counterDelta = afterActive?.active_frames - beforeActive?.active_frames;
  report.active3dUpdates = beforeActive?.active && afterActive?.active && counterDelta >= 0
    ? { count: counterDelta, elapsedSeconds: counterElapsedSeconds,
      updatesPerSecond: counterDelta / counterElapsedSeconds,
      meaning: 'Active 3D app updates over the whole measured interaction, including observation overhead; not display-presented FPS.' }
    : { unavailable: 'The active-3D counter was inactive or reset during this interaction.' };
  if (process.env.REQUIRE_EMPTY_PARTY === '1') {
    assert.deepEqual(after.status.party, [], 'walking must preserve the empty party without an automatic starter');
    assert.equal(after.status.screen, 'overworld', 'empty-party walking must remain playable in the overworld');
    report.emptyPartyAfter = after.status.party;
  }
  const times = [...metrics.frames].sort((a, b) => a - b);
  report = { ...report, url, mode, scenario, headless, audioControl: process.env.AUDIO_PREPARATION ?? 'default', loadBefore, loadAfter: os.loadavg(), platform: os.platform(), arch: os.arch(), browser: browser.version(), before: state.map_info, after: after.map_info, midpoint: midpoint?.map_info,
    graphicsContext: process.env.GRAPHICS_CONTEXT ?? 'Concurrent graphics workload not inventoried; no isolation established.',
    error: after.recent_events.error, errors, rendererBefore: state.presentation?.voxel,
    rendererAfter: after.presentation?.voxel,
    timingMeaning: 'visible focused requestAnimationFrame intervals; not GPU timestamps or presented FPS', ...metrics,
    median: times[Math.floor(times.length / 2)], p95: times[Math.floor(times.length * .95)],
    max: times.at(-1), uploadBytes: metrics.uploads.reduce((n, p) => n + p.width * p.height * 4, 0) };
  const diagnostics = await page.evaluate(() => ({
    rendererModule: window.__crystalRendererBundle?.module ?? null,
    mainLinearMemoryBytes: walkingRaw.memory.buffer.byteLength,
    jsHeapApproximate: performance.memory ? {
      usedBytes: performance.memory.usedJSHeapSize, totalBytes: performance.memory.totalJSHeapSize,
      limitBytes: performance.memory.jsHeapSizeLimit,
      meaning: 'Chromium nonstandard approximate main JavaScript heap; excludes worker heaps and cannot be added to Wasm high-water as a process-memory total.',
    } : null,
    actorWorker: typeof globalThis.__crystalActorWorkerReady === 'function' ? {
      readiness: globalThis.__crystalActorWorkerReady(), metrics: globalThis.__crystalActorWorkerMetrics,
      traces: globalThis.__crystalActorWorkerTraces,
    } : null,
    terrainWorker: typeof globalThis.__crystalTerrainWorkerReady === 'function' ? {
      readiness: globalThis.__crystalTerrainWorkerReady(),
      metrics: globalThis.__crystalTerrainWorkerMetrics,
      traces: globalThis.__crystalTerrainWorkerTraces,
    } : null,
    memory: typeof walkingWasm.crystal_loading_memory_read === 'function'
      ? JSON.parse(walkingWasm.crystal_loading_memory_read()) : null,
    trace: typeof walkingWasm.crystal_loading_trace_read === 'function'
      ? walkingWasm.crystal_loading_trace_read() : null,
  }));
  if (process.env.GPU_WATERFALL === '1') {
    report.gpuTiming = await page.evaluate(() => window.gpuFrameTiming.finish());
    const finalGpuValidation = await page.evaluate(() => ({
      gpuErrors: walkingMetrics.gpuErrors, deviceLosses: walkingMetrics.deviceLosses,
      contextLosses: walkingMetrics.contextLosses, shaderErrors: walkingMetrics.shaderErrors,
    }));
    Object.assign(metrics, finalGpuValidation);
    Object.assign(report, finalGpuValidation);
  }
  report.memory = diagnostics.memory;
  report.mainLinearMemoryBytes = diagnostics.mainLinearMemoryBytes;
  report.jsHeapApproximate = diagnostics.jsHeapApproximate;
  report.rendererModule = diagnostics.rendererModule;
  report.graphicsHardware = await page.evaluate(async () => {
    const adapter = await navigator.gpu?.requestAdapter();
    const webgpu = window.__crystalRendererBundle?.module?.includes('webgpu');
    const gl = webgpu ? null : document.querySelector('canvas')?.getContext('webgl2');
    const debug = gl?.getExtension('WEBGL_debug_renderer_info');
    return { webgpuAdapter: adapter ? { vendor: adapter.info.vendor,
      architecture: adapter.info.architecture, description: adapter.info.description,
      fallback: adapter.info.isFallbackAdapter ?? adapter.isFallbackAdapter ?? null } : null,
      webglRenderer: debug ? gl.getParameter(debug.UNMASKED_RENDERER_WEBGL) : null };
  });
  if (process.env.REQUIRE_HARDWARE_GPU === '1') {
    const gpu = report.graphicsHardware;
    const modern = report.rendererModule?.includes('webgpu');
    const identity = modern ? [gpu.webgpuAdapter?.vendor, gpu.webgpuAdapter?.architecture,
      gpu.webgpuAdapter?.description].filter(Boolean).join(' ') : gpu.webglRenderer ?? '';
    assert.ok(identity && !/swiftshader|llvmpipe|software/i.test(identity)
      && (!modern || gpu.webgpuAdapter?.fallback !== true),
      'performance acceptance requires an observed hardware renderer');
  }

  if (diagnostics.terrainWorker || new URL(url).searchParams.get('terrainWorker') === '1') {
    report.terrainWorker = diagnostics.terrainWorker;
    if (report.terrainWorker && diagnostics.trace) {
      const events = JSON.parse(diagnostics.trace).traceEvents.filter(e => e.ph === 'X' && !e.inProgress
        && (e.name.startsWith('worker.') || e.name === 'region.publish_assets'));
      report.terrainWorker.mainStages = Object.groupBy(events, e => e.name);
      report.terrainWorker.mainStages = Object.fromEntries(Object.entries(report.terrainWorker.mainStages)
        .map(([name, samples]) => [name, { count: samples.length,
          totalMs: samples.reduce((sum, e) => sum + e.dur, 0) / 1000,
          maxMs: Math.max(...samples.map(e => e.dur)) / 1000 }]));
    }
    assert.ok(report.terrainWorker, 'terrainWorker=1 requires the compiled QA bridge');
    assert.equal(report.terrainWorker.metrics.fallback, 0, 'worker QA must not silently use synchronous fallback');
    assert.equal(report.terrainWorker.metrics.initFailures, 0, 'worker QA initialization must succeed');
    assert.equal(report.terrainWorker.metrics.failed, 0, 'worker QA must not fail transport or restoration');
    if (mode === '2.5d') {
      assert.ok(report.terrainWorker.metrics.completed > 0, 'worker must actually complete a terrain job');
      assert.ok(report.terrainWorker.metrics.successfulJobIds.length > 0, 'worker must report successful job IDs');
    } else {
      assert.equal(report.terrainWorker.metrics.submitted, workerSubmissionsBeforeWalk,
        '2D walking must not submit terrain jobs');
    }
  }
  if (diagnostics.actorWorker || new URL(url).searchParams.get('actorWorker') === '1') {
    report.actorWorker = diagnostics.actorWorker;
    assert.ok(report.actorWorker, 'actorWorker=1 requires the compiled actor QA bridge');
    for (const counter of ['fallback', 'initFailures', 'failed']) {
      assert.equal(report.actorWorker.metrics[counter], 0, `actor worker ${counter} must remain zero`);
    }
    if (mode === '2.5d') assert.ok(report.actorWorker.metrics.completed > 0, 'actor worker must actually finish appearances');
    else assert.equal(report.actorWorker.metrics.submitted, actorSubmissionsBeforeWalk, '2D walking must not submit actor jobs');
    const stages = JSON.parse(diagnostics.trace ?? '{"traceEvents":[]}').traceEvents
      .filter(e => e.ph === 'X' && !e.inProgress && e.name.startsWith('actor_worker.'));
    report.actorWorker.mainStages = Object.fromEntries(Object.entries(Object.groupBy(stages, e => e.name))
      .map(([name, samples]) => [name, { count: samples.length,
        totalMs: samples.reduce((sum, e) => sum + e.dur, 0) / 1000,
        maxMs: Math.max(...samples.map(e => e.dur)) / 1000 }]));
    report.combinedLinearMemoryHighWaterBytes = diagnostics.mainLinearMemoryBytes
      + (diagnostics.terrainWorker?.metrics.workerMemory?.peakBytes ?? 0)
      + (diagnostics.actorWorker?.metrics.workerMemory?.peakBytes ?? 0);
    report.combinedMemoryMeaning = 'Sum of main/worker WebAssembly linear-memory high-water; excludes JavaScript, driver and browser overhead.';
    report.actorWorker.overlapWithTerrainMs = Math.max(0, ...(report.actorWorker.traces ?? []).flatMap(actor =>
      (report.terrainWorker?.traces ?? []).map(terrain => Math.min(actor.cpuEnd, terrain.cpuEnd) - Math.max(actor.cpuStart, terrain.cpuStart))));
  }
  if (process.env.REQUIRE_DEFAULT_WORKERS === '1') {
    assert.ok(report.terrainWorker && report.actorWorker, 'default candidate must install both compiled bridges');
  }
  if (process.env.REQUIRE_DORMANT_WORKERS === '1') {
    assert.equal(mode, '2d');
    for (const name of ['terrainWorker', 'actorWorker']) {
      assert.ok(report[name], name + ' dormant bridge must exist');
      assert.equal(report[name].metrics.initializations, 0, name + ' must not initialize in never-3D session');
      assert.equal(report[name].metrics.submitted, 0, name + ' must not submit in never-3D session');
      assert.equal(report[name].metrics.workerMemory ?? null, null, name + ' must not allocate a Wasm instance');
    }
    assert.deepEqual(networkRequests.filter(request => /(?:actor|terrain)[-_]worker.*\.wasm(?:\?|$)/.test(request.url)), [],
      'never-3D session must not download worker Wasm');
  }
  if (diagnostics.trace) {
    if (report.terrainWorker?.traces?.length) {
      const merged = mergeTerrainWorkerTrace(JSON.parse(diagnostics.trace), report.terrainWorker.traces);
      diagnostics.trace = JSON.stringify(merged.trace);
      report.terrainWorker.traceAlignment = merged.alignments;
    }
    if (report.actorWorker?.traces?.length) {
      const merged = mergeActorWorkerTrace(JSON.parse(diagnostics.trace), report.actorWorker.traces);
      diagnostics.trace = JSON.stringify(merged.trace);
      report.actorWorker.traceAlignment = merged.alignments;
    }
    report.waterfall = output.replace(/\.json$/, '-trace.json');
    await fs.writeFile(report.waterfall, diagnostics.trace);
    report.droppedTraceEvents = JSON.parse(diagnostics.trace).droppedEvents;
  }
  await fs.writeFile(output, JSON.stringify(report, null, 2));
  await page.screenshot({ path: output.replace(/\.json$/, '.png') });
  if (process.env.RENDERER_PAIR === '1') {
    const pairState = snapshot => ({ map: snapshot.map_info, status: snapshot.status,
      flow: snapshot.flow_state, text: snapshot.observe.text, menus: snapshot.observe.menus });
    const initial = pairState(await observe());
    report.rendererPair = { state: initial, hour: new URL(url).searchParams.get('preview') ? 16 : null, images: {} };
    for (const renderer of ['2d', '3d']) {
      const options = page.locator('#player-options > summary');
      if (!await toggle.isVisible()) await options.click();
      if ((await toggle.getAttribute('aria-pressed') === 'true') !== (renderer === '3d')) await toggle.click();
      if (await options.isVisible()) await options.click();
      await page.waitForTimeout(1000);
      const snapshot = await observe();
      assert.deepEqual(pairState(snapshot), initial, 'renderer pair must preserve authoritative game state');
      const image = output.replace(/\.json$/, `-pair-${renderer}.png`);
      await page.screenshot({ path: image });
      report.rendererPair.images[renderer] = image;
    }
    const pairGpuValidation = await page.evaluate(() => ({
      contextLosses: walkingMetrics.contextLosses, deviceLosses: walkingMetrics.deviceLosses,
      gpuErrors: walkingMetrics.gpuErrors, shaderErrors: walkingMetrics.shaderErrors,
    }));
    Object.assign(metrics, pairGpuValidation);
    Object.assign(report, pairGpuValidation);
  }
  assert.deepEqual(errors, []);
  assert.deepEqual(metrics.contextLosses, [], 'walking must not lose a WebGL context');
  assert.deepEqual(metrics.deviceLosses, [], 'walking must not lose a WebGPU device');
  assert.deepEqual(metrics.gpuErrors, [], 'walking must not cause GPU validation errors');
  assert.deepEqual(metrics.shaderErrors, [], 'walking shaders must compile');
  if (new URL(url).searchParams.get('loading-trace') === 'on') {
    assert.equal(report.droppedTraceEvents, 0, 'walking waterfall must retain every regional load');
  }
  if (scenario === 'stationary') {
    assert.equal(after.map_info.name, state.map_info.name);
    assert.deepEqual(after.map_info.player, state.map_info.player,
      'stationary comparison must retain position and facing');
  } else if (scenario === 'lab-exit') assert.equal(after.map_info.name, 'NewBarkTown');
  else if (scenario === 'route-exit') assert.equal(after.map_info.name, 'Route29');
  else assert.ok([midpoint, after, ...(report.walkObservations ?? []).map(sample => ({ map_info: sample.map }))].some(sample => sample &&
    (sample.map_info.player.x !== state.map_info.player.x || sample.map_info.player.y !== state.map_info.player.y)),
    'the player must actually move, even if the return walk ends at its starting tile');
  assert.equal(after.recent_events.error, null);
  assert.ok(metrics.visible && metrics.focused, 'measure only a visible, focused game');
  console.log(JSON.stringify({ mode, frames: times.length, median: report.median,
    p95: report.p95, max: report.max, allocations: metrics.allocations, uploadBytes: report.uploadBytes }));
  if (process.env.REQUIRE_RETAINED_TEXTURES === '1') {
    assert.equal(metrics.allocations, 0, 'walking must retain composed GPU texture allocations');
  }
  report.passed = true;
  await fs.writeFile(output, JSON.stringify(report, null, 2));
} catch (error) {
  await fs.writeFile(output, JSON.stringify({ ...report, errors, passed: false, url, mode, scenario, failure: hostDeadlineExpired ? 'Host deadline exceeded before a playable performance result' : String(error) }, null, 2));
  throw error;
} finally {
  clearTimeout(watchdog);
  // The disposable renderer may be stuck in a synchronous full-detail build.
  // Kill only this BrowserServer; graceful close can otherwise conceal a timeout.
  await browserServer.kill();
}
