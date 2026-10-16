/** Front-door allocation regression; all movement uses the production bridge. */
import { chromium, webkit, firefox } from 'playwright';
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { installTextureMemoryAccounting } from './browser-texture-memory.mjs';
import { installWebGpuMemoryAccounting } from './browser-webgpu-memory.mjs';
import { mergeTerrainWorkerTrace, mergeActorWorkerTrace } from './terrain-worker-trace.mjs';
const [url, output, engine = 'webkit', cyclesText = '6'] = process.argv.slice(2);
const cycles = Number(cyclesText);
assert.ok(url && output && ['webkit', 'chromium', 'firefox'].includes(engine) &&
  Number.isInteger(cycles) && cycles > 0,
  'usage: node tools/browser-house-memory.mjs URL OUTPUT_JSON [webkit|chromium|firefox] [CYCLES]');
assert.equal(new URL(url).searchParams.get('preview'), 'house-door');
assert.equal(new URL(url).searchParams.get('multiplayer'), 'off');
const report = { url, engine, cycles, traceRequested: new URL(url).searchParams.get('loading-trace') === 'on', samples: [], transitions: [], errors: [],
  quality: { requiredSceneryFullDetail: true },
  accounting: 'WASM linear-memory high-water, WebGL buffers and known-format texture storage; excludes renderbuffers and driver overhead. Unknown texture formats are reported.' };
let browser, page;
try {
  report.headless = process.env.BROWSER_HEADLESS !== '0';
  report.browserExecutable = process.env.BROWSER_EXECUTABLE ?? null;
  browser = await ({ chromium, webkit, firefox })[engine].launch({ headless: report.headless,
    ...(report.browserExecutable ? { executablePath: report.browserExecutable } : {}),
    ...(engine === 'chromium' && process.env.BROWSER_CHANNEL ?
      { channel: process.env.BROWSER_CHANNEL } : {}) });
  report.browser = browser.version();
  const context = await browser.newContext({ viewport: { width: 1280, height: 900 } });
  report.adapterDropControl = process.env.BROWSER_ADAPTER_DROP === 'second';
  if (report.adapterDropControl) {
    await context.addInitScript(() => {
      if (!navigator.gpu) throw Error('Adapter-loss control requires WebGPU');
      const proto = Object.getPrototypeOf(navigator.gpu);
      const request = proto.requestAdapter;
      let calls = 0;
      proto.requestAdapter = function (...args) {
        if (++calls === 2) return Promise.resolve(null);
        return request.apply(this, args);
      };
    });
  }
  const installBufferMemoryAccounting = () => {
    Error.stackTraceLimit = 32;
    localStorage.setItem('crystal.audio.muted', 'true');
    window.houseMemory = { backend:'webgl', largestBuffer: 0, bufferUploadBytes: 0,
      liveBufferBytes: 0, peakBufferBytes: 0, contextLost: 0,
      drawCalls: 0, framebufferBlits: 0, defaultFramebufferDrawCalls: 0,
      webglContextLosses: [] };
    const sizes = new WeakMap();
    const proto = WebGL2RenderingContext.prototype;
    const drawTargets = new WeakMap();
    const readTargets = new WeakMap();
    const originalBindFramebuffer = proto.bindFramebuffer;
    proto.bindFramebuffer = function (target, framebuffer) {
      const result = originalBindFramebuffer.call(this, target, framebuffer);
      if (target === this.FRAMEBUFFER || target === this.DRAW_FRAMEBUFFER) drawTargets.set(this, framebuffer);
      if (target === this.FRAMEBUFFER || target === this.READ_FRAMEBUFFER) readTargets.set(this, framebuffer);
      return result;
    };
    for (const name of ['drawArrays', 'drawElements', 'drawArraysInstanced', 'drawElementsInstanced']) {
      const original = proto[name];
      proto[name] = function (...args) {
        houseMemory.drawCalls++;
        const result = original.apply(this, args);
        if (!drawTargets.get(this)) {
          houseMemory.defaultFramebufferDrawCalls++;
          if (houseMemory.requestDefaultCapture) {
            houseMemory.requestDefaultCapture = false;
            // Read after the render callback completes but before WebGL can
            // discard its drawing buffer. Headless page screenshots can lose it.
            queueMicrotask(() => {
              try { window.houseCapturedFrame = this.canvas.toDataURL('image/png'); }
              catch (error) { window.houseCaptureError = String(error); }
            });
          }
          if (houseMemory.requestDefaultPixel && !readTargets.get(this)) {
            const pixel = new Uint8Array(4);
            this.readPixels(Math.floor(this.drawingBufferWidth / 2), Math.floor(this.drawingBufferHeight / 2),
              1, 1, this.RGBA, this.UNSIGNED_BYTE, pixel);
            houseMemory.defaultFramebufferPixel = [...pixel];
            houseMemory.requestDefaultPixel = false;
          }
        }
        return result;
      };
    }
    const originalBlit = proto.blitFramebuffer;
    proto.blitFramebuffer = function (...args) {
      houseMemory.framebufferBlits++;
      return originalBlit.apply(this, args);
    };
    const originalData = proto.bufferData;
    proto.bufferData = function (...args) {
      const data = args[1];
      const unit = data?.BYTES_PER_ELEMENT ?? 1;
      const bytes = typeof data === 'number' ? data :
        args[4] ? args[4] * unit : Math.max(0, (data?.byteLength ?? 0) - (args[3] ?? 0) * unit);
      const targets = new Map([[this.ARRAY_BUFFER, this.ARRAY_BUFFER_BINDING],
        [this.ELEMENT_ARRAY_BUFFER, this.ELEMENT_ARRAY_BUFFER_BINDING],
        [this.UNIFORM_BUFFER, this.UNIFORM_BUFFER_BINDING],
        [this.COPY_READ_BUFFER, this.COPY_READ_BUFFER_BINDING],
        [this.COPY_WRITE_BUFFER, this.COPY_WRITE_BUFFER_BINDING]]);
      const binding = targets.get(args[0]);
      const buffer = binding === undefined ? null : this.getParameter(binding);
      const result = originalData.apply(this, args);
      houseMemory.largestBuffer = Math.max(houseMemory.largestBuffer, bytes);
      houseMemory.bufferUploadBytes += bytes;
      if (buffer) {
        houseMemory.liveBufferBytes += bytes - (sizes.get(buffer) ?? 0);
        sizes.set(buffer, bytes);
        houseMemory.peakBufferBytes = Math.max(houseMemory.peakBufferBytes, houseMemory.liveBufferBytes);
      }
      return result;
    };
    const originalDelete = proto.deleteBuffer;
    proto.deleteBuffer = function (buffer) {
      const result = originalDelete.call(this, buffer);
      houseMemory.liveBufferBytes -= sizes.get(buffer) ?? 0;
      sizes.delete(buffer);
      return result;
    };
    document.addEventListener('webglcontextlost', event => {
      houseMemory.contextLost++;
      houseMemory.webglContextLosses.push({ time:performance.now(),
        gameCanvas:event.target === document.querySelector('canvas'),
        width:event.target.width, height:event.target.height,
        message:event.statusMessage ?? '' });
    }, true);
  };
  // Keep both installers in one script: separate init scripts have no ordering
  // contract, and the texture hook needs the shared meter created above.
  await context.addInitScript({ content:
    `(${installBufferMemoryAccounting.toString()})();(${installTextureMemoryAccounting.toString()})();(${installWebGpuMemoryAccounting.toString()})();` });
  page = await context.newPage();
  page.on('pageerror', error => report.errors.push(String(error)));
  page.on('crash', () => report.errors.push('browser page crashed'));
  page.on('console', message => {
    if (message.type() === 'error') {
      (report.consoleErrors ??= []).push(message.text());
      if (/shaderCompilationError/.test(message.text())) console.log(message.text());
    }
    if (message.type() === 'warning' || /%c(?:WARN|ERROR)%c/.test(message.text())) {
      const text = message.text();
      const counts = report.rendererDiagnosticCounts ??= {};
      counts[text] = (counts[text] ?? 0) + 1;
      if (counts[text] === 1) {
        (report.rendererDiagnostics ??= []).push(text);
        console.log(JSON.stringify({ rendererDiagnostic: text }));
      }
    }
  });
  await page.goto(url, { waitUntil: 'networkidle', timeout: 240000 });
  report.stage = 'startup';
  await fs.writeFile(output, JSON.stringify(report, null, 2) + '\n');
  await page.waitForFunction(() => document.querySelector('#loading')?.hidden ||
    document.querySelector('#startup-error')?.open, null, { timeout: 240000 });
  assert.equal(await page.locator('#startup-error').evaluate(e => e.open ? e.textContent : null), null);
  const artifacts = await page.evaluate(async () => {
    const scripts = [...document.scripts].map(s => s.textContent).join('\n');
    const path = window.__crystalRendererBundle?.module ?? scripts.match(/import\('(\.\/crystal-bevy(?:-[a-f0-9]{64})?\.js)'\)/)?.[1];
    if (!path) throw Error('Missing compiled game import');
    const wasm = await import(path);
    window.houseWasm = wasm;
    window.houseRaw = await wasm.default();
    wasm.crystal_set_muted(true);
    const { createGameBridge } = await import('./webmcp.js');
    window.houseBridge = createGameBridge(wasm, { timeoutMs: 120000 });
    const requests = performance.getEntriesByType('resource').map(entry => entry.name)
      .filter(name => /\/crystal-bevy(?:-webgpu)?(?:-[a-f0-9]{64}|_bg)\.wasm$/.test(new URL(name).pathname));
    return { script: new URL(path, location.href).href,
      wasm: [...new Set(requests)], wasmRequests: requests };
  });
  report.quality.requestedArtifacts = artifacts;
  assert.equal(artifacts.wasm.length, 1, 'Identify the single game WASM actually loaded');
  report.quality.artifacts = [];
  // Hash on the Node side so verification does not allocate a second WASM
  // response inside the browser whose memory this trial measures.
  for (const artifactUrl of [artifacts.script, artifacts.wasm[0]]) {
    const response = await fetch(artifactUrl, { signal: AbortSignal.timeout(120000) });
    assert.ok(response.ok, 'Retrieve compiled artifact: ' + response.status);
    const hash = createHash('sha256');
    let bytes = 0;
    for await (const chunk of response.body) { hash.update(chunk); bytes += chunk.length; }
    const sha256 = hash.digest('hex');
    // The packager's filename hashes the combined pre-rewrite JS/WASM bundle,
    // so it is a version identifier rather than either file's content digest.
    const bundleVersion = new URL(artifactUrl).pathname.match(/-([a-f0-9]{64})(?:_bg)?\.(?:js|wasm)$/)?.[1];
    report.quality.artifacts.push({ url: artifactUrl, sha256, bytes, bundleVersion,
      etag: response.headers.get('etag') });
  }
  assert.equal(report.quality.artifacts[0].bundleVersion,
    report.quality.artifacts[1].bundleVersion, 'Game glue and WASM have the same packaged version');
  const command = async input => {
    const state = await page.evaluate(input => houseBridge.execute(input), input);
    assert.ok(!state.recent_events?.error, state.recent_events?.error);
    return state;
  };
  const press = button => command({ kind: 'press', button, frames: 8 });
  const waitForTerrain = async map => {
    const deadline = Date.now() + 240000;
    let nextSnapshot = 0;
    while (Date.now() < deadline) {
      const state = await command({ kind: 'observe' });
      const voxel = state.presentation?.voxel;
      if (Date.now() >= nextSnapshot) {
        nextSnapshot = Date.now() + 5000;
        report.loading = { map, voxel, memory: await page.evaluate(() => ({
          heapBytes: houseRaw.memory.buffer.byteLength,
          allocator: houseWasm.crystal_loading_memory_read ? JSON.parse(houseWasm.crystal_loading_memory_read()) : null,
          ...houseMemory })) };
        await fs.writeFile(output, JSON.stringify(report, null, 2) + '\n');
        if (new URL(url).searchParams.get('loading-trace') === 'on') {
          await fs.writeFile(output.replace(/\.json$/, '-trace.json'),
            await page.evaluate(() => houseWasm.crystal_loading_trace_read()));
        }
        console.log(JSON.stringify({ loading: report.loading }));
      }
      if (!voxel) throw Error('Build must include location-tester voxel readiness diagnostics');
      if (voxel.scenery_full_detail !== true) throw Error('Full scenery detail is required for this memory trial');
      if (state.map_info.name === map && voxel.active &&
          voxel.terrain_map === map && voxel.terrain_pending === false &&
          !voxel.profiles_pending && voxel.active_frames >= 30) return;
      await new Promise(resolve => setTimeout(resolve, 250));
    }
    throw Error('Full-detail terrain did not finish for ' + map);
  };
  const sample = async label => {
    await page.evaluate(() => { houseMemory.requestDefaultPixel = true; });
    await page.waitForTimeout(250);
    report.samples.push(await page.evaluate(label => {
      const canvas = document.querySelector('canvas');
      const bounds = canvas.getBoundingClientRect();
      const style = getComputedStyle(canvas);
      return { label,
      heapBytes: houseRaw.memory.buffer.byteLength,
      workerMemory: {
        terrain: globalThis.__crystalTerrainWorkerMetrics?.workerMemory ?? null,
        actor: globalThis.__crystalActorWorkerMetrics?.workerMemory ?? null,
      },
      actorCache: globalThis.__crystalActorWorkerMetrics?.cache ?? null,
      shaderErrors: window.houseShaderErrors ?? [],
          allocator: houseWasm.crystal_loading_memory_read ? JSON.parse(houseWasm.crystal_loading_memory_read()) : null,
          ...houseMemory,
      canvas: { width: canvas.width, height: canvas.height,
        displayedWidth: bounds.width, displayedHeight: bounds.height,
        display: style.display, visibility: style.visibility, opacity: style.opacity },
      enabled: document.querySelector('#view-toggle').getAttribute('aria-pressed') };
    }, label));
    const current = report.samples.at(-1);
    const requestedWorkers = ['terrain', 'actor'].filter(name => current.workerMemory[name] != null || new URL(url).searchParams.get(name + 'Worker') === '1');
    current.combinedLinearMemoryHighWaterBytes = requestedWorkers.every(name => current.workerMemory[name]?.peakBytes != null)
      ? current.heapBytes + requestedWorkers.reduce((sum, name) => sum + current.workerMemory[name].peakBytes, 0) : null;
    report.samples.at(-1).voxel = (await command({ kind: 'observe' })).presentation?.voxel;
    report.stage = label;
    await fs.writeFile(output, JSON.stringify(report, null, 2) + '\n');
    console.log(JSON.stringify({ stage: label, ...report.samples.at(-1) }));
    assert.equal(report.samples.at(-1).contextLost, 0);
    assert.deepEqual(report.samples.at(-1).gpuErrors ?? [], [], 'GPU validation must succeed');
    assert.deepEqual(report.samples.at(-1).shaderErrors, [], 'Shader compilation must succeed');
    await page.evaluate(() => {
      window.houseCapturedFrame = null;
      window.houseCaptureError = null;
      houseMemory.requestDefaultCapture = true;
    });
    await page.waitForFunction(() => window.houseCapturedFrame || window.houseCaptureError,
      null, { timeout: 30000 });
    const captured = await page.evaluate(() => {
      const result = { png: window.houseCapturedFrame, error: window.houseCaptureError };
      window.houseCapturedFrame = null;
      return result;
    });
    assert.equal(captured.error, null, 'production drawing-buffer capture must succeed');
    assert.ok(captured.png.startsWith('data:image/png;base64,'));
    await fs.writeFile(output.replace(/\.json$/, `-${label}-rendered.png`),
      Buffer.from(captured.png.slice('data:image/png;base64,'.length), 'base64'));
    assert.deepEqual(report.errors, []);
  };
  let state = await command({ kind: 'observe' });
  assert.equal(state.status.screen, 'overworld');
  assert.equal(state.map_info.name, 'PlayersHouse1F');
  await sample('2d-house');
  await page.screenshot({ path: output.replace(/\.json$/, '-2d-house.png') });
  const toggle = page.locator('#view-toggle');
  if (!await toggle.isVisible()) await page.locator('#player-options > summary').click();
  if (await toggle.getAttribute('aria-pressed') !== 'true') await toggle.click();
  await waitForTerrain('PlayersHouse1F');
  await sample('3d-house');
  await page.screenshot({ path: output.replace(/\.json$/, '-3d-house.png') });
  assert.equal(report.samples.at(-1).enabled, 'true');
  assert.ok(report.samples.at(-1).largestBuffer > 0, '3D must upload real geometry');
  if (await page.locator('#player-options').evaluate(e => e.open)) {
    await page.locator('#player-options > summary').click();
  }
  await page.locator('canvas').focus();
  const reach = async (button, map) => {
    for (let attempt = 0; attempt < 20; attempt++) {
      state = await press(button);
      if (state.map_info.name === map) {
        assert.equal(state.status.screen, 'overworld');
        report.transitions.push({ map, player: state.map_info.player });
        return;
      }
    }
    throw Error('Production movement did not reach ' + map);
  };
  for (let cycle = 1; cycle <= cycles; cycle++) {
    const previousUploads = await page.evaluate(() => houseMemory.bufferUploadBytes);
    await reach('down', 'NewBarkTown');
    await page.waitForFunction(previous => houseMemory.bufferUploadBytes - previous >= 1024 ** 2,
      previousUploads, { timeout: 120000 });
    await waitForTerrain('NewBarkTown');
    await sample('exit-' + cycle);
    if (cycle === 1) await page.screenshot({ path: output.replace(/\.json$/, '-outside.png') });
    const previous = state.map_info.player;
    state = await press('down');
    assert.equal(state.map_info.name, 'NewBarkTown');
    assert.notDeepEqual(state.map_info.player, previous, 'walking must continue outside');
    const inputDeadline = Date.now() + 120000;
    while (state.flow_state.animating && Date.now() < inputDeadline) {
      await page.waitForTimeout(250);
      state = await command({ kind: 'observe' });
    }
    assert.equal(state.flow_state.animating, false, 'walking must release menu input');
    state = await command({ kind: 'press', button: 'start', frames: 8 });
    report.lastMenuObservation = state;
    assert.ok((state.observe.menus ?? []).length || /POK.MON|PACK|SAVE/.test(JSON.stringify(state.observe)));
    await command({ kind: 'press', button: 'b', frames: 8 });
    await reach('up', 'PlayersHouse1F');
    await waitForTerrain('PlayersHouse1F');
    await sample('entry-' + cycle);
  }
  const settleMs = Number(process.env.BROWSER_SETTLE_MS ?? 0);
  assert.ok(Number.isFinite(settleMs) && settleMs >= 0 && settleMs <= 60000);
  if (settleMs > 0) {
    await page.waitForTimeout(settleMs);
    await sample('settled-house');
    if (!await toggle.isVisible()) await page.locator('#player-options > summary').click();
    await toggle.click();
    await page.waitForTimeout(1000);
    await sample('2d-return');
    assert.equal(report.samples.at(-1).enabled, 'false');
  }
  assert.ok(!(report.rendererDiagnostics ?? []).some(text => /%cERROR%c/.test(text)),
    'Rendering validation errors are gameplay blockers');
  report.finalUrl = page.url();
  if (report.adapterDropControl) {
    assert.equal(new URL(report.finalUrl).searchParams.get('renderer'), 'webgl',
      'adapter loss must reload the compile-time-compatible GL bundle');
    assert.ok(report.samples.every(sample => sample.backend === 'webgl'));
  }
  report.gameplayPassed = true;
  report.allocationStacks = await page.evaluate(() => globalThis.__crystalAllocationStacks ?? []);
  report.heapHighWater = Math.max(...report.samples.map(s => s.heapBytes));
  assert.ok(!(report.rendererDiagnostics ?? []).some(text => /%cERROR%c/.test(text)),
    'Rendering validation errors are gameplay blockers');
  report.passed = true;
} catch (error) {
  report.failure = String(error);
  if (page && !page.isClosed()) {
    try {
      report.failureState = await page.evaluate(async () => {
        const scripts = [...document.scripts].map(s => s.textContent).join('\n');
        const path = window.__crystalRendererBundle?.module ?? scripts.match(/import\('(\.\/crystal-bevy(?:-[a-f0-9]{64})?\.js)'\)/)?.[1];
        let heapBytes = null;
        if (path) {
          const wasm = await import(path);
          window.houseWasm = wasm;
          heapBytes = (await wasm.default()).memory.buffer.byteLength;
        }
        return { heapBytes, ...window.houseMemory,
          loadingHidden: document.querySelector('#loading')?.hidden,
          startupError: document.querySelector('#startup-error')?.open
            ? document.querySelector('#startup-error-message')?.textContent : null,
          progress: [...document.querySelectorAll('.asset-progress')]
            .map(e => ({ phase: e.dataset.phase, text: e.textContent })),
          visible: document.visibilityState, focused: document.hasFocus(),
          activeElement: document.activeElement?.tagName,
          resources: performance.getEntriesByType('resource').map(e => e.name) };
      });
    } catch (diagnosticError) { report.diagnosticFailure = String(diagnosticError); }
  }
  process.exitCode = 1;
} finally {
  if (page && !page.isClosed()) {
    try {
      report.cpuWorkers = await page.evaluate(() => ({
        terrain: globalThis.__crystalTerrainWorkerMetrics ? { metrics: globalThis.__crystalTerrainWorkerMetrics, traces: globalThis.__crystalTerrainWorkerTraces } : null,
        actor: globalThis.__crystalActorWorkerMetrics ? { metrics: globalThis.__crystalActorWorkerMetrics, traces: globalThis.__crystalActorWorkerTraces } : null,
      }));
      for (const name of ['terrain', 'actor']) if (report.cpuWorkers[name] || new URL(url).searchParams.get(name + 'Worker') === '1') {
        assert.ok(report.cpuWorkers[name]?.metrics.completed > 0, name + ' worker must actually complete');
        for (const counter of ['fallback', 'failed', 'initFailures']) assert.equal(report.cpuWorkers[name].metrics[counter], 0, name + ' worker ' + counter);
      }
      report.combinedMemoryMeaning = 'Sum of requested main/worker WebAssembly linear-memory high-water; excludes JavaScript, browser and GPU driver overhead.';
      report.allocationStacks ??= await page.evaluate(() => globalThis.__crystalAllocationStacks ?? []);
      if (new URL(url).searchParams.get('allocation-stacks') === 'on') {
        assert.ok(report.allocationStacks.length > 0 && report.allocationStacks.length <= 64,
          'Requested allocation stacks must be captured within the diagnostic bound');
      }
      const trace = await page.evaluate(() => {
        if (typeof window.houseWasm?.crystal_loading_trace_read !== 'function') throw Error('Missing Rust loading waterfall');
        return window.houseWasm.crystal_loading_trace_read();
      });
      let capture = JSON.parse(trace);
      if (report.traceRequested) {
        for (const [name, merge] of [['terrain', mergeTerrainWorkerTrace], ['actor', mergeActorWorkerTrace]]) {
          const worker = report.cpuWorkers[name];
          if (worker?.traces?.length) {
            const merged = merge(capture, worker.traces);
            capture = merged.trace;
            worker.traceAlignment = merged.alignments;
          }
        }
      }
      report.traceEventCount = capture.traceEvents.length;
      if (report.traceRequested) {
        report.waterfall = output.replace(/\.json$/, '-trace.json');
        await fs.writeFile(report.waterfall, JSON.stringify(capture));
        assert.ok(capture.traceEvents.length > 0, 'Requested waterfall must contain events');
        assert.equal(capture.droppedEvents, 0, 'Waterfall must include every regional load');
      } else {
        assert.equal(capture.traceEvents.length, 0, 'Production memory control must keep tracing disabled');
      }
    } catch (error) { report.traceFailure = String(error); report.passed = false; process.exitCode = 1; }
  }
  await fs.writeFile(output, JSON.stringify(report, null, 2) + '\n');
  await browser?.close();
}
