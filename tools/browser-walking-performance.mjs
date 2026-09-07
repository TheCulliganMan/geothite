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
const browser = await chromium.launch({ headless: false });
try {
  const context = await browser.newContext({ viewport: { width: 1280, height: 900 }, storageState: storage });
  await context.addInitScript(() => {
    window.walkingMetrics = { active: false, last: 0, frames: [], uploads: [], allocations: 0, longTasks: [] };
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
  const errors = [];
  page.on('pageerror', e => { errors.push(e.message); console.error(e.message); });
  await page.goto(url);
  await page.waitForFunction(() => document.querySelector('#loading').hidden
    || document.querySelector('#startup-error').open, null, { timeout: 180000 });
  assert.equal(await page.locator('#startup-error').evaluate(e => e.open ? e.textContent : null), null);
  await page.evaluate(async synchronousAudio => {
    const wasm = await import('./crystal-bevy.js');
    const { createGameBridge } = await import('./webmcp.js');
    window.walkingBridge = createGameBridge(wasm);
    if (synchronousAudio) {
      // Experimental control only: execute the SAME synthesizer on the UI thread.
      const synth = await import('./audio-runtime/browser-synth.js');
      const { default: context } = await import('./audio-runtime/context.js');
      synth.initializeCrystalAudioSynth(context);
      globalThis.__crystalPollMidi = synth.synthesizeCrystalMidi;
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
  const observe = () => page.evaluate(() => walkingBridge.execute({ kind: 'observe' }));
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
  assert.equal(state.map_info.name, scenario === 'lab-exit' ? 'ElmsLab' : 'NewBarkTown', 'unexpected fixture map');
  assert.equal(state.status.screen, 'overworld');
  const toggle = page.locator('#view-toggle');
  if ((await toggle.getAttribute('aria-pressed') === 'true') !== (mode === '2.5d')) {
    const settings = page.locator('#player-options > summary');
    if (!await toggle.isVisible()) await settings.click();
    await toggle.click();
    if (await settings.isVisible()) await settings.click();
  }
  assert.equal(await toggle.getAttribute('aria-pressed'), String(mode === '2.5d'));
  await page.waitForTimeout(5000);
  await page.locator('canvas').focus();
  state = await observe();
  const loadBefore = os.loadavg();
  await page.evaluate(() => { walkingMetrics.active = true; });
  let midpoint = null;
  let leg = 0;
  for (const key of (scenario === 'lab-exit' ? ['ArrowDown', 'ArrowDown', 'ArrowDown'] : scenario === 'route-exit' ? Array(3).fill('ArrowLeft') : ['ArrowRight', 'ArrowLeft', 'ArrowRight', 'ArrowLeft', 'ArrowRight', 'ArrowLeft'])) {
    await page.keyboard.down(key);
    await page.waitForTimeout(1000);
    await page.keyboard.up(key);
    if (++leg === 3 && scenario === 'walking') {
      await page.evaluate(() => { walkingMetrics.active = false; walkingMetrics.last = 0; });
      midpoint = await observe();
      await page.evaluate(() => { walkingMetrics.active = true; });
    }
  }
  const metrics = await page.evaluate(() => {
    walkingMetrics.active = false;
    return { ...walkingMetrics, audioPreparation, visible: !document.hidden, focused: document.hasFocus() };
  });
  const after = await observe();
  const times = [...metrics.frames].sort((a, b) => a - b);
  const report = { url, mode, scenario, audioControl: process.env.AUDIO_PREPARATION ?? 'default', loadBefore, loadAfter: os.loadavg(), platform: os.platform(), arch: os.arch(), browser: browser.version(), before: state.map_info, after: after.map_info, midpoint: midpoint?.map_info,
    error: after.recent_events.error, errors, ...metrics,
    median: times[Math.floor(times.length / 2)], p95: times[Math.floor(times.length * .95)],
    max: times.at(-1), uploadBytes: metrics.uploads.reduce((n, p) => n + p.width * p.height * 4, 0) };
  await fs.writeFile(output, JSON.stringify(report, null, 2));
  await page.screenshot({ path: output.replace(/\.json$/, '.png') });
  assert.deepEqual(errors, []);
  if (scenario === 'lab-exit') assert.equal(after.map_info.name, 'NewBarkTown');
  else if (scenario === 'route-exit') assert.equal(after.map_info.name, 'Route29');
  else assert.ok([midpoint, after].some(sample => sample &&
    (sample.map_info.player.x !== state.map_info.player.x || sample.map_info.player.y !== state.map_info.player.y)),
    'the player must actually move, even if the return walk ends at its starting tile');
  assert.equal(after.recent_events.error, null);
  assert.ok(metrics.visible && metrics.focused, 'measure only a visible, focused game');
  console.log(JSON.stringify({ mode, frames: times.length, median: report.median,
    p95: report.p95, max: report.max, allocations: metrics.allocations, uploadBytes: report.uploadBytes }));
  if (process.env.REQUIRE_RETAINED_TEXTURES === '1') {
    assert.equal(metrics.allocations, 0, 'walking must retain composed GPU texture allocations');
  }
} finally { await browser.close(); }
