// Twenty live WASM games at one URL, real WebMCP/code mode, durable profile
// restart, same-slot writer rejection and bounded local saves. No game mocks.
import assert from 'node:assert/strict';
import { createServer } from 'node:http';
import { readFile, mkdtemp, rm, mkdir } from 'node:fs/promises';
import { resolve, extname } from 'node:path';
import { chromium } from 'playwright';
import { checkHome } from './tui-home-checks.mjs';
const root = resolve(process.env.TUI_TEST_ROOT ?? 'target/tui-web');
const pack = resolve(process.env.TUI_TEST_PACK ?? 'content-packs/realtime-clock.browser.crystalpack');
await mkdir('target/tui-session-smoke', { recursive: true });
const profile = await mkdtemp(resolve('target/tui-session-smoke/profile-'));
const server = createServer(async (req, res) => {
  try {
    const path = new URL(req.url, 'http://localhost').pathname;
    const file = path === '/realtime-clock.browser.crystalpack' ? pack : path === '/tui' || path === '/tui/' ? resolve(root, 'tui/index.html') : resolve(root, `.${path}`);
    if (file !== pack && !file.startsWith(`${root}/`)) { res.writeHead(403).end(); return; }
    const bytes = await readFile(file);
    res.writeHead(200, { 'content-type': ({ '.html': 'text/html', '.js': 'text/javascript', '.wasm': 'application/wasm', '.css': 'text/css' })[extname(file)] ?? 'application/octet-stream' });
    res.end(bytes);
  } catch { res.writeHead(404).end(); }
});
await new Promise(done => server.listen(0, '127.0.0.1', done));
const base = process.env.TUI_TEST_URL ?? `http://127.0.0.1:${server.address().port}/tui/`;
const options = { channel: 'chrome', headless: true, viewport: { width: 640, height: 480 }, reducedMotion: 'reduce', args: ['--enable-blink-features=WebMCP', '--enable-features=WebMCP'] };
let context;
const errors = [];
async function open(id) {
  const page = await context.newPage();
  page.on('pageerror', e => errors.push(e.message));
  const url = new URL(base); url.searchParams.set('session', id);
  await page.goto(url.href);
  await page.waitForFunction(() => !document.getElementById('screen').hidden, { timeout: 120000 });
  await page.evaluate(() => window.geothiteTui.ready);
  return page;
}
const view = page => page.evaluate(() => window.geothiteTui.observe());
const execute = (page, code) => page.evaluate(async code => {
  const tool = (await document.modelContext.getTools()).find(t => t.name === 'geothite_tui_execute');
  assertTool(tool);
  const result = await document.modelContext.executeTool(tool, { code });
  if (result?.isError) throw new Error(JSON.stringify(result));
  return typeof result === 'string' ? JSON.parse(result) : result;
  function assertTool(tool) { if (!tool) throw new Error('Real WebMCP execute tool missing'); }
}, code);
try {
  context = await chromium.launchPersistentContext(profile, options);
  await context.addInitScript(() => {
    window.saveWrites = 0;
    const write = Storage.prototype.setItem;
    Storage.prototype.setItem = function(key, value) { if (key.startsWith('crystal.save.v1.')) window.saveWrites++; return write.call(this, key, value); };
  });
  const pages = [];
  // All twenty remain alive. Batches avoid transient concurrent pack decoding
  // spikes while still checking concurrent sessions at the same origin/profile.
  for (let i = 0; i < 20; i += 4) pages.push(...await Promise.all(Array.from({ length: 4 }, (_, j) => open(`player-${i + j}`))));
  const initial = await Promise.all(pages.map(view));
  console.log('20 browser controllers started.');
  assert(initial.every(s => /PlayersHouse2F.*\(3, 3\)/.test(s.status_line)));
  const writes = await pages[0].evaluate(() => window.saveWrites);
  await execute(pages[0], 'for(let i=0;i<20;i++) await tools.observe(); return null;');
  await pages[0].waitForTimeout(1200);
  assert.equal(await pages[0].evaluate(() => window.saveWrites), writes, 'Observe/idle never write saves');
  await execute(pages[0], "await tools.press({button:'right'}); await tools.press({button:'right'}); return await tools.observe();");
  assert.match((await view(pages[0])).status_line, /\(4, 3\)/);
  for (let i = 1; i < 20; i++) assert.equal((await view(pages[i])).status_line, initial[i].status_line, 'One player cannot mutate the other 19');
  await Promise.all(pages.slice(1).map((page, i) => execute(page, `for(let i=0;i<${i % 2 ? 4 : 2};i++) await tools.press({button:'down'}); return await tools.observe();`)));
  const expected = await Promise.all(pages.map(view));
  for (let i = 1; i < 20; i++) assert.notEqual(expected[i].status_line, initial[i].status_line, `Player ${i}'s WebMCP inputs must reach their own controller`);
  const slots = await Promise.all(pages.map(page => page.evaluate(() => window.geothiteTui.session().savePath)));
  assert.equal(new Set(slots).size, 20);
  const stats = await pages[0].evaluate(slots => slots.map(path => {
    const primary = localStorage.getItem(`crystal.save.v1.${path}`);
    const backup = localStorage.getItem(`crystal.save.v1.${path}.bak`);
    return { bytes: atob(primary).length, recovered: primary === backup };
  }), slots);
  assert(stats.every(s => s.bytes < 65536 && s.recovered), 'Compact current-generation primary/recovery copies');
  const duplicate = await context.newPage();
  const duplicateUrl = new URL(base); duplicateUrl.searchParams.set('session', 'player-0');
  await duplicate.goto(duplicateUrl.href);
  await duplicate.waitForFunction(() => document.getElementById('status').textContent.includes('another tab'));
  assert(await duplicate.locator('#screen').evaluate(el => el.hidden), 'Rejected writer cannot start a divergent game');
  await duplicate.close();
  // Simulate an interrupted primary replacement: never start a new game over
  // the complete recovery generation when only .bak survived.
  await pages[0].evaluate(path => localStorage.removeItem(`crystal.save.v1.${path}`), slots[19]);
  console.log('Isolation and writer rejection passed; restarting the browser.');
  // Close the complete browser process, not just reload an in-memory game.
  await context.close(); context = await chromium.launchPersistentContext(profile, options);
  console.log('Browser process restarted; reconnecting all 20 slots.');
  const resumed = [];
  for (let i = 0; i < 20; i += 4) resumed.push(...await Promise.all(Array.from({ length: 4 }, (_, j) => open(`player-${i + j}`))));
  for (let i = 0; i < 20; i++) {
    assert.equal((await view(resumed[i])).status_line, expected[i].status_line, `Player ${i} resumes without an explicit save`);
    await execute(resumed[i], "await tools.press({button:'start'}); return await tools.observe();");
    assert((await view(resumed[i])).menu.some(line => line.text.includes('PACK')), 'Resumed controller is playable');
  }
  await Promise.all(resumed.map(page => page.close()));
  const home = await open('home-regression');
  const saved = () => home.evaluate(() => localStorage.getItem(`crystal.save.v1.${window.geothiteTui.session().savePath}`));
  await checkHome(() => view(home), async button => {
    const before = await saved();
    const result = (await execute(home, `return await tools.press({button:${JSON.stringify(button)}});`)).value;
    if (!result.status_line.startsWith('Overworld')) assert.equal(await saved(), before, 'Mom/dialogue/menu cannot overwrite a safe checkpoint');
    return result;
  });
  const homeView = await view(home);
  await home.reload(); await home.waitForFunction(() => !document.getElementById('screen').hidden);
  assert.equal((await view(home)).status_line, homeView.status_line, 'Completed Mom scene checkpoints and resumes');
  await execute(home, "await tools.press({button:'start'}); return await tools.observe();");
  assert((await view(home)).menu.some(line => line.text.includes('PACK')));
  assert.deepEqual(errors, []);
  console.log(`20 concurrent named WASM/WebMCP sessions: isolated, single writer, browser-process restart/resume, Start, no idle writes. Saves ${Math.min(...stats.map(s=>s.bytes))}–${Math.max(...stats.map(s=>s.bytes))} bytes each plus one recovery copy.`);
} finally {
  // Bound the persistent-context shutdown acknowledgement, but never remove
  // a connected profile. Some Chrome runs disconnect before resolving close.
  let timer;
  await Promise.race([context?.close(), new Promise(done => { timer = setTimeout(done, 5000); })]);
  clearTimeout(timer);
  server.closeAllConnections(); await new Promise(done => server.close(done));
  assert(!context?.browser()?.isConnected(), 'Browser must disconnect before profile cleanup');
  await rm(profile, { recursive: true, force: true });
}
