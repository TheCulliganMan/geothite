import assert from 'node:assert/strict';
import fs from 'node:fs';
import http from 'node:http';
import path from 'node:path';
import { chromium } from 'playwright';

const release = path.resolve(process.env.POKEDEX_WEB_RELEASE || '');
const gallery = process.env.POKEDEX_CRY_GALLERY;
const captures = process.env.POKEDEX_MEDIA_CAPTURES;
assert.ok(process.env.POKEDEX_WEB_RELEASE && gallery && captures, 'Set POKEDEX_WEB_RELEASE, POKEDEX_CRY_GALLERY and POKEDEX_MEDIA_CAPTURES');
fs.mkdirSync(captures, { recursive: true });
const releasePrefix = `/pokedex/releases/${path.basename(release)}/`;
const cryPrefix = fs.readFileSync(path.join(release, 'index.html'), 'utf8')
  .match(/data-cry="([^"]+\/)MN_\d+\.wav"/)[1];
const server = http.createServer((request, response) => {
  const url = new URL(request.url, 'http://localhost');
  let file;
  if (url.pathname.startsWith(cryPrefix)) file = path.join(gallery, path.basename(url.pathname));
  else if (url.pathname.startsWith(releasePrefix)) file = path.join(release, url.pathname.slice(releasePrefix.length));
  else if (url.pathname === '/pokedex/' || url.pathname === '/pokedex/index.html') file = path.join(release, 'index.html');
  else if (url.pathname.startsWith('/pokedex/species/')) file = path.join(release, 'species', path.basename(url.pathname));
  else if (url.pathname === '/pokedex/moves.html') file = path.join(release, 'moves.html');
  if (!file || !fs.existsSync(file) || !fs.statSync(file).isFile()) { response.writeHead(404).end(); return; }
  const type = { '.wav': 'audio/wav', '.png': 'image/png', '.html': 'text/html', '.js': 'text/javascript', '.css': 'text/css', '.svg': 'image/svg+xml' }[path.extname(file)] || 'application/octet-stream';
  response.writeHead(200, { 'Content-Type': type, 'Content-Length': fs.statSync(file).size });
  fs.createReadStream(file).pipe(response);
});
await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
const base = process.env.POKEDEX_WEB_URL || `http://127.0.0.1:${server.address().port}/pokedex/`;
const browser = await chromium.launch({ headless: true, channel: process.env.POKEDEX_BROWSER_CHANNEL || 'chromium', args: ['--mute-audio'] });
try {
  const context = await browser.newContext({ viewport: { width: 1280, height: 900 } });
  await context.addInitScript(() => {
    window.cryEvents = [];
    const NativeAudio = window.Audio;
    window.Audio = function (...args) {
      const audio = new NativeAudio(...args);
      for (const type of ['playing', 'ended', 'error']) audio.addEventListener(type, () => window.cryEvents.push({ type, source: audio.currentSrc, duration: audio.duration }));
      return audio;
    };
    window.Audio.prototype = NativeAudio.prototype;
  });
  const page = await context.newPage();
  const errors = [];
  const failedRequests = [];
  page.on('pageerror', error => errors.push(error.message));
  page.on('response', response => { if (response.status() >= 400) failedRequests.push(response.url()); });
  await page.goto(`${base}#guide`);
  const card = page.locator('.pokemon-card[data-id=MN_001]');
  await card.scrollIntoViewIfNeeded();
  const walker = card.locator('[data-walker]');
  await walker.locator('img').evaluate(image => image.decode());
  await page.waitForFunction(() => document.querySelector('.pokemon-card[data-id=MN_001] [data-walker]').hasAttribute('data-sprite-running'));
  assert.deepEqual(await walker.evaluate(node => [node.offsetWidth, node.offsetHeight]), [48, 48]);
  assert.deepEqual(await walker.locator('img').evaluate(node => [node.naturalWidth, node.naturalHeight, node.offsetWidth, node.offsetHeight]), [64, 16, 192, 48]);
  const transforms = new Set();
  for (let sample = 0; sample < 8; sample++) {
    transforms.add(await walker.locator('img').evaluate(node => getComputedStyle(node).transform));
    await page.waitForTimeout(115);
  }
  assert.ok(transforms.size >= 3, 'native walking cells must change in the browser');
  assert.deepEqual(await page.evaluate(() => window.cryEvents), [], 'cries never autoplay');
  await card.locator('[data-cry]').click();
  await page.waitForFunction(() => window.cryEvents.some(event => event.type === 'playing'));
  await page.waitForFunction(() => window.cryEvents.some(event => event.type === 'ended'));
  assert.equal(await card.locator('[data-cry]').getAttribute('aria-pressed'), 'false');
  const events = await page.evaluate(() => window.cryEvents);
  assert.ok(events.every(event => event.type !== 'error'));
  assert.ok(events.find(event => event.type === 'ended').duration > .25);
  await page.screenshot({ path: path.join(captures, 'index-desktop.png') });
  await page.locator('[data-walking-toggle]').click();
  assert.equal(await walker.getAttribute('data-sprite-running'), null);
  await page.locator('[data-walking-toggle]').click();
  await page.emulateMedia({ reducedMotion: 'reduce' });
  await page.waitForFunction(() => document.querySelector('[data-walking-toggle]').disabled);
  assert.equal(await walker.getAttribute('data-sprite-running'), null);
  assert.equal(await walker.locator('img').evaluate(node => getComputedStyle(node).animationName), 'none');
  await page.emulateMedia({ reducedMotion: 'no-preference' });
  await page.goto(`${base}species/birchit.html`);
  await page.locator('#shiny-toggle').click();
  assert.ok((await page.locator('[data-walk-normal]').getAttribute('src')).endsWith('-shiny.png'));
  await page.locator('[data-walk-normal]').evaluate(image => image.decode());
  await page.locator('[data-walking-toggle]').click();
  assert.equal(await page.locator('[data-walker]').getAttribute('data-sprite-running'), null);
  await page.screenshot({ path: path.join(captures, 'entry-desktop.png') });
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto(`${base}#guide`);
  await card.scrollIntoViewIfNeeded();
  assert.ok(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth), 'mobile layout must fit the screen');
  await page.screenshot({ path: path.join(captures, 'index-mobile.png') });
  await page.goto(`${base}species/birchit.html`);
  assert.ok(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth), 'mobile entry must fit the screen');
  await page.locator('[data-cry]').click();
  await page.waitForFunction(() => window.cryEvents.some(event => event.type === 'ended'));
  await page.screenshot({ path: path.join(captures, 'entry-mobile.png') });
  assert.deepEqual(errors, []);
  assert.deepEqual(failedRequests, []);
  console.log('PASS: real browser walking frames, native pixel sizing, audio playing/ended, shiny, pause/reduced motion, desktop/mobile layout and no failed assets');
} finally {
  await browser.close();
  await new Promise(resolve => server.close(resolve));
}
