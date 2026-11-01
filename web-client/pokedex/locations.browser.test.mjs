import assert from 'node:assert/strict';
import fs from 'node:fs';
import http from 'node:http';
import path from 'node:path';
import { chromium } from 'playwright';

const release = path.resolve(process.env.POKEDEX_WEB_RELEASE || '');
assert.ok(process.env.POKEDEX_WEB_RELEASE);
const prefix = `/pokedex/releases/${path.basename(release)}/`;
const server = http.createServer((request, response) => {
  const url = new URL(request.url, 'http://localhost');
  let relative;
  if (url.pathname.startsWith(prefix)) relative = url.pathname.slice(prefix.length);
  else if (url.pathname.startsWith('/pokedex/species/')) relative = `species/${path.basename(url.pathname)}`;
  else if (url.pathname === '/pokedex/') relative = 'index.html';
  const file = relative && path.resolve(release, relative);
  if (!file || !file.startsWith(`${release}/`) || !fs.existsSync(file) || !fs.statSync(file).isFile()) { response.writeHead(404).end(); return; }
  response.writeHead(200, {'Content-Type': {'.html':'text/html','.css':'text/css','.js':'text/javascript','.png':'image/png','.svg':'image/svg+xml'}[path.extname(file)] || 'application/octet-stream'});
  fs.createReadStream(file).pipe(response);
});
await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
const base = process.env.POKEDEX_WEB_URL || `http://127.0.0.1:${server.address().port}/pokedex/`;
const captures = process.env.POKEDEX_LOCATIONS_CAPTURES;
const progression = process.env.POKEDEX_EXPECT_PROGRESSION === '1';
if (captures) fs.mkdirSync(captures, {recursive:true});
const browser = await chromium.launch({headless:true, channel:process.env.POKEDEX_BROWSER_CHANNEL || 'chromium'});
try {
  for (const width of [1280,390]) {
    const page = await browser.newPage({viewport:{width,height:900}, reducedMotion:'reduce'});
    const errors = [];
    page.on('pageerror', error => errors.push(error.message));
    for (const [name, expected] of [['oatlet','Eastern harvest fields'],['cinderfin','Smokehouse'],['wickit','Lighthouse'],['trilume','Care Badge'],['driptip','Cave floor'],['keypout','Iron Range Pump Sump'],['scutlet','signed brick opening'],['leechimp','Pine Island'],['burrbud','Headbutt'],['burroot','Evolve'],['verdigrim','clean-bypass'],['pinit','Tall Tale'],['pailbaron',progression ? 'Evolve' : 'Minnesota State Fair'],['curdlet','Minnesota State Fair'],['pipulse','Free level-8']]) {
      await page.goto(`${base}species/${name}.html`);
      const panel = page.locator('.locations-panel');
      await panel.scrollIntoViewIfNeeded();
      assert.ok((await panel.textContent()).includes(expected), `${name} location content`);
      assert.ok(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth), `${name} fits ${width}px`);
      if (captures && ['cinderfin','trilume','verdigrim','scutlet','driptip','leechimp','pinit','pailbaron','curdlet','pipulse'].includes(name)) await panel.screenshot({path:path.join(captures, `${name}-${width}.png`)});
      if (['scutlet','leechimp'].includes(name)) {
        const details = panel.locator('details');
        if (await details.count()) await details.locator('summary').click();
        const row = panel.locator('li').filter({hasText: name === 'scutlet' ? 'signed brick opening' : 'Pine Island'}).first();
        await row.waitFor({state:'visible'});
        if (captures) await row.screenshot({path:path.join(captures, `${name}-habitat-${width}.png`)});
      }
    }
    await page.goto(base);
    await page.locator('#dex-search').fill('Smokehouse');
    await page.locator('.pokemon-card[data-name="Cinderfin"]:not([hidden])').waitFor();
    assert.ok((await page.locator('#result-count').textContent()).includes('Pokémon'));
    await page.locator('#dex-search').fill('Harvest Oats');
    await page.locator('.pokemon-card[data-name="Oatlet"]:not([hidden])').waitFor();
    await page.locator('#dex-search').fill('Voyageurs Ice Cavern');
    await page.locator('.pokemon-card[data-name="Driftpeg"]:not([hidden])').waitFor();
    await page.locator('#dex-search').fill('Moorhead');
    await page.locator('.pokemon-card[data-name="Scutlet"]:not([hidden])').waitFor();
    await page.locator('#dex-search').fill('St Paul');
    await page.locator('.pokemon-card[data-name="Pinglet"]:not([hidden])').waitFor();
    assert.deepEqual(errors, []);
    await page.close();
  }
  console.log('PASS: desktop/mobile locations, gifts, evolutions, all rare discoveries, specialist habitats, search, no overflow or page errors.');
} finally { await browser.close(); await new Promise(resolve => server.close(resolve)); }
