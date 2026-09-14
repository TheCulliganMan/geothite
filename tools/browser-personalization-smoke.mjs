/** Test the actual WASM profile menu and reload against a native-generated save fixture. */
import { chromium } from 'playwright';
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import http from 'node:http';
import path from 'node:path';

const directory = path.resolve(process.argv[2] ?? 'output/profile-browser');
const fixture = JSON.parse(await fs.readFile(path.join(directory, 'profile-fixture.json'), 'utf8'));
const save = (await fs.readFile(path.join(directory, 'profile.crystalsave'))).toString('base64');
const saveKey = `crystal.save.v1.saves/${fixture.modpack_id}-local.crystalsave`;
const mime = { '.js':'text/javascript', '.wasm':'application/wasm', '.html':'text/html', '.css':'text/css' };
const server = http.createServer(async (request, response) => {
  const route = new URL(request.url, 'http://localhost').pathname;
  if (route === '/v1/clock') {
    response.setHeader('Content-Type', 'application/json');
    response.end(JSON.stringify({ unixMillis: Date.now(), timeZone:'UTC' })); return;
  }
  const filename = path.resolve(directory, '.' + (route === '/' ? '/index.html' : route));
  if (!filename.startsWith(directory + path.sep)) { response.writeHead(403).end(); return; }
  try { response.setHeader('Content-Type', mime[path.extname(filename)] ?? 'application/octet-stream'); response.end(await fs.readFile(filename)); }
  catch { response.writeHead(404).end(); }
});
await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
const url = `http://127.0.0.1:${server.address().port}/?multiplayer=off`;
const browser = await chromium.launch({ channel:'chrome', headless:true });
const results = [];
try {
  for (const viewport of [{ width:1280,height:900 }, { width:390,height:844 }]) {
    const context = await browser.newContext({viewport});
    await context.addInitScript(({saveKey,save}) => {
      if (!localStorage.getItem(saveKey)) localStorage.setItem(saveKey, save);
    }, {saveKey,save});
    const page = await context.newPage();
    const errors = []; page.on('pageerror', error => errors.push(error.message));
    const boot = async () => {
      await page.goto(url);
      await page.waitForFunction(() => document.querySelector('#loading').hidden || document.querySelector('#startup-error').open, null, {timeout:180000});
      assert.equal(await page.locator('#startup-error').evaluate(el => el.open),false);
      await page.evaluate(async () => {
        window.profileWasm = await import('./crystal-bevy.js');
        const {createGameBridge} = await import('./webmcp.js');
        window.profileBridge = createGameBridge(window.profileWasm, {timeoutMs:60000});
      });
      await page.waitForFunction(() => JSON.parse(profileWasm.crystal_customization_poll()).can_edit, null, {timeout:60000});
    };
    await boot();
    // Enter via the actual START menu and original joypad input.
    let observed = await page.evaluate(() => profileBridge.execute({kind:'press',button:'start',frames:1}));
    let menu = observed.observe.menus.find(menu => menu.kind === 'start');
    assert.ok(menu?.entries.some(entry => entry.trim().replace(/^>/,'') === 'PROFILE'), 'default customization pack exposes PROFILE');
    for (let i=0; i<menu.entries.length && !menu.entries.some(entry => entry.startsWith('>PROFILE')); i++) {
      observed = await page.evaluate(() => profileBridge.execute({kind:'press',button:'down',frames:1}));
      menu = observed.observe.menus.find(menu => menu.kind === 'start');
    }
    assert.ok(menu.entries.some(entry => entry.startsWith('>PROFILE')));
    await page.evaluate(() => profileBridge.execute({kind:'press',button:'a',frames:1}));
    await page.locator('#personalization-dialog').waitFor({state:'visible'});
    await page.getByLabel('Trainer name',{exact:true}).fill('NOVA');
    await page.getByLabel('Online handle',{exact:true}).fill('Nova_7');
    await page.getByLabel('Player sprite',{exact:true}).selectOption('0');
    await page.getByRole('button',{name:'Save changes'}).click();
    await page.getByText('Saved. Your trainer and online profile are up to date.').waitFor({timeout:60000});
    const changed = await page.evaluate(() => JSON.parse(profileWasm.crystal_customization_poll()).profile);
    assert.deepEqual(changed,{name:'NOVA',handle:'Nova_7',sprite:0});
    await page.getByRole('button',{name:'Close',exact:true}).click();
    observed = await page.evaluate(() => profileBridge.execute({kind:'observe'}));
    assert.equal(observed.status.player_name,'NOVA');
    const screenshot = path.join(directory,`personalization-live-${viewport.width}.png`);
    await page.screenshot({path:screenshot});
    await boot();
    const restored = await page.evaluate(() => JSON.parse(profileWasm.crystal_customization_poll()).profile);
    assert.deepEqual(restored,changed,'all profile fields survive a real WASM reload');
    await page.locator('#player-options').evaluate(el => el.open=true);
    await page.getByRole('button',{name:'Personalization',exact:true}).click();
    await page.locator('#personalization-dialog').waitFor({state:'visible'});
    assert.equal(await page.getByLabel('Trainer name',{exact:true}).inputValue(),'NOVA');
    await page.keyboard.press('Escape');
    assert.equal(await page.locator('#personalization-dialog').evaluate(el=>el.open),false);
    assert.deepEqual(errors,[]);
    results.push({viewport,changed,restored,screenshot,errors});
    await context.close();
  }
} finally {
  await fs.writeFile(path.join(directory,'personalization-live-results.json'),JSON.stringify(results,null,2));
  await browser.close();
  await new Promise(resolve => server.close(resolve));
}
console.log(JSON.stringify(results,null,2));
