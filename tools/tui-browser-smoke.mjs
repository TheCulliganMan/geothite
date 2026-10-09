// Run against the built WASM, not mocks. All progression uses original buttons.
import assert from 'node:assert/strict';
import { createServer } from 'node:http';
import { readFile, mkdir } from 'node:fs/promises';
import { resolve, extname } from 'node:path';
import { chromium, webkit } from 'playwright';
import { checkMenus } from './tui-menu-checks.mjs';
import { checkVisibleInkMotion } from './tui-motion-checks.mjs';

const root = resolve(process.env.TUI_TEST_ROOT ?? 'target/tui-web');
const pack = resolve(process.env.TUI_TEST_PACK ?? 'content-packs/realtime-clock.browser.crystalpack');
const output = resolve('target/tui-smoke');
await mkdir(output, { recursive: true });
const server = createServer(async (req, res) => {
  try {
    const path = new URL(req.url, 'http://localhost').pathname;
    const file = path === '/realtime-clock.browser.crystalpack' ? pack
      : path === '/tui' || path === '/tui/' ? resolve(root, 'tui/index.html') : resolve(root, `.${path}`);
    if (file !== pack && !file.startsWith(`${root}/`)) { res.writeHead(403).end(); return; }
    const type = { '.html': 'text/html', '.js': 'text/javascript', '.wasm': 'application/wasm', '.css': 'text/css' }[extname(file)] ?? 'application/octet-stream';
    const bytes = await readFile(file);
    res.writeHead(200, { 'content-type': type });
    res.end(bytes);
  } catch { res.writeHead(404).end(); }
});
await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
const url = process.env.TUI_TEST_URL ?? `http://127.0.0.1:${server.address().port}/tui`;
// Real browser WebMCP, including discovery and execution; no contract shim.
const browser = await chromium.launch({ channel: 'chrome', headless: true, args: ['--enable-blink-features=WebMCP', '--enable-features=WebMCP'] });
try {
  const page = await browser.newPage({ viewport: { width: 1440, height: 960 } });
  await page.emulateMedia({ reducedMotion: 'reduce' });
  const errors = [];
  page.on('pageerror', error => errors.push(error.message));
  await page.goto(url);
  await page.waitForFunction(async () => (await document.modelContext?.getTools())?.filter(tool => tool.name.startsWith('geothite_tui_')).length === 4);
  await page.waitForFunction(() => !document.getElementById('screen').hidden, { timeout: 120000 });
  assert(await page.evaluate(() => performance.getEntriesByType('resource').some(entry => /ascii-mount-[a-f0-9]+\.js/.test(entry.name))), 'Use the pinned ascii.rest painter, not a decorative placeholder');
  assert(await page.evaluate(() => {
    const canvas = document.getElementById('ascii');
    const pixels = canvas.getContext('2d').getImageData(0, 0, canvas.width, canvas.height).data;
    let painted = 0;
    for (let i = 0; i < pixels.length; i += 4) if (pixels[i] !== 3 || pixels[i + 1] !== 3 || pixels[i + 2] !== 18) painted++;
    return painted > 1000;
  }), 'Actual colored game glyphs must be painted on the ASCII canvas');
  assert.equal(await page.locator('header,nav,input,#launcher,#touch-controls button:visible').count(), 0, 'No launcher or Game Boy buttons on desktop');
  assert.equal(await page.locator('#screen').getAttribute('data-view'), 'painted', 'Painted is default on desktop');
  const ink = await page.locator('#screen').textContent();
  assert(/[·•●]/.test(ink) && !ink.includes('▀'), 'Rust dithers real tiles and sprites, not half-block raster pixels');
  assert(await page.evaluate(() => {
    const rows = document.getElementById('screen').textContent.split('\n');
    const world = rows.slice(3, Math.floor(rows.length * .65)).join('');
    return (world.match(/[·•●]/g) ?? []).length > 100;
  }), 'The scene has real halftone coverage, not a blank floor with a player badge');
  assert(await page.evaluate(() => {
    const fine = [...document.querySelectorAll('canvas')].find(c => c.id !== 'ascii');
    return fine && fine.width > 0 && fine.height > 0;
  }), 'Browser paints the Rust high-density dot scene, independently of the readable text grid');
  const beforeToggle = await page.evaluate(() => window.geothiteTui.observe());
  await checkVisibleInkMotion(page);
  const canvasHash=()=>page.evaluate(async()=>{
    const c=document.getElementById('ascii');
    return [...new Uint8Array(await crypto.subtle.digest('SHA-256',c.getContext('2d').getImageData(0,0,c.width,c.height).data))].join(',');
  });
  await page.emulateMedia({reducedMotion:'no-preference'});
  const dotFrames=new Set();
  for(let i=0;i<4;i++) {
    await page.waitForTimeout(200);
    dotFrames.add(await canvasHash());
    assert.deepEqual(await page.evaluate(()=>window.geothiteTui.observe()),beforeToggle,'Ambient art cannot advance gameplay');
  }
  assert(dotFrames.size>1,'Real canvas dot sizes animate');
  await page.emulateMedia({reducedMotion:'reduce'});
  await page.waitForTimeout(200);
  const stillHash=await canvasHash();
  await page.waitForTimeout(400);
  assert.equal(await canvasHash(),stillHash,'Reduced motion stops ambient art without hiding updates');
  await page.locator('#view-toggle').click();
  assert.equal(await page.locator('#screen').getAttribute('data-view'), 'text');
  assert.deepEqual((await page.evaluate(() => window.geothiteTui.observe())).marker, beforeToggle.marker, 'View toggle never moves the trainer');
  await page.keyboard.press('v');
  assert.equal(await page.locator('#screen').getAttribute('data-view'), 'painted');
  await page.setViewportSize({ width: 390, height: 844 });
  await page.waitForTimeout(100);
  assert.equal(await page.locator('#touch-controls button:visible').count(), 0, 'A narrow non-touchscreen desktop must not get arrow buttons');
  await page.setViewportSize({ width: 1440, height: 960 });
  await page.waitForTimeout(100);
  const observe = () => page.evaluate(() => window.geothiteTui.observe());
  const press = button => page.evaluate(button => window.geothiteTui.press(button), button);
  let snapshot = await observe();
  assert.match(snapshot.status_line, /PlayersHouse2F.*\(3, 3\)/);
  await page.keyboard.press('ArrowRight');
  if (!(await observe()).status_line.includes('(4, 3)')) await page.keyboard.press('ArrowRight');
  assert.match((await observe()).status_line, /\(4, 3\)/);
  await move('left', 1);
  await page.evaluate(async () => {
    const tool = (await document.modelContext.getTools()).find(t => t.name === 'geothite_tui_move');
    await document.modelContext.executeTool(tool, { direction: 'right', steps: 2 });
  });
  assert.match((await observe()).status_line, /\(4, 3\)/, 'Native WebMCP movement must drive the keyboard session');
  await move('left', 1);
  await page.evaluate(async () => {
    const tool = (await document.modelContext.getTools()).find(t => t.name === 'geothite_tui_press');
    await document.modelContext.executeTool(tool, { button: 'start' });
  });
  assert((await observe()).menu.some(line => line.text.includes('PACK')));
  await press('b');
  async function move(button, count) {
    for (let step = 0; step < count; step++) {
      const before = await observe();
      let after;
      for (let attempt = 0; attempt < 4; attempt++) {
        after = await press(button);
        if (JSON.stringify(after.marker) !== JSON.stringify(before.marker) || after.viewport_title !== before.viewport_title) break;
      }
      assert(JSON.stringify(after.marker) !== JSON.stringify(before.marker) || after.viewport_title !== before.viewport_title,
        `Blocked ${button}: ${after.status_line}`);
    }
  }
  await move('right', 4); await move('up', 3); await move('down', 4);
  const mom = await observe();
  assert.match(mom.status_line, /Text.*PlayersHouse1F/);
  assert(mom.dialogue.some(line => line.text.includes('CHRIS')));
  await press('right');
  assert.deepEqual((await observe()).marker, mom.marker);
  await press('a');
  assert.notDeepEqual((await observe()).dialogue, mom.dialogue);
  for (let i = 0; i < 128 && !(await observe()).status_line.startsWith('Overworld'); i++) await press('a');
  await move('down', 2); await move('left', 2); await move('down', 4);
  await move('left', 11); await move('down', 1); await move('left', 1);
  assert((await observe()).dialogue.some(line => line.text.includes('Wait, CHRIS!')));
  for (let i = 0; i < 32 && !(await observe()).status_line.startsWith('Overworld'); i++) await press('a');
  assert.match((await observe()).status_line, /NewBarkTown.*\(5, 8\)/);
  await page.evaluate(async () => {
    const tool = (await document.modelContext.getTools()).find(t => t.name === 'geothite_tui_save');
    await document.modelContext.executeTool(tool, {});
  });
  const saved = await observe();
  await page.reload();
  await page.waitForFunction(() => !document.getElementById('screen').hidden, { timeout: 120000 });
  assert.deepEqual((await observe()).marker, saved.marker);
  assert.equal((await observe()).viewport_title, saved.viewport_title);
  const fixture = await readFile(resolve(output, 'battle.crystalsave'));
  await page.evaluate(base64 => {
    const key = Object.keys(localStorage).find(key => key.startsWith('crystal.save.v1.') && key.endsWith('-tui-local.crystalsave'));
    if (!key) throw new Error('TUI save key not found');
    localStorage.setItem(key, base64);
  }, fixture.toString('base64'));
  await page.reload();
  await page.waitForFunction(() => !document.getElementById('screen').hidden, { timeout: 120000 });
  assert.match((await observe()).status_line, /Route29.*\(46, 12\)/);
  assert((await observe()).viewport.some(line => /[↓↘↙]/.test(line.text)), 'Route29 ledges must be visible');
  for (let i = 0; i < 256 && !(await observe()).status_line.includes('Battle'); i++) await press(Math.floor(i / 2) % 2 === 0 ? 'right' : 'left');
  assert.match((await observe()).status_line, /Battle/);
  assert(!(await page.locator('#screen').textContent()).includes('sprite unavailable'), 'Real front and back assets must load');
  assert((await page.locator('#screen').textContent()).includes('█'), 'Live HP bars are painted');
  let sawFight = false, sawMoves = false, sawResult = false, repeated = 0;
  for (let i = 0; i < 256; i++) {
    const view = await observe();
    sawFight ||= view.menu.some(line => line.text.includes('FIGHT'));
    sawMoves ||= view.menu.some(line => /PP|SCRATCH|BITE|WATER GUN|RAGE/i.test(line.text));
    sawResult ||= view.dialogue.some(line => /fainted|EXP|experience/i.test(line.text));
    if (!view.status_line.includes('Battle') && !view.status_line.startsWith('Text')) break;
    const after = await press('a');
    const state = value => JSON.stringify([value.status_line, value.menu, value.dialogue, value.viewport]);
    repeated = state(view) === state(after) ? repeated + 1 : 0;
    assert(repeated <= 1, 'Battle is waiting for invisible animation/idle frames');
  }
  assert(sawFight && sawMoves && sawResult, `Missing battle presentation: ${JSON.stringify({ sawFight, sawMoves, sawResult })}`);
  assert.match((await observe()).status_line, /Overworld/);
  await move('up', 2);
  await press('start');
  assert((await observe()).menu.some(line => line.text.includes('SAVE')));
  await press('b');
  if (process.env.TUI_TEST_LOWLEVEL_FIXTURE) {
    const low = await readFile(resolve(process.env.TUI_TEST_LOWLEVEL_FIXTURE));
    await page.evaluate(base64 => {
      const key = Object.keys(localStorage).find(key => key.startsWith('crystal.save.v1.') && key.endsWith('-tui-local.crystalsave'));
      localStorage.setItem(key, base64);
    }, low.toString('base64'));
    await page.reload();
    await page.waitForFunction(() => !document.getElementById('screen').hidden, { timeout: 120000 });
    await page.emulateMedia({ reducedMotion: 'no-preference' });
    for (let i = 0; i < 256 && !(await observe()).status_line.includes('Battle'); i++) await press(Math.floor(i / 2) % 2 ? 'right' : 'left');
    let turns = 0, repeats = 0, replayChecked = false;
    for (let i = 0; i < 256; i++) {
      const before = await observe();
      if (before.status_line.startsWith('Overworld')) break;
      if (before.menu.some(line => /SCRATCH|TACKLE|EMBER/.test(line.text))) turns++;
      await page.keyboard.press('a');
      const after = await observe();
      if (!replayChecked && await page.locator('#screen').getAttribute('data-animation') === 'true') {
        const frames = new Set();
        for (let frame = 0; frame < 90; frame++) {
          frames.add(await page.locator('#screen').innerHTML());
          await page.waitForTimeout(50);
          assert.deepEqual(await observe(), after, 'Cosmetic replay never advances authoritative gameplay or semantic observation');
          if (frames.size>1 || await page.locator('#screen').getAttribute('data-animation')!=='true')break;
        }
        assert(frames.size > 1, `Actual WASM retains moving ASCII battle frames, not a static animation flag: ${JSON.stringify(after.status_line)} ${JSON.stringify(after.dialogue)}`);
        await page.screenshot({ path: resolve(output, 'ascii-battle-replay.png') });
        // Real WebMCP input interrupts, even while presentation is playing.
        await page.evaluate(async () => {
          const tool = (await document.modelContext.getTools()).find(t => t.name === 'geothite_tui_press');
          await document.modelContext.executeTool(tool, { button: 'a' });
        });
        replayChecked = true;
      }
      const visible = v => JSON.stringify([v.status_line, v.menu, v.dialogue, v.viewport, v.info]);
      repeats = visible(before) === visible(after) ? repeats + 1 : 0;
      assert(repeats <= 1, `Starter-level battle froze: ${visible(after)}`);
    }
    assert(turns >= 2, `Starter-level regression must execute multiple real turns; got ${turns}`);
    assert(replayChecked, 'A real starter-level turn must produce an authored ASCII replay');
    assert.match((await observe()).status_line, /^Overworld/, 'Multi-turn battle exits cleanly');
    await move('up', 2);
    await press('start');
    assert((await observe()).menu.some(line => line.text.includes('SAVE')));
    await press('b');
    await page.emulateMedia({ reducedMotion: 'reduce' });
    console.log(`Starter-level battle: ${turns} real turns, no invisible-animation wait, movement and Start afterward passed.`);
  }
  if (process.env.TUI_TEST_MENU_FIXTURE) {
    const menuFixture = await readFile(resolve(process.env.TUI_TEST_MENU_FIXTURE));
    await page.evaluate(base64 => {
      const key = Object.keys(localStorage).find(key => key.startsWith('crystal.save.v1.') && key.endsWith('-tui-local.crystalsave'));
      localStorage.setItem(key, base64);
    }, menuFixture.toString('base64'));
    await page.reload(); await page.evaluate(() => window.geothiteTui.ready);
    const toolPress = button => page.evaluate(async button => {
      const tool = (await document.modelContext.getTools()).find(tool => tool.name === 'geothite_tui_press');
      await document.modelContext.executeTool(tool, { button });
    }, button);
    let keyboardTurn = false;
    const menuPress = async button => {
      keyboardTurn = !keyboardTurn;
      if (keyboardTurn) await page.keyboard.press(({ a: 'a', b: 'x', start: 'Enter', select: 'Tab', up: 'ArrowUp', down: 'ArrowDown', left: 'ArrowLeft', right: 'ArrowRight' })[button]);
      else await toolPress(button);
    };
    await checkMenus(observe, menuPress, name => page.screenshot({ path: resolve(output, `menu-${name}.png`) }));
    console.log('Chrome keyboard + real WebMCP: Pack, Dex, all Gear cards, outgoing call and post-menu movement passed.');
  }
  await page.screenshot({ path: resolve(output, 'desktop.png') });
  assert.deepEqual(errors, []);
  console.log('Chromium: keyboard + WebMCP, Mom, gate, save/resume, battle victory, post-battle movement and Start passed.');

  const mobile = await webkit.launch({ headless: true });
  try {
    const page = await mobile.newPage({ viewport: { width: 390, height: 844 }, isMobile: true, hasTouch: true });
    await page.goto(url);
    await page.waitForFunction(() => !document.getElementById('screen').hidden, { timeout: 120000 });
    assert.equal(await page.locator('#touch-controls button:visible').count(), 8, 'Game Boy buttons are mobile-only');
    assert.equal(await page.locator('#screen').getAttribute('data-view'), 'painted', 'Painted is default on mobile');
    await checkVisibleInkMotion(page, false);
    await page.emulateMedia({reducedMotion:'no-preference'});
    const toggleMarker = (await page.evaluate(() => window.geothiteTui.observe())).marker;
    await page.locator('#view-toggle').tap();
    assert.equal(await page.locator('#screen').getAttribute('data-view'), 'text');
    assert.deepEqual((await page.evaluate(() => window.geothiteTui.observe())).marker, toggleMarker);
    assert.equal(await page.locator('#touch-controls button:visible').count(), 8, 'Text view retains touchscreen inputs');
    await page.locator('#view-toggle').tap();
    for (const button of await page.locator('#touch-controls button').all()) {
      const box = await button.boundingBox();
      assert(box.width >= 44 && box.height >= 44, 'Thumb targets must be at least 44px');
    }
    // WebKit's native touch tap exercises A on the terminal, no surrounding UI.
    await page.tap('#screen', { position: { x: 120, y: 180 } });
    // WebKit pointer-event gesture path (Playwright's public touch API only taps).
    const gesture = async (dx, dy, hold = false) => {
      await page.evaluate(() => document.getElementById('screen').dispatchEvent(new PointerEvent('pointerdown', { pointerType: 'touch', pointerId: 7, clientX: 100, clientY: 150, bubbles: true })));
      if (hold) await page.waitForTimeout(550);
      await page.evaluate(({ dx, dy }) => document.getElementById('screen').dispatchEvent(new PointerEvent('pointerup', { pointerType: 'touch', pointerId: 7, clientX: 100 + dx, clientY: 150 + dy, bubbles: true })), { dx, dy });
    };
    await page.locator('#touch-controls [data-button="right"]').tap();
    if (!(await page.evaluate(() => window.geothiteTui.observe())).status_line.includes('(4, 3)')) await page.locator('#touch-controls [data-button="right"]').tap();
    assert.match((await page.evaluate(() => window.geothiteTui.observe())).status_line, /\(4, 3\)/);
    await page.locator('#touch-controls [data-button="start"]').tap();
    assert((await page.evaluate(() => window.geothiteTui.observe())).menu.some(line => line.text.includes('PACK')));
    const beforeRotation = await page.evaluate(() => window.geothiteTui.observe());
    await page.setViewportSize({ width: 844, height: 390 });
    assert.deepEqual((await page.evaluate(() => window.geothiteTui.observe())).marker,beforeRotation.marker,'Rotation cannot change gameplay');
    for (const button of await page.locator('#touch-controls button').all()) {
      const box=await button.boundingBox();
      assert(box.width>=44 && box.height>=44 && box.x>=0 && box.y>=0 && box.x+box.width<=844 && box.y+box.height<=390,'Landscape controls stay usable');
    }
    await page.setViewportSize({ width: 390, height: 844 });
    await page.locator('#touch-controls [data-button="b"]').tap();
    await page.evaluate(base64 => {
      window.geothiteTui.save();
      const key = Object.keys(localStorage).find(key => key.startsWith('crystal.save.v1.') && key.endsWith('-tui-local.crystalsave'));
      localStorage.setItem(key, base64);
    }, fixture.toString('base64'));
    await page.reload();
    await page.waitForFunction(() => !document.getElementById('screen').hidden, { timeout: 120000 });
    const mobileObserve = () => page.evaluate(() => window.geothiteTui.observe());
    const tap = button => page.locator(`#touch-controls [data-button="${button}"]`).tap();
    assert.match((await mobileObserve()).status_line, /Route29.*\(46, 12\)/);
    for (let i=0; i<256 && !(await mobileObserve()).status_line.includes('Battle'); i++) await tap(Math.floor(i/2)%2 ? 'right' : 'left');
    assert((await mobileObserve()).status_line.includes('Battle'));
    let fight=false, moves=false;
    const battleMenus=[];
    for (let i=0; i<256 && !(await mobileObserve()).status_line.startsWith('Overworld'); i++) {
      const view=await mobileObserve();
      fight ||= view.menu.some(line=>line.text.includes('FIGHT'));
      moves ||= view.menu.some(line=>/PP|SCRATCH|BITE|WATER GUN|RAGE/i.test(line.text));
      if(view.menu.length) battleMenus.push(view.menu.map(line=>line.text));
      await tap('a');
    }
    assert(fight && moves, `Mobile production battle/move menus: ${JSON.stringify({fight,moves,battleMenus})}`);
    assert((await mobileObserve()).status_line.startsWith('Overworld'));
    const previous=(await mobileObserve()).marker;
    await tap('up'); await tap('up');
    assert.notDeepEqual((await mobileObserve()).marker,previous,'Mobile movement after victory');
    await tap('start');
    assert((await mobileObserve()).menu.some(line=>line.text.includes('SAVE')));
    await page.screenshot({ path: resolve(output, 'mobile.png') });
    if (process.env.TUI_TEST_MENU_FIXTURE) {
      const menuFixture = await readFile(resolve(process.env.TUI_TEST_MENU_FIXTURE));
      await page.evaluate(base64 => {
        const key = Object.keys(localStorage).find(key => key.startsWith('crystal.save.v1.') && key.endsWith('-tui-local.crystalsave'));
        localStorage.setItem(key, base64);
      }, menuFixture.toString('base64'));
      await page.reload(); await page.evaluate(() => window.geothiteTui.ready);
      await checkMenus(mobileObserve, tap, name => page.screenshot({ path: resolve(output, `mobile-menu-${name}.png`) }));
      console.log('Mobile WebKit touch: Pack, Dex, Gear phone/radio/map and movement passed.');
    }
    console.log('Mobile WebKit: 44px buttons, movement, Start, real battle/move menus, victory and post-battle movement passed.');
  } finally { await mobile.close(); }
} finally { await browser.close(); server.close(); }
