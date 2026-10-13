/** Real browser input, frame intervals and GPU upload accounting. No gameplay shortcuts. */
import { chromium } from 'playwright';
import fs from 'node:fs/promises';
import assert from 'node:assert/strict';
import os from 'node:os';
const [url, storage, mode, output] = process.argv.slice(2);
assert.ok(url && storage && ['2d', '2.5d'].includes(mode) && output,
  'usage: node tools/browser-walking-performance.mjs URL STORAGE_JSON 2d|2.5d OUTPUT_JSON');
const scenario = process.env.SCENARIO ?? 'walking';
assert.ok(['walking', 'lab-exit', 'route-exit'].includes(scenario));
const browserServer = await chromium.launchServer({ headless: false, args: ['--mute-audio'],
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
  await context.addInitScript(() => {
    localStorage.setItem('crystal.audio.muted', 'true');
    window.walkingMetrics = { active: false, last: 0, frames: [], uploads: [], allocations: 0, longTasks: [],
      drawCalls: 0, bindBufferRanges: 0, textureParameterCalls: 0, bufferUploadBytes: 0 };
    function frame(t) {
      if (walkingMetrics.active && walkingMetrics.last) walkingMetrics.frames.push(t - walkingMetrics.last);
      walkingMetrics.last = walkingMetrics.active ? t : 0;
      requestAnimationFrame(frame);
    }
    requestAnimationFrame(frame);
    new PerformanceObserver(list => {
      if (walkingMetrics.active) walkingMetrics.longTasks.push(...list.getEntries().map(e => e.duration));
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
  await page.goto(url);
  await page.waitForFunction(() => document.querySelector('#loading').hidden
    || document.querySelector('#startup-error').open, null, { timeout: 180000 });
  assert.equal(await page.locator('#startup-error').evaluate(e => e.open ? e.textContent : null), null);
  await page.evaluate(async synchronousAudio => {
    const scripts = [...document.scripts].map(script => script.textContent).join('\n');
    const bundle = scripts.match(/import\('(\.\/crystal-bevy(?:-[a-f0-9]{64})?\.js)'\)/)?.[1];
    if (!bundle) throw new Error('Missing compiled bundle import');
    const wasm = await import(bundle);
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
  assert.equal(state.map_info.name, preview === 'route29' ? 'Route29' : scenario === 'lab-exit' ? 'ElmsLab' : 'NewBarkTown', 'unexpected fixture map');
  assert.equal(state.status.screen, 'overworld');
  const toggle = page.locator('#view-toggle');
  if ((await toggle.getAttribute('aria-pressed') === 'true') !== (mode === '2.5d')) {
    const settings = page.locator('#player-options > summary');
    if (!await toggle.isVisible()) await settings.click();
    await toggle.click();
    if (await settings.isVisible()) await settings.click();
  }
  assert.equal(await toggle.getAttribute('aria-pressed'), String(mode === '2.5d'));
  await page.bringToFront();
  await page.waitForTimeout(5000);
  await page.locator('canvas').focus();
  state = await observe();
  if (mode === '2.5d') {
    const deadline = Date.now() + 180000;
    while (!state.presentation?.voxel?.active || state.presentation.voxel.profiles_pending
        || state.presentation.voxel.terrain_pending === true
        || ('terrain_map' in state.presentation.voxel && state.presentation.voxel.terrain_map !== state.map_info.name)
        || state.presentation.voxel.active_frames < 30) {
      assert.ok(Date.now() < deadline, '3D warmup timed out');
      await page.waitForTimeout(250); state = await observe();
    }
    assert.equal(state.presentation.voxel.scenery_full_detail, true, 'preserve full-detail models');
  }
  const loadBefore = os.loadavg();
  await page.evaluate(() => { walkingMetrics.active = true; });
  let midpoint = null;
  let leg = 0;
  for (const key of (scenario === 'lab-exit' ? ['ArrowDown', 'ArrowDown', 'ArrowDown'] : scenario === 'route-exit' ? Array(3).fill('ArrowLeft') : ['ArrowRight', 'ArrowLeft', 'ArrowRight', 'ArrowLeft', 'ArrowRight', 'ArrowLeft'])) {
    await page.keyboard.down(key);
    await page.waitForTimeout(1000);
    await page.keyboard.up(key);
    ++leg;
    if (scenario === 'walking') {
      // Every leg is observable. Two endpoints can alias after a round trip.
      // Keep observation overhead outside the walking interval sample.
      await page.evaluate(() => { walkingMetrics.active = false; walkingMetrics.last = 0; });
      const legState = await observe();
      report.walkObservations ??= [];
      report.walkObservations.push({ leg, key, map: legState.map_info, flow: legState.flow_state });
      if (leg === 3) midpoint = legState;
      await page.evaluate(() => { walkingMetrics.active = true; });
    }
  }
  const metrics = await page.evaluate(() => {
    walkingMetrics.active = false;
    return { ...walkingMetrics, audioPreparation, visible: !document.hidden, focused: document.hasFocus() };
  });
  const after = await observe();
  const times = [...metrics.frames].sort((a, b) => a - b);
  report = { ...report, url, mode, scenario, audioControl: process.env.AUDIO_PREPARATION ?? 'default', loadBefore, loadAfter: os.loadavg(), platform: os.platform(), arch: os.arch(), browser: browser.version(), before: state.map_info, after: after.map_info, midpoint: midpoint?.map_info,
    error: after.recent_events.error, errors, rendererBefore: state.presentation?.voxel,
    rendererAfter: after.presentation?.voxel,
    timingMeaning: 'visible focused requestAnimationFrame intervals; not GPU timestamps or presented FPS', ...metrics,
    median: times[Math.floor(times.length / 2)], p95: times[Math.floor(times.length * .95)],
    max: times.at(-1), uploadBytes: metrics.uploads.reduce((n, p) => n + p.width * p.height * 4, 0) };
  await fs.writeFile(output, JSON.stringify(report, null, 2));
  await page.screenshot({ path: output.replace(/\.json$/, '.png') });
  assert.deepEqual(errors, []);
  if (scenario === 'lab-exit') assert.equal(after.map_info.name, 'NewBarkTown');
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
