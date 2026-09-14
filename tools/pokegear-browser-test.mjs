import { chromium } from 'playwright';
import { readFile, writeFile } from 'node:fs/promises';
import assert from 'node:assert/strict';

const directory = new URL('../output/ui-fidelity/', import.meta.url);
const save = await readFile(new URL('pokegear-browser.crystalsave', directory));
const browser = await chromium.launch({ headless: false });
try {
  const context = await browser.newContext({ viewport: { width: 800, height: 800 } });
  await context.addInitScript(value => localStorage.setItem(
    'crystal.save.v1.saves/core-modular+all-251-catchable-v1+realtime-clock-local.crystalsave', value), save.toString('base64'));
  const page = await context.newPage();
  const errors = [];
  page.on('pageerror', error => { errors.push(error.message); console.error(error.stack); });
  page.on('console', message => { if (message.type() === 'error') console.error(message.text()); });
  await page.goto(new URL('/?multiplayer=off', process.env.GEOTHITE_URL ?? 'http://localhost:8091').href);
  await page.waitForFunction(() => document.querySelector('#startup-error').open ||
    (document.querySelector('#loading').hidden && !document.querySelector('#touch-controls').disabled),
    null, { timeout: 180000 });
  await page.screenshot({ path: new URL('browser-startup.png', directory).pathname });
  assert.equal(await page.locator('#startup-error').evaluate(dialog => dialog.open ? dialog.textContent : null), null);
  await page.evaluate(async () => {
    const entry = [...document.scripts].map(script => script.textContent).join('\n')
      .match(/import\(['"]([^'"]*crystal-bevy[^'"]*\.js)['"]\)/)?.[1];
    if (!entry) throw new Error('The page must declare its game bundle');
    const wasm = await import(new URL(entry, location.href).href);
    const { createGameBridge } = await import('/webmcp.js');
    window.qaBridge = createGameBridge(wasm);
  });
  const observe = () => page.evaluate(() => qaBridge.execute({ kind: 'observe' }));
  const press = button => page.evaluate(button => qaBridge.execute({ kind: 'press', button, frames: 1 }), button);
  const menu = (state, kind) => state.observe.menus.find(menu => menu.kind === kind);
  let state = await press('start');
  for (let index = 0; index < 8 && !menu(state, 'start')?.entries.some(entry => /^>.*GEAR/.test(entry)); index++) {
    assert.ok(menu(state, 'start'), JSON.stringify(state));
    state = await press('down');
  }
  assert.ok(menu(state, 'start')?.entries.some(entry => /^>.*GEAR/.test(entry)));
  await page.keyboard.down('z');
  await page.waitForTimeout(2100);
  state = await observe();
  assert.ok(menu(state, 'pokegear'), 'held opening A must leave Pokégear open');
  const clock = menu(state, 'pokegear').entries;
  await page.screenshot({ path: new URL('browser-pokegear-clock.png', directory).pathname });
  await page.keyboard.press('ArrowRight', { delay: 100 });
  await page.waitForTimeout(250);
  state = await observe();
  assert.ok(menu(state, 'pokegear'));
  assert.notDeepEqual(menu(state, 'pokegear').entries, clock, 'new direction must switch to map while A is held');
  await page.screenshot({ path: new URL('browser-pokegear-map.png', directory).pathname });
  await page.keyboard.press('ArrowLeft', { delay: 100 });
  await page.waitForTimeout(250);
  assert.deepEqual(menu(await observe(), 'pokegear').entries, clock);
  await page.keyboard.up('z');
  await page.waitForTimeout(100);
  await page.keyboard.press('z', { delay: 100 });
  await page.waitForTimeout(500);
  state = await observe();
  assert.equal(menu(state, 'pokegear'), undefined);
  assert.ok(menu(state, 'start'));
  assert.deepEqual(errors, []);
  await writeFile(new URL('browser-pokegear-result.json', directory), JSON.stringify({ passed: true, state }, null, 2));
  console.log('PASS: browser held-A opening, card navigation, release/repress exit');
  for (let index = 0; index < 8 && !menu(state, 'start')?.entries.includes('>#MON'); index++) {
    state = await press('down');
  }
  assert.ok(menu(state, 'start')?.entries.includes('>#MON'));
  state = await press('a');
  assert.ok(menu(state, 'party'));
  await page.screenshot({ path: new URL('browser-pokemon-menu.png', directory).pathname });
  await press('a');
  await press('a');
  await page.waitForTimeout(1800);
  for (let number = 1; number <= 3; number++) {
    state = await observe();
    assert.equal(state.recent_events.error, null);
    await page.screenshot({ path: new URL(`browser-pokemon-stats-${number}.png`, directory).pathname });
    if (number < 3) await press('right');
  }
  await press('b');
  state = await press('b');
  assert.ok(menu(state, 'start'));
  assert.deepEqual(errors, []);
  console.log('PASS: browser party menu, three stats pages, return to Start');

} finally {
  await browser.close();
}
