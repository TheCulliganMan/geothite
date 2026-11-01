/**
 * Isolated production-game regression, not a renderer-only smoke test.
 *
 * node tools/new-bark-3d-playtest.mjs URL OUTPUT.json [--fixture STORAGE.json|--fresh]
 *
 * Uses /usr/bin/chromium headlessly and real keyboard/WebMCP joypad input only.
 * A URL containing preview=new-bark uses the feature-gated production location
 * preview without injecting a fixture or modifying authoritative state.
 * --fresh plays the original intro, bedroom and Mom scene instead of loading a
 * fixture. Incompatible fixtures FAIL explicitly; they are never rewritten.
 * Output belongs in ignored target/ or outside the checkout, never in fixtures/.
 * Browser timings identify SwiftShader and are not hardware-GPU benchmarks.
 */
import { chromium } from 'playwright';
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import { createHash } from 'node:crypto';

const [url, output, ...args] = process.argv.slice(2);
assert.ok(url && output && /\.json$/i.test(output),
  'usage: node tools/new-bark-3d-playtest.mjs URL OUTPUT.json [--fixture STORAGE.json|--fresh]');
assert.ok(new URL(url).searchParams.get('multiplayer') === 'off', 'Use ?multiplayer=off for isolated QA');
const fresh = args.includes('--fresh');
const preview = new URL(url).searchParams.get('preview') === 'new-bark';
assert.ok(!(fresh && preview), '--fresh needs a normal URL without preview=new-bark');
const fixtureIndex = args.indexOf('--fixture');
assert.ok(!((fresh || preview) && fixtureIndex >= 0), 'Use a fixture only with a normal non-preview URL');
assert.ok(args.every((arg, i) => arg === '--fresh' || arg === '--fixture' || i === fixtureIndex + 1 && fixtureIndex >= 0), 'Unknown argument');
const fixture = fresh || preview ? null : (fixtureIndex >= 0 ? args[fixtureIndex + 1] : new URL('./fixtures/new-bark-saved-game.json', import.meta.url));
assert.ok(fresh || preview || fixture, '--fixture needs a storage-state JSON path');
await fs.mkdir(path.dirname(path.resolve(output)), { recursive: true });
const screenshotPath = label => output.replace(/\.json$/i, `-${label}.png`);
const hash = value => createHash('sha256').update(value).digest('hex');
const report = {
  version: 1, url, mode: preview ? 'production-location-preview' : fresh ? 'real-new-game' : 'unmodified-saved-game',
  startedAt: new Date().toISOString(), status: 'running', checks: [], trace: [], metrics: [], screenshots: [],
  errors: [], warnings: [], networkFailures: [],
  routeProvenance: [
    'crates/crystal-flygon/src/breadcrumbs.rs: bedroom stairs (7,0), home exits (6,7)/(7,7)',
    'crates/crystal-bevy/src/bevy_shell/tests/story_progression.rs: Mom interception and New Bark gate (1,8), return (5,8)',
    'Verified external pack collision/warp metadata: NewBarkTown home door (13,5), exterior (13,6)',
  ],
};
let storage = { cookies: [], origins: [] };
if (fixture) {
  storage = JSON.parse(await fs.readFile(fixture, 'utf8'));
  report.fixture = { path: String(fixture), saves: [] };
  for (const origin of storage.origins) {
    origin.origin = new URL(url).origin;
    for (const item of origin.localStorage.filter(item => item.name.startsWith('crystal.save.') && !item.name.endsWith('.bak'))) {
      const bytes = Buffer.from(item.value, 'base64');
      const info = { key: item.name, bytes: bytes.length, sha256: hash(bytes) };
      // Read only version/identity metadata from the documented fixed-width
      // bincode save envelope. The production loader remains the authority.
      if (bytes.subarray(0, 12).equals(Buffer.from('CRYSTALSAVE\0')) && bytes.length >= 24) {
        info.version = bytes.readUInt16BE(12);
        let cursor = 24;
        const string = () => {
          const count = Number(bytes.readBigUInt64LE(cursor)); cursor += 8;
          assert.ok(count <= 1024 && cursor + count <= bytes.length, 'Invalid fixture identity metadata');
          const value = bytes.toString('utf8', cursor, cursor + count); cursor += count; return value;
        };
        try { info.modpackId = string(); info.modpackHash = string(); info.contentHash = string(); }
        catch (error) { info.metadataError = String(error); }
      }
      report.fixture.saves.push(info);
    }
  }
  assert.ok(report.fixture.saves.length, 'Fixture contains no game saves');
}
const browser = await chromium.launch({
  executablePath: '/usr/bin/chromium', headless: true,
  args: ['--use-gl=angle', '--use-angle=swiftshader', '--enable-unsafe-swiftshader', '--disable-dev-shm-usage'],
});
let page;
let state;
let phase = 'startup';
const DIRECTIONS = [ ['up', 0, -1], ['right', 1, 0], ['down', 0, 1], ['left', -1, 0] ];
const keyOf = p => `${p.x},${p.y}`;
const position = o => ({ map: o.map_info.name, x: o.map_info.player.x, y: o.map_info.player.y });
const samePosition = (a, b) => a.map === b.map && a.x === b.x && a.y === b.y;
const dialogue = o => o.observe?.visible_dialogue || null;
const menus = o => o.observe?.menus || [];
const busy = o => Boolean(dialogue(o) || menus(o).length);
const summarize = o => ({ frame: o.frame, screen: o.status?.screen, ...position(o), facing: o.map_info.player.facing,
  dialogue: Boolean(dialogue(o)), menuKinds: menus(o).map(m => m.kind), animating: Boolean(o.flow_state?.animating), error: o.recent_events?.error ?? null });
const check = (name, detail = {}) => { report.checks.push({ name, status: 'passed', ...detail }); console.log(JSON.stringify({ check: name, ...detail })); };
const terrainByMap = new Map();
const blockedEdges = new Set();
function remember(o) {
  const map = o.map_info.name;
  if (!terrainByMap.has(map)) terrainByMap.set(map, new Map());
  const terrain = o.map_info.terrain;
  for (let dy = 0; dy < (terrain?.rows?.length || 0); dy++) {
    for (let dx = 0; dx < terrain.rows[dy].length; dx++) {
      const tile = terrain.rows[dy][dx];
      if (tile) terrainByMap.get(map).set(`${terrain.origin_x + dx},${terrain.origin_y + dy}`, tile);
    }
  }
}
function warpTile(tile) { return tile && tile.permission >= 0x60 && tile.permission <= 0x7f; }
function planTo(goal, o, { permitGoalWarp = false, allowFrontier = true } = {}) {
  const map = o.map_info.name, start = o.map_info.player, grid = terrainByMap.get(map);
  const occupied = new Set(o.map_info.objects.map(keyOf));
  const queue = [{ x: start.x, y: start.y, route: [] }], seen = new Set([keyOf(start)]);
  let best = null;
  for (let index = 0; index < queue.length; index++) {
    const at = queue[index];
    if (at.x === goal.x && at.y === goal.y) return at.route;
    const distance = Math.abs(at.x - goal.x) + Math.abs(at.y - goal.y);
    if (at.route.length && (!best || distance < best.distance || distance === best.distance && at.route.length < best.route.length)) best = { ...at, distance };
    for (const [button, dx, dy] of DIRECTIONS) {
      const next = { x: at.x + dx, y: at.y + dy }, key = keyOf(next), tile = grid?.get(key);
      if (seen.has(key) || occupied.has(key) || !tile || tile.terrain !== 'Land') continue;
      if (warpTile(tile) && !(permitGoalWarp && next.x === goal.x && next.y === goal.y)) continue;
      if (blockedEdges.has(`${map}:${keyOf(at)}:${button}`)) continue;
      seen.add(key); queue.push({ ...next, route: [...at.route, button] });
    }
  }
  // Walk a discovered frontier toward the target, then obtain more real terrain.
  const startDistance = Math.abs(start.x - goal.x) + Math.abs(start.y - goal.y);
  return allowFrontier && best && best.distance < startDistance ? best.route : null;
}
try {
  report.browser = { version: browser.version(), executable: '/usr/bin/chromium', headless: true, rendering: 'ANGLE SwiftShader (software)' };
  const context = await browser.newContext({ viewport: { width: 1280, height: 900 }, deviceScaleFactor: 1, storageState: storage });
  await context.addInitScript(() => {
    const m = globalThis.__newBarkMetrics = { active: false, last: 0, frames: [], longTasks: [], contextLost: 0,
      bufferDataCalls: 0, bufferSubDataCalls: 0, bufferUploadBytes: 0, largestBuffer: 0, textureAllocations: 0, contexts: [] };
    const contexts = new WeakSet();
    const originalContext = HTMLCanvasElement.prototype.getContext;
    HTMLCanvasElement.prototype.getContext = function (...args) {
      const gl = originalContext.apply(this, args);
      if (gl && /webgl/.test(args[0]) && !contexts.has(gl)) {
        contexts.add(gl);
        const extension = gl.getExtension('WEBGL_debug_renderer_info');
        m.contexts.push({ type: args[0], renderer: gl.getParameter(extension?.UNMASKED_RENDERER_WEBGL ?? gl.RENDERER), vendor: gl.getParameter(extension?.UNMASKED_VENDOR_WEBGL ?? gl.VENDOR) });
      }
      return gl;
    };
    for (const prototype of [globalThis.WebGLRenderingContext?.prototype, globalThis.WebGL2RenderingContext?.prototype].filter(Boolean)) {
      for (const name of ['bufferData', 'bufferSubData', 'createTexture']) {
        const original = prototype[name];
        prototype[name] = function (...args) {
          if (name === 'createTexture') m.textureAllocations++;
          else {
            const data = args[name === 'bufferData' ? 1 : 2];
            const offset = args[name === 'bufferData' ? 3 : 3];
            const length = args[name === 'bufferData' ? 4 : 4];
            const bytes = typeof data === 'number' ? data : typeof length === 'number' && length > 0 ? length * (data?.BYTES_PER_ELEMENT || 1) : Math.max(0, (data?.byteLength || 0) - (offset || 0) * (data?.BYTES_PER_ELEMENT || 1));
            m[`${name}Calls`]++; m.bufferUploadBytes += bytes;
            if (name === 'bufferData') m.largestBuffer = Math.max(m.largestBuffer, bytes);
          }
          return original.apply(this, args);
        };
      }
    }
    document.addEventListener('webglcontextlost', () => m.contextLost++, true);
    const frame = time => { if (m.active && m.last) m.frames.push(time - m.last); m.last = m.active ? time : 0; requestAnimationFrame(frame); };
    requestAnimationFrame(frame);
    if (PerformanceObserver.supportedEntryTypes.includes('longtask')) new PerformanceObserver(list => {
      if (m.active) m.longTasks.push(...list.getEntries().map(e => e.duration));
    }).observe({ type: 'longtask' });
  });
  page = await context.newPage();
  page.on('crash', () => report.errors.push({ phase, type: 'crash', message: 'Browser page process crashed' }));
  page.on('pageerror', error => report.errors.push({ phase, type: 'pageerror', message: error.message }));
  page.on('console', message => {
    if (message.type() === 'error') report.errors.push({ phase, type: 'console', message: message.text() });
    else if (message.type() === 'warning' && report.warnings.length < 100) report.warnings.push({ phase, message: message.text() });
  });
  page.on('requestfailed', request => report.networkFailures.push({ phase, url: request.url(), error: request.failure()?.errorText }));
  await page.goto(url, { waitUntil: 'domcontentloaded', timeout: 180000 });
  await page.waitForFunction(() => document.querySelector('#loading')?.hidden || document.querySelector('#startup-error')?.open, null, { timeout: 180000 });
  const startupError = await page.locator('#startup-error').evaluate(e => e.open ? e.textContent : null);
  assert.equal(startupError, null, `Production startup failed${fixture ? '; fixture may be incompatible with this build/pack' : ''}: ${startupError}`);
  report.bundle = await page.evaluate(async () => {
    const scripts = [...document.scripts].map(s => s.textContent).join('\n');
    const bundle = scripts.match(/import\(['"](.\/crystal-bevy(?:-[a-f0-9]{64})?\.js)['"]\)/)?.[1];
    if (!bundle) throw new Error('Missing production WASM import');
    const wasm = await import(bundle);
    const raw = await wasm.default(); // wasm-bindgen returns the already initialized instance.
    const { createGameBridge } = await import('./webmcp.js');
    globalThis.__newBarkQA = { bridge: createGameBridge(wasm, { timeoutMs: 45000 }), raw };
    return { bundle, packs: performance.getEntriesByType('resource').map(r => r.name).filter(url => /\.crystalpack(?:\?|$)/.test(url)) };
  });
  const observe = async () => { state = await page.evaluate(() => __newBarkQA.bridge.execute({ kind: 'observe' })); remember(state); return state; };
  const settle = async () => {
    for (let i = 0; state.flow_state?.animating && !busy(state) && i < 180; i++) { await page.waitForTimeout(50); await observe(); }
    assert.ok(!state.flow_state?.animating || busy(state), 'Production input did not settle within 9 seconds');
    assert.equal(state.recent_events?.error ?? null, null, 'Game reported an input/runtime error');
    return state;
  };
  const press = async (button, label = button) => {
    state = await page.evaluate(button => __newBarkQA.bridge.execute({ kind: 'press', button, frames: 1 }), button);
    remember(state); await settle(); report.trace.push({ phase, input: label, ...summarize(state) }); return state;
  };
  const capture = async label => {
    const file = screenshotPath(label); await page.screenshot({ path: file, timeout: 30000 });
    report.screenshots.push({ label, path: file, state: summarize(state) });
  };
  const metricStart = async label => {
    phase = label;
    await page.evaluate(() => { const m = __newBarkMetrics; m.active = true; m.last = 0; m.frames = []; m.longTasks = []; m.start = { bufferUploadBytes: m.bufferUploadBytes, bufferDataCalls: m.bufferDataCalls, bufferSubDataCalls: m.bufferSubDataCalls, textureAllocations: m.textureAllocations }; });
  };
  const metricEnd = async label => {
    const sample = await page.evaluate(label => {
      const m = __newBarkMetrics; m.active = false; m.last = 0;
      const sorted = [...m.frames].sort((a, b) => a - b);
      const percentile = fraction => sorted.length ? sorted[Math.min(sorted.length - 1, Math.floor(sorted.length * fraction))] : null;
      return { label, frameCount: sorted.length, frameMs: { median: percentile(.5), p95: percentile(.95), p99: percentile(.99), maximum: sorted.at(-1) ?? null },
        longTasks: { count: m.longTasks.length, totalMs: m.longTasks.reduce((sum, x) => sum + x, 0), maximumMs: Math.max(0, ...m.longTasks) },
        wasmHeapBytes: __newBarkQA.raw.memory.buffer.byteLength, jsHeapBytes: performance.memory?.usedJSHeapSize ?? null,
        gpu: { bufferDataCalls: m.bufferDataCalls - m.start.bufferDataCalls, bufferSubDataCalls: m.bufferSubDataCalls - m.start.bufferSubDataCalls,
          bufferUploadBytes: m.bufferUploadBytes - m.start.bufferUploadBytes, textureAllocations: m.textureAllocations - m.start.textureAllocations,
          cumulativeBufferUploadBytes: m.bufferUploadBytes, largestBuffer: m.largestBuffer, contexts: m.contexts },
        contextLost: m.contextLost, visible: !document.hidden, focused: document.hasFocus() };
    }, label);
    report.metrics.push(sample);
    assert.ok(sample.wasmHeapBytes <= Number(process.env.NEW_BARK_HEAP_MIB || 512) * 1024 * 1024, 'WASM high-water heap exceeded budget');
    assert.ok(sample.gpu.largestBuffer <= 32 * 1024 * 1024, 'Single GPU buffer exceeded 32 MiB budget');
    assert.equal(sample.contextLost, 0, 'WebGL context was lost');
    assert.ok(sample.visible && sample.focused, 'Only visible/focused headless pages give meaningful frame samples');
    return sample;
  };
  const moveOne = async button => {
    const before = position(state);
    for (let attempt = 0; attempt < 3; attempt++) {
      await press(button);
      if (!samePosition(position(state), before) || busy(state)) return true;
    }
    blockedEdges.add(`${before.map}:${before.x},${before.y}:${button}`);
    return false;
  };
  const moveTo = async (goal, { permitGoalWarp = false, expectMap = state.map_info.name, allowDialogue = false } = {}) => {
    for (let i = 0; i < 160; i++) {
      if (state.map_info.name !== expectMap) return;
      if (busy(state)) { if (allowDialogue) return; throw new Error(`Unexpected dialogue/menu while navigating ${expectMap}`); }
      if (state.map_info.player.x === goal.x && state.map_info.player.y === goal.y) return;
      const route = planTo(goal, state, { permitGoalWarp });
      assert.ok(route?.length, `No observed walkable route to ${expectMap} (${goal.x},${goal.y}); not teleporting`);
      const before = position(state); await moveOne(route[0]);
      if (state.map_info.name === expectMap) {
        const distance = Math.abs(state.map_info.player.x - before.x) + Math.abs(state.map_info.player.y - before.y);
        assert.ok(distance <= 1 || busy(state), `One-frame input unexpectedly crossed ${distance} tiles`);
      }
    }
    throw new Error(`Navigation failed to reach ${expectMap} (${goal.x},${goal.y}) in 160 real-input steps`);
  };
  const closeDialogue = async (limit = 128) => {
    for (let i = 0; i < limit; i++) {
      await settle();
      if (!busy(state)) return;
      await press('a');
    }
    throw new Error('Dialogue did not release input after 128 confirmation taps');
  };
  const testDialogueOwnership = async label => {
    assert.ok(dialogue(state), `${label} must show actual visible dialogue`);
    const before = position(state), first = String(dialogue(state));
    await press('right', `${label}: direction while dialogue owns input`);
    assert.deepEqual(position(state), before, 'Directional input moved the player under dialogue');
    let advanced = false;
    for (let i = 0; i < 12 && dialogue(state); i++) { await press('a'); if (String(dialogue(state)) !== first) { advanced = true; break; } }
    assert.ok(advanced, 'A must change a dialogue page or finish the page sequence');
    check(label, { blockedMovement: true, pageChanged: true, firstPageSha256: hash(first) });
    await closeDialogue();
  };

  await observe();
  phase = preview ? 'production-location-preview' : fresh ? 'original-new-game' : 'restore-fixture';
  for (let i = 0; state.status.screen === 'intro' && i < 12; i++) await press('a');
  for (let i = 0; state.status.screen === 'title' && !/NEW GAME/.test(state.observe.text) && i < 12; i++) await press('start');
  if (!fresh && !preview && state.status.screen === 'title') {
    assert.match(state.observe.text, /CONTINUE/, 'FIXTURE_INCOMPATIBLE: no Continue option; provide a genuine matching storage fixture or rerun --fresh. Save identity was not rewritten.');
    for (let i = 0; !/^\s*>\s*CONTINUE/m.test(state.observe.text) && i < 5; i++) await press('up');
    assert.match(state.observe.text, /^\s*>\s*CONTINUE/m, 'Could not select production Continue option');
    for (let i = 0; state.status.screen === 'title' && i < 8; i++) await press('a');
  } else if (fresh) {
    assert.equal(state.status.screen, 'title', 'Fresh session should reach the original title menu');
    for (let i = 0; state.status.screen !== 'overworld' && i < 240; i++) {
      const nameChoice = menus(state).find(menu => menu.kind === 'name_choices');
      if (nameChoice) {
        const wanted = nameChoice.options.findIndex(option => /CHRIS/.test(option));
        const target = wanted >= 0 ? wanted : nameChoice.options.findIndex(option => !/NEW NAME/i.test(option));
        assert.ok(target >= 0, 'No existing trainer name choice; refusing to manipulate name/controller state');
        await press(nameChoice.selected === target ? 'a' : nameChoice.selected < target ? 'down' : 'up');
      } else {
        assert.ok(!menus(state).some(menu => menu.kind === 'name_input'), 'Unexpected name keyboard; select an authored trainer name with normal input');
        await press('a');
      }
    }
    assert.equal(state.status.screen, 'overworld', 'Original intro did not finish');
    assert.equal(state.map_info.name, 'PlayersHouse2F', 'Fresh game must begin in original bedroom');
    check('Original intro and trainer identity', { trainer: state.status.player_name, map: state.map_info.name });
    await moveTo({ x: 7, y: 0 }, { permitGoalWarp: true });
    if (state.map_info.name === 'PlayersHouse2F') await moveOne('up');
    assert.equal(state.map_info.name, 'PlayersHouse1F', 'Bedroom stairs must execute the actual warp');
    for (let i = 0; !dialogue(state) && i < 8; i++) await moveOne('down');
    assert.ok(dialogue(state), 'Mom must intercept the player before leaving');
    await testDialogueOwnership('Mom dialogue owns directions and advances pages');
    await moveTo({ x: 7, y: 7 }, { permitGoalWarp: true });
    if (state.map_info.name === 'PlayersHouse1F') await moveOne('down');
    assert.equal(state.map_info.name, 'NewBarkTown', 'Original home exit must reach New Bark');
    check('Bedroom, Mom and home exit production path');
  }
  assert.equal(state.status.screen, 'overworld', 'FIXTURE_INCOMPATIBLE: save did not restore an overworld');
  assert.equal(state.map_info.name, 'NewBarkTown', 'Expected a genuine NewBarkTown session; coordinates were not overridden');
  assert.ok(!busy(state), 'New Bark must start with free gameplay input');
  report.initial = summarize(state);
  check(preview ? 'Production location preview reached New Bark' : fresh ? 'Genuine New Bark arrival' : 'Fixture restored through production Continue', { position: position(state) });
  phase = '3d-warmup';
  const toggle = page.locator('#view-toggle');
  if (await toggle.getAttribute('aria-pressed') !== 'true') {
    await page.locator('#player-options > summary').click(); await toggle.click();
    if (await page.locator('#player-options').evaluate(e => e.open)) await page.locator('#player-options > summary').click();
  }
  assert.equal(await toggle.getAttribute('aria-pressed'), 'true', 'Modeled 3D view must be enabled');
  await page.locator('canvas').focus();
  await page.waitForTimeout(5000);
  await observe(); await metricStart('warm-new-bark-idle'); await page.waitForTimeout(5000); await metricEnd('warm-new-bark-idle');
  await capture('warm-new-bark-overview');

  phase = 'production-menus';
  const menuOrigin = position(state);
  await press('start');
  let start = menus(state).find(menu => menu.kind === 'start');
  assert.ok(start, 'Start must expose the production Start menu');
  assert.ok(start.entries.some(entry => /PACK/.test(entry)), 'Production Start menu must include PACK');
  await press('down'); assert.deepEqual(position(state), menuOrigin, 'Start menu owns directional input');
  for (let i = 0; i < 15; i++) {
    start = menus(state).find(menu => menu.kind === 'start');
    assert.ok(start, 'Start menu unexpectedly closed during selection');
    if (start.entries.some(entry => /^\s*>\s*PACK\b/.test(entry))) break;
    await press('down');
  }
  assert.ok(start.entries.some(entry => /^\s*>\s*PACK\b/.test(entry)), 'Could not select PACK with production cursor');
  await press('a'); assert.ok(menus(state).some(menu => menu.kind === 'pack'), 'PACK must open its production menu');
  await press('right'); assert.deepEqual(position(state), menuOrigin, 'PACK owns directional input');
  for (let i = 0; busy(state) && i < 5; i++) await press('b');
  assert.ok(!busy(state), 'B must return menu ownership to overworld');
  check('Start and PACK menus own input and close normally');

  phase = 'keyboard-walking';
  // The known home approach is verified against live collision observations.
  await moveTo({ x: 13, y: 6 });
  const beforeKeyboard = position(state);
  const adjacent = DIRECTIONS.find(([, dx, dy]) => {
    const p = { x: beforeKeyboard.x + dx, y: beforeKeyboard.y + dy };
    const tile = terrainByMap.get('NewBarkTown').get(keyOf(p));
    return tile?.terrain === 'Land' && !warpTile(tile) && !state.map_info.objects.some(o => keyOf(o) === keyOf(p));
  });
  assert.ok(adjacent, 'Need a genuinely walkable neighboring tile for keyboard regression');
  await metricStart('keyboard-walking');
  const key = `Arrow${adjacent[0][0].toUpperCase()}${adjacent[0].slice(1)}`;
  for (let i = 0; samePosition(position(state), beforeKeyboard) && i < 3; i++) {
    await page.keyboard.down(key); await page.waitForTimeout(120); await page.keyboard.up(key);
    await page.waitForTimeout(250); await observe(); await settle();
  }
  assert.equal(state.map_info.name, 'NewBarkTown');
  assert.ok(!samePosition(position(state), beforeKeyboard), 'Actual keyboard input must commit a walking tile');
  report.trace.push({ phase, input: `physical ${key}`, ...summarize(state) });
  await moveTo({ x: beforeKeyboard.x, y: beforeKeyboard.y });
  await metricEnd('keyboard-walking');
  check('Physical keyboard walking and return', { returnedTo: position(state) });

  phase = 'collision';
  const origin = position(state), grid = terrainByMap.get('NewBarkTown');
  const collisionTargets = [];
  for (const [key, tile] of grid) {
    if (tile.terrain !== 'Land' || warpTile(tile)) continue;
    const [x, y] = key.split(',').map(Number), route = planTo({ x, y }, state, { allowFrontier: false });
    if (!route) continue;
    for (const [button, dx, dy] of DIRECTIONS) if (grid.get(`${x + dx},${y + dy}`)?.terrain === 'Wall') collisionTargets.push({ x, y, button, routeLength: route.length });
  }
  collisionTargets.sort((a, b) => a.routeLength - b.routeLength);
  assert.ok(collisionTargets.length, 'Need observed solid terrain for collision proof');
  const wall = collisionTargets[0]; await moveTo(wall);
  const beforeWall = position(state);
  await press(wall.button); await press(wall.button);
  assert.deepEqual(position(state), beforeWall, 'Solid wall must reject repeated directional walking');
  assert.ok(!busy(state), 'Collision check must not accidentally pass because a menu/dialogue owns input');
  check('Actual solid-wall collision', { position: beforeWall, attemptedDirection: wall.button });
  await moveTo({ x: origin.x, y: origin.y });

  phase = 'home-warp-round-trip';
  await moveTo({ x: 13, y: 6 });
  await metricStart('home-warp-round-trip');
  for (let i = 0; state.map_info.name === 'NewBarkTown' && i < 3; i++) await moveOne('up');
  assert.equal(state.map_info.name, 'PlayersHouse1F', 'Walking into the modeled home door must execute the original warp');
  const arrival = position(state);
  await capture('home-interior');
  // Interact with a real observed Mom object; NPC collision/roaming may require
  // replanning, but never fabricated object positions or controller mutation.
  let interacted = false;
  for (let i = 0; i < 16 && !interacted; i++) {
    const mom = state.map_info.objects.find(object => /MOM/i.test(object.name));
    assert.ok(mom, 'Production home observation must expose Mom');
    const options = DIRECTIONS.map(([button, dx, dy]) => ({ x: mom.x - dx, y: mom.y - dy, button })).map(candidate => ({ ...candidate, route: planTo(candidate, state, { allowFrontier: false }) })).filter(candidate => candidate.route).sort((a, b) => a.route.length - b.route.length);
    assert.ok(options.length, 'No observed reachable tile beside Mom');
    const target = options[0];
    if (target.route.length) { await moveOne(target.route[0]); continue; }
    await press(target.button); // Face the adjacent real object; its collision prevents stepping.
    await press('a'); interacted = Boolean(dialogue(state));
  }
  assert.ok(interacted, 'A facing Mom must launch authored dialogue');
  await testDialogueOwnership('Mom interaction owns directions and advances pages');
  await moveTo({ x: 7, y: 7 }, { permitGoalWarp: true });
  for (let i = 0; state.map_info.name === 'PlayersHouse1F' && i < 3; i++) await moveOne('down');
  assert.equal(state.map_info.name, 'NewBarkTown', 'Original home exit must restore modeled New Bark');
  assert.equal(await toggle.getAttribute('aria-pressed'), 'true', '3D view must survive indoor/outdoor warps');
  await moveTo({ x: 13, y: 6 });
  await metricEnd('home-warp-round-trip');
  await capture('walk-warp-restored');
  check('Home entry, interaction, exit and restored 3D town', { interiorArrival: arrival, returnedTo: position(state) });

  phase = 'new-bark-west-gate';
  const scene = state.reward_state?.scenes?.NewBarkTown;
  if (typeof scene === 'string' && /TEACHER_STOPS_YOU/.test(scene)) {
    await metricStart('new-bark-west-gate');
    await moveTo({ x: 2, y: 8 });
    for (let i = 0; !dialogue(state) && i < 3; i++) await moveOne('left');
    assert.equal(state.map_info.name, 'NewBarkTown');
    assert.match(String(dialogue(state)), /Wait,/i, 'Uncleared New Bark gate must launch authored teacher dialogue');
    const playerName = state.status.player_name;
    assert.ok(String(dialogue(state)).includes(playerName), 'Gate dialogue must refer to the actual trainer');
    await testDialogueOwnership('New Bark west gate owns input and advances pages');
    await settle();
    assert.deepEqual(position(state), { map: 'NewBarkTown', x: 5, y: 8 }, 'Completing gate must return player to source-authored (5,8)');
    await metricEnd('new-bark-west-gate');
    await capture('gate-return');
    check('New Bark gate returns player to authored position', { scene, returnedTo: position(state) });
  } else {
    report.checks.push({ name: 'New Bark west gate', status: 'skipped', reason: 'Current genuine save does not have the teacher-stop scene active; no flags or coordinates were modified', scene: scene ?? null });
  }
  phase = 'final-warm-restored';
  await moveTo({ x: 13, y: 6 });
  await page.waitForTimeout(2000);
  await metricStart('final-warm-restored'); await page.waitForTimeout(3000); await metricEnd('final-warm-restored');
  await capture('final-restored-overview');
  report.final = summarize(state);
  assert.ok(report.metrics.every(sample => sample.frameCount > 0), 'Expected real animation-frame timing samples');
  assert.ok(report.metrics.some(sample => sample.gpu.contexts.length > 0 && sample.gpu.cumulativeBufferUploadBytes > 0), 'Expected live WebGL context and real GPU buffer instrumentation');
  assert.deepEqual(report.errors, [], 'Console/runtime errors occurred; see report.errors');
  report.status = 'passed';
} catch (error) {
  report.status = 'failed'; report.failure = { phase, name: error.name, message: error.message, stack: error.stack };
  if (state) report.last = summarize(state);
  if (page && !page.isClosed()) {
    const file = screenshotPath('failure');
    await page.screenshot({ path: file, timeout: 15000 }).then(() => report.screenshots.push({ label: 'failure', path: file })).catch(() => {});
  }
  process.exitCode = 1;
  console.error(`${phase}: ${error.message}`);
} finally {
  report.finishedAt = new Date().toISOString();
  await fs.writeFile(output, `${JSON.stringify(report, null, 2)}\n`);
  await browser.close();
  console.log(JSON.stringify({ status: report.status, report: output, checks: report.checks.map(({ name, status }) => ({ name, status })), screenshots: report.screenshots.map(s => s.path) }));
}
