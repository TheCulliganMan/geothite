/** Offscreen production route/battle handoff regression. Gameplay uses the real bridge. */
import { chromium } from 'playwright';
import fs from 'node:fs/promises';
import assert from 'node:assert/strict';
const [url, out, mode = 'battle'] = process.argv.slice(2);
assert.ok(url && out && ['battle','battle-cycle','crossing','cold-crossing','reference'].includes(mode),
 'usage: node tools/browser-transition-check.mjs URL OUTPUT_DIR [battle|battle-cycle|crossing|cold-crossing|reference]');
assert.equal(new URL(url).searchParams.get('multiplayer'), 'off', 'use a disposable local preview');
assert.equal(new URL(url).searchParams.get('loading-trace'), 'on', 'per-frame state evidence is required');
const crossing = mode === 'crossing' || mode === 'cold-crossing';
const reference = mode === 'reference';
await fs.mkdir(out, { recursive: true });
const server = await chromium.launchServer({ headless: true, args: ['--mute-audio','--enable-gpu'] });
const browser = await chromium.connect(server.wsEndpoint());
const report = { url, errors: [], browser: browser.version(), meaning: 'App presentation-state trace and requestAnimationFrame intervals, not displayed FPS' };
let page;
let context;
const timeout = setTimeout(() => { void fs.writeFile(`${out}/timeout.json`, JSON.stringify(report)).finally(() => server.kill()); }, 360000);
try {
 context = await browser.newContext({ viewport: { width: 1280, height: 800 },
  ...(process.env.RECORD_TRANSITION_VIDEO === '1'
    ? { recordVideo: { dir: `${out}/video`, size: { width: 1280, height: 800 } } } : {}) });
 await context.addInitScript(() => {
  localStorage.setItem('crystal.audio.preference', 'muted');
  window.entryIntervals = []; window.entryMeasuring = false;
  window.entryGpu = { lost: [], errors: [], shaderErrors: [] };
  if (typeof GPUAdapter !== 'undefined') {
   const createModule = GPUDevice.prototype.createShaderModule;
   GPUDevice.prototype.createShaderModule = function (descriptor) {
    const module = createModule.call(this, descriptor);
    module.getCompilationInfo().then(info => {
     for (const m of info.messages) if (m.type === 'error') entryGpu.shaderErrors.push({
      message: m.message, line: m.lineNum, column: m.linePos, code: String(descriptor.code) });
    });
    return module;
   };
   const request = GPUAdapter.prototype.requestDevice;
   GPUAdapter.prototype.requestDevice = async function (...args) {
    const device = await request.apply(this, args);
    device.lost.then(info => entryGpu.lost.push({ reason: info.reason, message: info.message }));
    device.addEventListener('uncapturederror', e => entryGpu.errors.push({ type: e.error.constructor.name, message: e.error.message }));
    return device;
   };
  }
  let previous;
  const tick = t => { if (window.entryMeasuring && previous) window.entryIntervals.push(t - previous); previous = window.entryMeasuring ? t : null; requestAnimationFrame(tick); }; requestAnimationFrame(tick);
 });
 const videoStarted = Date.now();
 page = await context.newPage();
 page.on('pageerror', error => report.errors.push(String(error)));
 page.on('console', msg => { if (msg.type() === 'error') report.errors.push(msg.text()); if (['warn','error'].includes(msg.type())) { report.console ??= []; if (report.console.length < 100) report.console.push({ type:msg.type(), text:msg.text() }); } });
 await page.goto(url);
 await page.waitForFunction(() => document.querySelector('#loading').hidden || document.querySelector('#startup-error').open, null, { timeout: 180000 });
 assert.equal(await page.locator('#startup-error').evaluate(e => e.open ? e.textContent : null), null);
 await page.evaluate(async () => {
  const bundle = window.__crystalRendererBundle?.module ?? './crystal-bevy.js';
  window.entryWasm = await import(bundle); await entryWasm.default();
  const { createGameBridge } = await import('./webmcp.js');
  window.entryBridge = createGameBridge(entryWasm, { timeoutMs: 30000 });
 });
 const observe = () => page.evaluate(() => entryBridge.execute({ kind: 'observe' }));
 let state;
 const deadline = Date.now() + 180000;
 do {
  state = await observe();
  const workers = await page.evaluate(() => ({ terrain: globalThis.__crystalTerrainWorkerMetrics,
   actors: globalThis.__crystalActorWorkerMetrics }));
  assert.ok(Object.values(workers).every(m => !m || ['fallback','initFailures','failed'].every(key => m[key] === 0)),
   `worker warmup failed: ${JSON.stringify(workers)}`);
  const workersReady = Object.values(workers).length === 2 && Object.values(workers).every(m => m?.completed > 0);
  const voxel = state.presentation?.voxel;
  if (mode === 'cold-crossing'
    ? voxel?.active && !voxel.profiles_pending && voxel.active_frames >= 2
    : voxel?.active && !voxel.terrain_pending && !voxel.profiles_pending && voxel.active_frames >= 30 && workersReady) break;
  assert.ok(Date.now() < deadline, '3D terrain warmup timed out');
  await page.waitForTimeout(250);
 } while (true);
 await page.bringToFront();
 await page.locator('canvas').focus();
 // A trusted gesture starts the muted source-audio clock too. Without it,
 // authored WaitSFX (including the badge fanfare) can wait for audio unlock.
 await page.keyboard.press('ControlLeft');
 report.audioControl = 'Muted browser output and game gain; trusted modifier gesture unlocks source WaitSFX timing.';
 report.before = state;
 console.log('phase: field ready');
 report.renderer = await page.evaluate(async () => ({ bundle: window.__crystalRendererBundle, adapter: await navigator.gpu?.requestAdapter().then(a => a ? { vendor: a.info.vendor, architecture: a.info.architecture, device: a.info.device, description: a.info.description, fallback: a.isFallbackAdapter } : null) }));
 if (process.env.REQUIRE_WEBGPU === '1') assert.match(report.renderer.bundle?.binary ?? '', /webgpu/, 'this QA run requires the WebGPU renderer bundle');
 await page.screenshot({ path: `${out}/before.png` });
 if (process.env.RECORD_TRANSITION_VIDEO === '1') {
  await page.evaluate(() => {
   const marker = document.createElement('div'); marker.id = 'qa-video-marker';
   marker.style.cssText = 'position:fixed;left:0;top:0;width:16px;height:16px;background:rgb(0,255,0);z-index:2147483647';
   document.body.append(marker);
  });
  await page.waitForTimeout(500);
  await page.evaluate(() => document.querySelector('#qa-video-marker').remove());
  report.videoAlignment = 'The last green marker frame immediately precedes trace start; use video PTS rather than wall time under load.';
 }
 report.videoTraceOffsetSeconds = (Date.now() - videoStarted) / 1000;
 const traceStarted = await page.evaluate(() => {
  const now = performance.now(); entryWasm.crystal_loading_trace_start(); entryMeasuring = true; return now;
 });
 let traceEpoch = traceStarted;
 const traceChunks = [];
 const collectTrace = async () => {
  const slice = await page.evaluate(() => {
   const trace = JSON.parse(entryWasm.crystal_loading_trace_read());
   const next = performance.now(); entryWasm.crystal_loading_trace_start(); return { trace, next };
  });
  const offset = (traceEpoch - traceStarted) * 1000;
  traceChunks.push({ ...slice.trace, traceEvents: slice.trace.traceEvents
   .filter(event => !event.inProgress).map(event => ({ ...event, ts: event.ts + offset })) });
  traceEpoch = slice.next;
 };
 if (reference) {
  const menu = await page.evaluate(() => entryBridge.execute({ kind: 'press', button: 'start', frames: 8 }));
  assert.ok(menu.observe.menus.length > 0, 'the production Start menu must own reference input');
 } else if (crossing) {
  const button = new URL(url).searchParams.get('preview') === 'route26-border' ? 'left' : 'right';
  report.steps = [];
  for (let i = 0; i < 4; i++) {
   const started = Date.now();
   const result = await page.evaluate(button => entryBridge.execute({ kind: 'press', button, frames: 8 }), button);
   report.steps.push({ wallMs: Date.now() - started, ...result });
  }
 } else {
  const trainer = new URL(url).searchParams.get('preview') === 'falkner-battle';
  report.encounterInputs = [];
  if (trainer) await page.evaluate(() => entryBridge.execute({ kind: 'press', button: 'right', frames: 8 }));
  for (let i = 0; i < 64; i++) {
   const button = trainer ? 'a' : i % 2 === 0 ? 'right' : 'left';
   console.log(`phase: encounter input ${i + 1} ${button}`);
   const next = await page.evaluate(button => entryBridge.execute({ kind: 'press', button, frames: 8 }), button);
   report.encounterInputs.push({ button, status: next.status, map: next.map_info });
   if (next.status.screen === 'battle') break;
  }
  assert.equal((await observe()).status.screen, 'battle', 'ordinary inputs must trigger the production encounter');
 }
 if (mode === 'battle-cycle') {
  report.cycle = [];
  let openingStatusUsed = false;
  for (let i = 0; i < 600; i++) {
   const current = await observe();
   report.cycle.push({ status: current.status, battle: current.reward_state?.battle,
    menus: current.observe.menus, text: current.observe.battle_message,
    dialogue: current.observe.visible_dialogue,
    view: current.presentation?.battle_view });
   if (i % 25 === 0) {
    console.log(`phase: cycle ${i} ${current.status.screen} ${JSON.stringify(current.observe.visible_dialogue ?? current.observe.battle_message ?? '')}`);
    await fs.writeFile(`${out}/cycle-progress.json`, JSON.stringify(report.cycle));
    await collectTrace();
   }
   if (current.status.screen !== 'battle' && !current.presentation?.battle_view?.active
     && !current.observe.visible_dialogue && !current.flow_state.animating) break;
   if (process.env.BATTLE_COMPLETE !== '1' && /fainted/i.test(current.observe.battle_message ?? '')) break;
   if (current.flow_state.animating) { await page.waitForTimeout(100); continue; }
   if (current.status.screen !== 'battle') {
    // Complete authored field dialogue with ordinary browser keyboard input,
    // including its real key-up edge, before taking a stable reference pair.
    await page.locator('canvas').focus();
    await page.keyboard.press('KeyZ', { delay: 80 });
    await page.waitForTimeout(150);
    continue;
   }
   const entries = current.observe.menus.find(menu => menu.kind === 'battle')?.entries ?? [];
   const opening = !openingStatusUsed ? entries.findIndex(entry => /\bLEER\b/.test(entry)) : -1;
   const attack = opening >= 0 ? opening : entries.findIndex(entry => /EMBER|WATER.GUN|TACKLE|SCRATCH/.test(entry));
   const cursor = entries.findIndex(entry => entry.startsWith('>'));
   const button = attack >= 0 && cursor >= 0 && attack !== cursor ? 'down' : 'a';
   if (opening >= 0 && cursor === opening && button === 'a') { openingStatusUsed = true; report.openingStatusMove = 'LEER'; }
   await page.evaluate(button => entryBridge.execute({ kind: 'press', button, frames: 8 }), button);
  }
  await fs.writeFile(`${out}/cycle.json`, JSON.stringify(report.cycle, null, 2));
  assert.ok(report.cycle.some(s => s.status.party.some((p, i) =>
   JSON.stringify(p.moves) !== JSON.stringify(report.cycle[0].status.party[i]?.moves))),
   'battle cycle must spend PP through ordinary inputs');
 }
 await page.waitForTimeout(reference ? 1000 : crossing ? 5000 : 20000);
 report.gpu = await page.evaluate(() => entryGpu);
 for (let i = 0; i < report.gpu.shaderErrors.length; i++) {
  const shader = report.gpu.shaderErrors[i];
  await fs.writeFile(`${out}/shader-error-${i}.wgsl`, shader.code);
  shader.path = `${out}/shader-error-${i}.wgsl`; delete shader.code;
 }
 report.workers = await page.evaluate(() => ({ terrain: globalThis.__crystalTerrainWorkerMetrics,
  actors: globalThis.__crystalActorWorkerMetrics }));
 report.workerPolicyHealthy = Object.values(report.workers).length === 2
  && Object.values(report.workers).every(m => m && m.completed > 0
   && ['fallback', 'initFailures', 'failed'].every(key => m[key] === 0));
 await collectTrace();
 const trace = { traceEvents: traceChunks.flatMap(chunk => chunk.traceEvents),
  droppedEvents: traceChunks.reduce((sum, chunk) => sum + chunk.droppedEvents, 0), displayTimeUnit: 'ms' };
 report.traceCollection = 'Complete trace epochs rebased with performance.now; periodic collection can delay rAF and is not an FPS benchmark.';
 const intervals = await page.evaluate(() => { entryMeasuring = false; return entryIntervals; });
 await fs.writeFile(`${out}/trace.json`, JSON.stringify(trace));
 const rows = trace.traceEvents.filter(e => e.name === 'presentation.frame' && !e.inProgress);
 const entries = rows.filter(e => e.args.entry_fx_active);
 const battles = rows.filter(e => e.args.battle_frame_active && !e.args.battle_preparing);
 const gaps = rows.filter(e => !e.args.world_3d_active && !e.args.battle_3d_active);
 const rendererSequence = rows.map(e => e.args.battle_3d_active ? 'battle' : e.args.world_3d_active ? 'world' : 'classic')
  .filter((value, index, all) => index === 0 || value !== all[index - 1]);
 report.rendererSequence = rendererSequence;
 report.faintRows = rows.filter(e => e.args.battle_cues?.some(c => c.startsWith('Faint:'))).length;
 report.enemyMoveCues = [...new Set(rows.flatMap(e => e.args.battle_cues ?? [])
  .filter(cue => cue.startsWith('Move:') && cue.endsWith(':Enemy')))];
 report.enemyTurnVerified = report.cycle?.some(state => state.battle?.enemy_turns > 0) ?? false;
 report.playerDamage = report.cycle?.length ? Math.max(0,
  report.cycle[0].status.party[0].hp - Math.min(...report.cycle.map(state => state.status.party[0].hp))) : 0;
 const ordered = [...intervals].sort((a,b) => a-b);
 report.presentation = { rows: rows.length, entryRows: entries.length, entryNot3d: entries.filter(e => !e.args.world_3d_active).length,
  battleRows: battles.length, battleNot3d: battles.filter(e => !e.args.battle_3d_active).length,
  neither3d: gaps.length, connectedRetentionRows: rows.filter(e => e.args.connected_terrain).length, firstGaps: gaps.slice(0,20), traceDropped: trace.droppedEvents };
 report.intervals = { count: ordered.length, median: ordered[Math.floor(ordered.length*.5)], p95: ordered[Math.floor(ordered.length*.95)], maximum: ordered.at(-1) };
 await page.screenshot({ path: `${out}/after-3d.png` });
 console.log('phase: entry trace captured');
 await fs.writeFile(`${out}/checkpoint.json`, JSON.stringify(report,null,2));
 assert.deepEqual(report.gpu.errors, [], 'transition must not cause GPU validation errors');
 assert.deepEqual(report.gpu.shaderErrors, [], 'transition shaders must compile');
 assert.deepEqual(report.gpu.lost, [], 'transition must not lose its GPU device');
 report.after = await observe();
 if (mode === 'battle-cycle') {
  if (report.openingStatusMove) assert.ok(report.enemyTurnVerified && report.playerDamage > 0,
   'the opening status move must allow an authoritative damaging enemy turn');
  assert.ok(report.faintRows > 0, 'the cycle must actually render a source faint');
  assert.equal(gaps.length, 0, 'moves, faints and result narration must retain 3D');
  if (process.env.BATTLE_COMPLETE === '1') {
   assert.equal(report.after.status.screen, 'overworld', 'the full cycle must finish the battle');
   assert.equal(report.after.presentation?.battle_view?.active, false,
    'result dialogue must finish and release the retained battle camera');
   assert.ok(!report.after.observe.visible_dialogue, 'finish post-battle script pages before comparing views');
  }
 }
 const toggle = page.locator('#view-toggle');
 const options = page.locator('#player-options');
 if (!await toggle.isVisible()) await options.locator('summary').click();
 if (await toggle.getAttribute('aria-pressed') === 'true') await toggle.click();
 if (await options.getAttribute('open') !== null) await options.locator('summary').click();
 await page.waitForTimeout(500);
 report.classic = await observe();
 report.classicPresentation = await page.evaluate(() => JSON.parse(entryWasm.crystal_loading_trace_read()).traceEvents.filter(e => e.name === 'presentation.frame' && !e.inProgress).at(-1)?.args);
 assert.ok(report.classicPresentation && !report.classicPresentation.world_3d_active && !report.classicPresentation.battle_3d_active, 'the production view toggle must expose the classic renderer');
 await page.screenshot({ path: `${out}/after-2d.png` });
 const pairState = state => ({ map: state.map_info, status: state.status, flow: state.flow_state,
  text: state.observe.text, menus: state.observe.menus });
 report.pairPreserved = JSON.stringify(pairState(report.after)) === JSON.stringify(pairState(report.classic));
 report.passed = report.pairPreserved && report.workerPolicyHealthy && report.gpu.lost.length === 0 && report.gpu.errors.length === 0 && report.gpu.shaderErrors.length === 0 && (reference
  ? rows.length > 0 && gaps.length === 0 && report.after.observe.menus.length > 0 && report.after.map_info.name === report.before.map_info.name
  : crossing
  ? report.after.map_info.name === (new URL(url).searchParams.get('preview') === 'route26-border' ? 'Route27' : 'NewBarkTown') && gaps.length === 0 && rows.every(e => !e.args.terrain_pending
      || (e.args.world_3d_active && (e.args.connected_terrain || e.args.terrain_map === e.args.map)))
  : entries.length > 0 && battles.length > 0 && report.presentation.entryNot3d === 0 && report.presentation.battleNot3d === 0)
  && report.presentation.traceDropped === 0 && report.errors.length === 0;
 await fs.writeFile(`${out}/report.json`, JSON.stringify(report, null, 2));
 console.log(JSON.stringify({ passed: report.passed, presentation: report.presentation, intervals: report.intervals, errors: report.errors }));
 assert.ok(report.passed, `transition regression failed; inspect ${out}/report.json`);
} catch (error) {
 report.failure = String(error); report.passed = false;
 if (page && !page.isClosed()) {
  try { await page.screenshot({ path: `${out}/failure.png`, timeout: 5000 }); } catch {}
 }
 await fs.writeFile(`${out}/report.json`, JSON.stringify(report, null, 2));
 throw error;
} finally {
 clearTimeout(timeout);
 const video = page?.video();
 try {
  await context?.close();
  if (video) await video.saveAs(`${out}/transition.webm`);
 } finally { await server.kill(); }
}
