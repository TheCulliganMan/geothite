import { chromium } from 'playwright';
import { readFile, writeFile } from 'node:fs/promises';
import assert from 'node:assert/strict';

const kind = process.argv[2];
assert.ok(['pc', 'mart', 'elm', 'pokedex', 'battle'].includes(kind));
const directory = new URL('../output/ui-fidelity/', import.meta.url);
const save = await readFile(new URL(`${kind}-browser.crystalsave`, directory));
const browser = await chromium.launch({ headless: false });
try {
  const context = await browser.newContext({ viewport: { width: 800, height: 800 } });
  await context.addInitScript(value => localStorage.setItem(
    'crystal.save.v1.saves/core-modular+all-251-catchable-v1+realtime-clock-local.crystalsave', value), save.toString('base64'));
  const page = await context.newPage();
  const errors = [];
  page.on('pageerror', error => errors.push(error.stack));
  page.on('console', message => { if (message.type() === 'error') console.error(message.text()); });
  await page.goto('http://localhost:8091/?multiplayer=off');
  await page.waitForFunction(() => document.querySelector('#startup-error').open ||
    (document.querySelector('#loading').hidden && !document.querySelector('#touch-controls').disabled), null, { timeout: 180000 });
  assert.equal(await page.locator('#startup-error').evaluate(dialog => dialog.open ? dialog.textContent : null), null);
  await page.evaluate(async () => {
    const wasm = await import('/crystal-bevy.js');
    const { createGameBridge } = await import('/webmcp.js');
    window.qaBridge = createGameBridge(wasm);
  });
  const observe = () => page.evaluate(() => qaBridge.execute({ kind: 'observe' }));
  const press = async button => {
    if (['mart', 'battle'].includes(kind) && button === 'a') {
      await page.keyboard.down('z');
      await page.waitForTimeout(100);
      await page.keyboard.up('z');
    } else {
      await page.evaluate(button => qaBridge.execute({ kind: 'press', button, frames: 1 }), button);
    }
    await page.waitForTimeout(350);
    return observe();
  };
  const text = state => [state.observe.text, state.observe.battle, state.observe.battle_message,
    ...state.observe.rendered_text].join('\n');
  const capture = async name => {
    const state = await observe();
    assert.equal(state.recent_events.error, null);
    await page.screenshot({ path: new URL(`browser-${name}.png`, directory).pathname });
    await writeFile(new URL(`browser-${name}.json`, directory), JSON.stringify(state, null, 2));
    return state;
  };
  const advanceTo = async pattern => {
    let state = await observe();
    for (let count = 0; count < 40 && !pattern.test(text(state)); count++) {
      state = await press('a');
      assert.equal(state.recent_events.error, null);
    }
    assert.match(text(state), pattern, JSON.stringify(state));
    return state;
  };
  if (kind === 'pc') {
    await press('up');
    await advanceTo(/Access whose PC/);
    await capture('pc-hub');
    await press('a');
    await advanceTo(/SEE YA/);
    const bills = await capture('pc-bills');
    assert.match(text(bills), /MOVE.*MAIL/);
    await press('b');
    await press('down');
    await capture('pc-cursor');
    await press('b');
    const before = await observe();
    const after = await press('down');
    assert.notDeepEqual(after.map_info.player, before.map_info.player);
  } else if (kind === 'mart') {
    await capture('mart-before');
    console.log('initial', (await observe()).map_info.player);
    await page.keyboard.down('ArrowLeft');
    await page.waitForTimeout(250);
    await page.keyboard.up('ArrowLeft');
    await page.waitForTimeout(350);
    console.log('facing', (await observe()).map_info.player);
    await advanceTo(/BUY[\s\S]*SELL[\s\S]*QUIT/);
    await capture('mart-top');
    await press('a');
    await press('down');
    await press('down');
    const buy = await capture('mart-buy');
    assert.match(text(buy), /PARLYZ HEAL/);
    await press('b');
    await advanceTo(/BUY[\s\S]*SELL[\s\S]*QUIT/);
    await press('down');
    await press('a');
    const sell = await capture('mart-sell');
    assert.match(text(sell), /PARLYZ HEAL/);
  } else if (kind === 'battle') {
    await press('up');
    await advanceTo(/FIGHT/);
    await capture('battle-command');
    if (!process.argv.includes('--display-only')) {
    await advanceTo(/SWIFT/);
    let state = await capture('battle-moves');
    assert.match(text(state), /SWIFT/);
    assert.ok(state.map_info.objects.some(object => object.name === 'ROUTE36_WEIRD_TREE'),
      'post-battle disappearance must not execute before the first move');
    await press('a');
    const messages = [];
    for (let index = 0; index < 60; index++) {
      state = await capture(`battle-turn-${String(index).padStart(2, '0')}`);
      messages.push(text(state));
      if (messages.some(message => /fainted/i.test(message)) && !state.observe.battle &&
          !state.observe.battle_message) break;
      await press('a');
    }
    await writeFile(new URL('browser-battle-messages.json', directory), JSON.stringify(messages, null, 2));
    assert.ok(messages.some(message => /fainted/i.test(message)), 'finishing turn must present faint text');
    await page.waitForTimeout(1500);
    state = await capture('battle-after');
    assert.equal(state.status.screen, 'overworld');
    assert.ok(!state.map_info.objects.some(object => object.name === 'ROUTE36_WEIRD_TREE'),
      'the encounter script must finish after the battle');
    const before = state.map_info.player;
    await press('down');
    const after = (await press('down')).map_info.player;
    assert.notDeepEqual([after.x, after.y], [before.x, before.y],
      'the player must regain overworld movement after the final faint');
    }
  } else if (kind === 'pokedex') {
    let state = await press('start');
    const start = state => state.observe.menus.find(menu => menu.kind === 'start');
    for (let index = 0; index < 8 && !start(state)?.entries.some(entry => /^>.*DEX/.test(entry)); index++) {
      state = await press('down');
    }
    assert.ok(start(state)?.entries.some(entry => /^>.*DEX/.test(entry)));
    await press('a');
    state = await capture('pokedex-list');
    assert.match(text(state), /CHIKORITA/);
    await press('down'); await press('down'); await press('down');
    await press('a');
    state = await capture('pokedex-entry');
    assert.match(text(state), /CYNDAQUIL/);
    await press('a');
    await capture('pokedex-page2');
    await press('right'); await press('a');
    await capture('pokedex-area');
    await press('a'); await press('b');
    await press('select');
    state = await capture('pokedex-options');
    assert.match(text(state), /OLD/);
    await press('b'); await press('start');
    state = await capture('pokedex-search');
    assert.match(text(state), /SEARCH/);
    await press('b'); await press('b');
    assert.ok(start(await observe()));
  } else {
    await press('down');
    await page.waitForTimeout(700);
    await capture('elm-ring');
    await page.waitForTimeout(3500);
    const call = await capture('elm-disaster');
    assert.match(call.observe.text, /ElmPhoneDisasterText/);
    assert.match(text(call), /Hello|disaster|TEST/);
  }
  assert.deepEqual(errors, []);
  console.log(`PASS: browser ${kind}${process.argv.includes('--display-only') ? ' display' : ''} real interaction and rendered content`);
} finally {
  await browser.close();
}
