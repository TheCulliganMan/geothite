import assert from 'node:assert/strict';
import fs from 'node:fs';
import http from 'node:http';
import path from 'node:path';
import { chromium } from 'playwright';

assert.ok(process.env.POKEDEX_WEB_RELEASE, 'Set POKEDEX_WEB_RELEASE');
const release = path.resolve(process.env.POKEDEX_WEB_RELEASE);
const prefix = `/pokedex/releases/${path.basename(release)}/`;
const captures = process.env.POKEDEX_MACHINE_CAPTURES;
if (captures) fs.mkdirSync(captures, { recursive: true });
const server = http.createServer((request, response) => {
 const url = new URL(request.url, 'http://localhost');
 let relative;
 if (url.pathname.startsWith(prefix)) relative = url.pathname.slice(prefix.length);
 else if (url.pathname.startsWith('/pokedex/species/')) relative = `species/${path.basename(url.pathname)}`;
 else if (url.pathname === '/pokedex/') relative = 'index.html';
 const file = relative && path.resolve(release, relative);
 if (!file || !file.startsWith(`${release}/`) || !fs.existsSync(file) || !fs.statSync(file).isFile()) {
  response.writeHead(404).end(); return;
 }
 const type = { '.html': 'text/html', '.css': 'text/css', '.js': 'text/javascript', '.png': 'image/png', '.svg': 'image/svg+xml' }[path.extname(file)] || 'application/octet-stream';
 response.writeHead(200, { 'Content-Type': type });
 fs.createReadStream(file).pipe(response);
});
await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
const base = process.env.POKEDEX_WEB_URL || `http://127.0.0.1:${server.address().port}/pokedex/`;
const browser = await chromium.launch({ headless: true, channel: process.env.POKEDEX_BROWSER_CHANNEL || 'chromium' });
try {
 for (const width of [1280, 390]) {
  const context = await browser.newContext({ viewport: { width, height: 900 }, reducedMotion: 'reduce' });
  const page = await context.newPage();
  const errors = [];
  page.on('pageerror', error => errors.push(error.message));
  for (const name of ['pouchid', 'voyagear', 'borealynx']) {
   await page.goto(`${base}species/${name}.html`);
   await page.locator('#tab-machine').click();
   await page.locator('#moves-machine').waitFor({ state: 'visible' });
   assert.equal(await page.locator('#tab-machine').getAttribute('aria-selected'), 'true');
   const methods = await page.locator('#moves-machine tbody tr td:first-child').allTextContents();
   assert.ok(methods.length > 0 && methods.every(method => /^(TM\d{2}|HM0[1-7])$/.test(method)));
   assert.equal(await page.locator('#moves-level').isVisible(), false);
   assert.ok(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth), `${name} must fit ${width}px`);
   if (name === 'voyagear') {
    assert.ok(methods.includes('HM03') && methods.includes('HM06') && methods.includes('HM07'));
   }
   if (captures) await page.locator('.move-panel').screenshot({ path: path.join(captures, `${name}-${width}-machines.png`) });
   await page.locator('#tab-machine').focus();
   await page.keyboard.press('ArrowRight');
   assert.equal(await page.locator('#tab-tutor').getAttribute('aria-selected'), 'true');
   assert.equal(await page.locator('#moves-tutor').isVisible(), true);
   assert.equal(await page.locator('#moves-machine').isVisible(), false);
   if (name === 'borealynx') {
    assert.match(await page.locator('#moves-tutor').textContent(), /Ice Beam/);
    assert.match(await page.locator('#moves-tutor').textContent(), /Flamethrower/);
   } else {
    assert.equal(await page.locator('#moves-tutor tbody tr').count(), 0);
    assert.match(await page.locator('#moves-tutor').textContent(), /No moves listed/);
   }
   await page.keyboard.press('End');
   assert.equal(await page.locator('#moves-egg').isVisible(), true);
   await page.keyboard.press('Home');
   assert.equal(await page.locator('#moves-level').isVisible(), true);
  }
  assert.deepEqual(errors, []);
  await context.close();
 }
 console.log('PASS: desktop/mobile numbered TM/HM learnsets, separated tutors, keyboard navigation and no page overflow');
} finally {
 await browser.close();
 await new Promise(resolve => server.close(resolve));
}
