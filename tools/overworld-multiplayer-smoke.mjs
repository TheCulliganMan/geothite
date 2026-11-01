/** Real hosted server + WASM clients. Generate saves with hosted_overworld_fixture
 * against the server's exact composed pack; never use production save slots.
 * Usage: node tools/overworld-multiplayer-smoke.mjs URL FIXTURE_DIRECTORY
 * The local server must use CRYSTAL_AUTH_SECRET below (or MP_TEST_SECRET).
 */
import { chromium } from 'playwright';
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import path from 'node:path';
import { createHmac } from 'node:crypto';

const origin = process.argv[2] ?? 'http://127.0.0.1:18743/';
const directory = path.resolve(process.argv[3] ?? 'target/hosted-overworld-fixtures');
const fixture = JSON.parse(await fs.readFile(path.join(directory, 'fixtures.json'), 'utf8'));
const secret = process.env.MP_TEST_SECRET ?? 'hosted-overworld-test-secret-local-only-2026';
const browser = await chromium.launch({channel:'chrome', headless:true, args:[
  '--disable-background-timer-throttling', '--disable-renderer-backgrounding',
  '--use-gl=angle', '--use-angle=swiftshader', '--enable-unsafe-swiftshader',
]});
const clients = [];
const errors = [];
const checks = [];
checks.push = function (...items) { for (const item of items) console.log(`PASS ${item}`); return Array.prototype.push.apply(this,items); };
const observe = page => page.evaluate(() => mpBridge.execute({kind:'observe'}));
const press = (page, button, frames = 1) => page.evaluate(({button,frames}) => mpBridge.execute({kind:'press',button,frames}), {button,frames});
const until = async (fn, label, timeout = 30000) => {
  const deadline = Date.now() + timeout;
  while (Date.now() < deadline) { if (await fn()) return; await new Promise(resolve => setTimeout(resolve,150)); }
  throw new Error(`Timed out: ${label}`);
};
const settled = async page => { await until(async () => !(await observe(page)).flow_state.animating, 'walk settles'); return observe(page); };
const boot = async (page, url) => {
  await page.goto(url);
  await page.waitForFunction(() => document.querySelector('#loading').hidden || document.querySelector('#startup-error').open, null, {timeout:180000});
  assert.equal(await page.locator('#startup-error').evaluate(el => el.open), false, await page.locator('#startup-error').textContent());
  await page.evaluate(async () => {
    const script = [...document.querySelectorAll('script[type="module"]')].map(node => node.textContent).join('\n');
    const modulePath = script.match(/import\('([^']*crystal-bevy[^']*\.js)'\)/)?.[1];
    if (!modulePath) throw new Error('Game module import missing from the served page');
    const wasm = await import(modulePath);
    const {createGameBridge} = await import('./webmcp.js');
    window.mpBridge = createGameBridge(wasm,{timeoutMs:60000});
  });
  await page.locator('.chat-status[data-connected="true"]').waitFor({state:'attached',timeout:60000});
};
const say = async (page, message) => {
  if (!await page.locator('#chat-message').isVisible()) await page.locator('.chat-toggle').click();
  await page.locator('#chat-message').fill(message);
  await page.locator('button[aria-label="Send message"]').click();
};
const close = async page => { if (await page.locator('.chat-close').isVisible()) await page.locator('.chat-close').click(); };
const logHas = (page,text) => page.locator('.chat-log').textContent().then(log => log.includes(text));
try {
  for (const [index, player] of fixture.players.entries()) {
    const context = await browser.newContext({viewport:index === 2 ? {width:390,height:844} : {width:1024,height:768},hasTouch:index === 2,isMobile:index === 2});
    const save = (await fs.readFile(path.join(directory,player.save))).toString('base64');
    await context.addInitScript(({key,save}) => { if (!localStorage.getItem(key)) localStorage.setItem(key,save); }, {key:`crystal.save.v1.saves/${fixture.modpack_id}-player-${player.id}.crystalsave`,save});
    const page = await context.newPage();
    page.on('pageerror', error => errors.push(`${player.name}: ${error.message}`));
    const payload = Buffer.from(JSON.stringify({user_id:`player-${player.id}`,expires_at:Math.floor(Date.now()/1000)+3600})).toString('base64url');
    const token = `${payload}.${createHmac('sha256',secret).update(payload).digest('base64url')}`;
    const url = new URL(origin); url.searchParams.set('token',token); url.searchParams.set('player_name',player.name);
    clients.push({context,page,player,url:url.href});
    await boot(page,url.href);
    console.log(`READY ${player.name}`);
    const state = await observe(page);
    assert.equal(state.map_info.name,fixture.map);
    assert.equal(state.status.player_name,player.name);
  }
  const [a,b,c] = clients.map(client=>client.page);
  await until(async () => (await observe(a)).map_info.players.length === 2, 'all remote players visible');
  assert.deepEqual((await observe(a)).map_info.players.map(p=>p.user_id).sort(),['player-41002','player-41003']);
  checks.push('three independent authenticated players and identifiable remote sprites');
  await a.locator('canvas').focus();
  await a.keyboard.press('z');
  await a.locator('.chat-actions button[data-action="time_capsule"]').waitFor({state:'visible'});
  assert.deepEqual(await a.locator('.chat-actions button').allTextContents(),['Whisper','Battle','Trade','Time Capsule']);
  await close(a);
  checks.push('physical confirm selects the facing player and opens every interaction action');

  await say(b,'walking-public-chat');
  await until(async () => (await Promise.all([a,b,c].map(p=>logHas(p,'walking-public-chat')))).every(Boolean),'nearby public chat');
  const bubble = a.locator('.speech-bubble').filter({hasText:'walking-public-chat'});
  await bubble.waitFor({state:'visible'});
  const before = await bubble.boundingBox();
  await close(b);
  const old = (await settled(b)).map_info.player;
  await press(b,'right',12);
  await press(b,'right',12); // First directional tap may only turn the trainer.
  const moved = (await settled(b)).map_info.player;
  assert.notEqual(moved.x,old.x,'real walking moves sender');
  await until(async () => (await observe(a)).map_info.players.some(p=>p.user_id==='player-41002' && p.x===moved.x && p.y===moved.y), 'remote movement replication');
  await until(async () => {const box=await bubble.boundingBox();return box && Math.abs(box.x-before.x)>2;},'speech bubble follows rendered remote sprite');
  checks.push('live walking replication and speech bubbles following remote sprites');

  await say(a,'/w player-41002 private-pair-message');
  await until(async () => await logHas(b,'private-pair-message'),'private whisper delivered');
  assert.equal(await logHas(c,'private-pair-message'),false);
  assert.equal(await c.locator('.speech-bubble').filter({hasText:'private-pair-message'}).count(),0);
  await say(b,'/r private-reply-message');
  await until(async () => await logHas(a,'private-reply-message'),'whisper reply delivered');
  assert.equal(await logHas(c,'private-reply-message'),false);
  checks.push('whisper and reply privacy with a third player present');

  await say(c,'/w player-41001 mobile-private-message');
  await until(async () => await logHas(a,'mobile-private-message'),'mobile chat reaches selected peer');
  await close(c);
  const dpad = await c.locator('#dpad').boundingBox();
  assert.ok(dpad,'real mobile direction control is available');
  await c.touchscreen.tap(dpad.x+dpad.width*.85,dpad.y+dpad.height*.5);
  await settled(c);
  await c.touchscreen.tap(dpad.x+dpad.width*.85,dpad.y+dpad.height*.5);
  const mobileMoved = (await settled(c)).map_info.player;
  assert.ok(mobileMoved.x>14,'touch moves the actual player');
  await until(async () => (await observe(a)).map_info.players.some(p=>p.user_id==='player-41003' && p.x===mobileMoved.x),'mobile walking replicated to desktop');
  checks.push('mobile touch walking and player-to-player chat');

  await close(a); await a.locator('canvas').focus();
  await a.keyboard.down('ArrowRight');
  await a.keyboard.press('Enter');
  await a.locator('#chat-message').waitFor({state:'visible'});
  const focused = (await settled(a)).map_info.player;
  await a.locator('#chat-message').pressSequentially('wasdzx',{delay:80});
  assert.deepEqual((await settled(a)).map_info.player,focused,'typing and held keys cannot drive gameplay');
  await a.keyboard.up('ArrowRight'); await a.keyboard.press('Escape');
  await press(a,'down',12);
  await press(a,'down',12);
  assert.notDeepEqual((await settled(a)).map_info.player,focused,'movement resumes after chat');
  checks.push('held-key handoff, safe typing, and movement after closing chat');

  for (const command of ['battle','tradewith','timecapsule']) {
    await close(b);
    const beforeInvite = (await settled(b)).map_info.player;
    const walkingKey = command==='tradewith' ? 'ArrowUp' : 'ArrowDown';
    await b.locator('canvas').focus();
    await b.keyboard.down(walkingKey);
    await say(a,`/${command} player-41002`);
    await b.locator('.chat-requests button[data-action="decline"]').waitFor({state:'visible'});
    await b.keyboard.up(walkingKey);
    assert.notDeepEqual((await settled(b)).map_info.player,beforeInvite,'the recipient remains free to walk while invited');
    await b.locator('.chat-requests button[data-action="decline"]').click();
    await until(async () => (await observe(b)).multiplayer?.pending_request == null,`${command} decline releases recipient`);
    await until(async () => await b.locator('.chat-requests button').count() === 0,`${command} invitation cleaned`);
    await close(a);
  }
  checks.push('Battle, Trade, and Time Capsule invitations decline and release both players');

  await close(b); await press(b,'start');
  assert.ok((await observe(b)).observe.menus.some(menu=>menu.kind==='start'));
  const declines = (await a.locator('.chat-log').textContent()).match(/Request declined or cancelled\./g)?.length ?? 0;
  await say(a,'/battle player-41002');
  await until(async () => ((await a.locator('.chat-log').textContent()).match(/Request declined or cancelled\./g)?.length ?? 0)>declines && (await observe(b)).multiplayer.pending_request == null,'busy player declines invitation');
  assert.ok((await observe(b)).observe.menus.some(menu=>menu.kind==='start'),'incoming invitation preserves existing menu');
  await press(b,'b'); await close(a);
  checks.push('invitations preserve existing gameplay menus');

  await say(a,'/tradewith player-41002');
  await b.locator('.chat-requests button[data-action="accept"]').waitFor({state:'visible'});
  await say(a,'/cancel');
  await until(async () => await b.locator('.chat-requests button').count()===0,'sender cancellation removes recipient invitation');
  await close(a); await close(b);
  checks.push('sender cancellation removes both invitation states');

  for (const command of ['tradewith','timecapsule']) {
    const parties = await Promise.all([a,b].map(async page => (await observe(page)).status.party[0].nickname));
    await say(a,`/${command} player-41002`);
    await b.locator('.chat-requests button[data-action="accept"]').waitFor({state:'visible'});
    await b.locator('.chat-requests button[data-action="accept"]').click();
    await until(async () => (await Promise.all([a,b].map(async page => (await observe(page)).observe.menus.some(m=>m.kind==='party')))).every(Boolean),`${command} opens real party selection`,60000);
    assert.ok((await observe(a)).multiplayer.connected,'social connection remains available in a link session');
    await say(a,`${command}-session-chat`);
    await until(async () => await logHas(b,`${command}-session-chat`),'chat remains live during an exchange');
    await close(a);
    await Promise.all([a,b].map(page=>press(page,'a')));
    await until(async () => (await Promise.all([a,b].map(async page => (await observe(page)).observe.menus.some(m=>m.kind==='yes_no')))).every(Boolean),`${command} shows real exchange confirmation`);
    await Promise.all([a,b].map(page=>press(page,'a')));
    await until(async () => {
      const received = await Promise.all([a,b].map(async page=>(await observe(page)).status.party[0].nickname));
      return received[0]===parties[1] && received[1]===parties[0];
    },`${command} exchanges actual party Pokemon`,60000);
    await until(async () => (await Promise.all([a,b].map(async page => (await observe(page)).observe.menus.some(m=>m.kind==='party')))).every(Boolean),`${command} allows another trade`);
    await press(a,'b');
    await until(async () => (await Promise.all([a,b].map(async page=>!(await observe(page)).multiplayer.session_active))).every(Boolean),`${command} exits on both peers`,60000);
    await Promise.all([a,b].map(page=>page.locator('.chat-status[data-connected="true"]').waitFor({state:'attached',timeout:60000})));
    await press(b,'start');
    assert.ok((await observe(b)).observe.menus.some(menu=>menu.kind==='start'));
    await press(b,'b');
    const received = (await observe(b)).status.party[0].nickname;
    await boot(b,clients[1].url);
    assert.equal((await observe(b)).status.party[0].nickname,received,'the completed exchange survives a real browser reload');
    await until(async () => (await observe(a)).map_info.players.some(p=>p.user_id==='player-41002'),'exchanged peer reconnects');
    checks.push(`${command} completes a confirmed exchange, saves it, and returns both players to usable overworld`);
  }

  await say(a,'/battle player-41002');
  await b.locator('.chat-requests button[data-action="accept"]').waitFor({state:'visible'});
  await b.locator('.chat-requests button[data-action="accept"]').click();
  await until(async () => (await Promise.all([a,b].map(async page=>(await observe(page)).status.screen==='battle'))).every(Boolean),'real link battle begins',60000);
  await close(a); await close(b);
  let battleFinished = false;
  let damaged = false;
  for (let step=0;step<600;step++) {
    const states = await Promise.all([a,b].map(observe));
    damaged ||= states.some(state=>state.status.party[0].hp < state.status.party[0].max_hp);
    if (step % 30 === 0) console.log(`BATTLE step ${step}: ${states.map(state=>`${state.status.screen} HP ${state.status.party[0].hp}`).join(' / ')}`);
    if (states.every(state=>state.status.screen==='overworld' && !state.multiplayer.session_active)) {battleFinished=true;break;}
    await Promise.all([a,b].map(async (page,index)=> {
      if (states[index].flow_state.animating || states[index].status.screen!=='battle') return;
      const menu = states[index].observe.menus.find(m=>m.kind==='battle');
      if (menu && menu.surface==='commands') {
        assert.ok(menu.entries.some(entry=>entry.includes('FIGHT')),'real battle command menu');
      }
      // These level-five fixture parties lead with their authored damaging move.
      await press(page,'a');
    }));
    await new Promise(resolve=>setTimeout(resolve,120));
  }
  assert.ok(damaged,'link turns change actual HP');
  assert.ok(battleFinished,'link battle reaches a settled result on both peers');
  for (const page of [a,b]) {await press(page,'start');assert.ok((await observe(page)).observe.menus.some(menu=>menu.kind==='start'));await press(page,'b');}
  checks.push('complete synchronized link battle, real HP changes, result settlement, and usable Start afterward');

  const saved = (await settled(b)).map_info.player;
  await press(b,'start'); await press(b,'b');
  await b.keyboard.press('F5');
  await new Promise(resolve=>setTimeout(resolve,400));
  await boot(b,clients[1].url);
  assert.deepEqual((await settled(b)).map_info.player,saved,'reload resumes exact position and facing');
  await until(async () => (await observe(a)).map_info.players.some(p=>p.user_id==='player-41002'),'reconnected remote player visible');
  await press(b,'start');
  assert.ok((await observe(b)).observe.menus.some(menu=>menu.kind==='start'),'Start remains usable after reconnect');
  await press(b,'b');
  // Enter the actual house through its door using only normal movement.
  for (const [axis,target,negative,positive] of [['x',13,'left','right'],['y',6,'up','down']]) {
    for (let taps=0;taps<20 && (await settled(b)).map_info.player[axis]!==target;taps++) {
      const pos=(await observe(b)).map_info.player[axis];
      await press(b,pos>target?negative:positive,8);
    }
    assert.equal((await settled(b)).map_info.player[axis],target);
  }
  await press(b,'up',8); await press(b,'up',8);
  await until(async () => (await observe(b)).map_info.name!==fixture.map,'normal door warp changes map',60000);
  await until(async () => !(await observe(a)).map_info.players.some(p=>p.user_id==='player-41002'),'map transition removes former remote sprite');
  await say(b,'inside-house-local-message');
  await until(async () => await logHas(b,'inside-house-local-message'),'local say echo in new map');
  assert.equal(await logHas(a,'inside-house-local-message'),false,'say does not cross maps');
  await say(b,'/w player-41001 across-map-whisper');
  await until(async () => await logHas(a,'across-map-whisper'),'private messages work across maps');
  checks.push('real door warp, map-scoped public chat, and private chat across maps');
  assert.equal((await c.locator('.chat-log').textContent()).includes('Multiplayer disconnected:'),false,'mobile has no startup or gameplay network failures');
  await clients[2].context.close();
  await until(async () => !(await observe(a)).map_info.players.some(p=>p.user_id==='player-41003'),'disconnect removes remote sprite');
  checks.push('save/reload, reconnect, usable Start, and disconnect cleanup');
  await a.screenshot({path:path.join(directory,'overworld-multiplayer.png')});
  for (const page of [a,b]) assert.equal((await page.locator('.chat-log').textContent()).includes('Multiplayer disconnected:'),false,'no hidden multiplayer failures during gameplay');
  assert.deepEqual(errors,[],'no browser runtime errors');
  console.log(JSON.stringify({passed:true,checks},null,2));
} catch (error) {
  for (const [index,client] of clients.entries()) {
    if (client.page.isClosed()) continue;
    await client.page.screenshot({path:path.join(directory,`failure-${index}.png`)}).catch(()=>{});
    const state = await observe(client.page).catch(()=>null);
    console.error(client.player.name,JSON.stringify(state && {status:state.status,player:state.map_info.player,multiplayer:state.multiplayer,observe:state.observe,recent_events:state.recent_events}),await client.page.locator('#social-chat').textContent().catch(()=>''));
  }
  throw error;
} finally { await browser.close(); }
