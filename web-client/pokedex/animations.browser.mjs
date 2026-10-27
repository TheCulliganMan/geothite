// Headless checks against a completed Rust-exported release; never opens a window.
import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import http from 'node:http';
import path from 'node:path';
import { chromium } from 'playwright';

const release = path.resolve(process.env.POKEDEX_WEB_RELEASE || 'target/minnesota-pokedex-web/pokedex/current');
const proof = process.env.POKEDEX_ANIMATION_PROOF;
if (proof) await fs.mkdir(proof, { recursive: true });
const types = { '.html': 'text/html', '.js': 'text/javascript', '.css': 'text/css', '.png': 'image/png', '.svg': 'image/svg+xml', '.json': 'application/json', '.woff2': 'font/woff2' };
const server = http.createServer(async (request, response) => {
  try {
    const url = new URL(request.url, 'http://localhost');
    const relative = decodeURIComponent(url.pathname).replace(/^\/pokedex\/(?:releases\/[^/]+\/)?/, '');
    const file = path.resolve(release, relative || 'index.html');
    if (!file.startsWith(`${release}${path.sep}`)) throw new Error('outside release');
    response.setHeader('Content-Type', types[path.extname(file)] || 'application/octet-stream');
    response.end(await fs.readFile(file));
  } catch {
    response.writeHead(404); response.end('Not found');
  }
});
await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
let browser;
try {
  browser = await chromium.launch({ headless: true });
  for (const width of [390, 1280]) {
    const context = await browser.newContext({ viewport: { width, height: 850 }, reducedMotion: 'reduce' });
    const page = await context.newPage();
    const errors = [];
    page.on('pageerror', error => errors.push(error.message));
    page.on('response', response => { if (response.status() >= 400) errors.push(`${response.status()} ${response.url()}`); });
    await page.goto(`http://127.0.0.1:${server.address().port}/pokedex/moves.html`);
    const cards = page.locator('.move-animation-card');
    assert.equal(await cards.count(), 19);
    assert.ok(await page.locator('.main-nav a[href="/pokedex/moves.html"]').isVisible());
    assert.equal(await page.locator('[data-animation-play][aria-pressed=true]').count(), 0);
    assert.ok(await page.locator('[data-animation-base] img').evaluateAll(images => images.every(image => image.getAttribute('src').endsWith('-still.png') && image.loading === 'lazy')));
    assert.ok(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth), `Moves page overflows at ${width}px`);
    for (const card of await cards.all()) {
      await card.scrollIntoViewIfNeeded();
      const image = card.locator('.battle-preview img');
      await image.evaluate(image => image.decode());
      await card.locator('[data-animation-side]').click();
      await card.locator('[data-animation-shiny]').click();
      await card.locator('[data-animation-play]').click();
      await image.evaluate(image => image.decode());
      assert.match(await image.getAttribute('src'), /enemy-shiny.png$/);
      assert.match(await card.locator('.animation-download').getAttribute('href'), /enemy-shiny.png$/);
      assert.equal(await page.locator('[data-animation-play][aria-pressed=true]').count(), 1);
      await card.locator('[data-animation-play]').click();
      assert.match(await image.getAttribute('src'), /enemy-shiny-still.png$/);
    }
    // Changing the OS preference while an explicitly started preview runs stops it.
    await page.emulateMedia({ reducedMotion: 'no-preference' });
    await cards.first().locator('[data-animation-play]').click();
    await page.emulateMedia({ reducedMotion: 'reduce' });
    await page.waitForFunction(() => !document.querySelector('[data-animation-play][aria-pressed=true]'));
    if (proof) {
      await page.evaluate(() => scrollTo(0, 0));
      await page.screenshot({ path: path.join(proof, `moves-${width}.png`) });
      await page.locator('#fairy_wind').scrollIntoViewIfNeeded();
      await page.screenshot({ path: path.join(proof, `fairy-wind-${width}.png`) });
    }
    await page.goto(`http://127.0.0.1:${server.address().port}/pokedex/species/cyprielle.html`);
    assert.ok(await page.locator('#fairy_wind').count());
    assert.ok(await page.locator('#slipper_waltz').count());
    assert.ok(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth), `Species page overflows at ${width}px`);
    await page.locator('#fairy_wind [data-animation-play]').click();
    await page.locator('#fairy_wind .battle-preview img').evaluate(image => image.decode());
    assert.match(await page.locator('#fairy_wind .battle-preview img').getAttribute('src'), /player-normal.png$/);
    if (proof) await page.screenshot({ path: path.join(proof, `cyprielle-moves-${width}.png`) });
    assert.deepEqual(errors, [], `Browser errors at ${width}px`);
    await context.close();
  }
  console.log('PASS: all 19 native previews play both side/palette controls at 390px and 1280px; lazy/reduced-motion safety, native images, species cards and no overflow/browser errors');
} finally {
  if (browser) await browser.close();
  await new Promise(resolve => server.close(resolve));
}
