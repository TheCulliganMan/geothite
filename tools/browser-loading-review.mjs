// Background render check using actual response streams and throttled networking.
// Usage: node tools/browser-loading-review.mjs URL OUTPUT_DIRECTORY
import { chromium } from 'playwright';
import fs from 'node:fs/promises';
import assert from 'node:assert/strict';
const [base = 'http://127.0.0.1:33104', output = 'target/loading-evidence'] = process.argv.slice(2);
await fs.mkdir(output, { recursive: true });
const browser = await chromium.launch({ headless: true, args: ['--use-gl=angle', '--use-angle=swiftshader', '--enable-unsafe-swiftshader'] });
const results = [];
try {
  for (const [name, path, width, label] of [
    ['game-desktop', '/?multiplayer=off', 1280, 'Game engine'],
    ['brain-mobile', '/flygon?multiplayer=off', 390, 'Connectome'],
  ]) {
    const context = await browser.newContext({ viewport: { width, height: 900 } });
    const page = await context.newPage();
    const errors = [];
    page.on('pageerror', error => errors.push(error.message));
    page.on('download', () => errors.push('Unexpected browser download'));
    const cdp = await context.newCDPSession(page);
    await cdp.send('Network.enable');
    await cdp.send('Network.setCacheDisabled', { cacheDisabled: true });
    await cdp.send('Network.emulateNetworkConditions', { offline: false, latency: 30, downloadThroughput: 12 * 1048576, uploadThroughput: 1048576 });
    await page.goto(new URL(path, base).href);
    await page.waitForFunction(label => [...document.querySelectorAll('.asset-progress')].some(row => row.querySelector('strong').textContent === label && row.dataset.phase === 'downloading' && row.querySelector('progress').value > 0 && row.querySelector('progress').value < 1), label, { timeout: 90000 });
    const progress = await page.locator('.asset-progress').allTextContents();
    await page.screenshot({ path: `${output}/${name}-loading.png`, fullPage: true });
    if (name.startsWith('game')) {
      await page.waitForFunction(() => document.querySelector('#loading').hidden || document.querySelector('#startup-error').open, null, { timeout: 180000 });
      assert.equal(await page.locator('#startup-error').evaluate(e => e.open), false);
      assert.equal(await page.locator('#loading').evaluate(e => e.hidden), true);
    } else {
      await page.waitForFunction(() => Number(document.querySelector('#brain').dataset.frames) > 0, null, { timeout: 180000 });
      assert.equal(await page.locator('#view-empty').evaluate(e => e.hidden), true);
    }
    assert.deepEqual(errors, []);
    const overflow = await page.evaluate(() => document.documentElement.scrollWidth > innerWidth);
    assert.equal(overflow, false);
    await page.screenshot({ path: `${output}/${name}-ready.png`, fullPage: true });
    results.push({ name, progress, ready: true, errors, overflow });
    await context.close();
  }
  await fs.writeFile(`${output}/results.json`, JSON.stringify(results, null, 2));
  console.log(JSON.stringify(results, null, 2));
} finally { await browser.close(); }
