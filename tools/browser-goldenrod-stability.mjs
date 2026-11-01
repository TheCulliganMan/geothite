/** Disposable preview; every game action uses the production bridge or UI. */
import { chromium } from 'playwright';
import fs from 'node:fs/promises';
import assert from 'node:assert/strict';
import { execFile } from 'node:child_process';
import { promisify } from 'node:util';
const run = promisify(execFile);
const [url, output, cyclesText = '6'] = process.argv.slice(2);
const cycles = Number(cyclesText);
assert.ok(url && output && Number.isInteger(cycles) && cycles > 0,
  'usage: node tools/browser-goldenrod-stability.mjs URL OUTPUT_JSON [CYCLES]');
assert.equal(new URL(url).searchParams.get('preview'), 'department-store');
assert.equal(new URL(url).searchParams.get('multiplayer'), 'off');
const report = { url, cycles, samples: [], transitions: [], errors: [], processSamples: [],
  consoleDiagnostics: [],
  budgets: { wasmBytes: Number(process.env.WASM_BUDGET_MIB ?? 512) * 1024 ** 2,
    originalWasmBytes: 512 * 1024 ** 2, singleBufferBytes: 32 * 1024 ** 2 },
  timingMeaning: 'requestAnimationFrame intervals; not GPU completion or presented FPS',
  bufferAccounting: 'Requested WebGL bufferData storage; excludes textures and driver overhead',
  bufferAccountingEnabled: process.env.DISABLE_BUFFER_ACCOUNTING !== '1',
  quality: 'Run with CRYSTAL_SCENERY_FULL_DETAIL=1 at browser build time; record build revision separately.' };
let browser;
let page;
let sampler;
const bounded = async (promise, milliseconds) => {
  let timer;
  try {
    return await Promise.race([promise, new Promise((_, reject) => {
      timer = setTimeout(() => reject(Error(`Browser operation exceeded ${milliseconds} ms`)), milliseconds);
    })]);
  } finally { clearTimeout(timer); }
};
try {
  report.channel = process.env.BROWSER_CHANNEL ?? 'playwright-chromium';
  browser = await chromium.launch({ headless: false, args: ['--mute-audio'],
    ...(process.env.BROWSER_CHANNEL ? { channel: process.env.BROWSER_CHANNEL } : {}) });
  report.browser = browser.version();
  const browserCDP = await browser.newBrowserCDPSession();
  const processSample = async () => {
    const { processInfo } = await browserCDP.send('SystemInfo.getProcessInfo');
    if (process.platform !== 'win32') return;
    const ids = processInfo.map(p => Number(p.id)).filter(Number.isSafeInteger);
    if (!ids.length) return;
    const { stdout } = await run('powershell.exe', ['-NoProfile', '-Command',
      `Get-Process -Id ${ids.join(',')} -ErrorAction SilentlyContinue | Select-Object Id,WorkingSet64,PrivateMemorySize64 | ConvertTo-Json -Compress`]);
    const rows = JSON.parse(stdout || '[]');
    const processes = (Array.isArray(rows) ? rows : [rows]).map(p => ({ ...p,
      type: processInfo.find(info => Number(info.id) === p.Id)?.type ?? 'unknown' }));
    report.processSamples.push({ time: Date.now(), processes });
  };
  let sampling = false;
  sampler = setInterval(async () => {
    if (sampling) return;
    sampling = true;
    try { await processSample(); } catch (e) { report.processSamplingError = String(e); }
    finally { sampling = false; }
  }, 1000);
  const context = await browser.newContext({ viewport: { width: 1280, height: 900 } });
  await context.addInitScript(accountBuffers => {
    localStorage.setItem('crystal.audio.preference', 'muted');
    window.goldenrodMetrics = { largestBuffer: 0, bufferUploadBytes: 0, liveBufferBytes: 0,
      peakBufferBytes: 0, contextLost: 0,
      active: false, last: 0, intervals: [], longTasks: [], renderer: null };
    if (!accountBuffers) return;
    const proto = WebGL2RenderingContext.prototype;
    const allocations = new WeakMap();
    const deleteBuffer = proto.deleteBuffer;
    proto.deleteBuffer = function (buffer) {
      if (buffer) {
        goldenrodMetrics.liveBufferBytes -= allocations.get(buffer) ?? 0;
        allocations.delete(buffer);
      }
      return deleteBuffer.call(this, buffer);
    };
    const bufferData = proto.bufferData;
    proto.bufferData = function (...args) {
      const m = goldenrodMetrics;
      const data = args[1];
      const unit = data?.BYTES_PER_ELEMENT ?? 1;
      const bytes = typeof data === 'number' ? data : args[4] ? args[4] * unit :
        Math.max(0, (data?.byteLength ?? 0) - (args[3] ?? 0) * unit);
      m.largestBuffer = Math.max(m.largestBuffer, bytes);
      m.bufferUploadBytes += bytes;
      const binding = new Map([
        [this.ARRAY_BUFFER, this.ARRAY_BUFFER_BINDING],
        [this.ELEMENT_ARRAY_BUFFER, this.ELEMENT_ARRAY_BUFFER_BINDING],
        [this.COPY_READ_BUFFER, this.COPY_READ_BUFFER_BINDING],
        [this.COPY_WRITE_BUFFER, this.COPY_WRITE_BUFFER_BINDING],
        [this.PIXEL_PACK_BUFFER, this.PIXEL_PACK_BUFFER_BINDING],
        [this.PIXEL_UNPACK_BUFFER, this.PIXEL_UNPACK_BUFFER_BINDING],
        [this.TRANSFORM_FEEDBACK_BUFFER, this.TRANSFORM_FEEDBACK_BUFFER_BINDING],
        [this.UNIFORM_BUFFER, this.UNIFORM_BUFFER_BINDING],
      ]).get(args[0]);
      const buffer = binding ? this.getParameter(binding) : null;
      if (buffer) {
        m.liveBufferBytes += bytes - (allocations.get(buffer) ?? 0);
        allocations.set(buffer, bytes);
        m.peakBufferBytes = Math.max(m.peakBufferBytes, m.liveBufferBytes);
      }
      if (!m.renderer) {
        const debug = this.getExtension('WEBGL_debug_renderer_info');
        m.renderer = debug ? this.getParameter(debug.UNMASKED_RENDERER_WEBGL) : this.getParameter(this.RENDERER);
      }
      return bufferData.apply(this, args);
    };
    document.addEventListener('webglcontextlost', () => goldenrodMetrics.contextLost++, true);
    const frame = t => {
      const m = goldenrodMetrics;
      if (m.active && m.last && m.intervals.length < 60000) m.intervals.push(t - m.last);
      m.last = m.active ? t : 0;
      requestAnimationFrame(frame);
    };
    requestAnimationFrame(frame);
    new PerformanceObserver(list => {
      if (goldenrodMetrics.active) goldenrodMetrics.longTasks.push(...list.getEntries().map(e => e.duration));
    }).observe({ type: 'longtask' });
  }, process.env.DISABLE_BUFFER_ACCOUNTING !== '1');
  page = await context.newPage();
  page.on('crash', () => report.errors.push('Page process crashed'));
  page.on('pageerror', e => report.errors.push(e.message));
  page.on('console', message => {
    // Bevy's styled tracing writes ERROR/WARN through console.log.
    if (message.type() !== 'error' && message.type() !== 'warning' &&
        !/Caught rendering error|Quitting the application|%cERROR%c|%cWARN%c/.test(message.text())) return;
    if (/%cERROR%c|Caught rendering error|Quitting the application/.test(message.text())) {
      report.errors.push(message.text().slice(0, 4000));
    }
    report.consoleDiagnostics.push({ type: message.type(), text: message.text().slice(0, 4000) });
    if (report.consoleDiagnostics.length > 30) report.consoleDiagnostics.shift();
  });
  await page.goto(url);
  await page.waitForFunction(() => document.querySelector('#loading').hidden ||
    document.querySelector('#startup-error').open, null, { timeout: 240000 });
  assert.equal(await page.locator('#startup-error').evaluate(e => e.open ? e.textContent : null), null);
  await page.bringToFront();
  await page.locator('canvas').click();
  await page.evaluate(async () => {
    const scripts = [...document.scripts].map(s => s.textContent).join('\n');
    const path = scripts.match(/import\('(\.\/crystal-bevy(?:-[a-f0-9]{64})?\.js)'\)/)?.[1];
    assertPath(path);
    function assertPath(p) { if (!p) throw Error('Missing compiled game import'); }
    const wasm = await import(path);
    window.goldenrodRaw = await wasm.default();
    wasm.crystal_set_muted(true);
    const { createGameBridge } = await import('./webmcp.js');
    window.goldenrodBridge = createGameBridge(wasm, { timeoutMs: 120000 });
  });
  const healthy = state => {
    assert.ok(!state.error, `production bridge error: ${state.error}`);
    assert.ok(!state.recent_events?.error, `game error: ${state.recent_events?.error}`);
    return state;
  };
  const observe = async () => healthy(await bounded(page.evaluate(() => goldenrodBridge.execute({ kind: 'observe' })), 150000));
  const ready3d = async () => {
    const deadline = Date.now() + 240000;
    while (Date.now() < deadline) {
      const state = await observe();
      const voxel = state.presentation?.voxel;
      assert.ok(voxel, 'current production bridge must expose renderer readiness');
      assert.equal(voxel.scenery_full_detail, true, 'trial must preserve full-detail models');
      const currentTerrain = !('terrain_map' in voxel) || voxel.terrain_map === state.map_info.name;
      if (voxel.active && currentTerrain && voxel.terrain_pending !== true
          && !voxel.profiles_pending && voxel.active_frames >= 30) return state;
      await page.waitForTimeout(250);
    }
    throw Error('Production 3D renderer did not finish its active-frame warmup');
  };
  const press = async (button, frames = 1) => healthy(await bounded(page.evaluate(({ button, frames }) =>
    goldenrodBridge.execute({ kind: 'press', button, frames }), { button, frames }), 150000));
  const sample = async label => {
    report.samples.push(await page.evaluate(label => ({ label, time: Date.now(),
      heapBytes: goldenrodRaw.memory.buffer.byteLength, ...goldenrodMetrics,
      intervals: undefined, longTasks: undefined, visible: !document.hidden,
      focused: document.hasFocus(), enabled: document.querySelector('#view-toggle').getAttribute('aria-pressed'),
      muted: document.querySelector('#mute').getAttribute('aria-pressed') }), label));
    assert.equal(report.samples.at(-1).muted, 'true');
    assert.equal(report.samples.at(-1).contextLost, 0);
    assert.deepEqual(report.errors, []);
  };
  let state = await observe();
  assert.equal(state.status.screen, 'overworld');
  assert.equal(state.map_info.name, 'GoldenrodDeptStore1F');
  assert.deepEqual({ x: state.map_info.player.x, y: state.map_info.player.y }, { x: 8, y: 4 });
  report.initial = state;
  await sample('preview-start');
  const settings = page.locator('#player-options > summary');
  const toggle = page.locator('#view-toggle');
  if (!await toggle.isVisible()) await settings.click();
  if (await toggle.getAttribute('aria-pressed') === 'true') {
    await toggle.click();
    await page.waitForTimeout(1000);
  }
  await sample('2d-store');
  const store2d = await observe();
  if (await page.locator('#player-options').evaluate(e => e.open)) await settings.click();
  await page.screenshot({ path: output.replace(/\.json$/, '-store-2d.png') });
  if (!await toggle.isVisible()) await settings.click();
  if (await toggle.getAttribute('aria-pressed') !== 'true') await toggle.click();
  if (await page.locator('#player-options').evaluate(e => e.open)) await settings.click();
  await page.locator('canvas').focus();
  await page.waitForTimeout(5000);
  await ready3d();
  await sample('3d-store');
  const store3d = await observe();
  assert.equal(store3d.map_info.name, store2d.map_info.name);
  assert.deepEqual(store3d.map_info.player, store2d.map_info.player,
    'paired store views must retain the authoritative player state');
  assert.equal(store3d.status.screen, store2d.status.screen);
  report.pairedStore = { classic: store2d, modeled: store3d };
  assert.equal(report.samples.at(-1).enabled, 'true');
  assert.ok(report.samples.at(-1).renderer && report.samples.at(-1).largestBuffer > 0,
    'WebGL backend must be active');
  await page.screenshot({ path: output.replace(/\.json$/, '-store.png') });
  await page.evaluate(() => { goldenrodMetrics.active = true; });
  const reach = async (button, map) => {
    for (let i = 0; i < 16; i++) {
      state = await press(button, 8);
      if (state.map_info.name === map) {
        assert.equal(state.status.screen, 'overworld');
        report.transitions.push({ time: Date.now(), map, player: state.map_info.player });
        return;
      }
    }
    throw Error(`Production ${button} input did not reach ${map}`);
  };
  for (let cycle = 1; cycle <= cycles; cycle++) {
    await reach('down', 'GoldenrodCity');
    await page.waitForTimeout(4000);
    await ready3d();
    await sample(`exit-${cycle}`);
    if (cycle === 1 || cycle === cycles) {
      await page.screenshot({ path: output.replace(/\.json$/, `-exit-${cycle}.png`) });
    }
    if (cycle === 1) {
      const pairedBefore = await observe();
      if (!await toggle.isVisible()) await settings.click();
      await toggle.click();
      if (await page.locator('#player-options').evaluate(e => e.open)) await settings.click();
      await page.waitForTimeout(1000);
      await page.screenshot({ path: output.replace(/\.json$/, '-exit-1-2d.png') });
      const pairedAfter = await observe();
      assert.equal(pairedAfter.map_info.name, pairedBefore.map_info.name);
      const tile = state => ({ x: state.map_info.player.x, y: state.map_info.player.y,
        facing: state.map_info.player.facing });
      assert.deepEqual(tile(pairedAfter), tile(pairedBefore),
        'paired visual review must retain the authoritative player state');
      report.pairedCity = pairedAfter;
      if (!await toggle.isVisible()) await settings.click();
      await toggle.click();
      if (await page.locator('#player-options').evaluate(e => e.open)) await settings.click();
      await page.locator('canvas').focus();
      await page.waitForTimeout(4000);
      await ready3d();
      await sample('paired-city-restored-3d');
    }
    const before = await observe();
    state = await press('down', 8);
    assert.equal(state.map_info.name, 'GoldenrodCity');
    assert.ok(state.map_info.player.x !== before.map_info.player.x ||
      state.map_info.player.y !== before.map_info.player.y, 'authoritative tiles must move after exit');
    // Wait for the production movement animation to release input.
    for (let settle = 0; settle < 120 && state.flow_state?.animating; settle++) {
      await page.waitForTimeout(100);
      state = await observe();
    }
    assert.equal(state.flow_state?.animating, false, 'movement must release input before Start');
    state = await press('start');
    report.lastMenuObservation = state;
    assert.ok((state.observe.menus ?? []).length > 0 || /POK.MON|PACK|SAVE/.test(JSON.stringify(state.observe)),
      'production Start menu must be visible');
    await press('b');
    if (!await page.locator('#rotate-right').isVisible()) await settings.click();
    for (let i = 0; i < 4; i++) await page.locator('#rotate-right').click();
    if (await page.locator('#player-options').evaluate(e => e.open)) await settings.click();
    await page.locator('canvas').focus();
    await page.waitForTimeout(2000);
    await sample(`camera-${cycle}`);
    await reach('up', 'GoldenrodDeptStore1F');
    await page.waitForTimeout(2000);
    await ready3d();
    await sample(`entry-${cycle}`);
  }
  report.final = await observe();
  const timing = await page.evaluate(() => {
    goldenrodMetrics.active = false;
    return { intervals: goldenrodMetrics.intervals, longTasks: goldenrodMetrics.longTasks };
  });
  report.timing = timing;
  const sorted = [...timing.intervals].sort((a, b) => a - b);
  report.intervalSummary = { count: sorted.length, median: sorted[Math.floor(sorted.length / 2)],
    p95: sorted[Math.floor(sorted.length * .95)], max: sorted.at(-1) };
  await page.screenshot({ path: output.replace(/\.json$/, '.png') });
  report.heapHighWater = Math.max(...report.samples.map(s => s.heapBytes));
  report.largestBuffer = Math.max(...report.samples.map(s => s.largestBuffer));
  // Preserve every transition before enforcing the existing browser budgets.
  assert.ok(report.heapHighWater <= report.budgets.wasmBytes, 'WASM memory budget exceeded');
  assert.ok(report.largestBuffer <= report.budgets.singleBufferBytes, 'single GPU buffer exceeds 32 MiB');
  report.passed = true;
} catch (error) {
  report.failure = String(error);
  try {
    report.failureSnapshot = await bounded(page.evaluate(() => ({
      heapBytes: window.goldenrodRaw?.memory.buffer.byteLength ?? null,
      largestBuffer: window.goldenrodMetrics?.largestBuffer ?? null,
      contextLost: window.goldenrodMetrics?.contextLost ?? null,
      visible: !document.hidden,
      focused: document.hasFocus(),
      activeElement: document.activeElement?.tagName,
      startupError: document.querySelector('#startup-error')?.open
        ? document.querySelector('#startup-error-message')?.textContent : null,
    })), 5000);
    await bounded(page.screenshot({ path: output.replace(/\.json$/, '-failure.png') }), 5000);
  } catch (snapshotError) { report.failureSnapshotError = String(snapshotError); }
  process.exitCode = 1;
} finally {
  clearInterval(sampler);
  report.heapHighWater = Math.max(0, ...report.samples.map(s => s.heapBytes), report.failureSnapshot?.heapBytes ?? 0);
  report.largestBuffer = Math.max(0, ...report.samples.map(s => s.largestBuffer), report.failureSnapshot?.largestBuffer ?? 0);
  report.originalBudgetsPassed = report.passed === true &&
    report.heapHighWater <= report.budgets.originalWasmBytes;
  report.processMemoryMeaning = 'Sum across sampled browser process IDs; working sets may count shared pages more than once; excludes VRAM';
  report.processMemoryHighWater = {
    workingSetBytes: Math.max(0, ...report.processSamples.map(s => s.processes.reduce((n, p) => n + p.WorkingSet64, 0))),
    privateBytes: Math.max(0, ...report.processSamples.map(s => s.processes.reduce((n, p) => n + p.PrivateMemorySize64, 0))),
  };
  await fs.writeFile(output, JSON.stringify(report, null, 2) + '\n');
  await browser?.close();
}
