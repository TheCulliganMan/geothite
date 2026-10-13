/** Front-door allocation regression; all movement uses the production bridge. */
import { chromium, webkit } from 'playwright';
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import { createHash } from 'node:crypto';
import { installTextureMemoryAccounting } from './browser-texture-memory.mjs';
const [url, output, engine = 'webkit', cyclesText = '6'] = process.argv.slice(2);
const cycles = Number(cyclesText);
assert.ok(url && output && ['webkit', 'chromium'].includes(engine) &&
  Number.isInteger(cycles) && cycles > 0,
  'usage: node tools/browser-house-memory.mjs URL OUTPUT_JSON [webkit|chromium] [CYCLES]');
assert.equal(new URL(url).searchParams.get('preview'), 'house-door');
assert.equal(new URL(url).searchParams.get('multiplayer'), 'off');
const report = { url, engine, cycles, samples: [], transitions: [], errors: [],
  quality: { requiredSceneryFullDetail: true },
  accounting: 'WASM linear-memory high-water, WebGL buffers and known-format texture storage; excludes renderbuffers and driver overhead. Unknown texture formats are reported.' };
let browser;
try {
  browser = await ({ chromium, webkit })[engine].launch({ headless: true,
    ...(engine === 'chromium' && process.env.BROWSER_CHANNEL ?
      { channel: process.env.BROWSER_CHANNEL } : {}) });
  report.browser = browser.version();
  const context = await browser.newContext({ viewport: { width: 1280, height: 900 } });
  const installBufferMemoryAccounting = () => {
    localStorage.setItem('crystal.audio.muted', 'true');
    window.houseMemory = { largestBuffer: 0, bufferUploadBytes: 0,
      liveBufferBytes: 0, peakBufferBytes: 0, contextLost: 0 };
    const sizes = new WeakMap();
    const proto = WebGL2RenderingContext.prototype;
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
    document.addEventListener('webglcontextlost', () => houseMemory.contextLost++, true);
  };
  // Keep both installers in one script: separate init scripts have no ordering
  // contract, and the texture hook needs the shared meter created above.
  await context.addInitScript({ content:
    `(${installBufferMemoryAccounting.toString()})();(${installTextureMemoryAccounting.toString()})();` });
  const page = await context.newPage();
  page.on('pageerror', error => report.errors.push(String(error)));
  page.on('crash', () => report.errors.push('browser page crashed'));
  page.on('console', message => {
    if (message.type() === 'error') (report.consoleErrors ??= []).push(message.text());
  });
  await page.goto(url, { waitUntil: 'networkidle', timeout: 240000 });
  report.stage = 'startup';
  await fs.writeFile(output, JSON.stringify(report, null, 2) + '\n');
  await page.waitForFunction(() => document.querySelector('#loading')?.hidden ||
    document.querySelector('#startup-error')?.open, null, { timeout: 240000 });
  assert.equal(await page.locator('#startup-error').evaluate(e => e.open ? e.textContent : null), null);
  const artifacts = await page.evaluate(async () => {
    const scripts = [...document.scripts].map(s => s.textContent).join('\n');
    const path = scripts.match(/import\('(\.\/crystal-bevy(?:-[a-f0-9]{64})?\.js)'\)/)?.[1];
    if (!path) throw Error('Missing compiled game import');
    const wasm = await import(path);
    window.houseRaw = await wasm.default();
    wasm.crystal_set_muted(true);
    const { createGameBridge } = await import('./webmcp.js');
    window.houseBridge = createGameBridge(wasm, { timeoutMs: 120000 });
    const requests = performance.getEntriesByType('resource').map(entry => entry.name)
      .filter(name => /\/crystal-bevy(?:-[a-f0-9]{64}|_bg)\.wasm$/.test(new URL(name).pathname));
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
    await page.waitForFunction(async map => {
      const state = await houseBridge.execute({ kind: 'observe' });
      const voxel = state.presentation?.voxel;
      if (!voxel) throw Error('Build must include location-tester voxel readiness diagnostics');
      if (voxel.scenery_full_detail !== true) throw Error('Full scenery detail is required for this memory trial');
      return state.map_info.name === map && voxel.active &&
        !voxel.profiles_pending && voxel.active_frames >= 30;
    }, map, { timeout: 240000, polling: 250 });
  };
  const sample = async label => {
    report.samples.push(await page.evaluate(label => ({ label,
      heapBytes: houseRaw.memory.buffer.byteLength, ...houseMemory,
      enabled: document.querySelector('#view-toggle').getAttribute('aria-pressed') }), label));
    report.samples.at(-1).voxel = (await command({ kind: 'observe' })).presentation?.voxel;
    report.stage = label;
    await fs.writeFile(output, JSON.stringify(report, null, 2) + '\n');
    console.log(JSON.stringify({ stage: label, ...report.samples.at(-1) }));
    assert.equal(report.samples.at(-1).contextLost, 0);
    assert.deepEqual(report.errors, []);
  };
  let state = await command({ kind: 'observe' });
  assert.equal(state.status.screen, 'overworld');
  assert.equal(state.map_info.name, 'PlayersHouse1F');
  await sample('2d-house');
  const toggle = page.locator('#view-toggle');
  if (!await toggle.isVisible()) await page.locator('#player-options > summary').click();
  if (await toggle.getAttribute('aria-pressed') !== 'true') await toggle.click();
  await waitForTerrain('PlayersHouse1F');
  await sample('3d-house');
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
    state = await command({ kind: 'press', button: 'start', frames: 1 });
    assert.ok((state.observe.menus ?? []).length || /POK.MON|PACK|SAVE/.test(JSON.stringify(state.observe)));
    await command({ kind: 'press', button: 'b', frames: 1 });
    await reach('up', 'PlayersHouse1F');
    await waitForTerrain('PlayersHouse1F');
    await sample('entry-' + cycle);
  }
  report.heapHighWater = Math.max(...report.samples.map(s => s.heapBytes));
  assert.ok(report.heapHighWater <= 512 * 1024 ** 2, 'WASM exceeded 512 MiB');
  assert.ok(report.samples.every(s => s.largestBuffer <= 32 * 1024 ** 2), 'GPU buffer exceeded 32 MiB');
  report.passed = true;
} catch (error) {
  report.failure = String(error);
  process.exitCode = 1;
} finally {
  await fs.writeFile(output, JSON.stringify(report, null, 2) + '\n');
  await browser?.close();
}
