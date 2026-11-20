// Exercise the shipped controls with trusted browser touch input. MCP only observes.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import { chromium, webkit, devices } from 'playwright';
const [base, output, selectedEngine] = process.argv.slice(2);
assert.ok(base && output, 'usage: node tools/browser-mobile-smoke.mjs URL REPORT_JSON [webkit|chromium]');
assert.ok(!selectedEngine || ['webkit', 'chromium'].includes(selectedEngine), 'unknown browser engine');
const results = [];
for (const [name, engine] of [['webkit', webkit], ['chromium', chromium]]) {
 if (selectedEngine && name !== selectedEngine) continue;
 const browser = await engine.launch({ headless: true, args: name === 'chromium' ? (process.platform === 'darwin' ? ['--use-angle=metal'] : ['--use-angle=swiftshader', '--enable-unsafe-swiftshader']) : undefined });
 const initialView = '2d';
 const report = { engine: name, initialView, errors: [], consoleErrors: [], intro: [], layouts: [] }; results.push(report);
 try {
  const context = await browser.newContext({ ...devices['iPhone 13'] });
  const page = await context.newPage();
  page.on('pageerror', error => report.errors.push(error.message));
  page.on('console', message => { if (message.type() === 'error') report.consoleErrors.push(message.text()); });
  await page.goto(new URL(`/?view=${initialView}&multiplayer=off`, base).href);
  await page.waitForFunction(() => document.querySelector('#loading').hidden || document.querySelector('#startup-error').open, null, { timeout: 240000 });
  assert.equal(await page.locator('#startup-error').evaluate(e => e.open ? e.textContent : null), null);
  await page.evaluate(async () => {
   const wasm = await import(new URL(__crystalRendererBundle.module, document.baseURI));
   const { createGameBridge } = await import('/webmcp.js');
   window.mobileTestBridge = createGameBridge(wasm, { timeoutMs: 60000 });
   window.mobileTestAllows3D = wasm.crystal_voxel_view_allowed?.() !== false;
  });
  const observe = () => page.evaluate(() => mobileTestBridge.execute({ kind: 'observe' }));
  let state = await observe();
  const minnesota = state.map_info.name.startsWith('Mn');
  const startingMap = minnesota ? 'MnWinonaHome2F' : 'PlayersHouse2F';
  report.region = minnesota ? 'minnesota' : 'johto';
  report.allow3D = await page.evaluate(() => mobileTestAllows3D);
  assert.equal(state.map_info.name, startingMap, 'new session must use the regional starting map');
  if (minnesota) {
   assert.equal(report.allow3D, false, 'Minnesota is native 2D');
   assert.equal(await page.locator('#view-navigation [data-view="3d"]').isVisible(), false);
  }
  // Let the complete, unskipped movie render. Fail on errors even if later scenes recover.
  for (let i = 0; state.status.screen === 'intro' && i < 180; i++) {
   assert.equal(state.recent_events.error, null, JSON.stringify(state.recent_events));
   const scene = state.observe.text;
   if (report.intro.at(-1) !== scene) { report.intro.push(scene); console.log(name, scene); }
   if (/IntroScene(?:16|20)$/.test(scene)) await page.screenshot({ path: output.replace(/\.json$/, `-${name}-${scene.match(/IntroScene\d+/)[0]}.png`) });
   await page.waitForTimeout(500); state = await observe();
  }
  assert.equal(state.status.screen, 'title', 'the full intro must hand off to the title');
  assert.equal(state.recent_events.error, null);
  // Existing Minnesota packs open the regional menu directly; packs with the
  // authored Northstar movie opt in. Whenever a movie starts, watch it fully.
  if (!minnesota || report.intro.length) {
   for (const scene of ['IntroScene16', 'IntroScene20']) assert.ok(report.intro.some(text => text.endsWith(scene)), `${scene} must actually play`);
  } else assert.match(state.observe.text, /NEW GAME/, 'Minnesota direct startup must expose its game menu');
  const layout = async label => {
   const controls = await page.locator('#touch-controls button').evaluateAll(buttons => buttons.map(button => {
    const r = button.getBoundingClientRect();
    const hit = document.elementFromPoint(r.x + r.width / 2, r.y + r.height / 2);
    return { button: button.dataset.gameButton, x: r.x, y: r.y, width: r.width, height: r.height, hit: hit === button, visible: getComputedStyle(button).visibility === 'visible' };
   }));
   assert.equal(controls.length, 8);
   for (const control of controls) { assert.ok(control.width >= 44 && control.height >= 44 && control.hit && control.visible, `${label}: ${JSON.stringify(control)}`); }
   for (let i = 0; i < controls.length; i++) for (const other of controls.slice(i + 1)) {
    const control = controls[i];
    const overlapX = Math.min(control.x + control.width, other.x + other.width) - Math.max(control.x, other.x);
    const overlapY = Math.min(control.y + control.height, other.y + other.height) - Math.max(control.y, other.y);
    assert.ok(overlapX <= .5 || overlapY <= .5, `${label}: ${control.button} overlaps ${other.button}`);
   }
   const stage = await page.locator('#game-stage').boundingBox();
   assert.ok(stage.width >= 160 && stage.height >= 144, `${label}: readable game surface`);
   report.layouts.push({ label, controls, stage });
   await page.screenshot({ path: output.replace(/\.json$/, `-${name}-${label}.png`) });
  };
  await layout('portrait');
  let taps = 0;
  const welcome = new Set();
  const tap = async button => {
   const box = await page.locator(`[data-game-button="${button}"]`).boundingBox();
   await page.touchscreen.tap(box.x + box.width / 2, box.y + box.height / 2);
   await page.waitForTimeout(180); state = await observe();
   if (state.status.screen === 'introduction' && state.observe.text) welcome.add(state.observe.text);
   if (++taps % 15 === 0) console.log(name, 'touch', taps, button, state.status.screen, state.observe.text.slice(0,160));
   return state;
  };
  for (let i = 0; state.status.screen === 'title' && !/NEW GAME/.test(state.observe.text) && i < 15; i++) await tap('start');
  assert.match(state.observe.text, /NEW GAME/, 'touch START must open the title menu');
  for (let i = 0; state.status.screen !== 'overworld' && i < 240; i++) {
   const names = state.observe.menus?.find(menu => menu.kind === 'name_choices');
   if (names) {
    const target = names.options.findIndex(option => /CHRIS/.test(option)); assert.ok(target >= 0);
    await tap(names.selected === target ? 'a' : names.selected < target ? 'down' : 'up');
   } else await tap('a');
   assert.equal(state.recent_events.error, null, JSON.stringify(state.recent_events));
  }
  assert.equal(state.status.screen, 'overworld', `touch opening did not finish: ${state.observe.text}`);
  assert.equal(state.map_info.name, startingMap);
  assert.equal(state.status.player_name, 'CHRIS');
  report.welcome = [...welcome];
  report.newGame = { name: state.status.player_name, map: state.map_info.name, taps };
  const before = state.map_info.player;
  for (let i = 0; i < 4 && state.map_info.player.x === before.x; i++) await tap('right');
  assert.ok(state.map_info.player.x > before.x, 'touch D-pad must move the authoritative player');
  report.movement = { before, after: state.map_info.player };
  if (name === 'chromium') {
   const session = await context.newCDPSession(page);
   const point = async (button, id) => {
    const r = await page.locator(`[data-game-button="${button}"]`).boundingBox();
    return { x: r.x + r.width / 2, y: r.y + r.height / 2, id };
   };
   await session.send('Input.dispatchTouchEvent', { type: 'touchStart', touchPoints: [await point('left', 1), await point('b', 2)] });
   await page.waitForTimeout(250);
   assert.equal(await page.locator('[data-game-button="left"]').evaluate(e => e.classList.contains('held')), true);
   assert.equal(await page.locator('[data-game-button="b"]').evaluate(e => e.classList.contains('held')), true);
   await session.send('Input.dispatchTouchEvent', { type: 'touchCancel', touchPoints: [] });
   await page.waitForTimeout(350);
   assert.equal(await page.locator('#touch-controls .held').count(), 0);
   const released = (await observe()).map_info.player;
   await page.waitForTimeout(350); state = await observe();
   assert.deepEqual(state.map_info.player, released, 'canceled touches must not leave movement held');
   report.multitouch = { released, stopped: state.map_info.player };
   const stage = await page.locator('#game-stage').boundingBox();
   const center = { x: stage.x + stage.width / 2, y: stage.y + stage.height / 2 };
   const fingers = distance => [
    { x: center.x - distance, y: center.y, id: 1 },
    { x: center.x + distance, y: center.y, id: 2 },
   ];
   const scale = await page.evaluate(() => visualViewport.scale);
   await session.send('Input.dispatchTouchEvent', { type: 'touchStart', touchPoints: fingers(20) });
   for (let distance = 30; distance <= 110; distance += 10) {
    await session.send('Input.dispatchTouchEvent', { type: 'touchMove', touchPoints: fingers(distance) });
    await page.waitForTimeout(20);
   }
   await session.send('Input.dispatchTouchEvent', { type: 'touchEnd', touchPoints: [] });
   assert.equal(await page.evaluate(() => visualViewport.scale), scale, 'pinch during play must not zoom the page');
   state = await observe();
   assert.deepEqual(state.map_info.player, released, 'pinching must not move the player');
   report.pinch = { before: scale, after: await page.evaluate(() => visualViewport.scale) };
   await session.detach();
  }
  for (let i = 0; !state.observe.menus?.some(menu => menu.kind === 'start') && i < 5; i++) await tap('start');
  assert.ok(state.observe.menus?.some(menu => menu.kind === 'start'), 'touch START must expose the production Start menu');
  const menu = JSON.stringify(state.observe.menus); await tap('down');
  assert.notEqual(JSON.stringify(state.observe.menus), menu, 'D-pad must change the menu selection');
  await tap('b'); assert.equal(state.observe.menus?.length ?? 0, 0, 'B closes the menu');
  await layout('portrait-gameplay');
  await page.setViewportSize({ width: 320, height: 568 }); await page.waitForTimeout(600);
  await layout('small-portrait');
  await page.setViewportSize({ width: 844, height: 390 }); await page.waitForTimeout(600);
  await layout('landscape');
  for (const mode of report.allow3D ? ['3d', 'tui', '2d'] : ['tui', '2d']) {
   const beforeView = { map: state.map_info.name, player: state.map_info.player, name: state.status.player_name };
   await page.locator(`#view-navigation [data-view="${mode}"]`).tap(); await page.waitForTimeout(600);
   await layout(`landscape-${mode}`);
   state = await observe();
   assert.equal(state.status.screen, 'overworld');
   assert.deepEqual({ map: state.map_info.name, player: state.map_info.player, name: state.status.player_name }, beforeView, 'changing presentation must preserve the session');
  }
  assert.deepEqual(report.errors, []);
  assert.deepEqual(report.consoleErrors, []);
  console.log(JSON.stringify({ engine: name, introScenes: report.intro.length, movement: report.movement, layouts: report.layouts.length, errors: report.errors }));
  await context.close();
 } finally { await browser.close(); await fs.writeFile(output, JSON.stringify({ base, results }, null, 2) + '\n'); }
}
