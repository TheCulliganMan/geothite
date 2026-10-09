// Visual evidence from the actual WASM and live controller, not mock screens.
import assert from 'node:assert/strict';
import { chromium, webkit } from 'playwright';
import { createServer } from 'node:http';
import { readFile, readdir, mkdir, writeFile } from 'node:fs/promises';
import { resolve, extname } from 'node:path';

const root = resolve(process.env.TUI_EVAL_ROOT ?? 'target/tui-web');
const output = resolve(process.env.TUI_EVAL_OUTPUT ?? 'target/tui-art-eval/after');
const pack = resolve(process.env.TUI_EVAL_PACK ?? 'content-packs/realtime-clock.browser.crystalpack');
await mkdir(output, { recursive: true });
const server = createServer(async (request, response) => {
  try {
    const path = new URL(request.url, 'http://localhost').pathname;
    const file = path === '/realtime-clock.browser.crystalpack' ? pack : resolve(root, path === '/tui/' ? 'tui/index.html' : `.${path}`);
    if (file !== pack && !file.startsWith(`${root}/`)) throw new Error('outside root');
    response.setHeader('content-type', { '.html': 'text/html', '.js': 'text/javascript', '.css': 'text/css', '.wasm': 'application/wasm' }[extname(file)] ?? 'application/octet-stream');
    response.end(await readFile(file));
  } catch { response.writeHead(404).end(); }
});
await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
const url = `http://127.0.0.1:${server.address().port}/tui/`;
const results = [];
try {
  for (const [name, size, engine] of [
    ['desktop', [1440, 960], chromium], ['desktop-small', [1024, 768], chromium],
    ['iphone-se', [320, 568], webkit], ['iphone', [390, 844], webkit],
    ['iphone-large', [430, 932], webkit], ['phone-landscape', [844, 390], webkit],
  ]) {
    const browser = await engine.launch({ headless: true });
    try {
      const phone = name.includes('phone');
      const page = await browser.newPage({ viewport: { width: size[0], height: size[1] }, isMobile: phone, hasTouch: phone });
      const errors = [];
      page.on('pageerror', error => errors.push(error.message));
      await page.emulateMedia({ reducedMotion: 'reduce' });
      await page.goto(url);
      await page.evaluate(() => window.geothiteTui.ready);
      await page.waitForFunction(() => !document.getElementById('screen').hidden, { timeout: 120000 });
      const press = button => page.evaluate(button => window.geothiteTui.press(button), button);
      const observe = () => page.evaluate(() => window.geothiteTui.observe());
      const capture = async scene => {
        await page.evaluate(async () => {
          await document.fonts.ready;
          await new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve)));
        });
        const metrics = await page.evaluate(async () => {
          const canvas = document.getElementById('ascii');
          const box = canvas.getBoundingClientRect();
          const bytes = canvas.getContext('2d').getImageData(0, 0, canvas.width, canvas.height).data;
          const hash = [...new Uint8Array(await crypto.subtle.digest('SHA-256', bytes))].map(x => x.toString(16).padStart(2, '0')).join('');
          const controls = [...document.querySelectorAll('#touch-controls button')].filter(button => button.getBoundingClientRect().width).map(button => { const b=button.getBoundingClientRect(); return { button:button.dataset.button,x:b.x,y:b.y,width:b.width,height:b.height }; });
          return { hash, view: document.getElementById('screen').dataset.view, width: box.width, height: box.height, x: box.x, y: box.y, viewport: [innerWidth, innerHeight], scrollWidth: document.documentElement.scrollWidth, scrollHeight: document.documentElement.scrollHeight, controls };
        });
        await page.screenshot({ path: resolve(output, `${name}-${scene}.png`) });
        results.push({ name, scene, metrics, snapshot: await observe(), errors: [...errors] });
      };
      await capture('room');
      assert.equal(await page.locator('#screen').getAttribute('data-view'), 'painted');
      const initial = await observe();
      await page.locator('#view-toggle').click();
      assert.equal(await page.locator('#screen').getAttribute('data-view'), 'text');
      assert.deepEqual((await observe()).marker, initial.marker, 'Toggling is presentation-only');
      await capture('room-text');
      await page.locator('#view-toggle').click();
      await press('start'); await capture('menu'); await press('b');
      async function move(button, count) {
        for (let i = 0; i < count; i++) {
          const before = await observe();
          for (let attempt = 0; attempt < 4; attempt++) {
            const after = await press(button);
            if (JSON.stringify(after.marker) !== JSON.stringify(before.marker) || after.viewport_title !== before.viewport_title) break;
          }
        }
      }
      await move('right', 4); await move('up', 3); await move('down', 4);
      assert((await observe()).dialogue.some(line => line.text.includes('CHRIS')), 'Authored Mom dialogue');
      await capture('dialogue');
      await press('a'); await capture('dialogue-page2');
      if (process.env.TUI_EVAL_FIXTURE) {
        const fixture = await readFile(resolve(process.env.TUI_EVAL_FIXTURE));
        await page.evaluate(() => window.geothiteTui.save());
        await page.evaluate(base64 => {
          const key = Object.keys(localStorage).find(key => key.startsWith('crystal.save.v1.') && key.endsWith('-tui-local.crystalsave'));
          if (!key) throw new Error('Missing authoritative save key');
          localStorage.setItem(key, base64);
        }, fixture.toString('base64'));
        await page.reload();
        await page.evaluate(() => window.geothiteTui.ready);
        await page.waitForFunction(() => !document.getElementById('screen').hidden, { timeout: 120000 });
        assert.match((await observe()).status_line, /Route29.*\(46, 12\)/, 'Actually loaded the intended route fixture');
        await capture('route');
        for (let i = 0; i < 256 && !(await observe()).status_line.includes('Battle'); i++) await press(Math.floor(i / 2) % 2 ? 'right' : 'left');
        for (let i = 0; i < 64 && !(await observe()).menu.some(line => line.text.includes('FIGHT')); i++) await press('a');
        assert((await observe()).menu.some(line => line.text.includes('FIGHT')), 'Actually in the production battle menu');
        const painting = await page.locator('#screen').textContent();
        assert(!painting.includes('sprite unavailable'), 'Both production sprite assets load');
        assert.equal((painting.match(/HP \d+\/\d+/g) ?? []).length, 2, 'Enemy and player health are painted');
        assert(painting.includes('█'), 'HP bars are painted, not just labels');
        await capture('battle');
      }
      if (process.env.TUI_EVAL_OVERWORLD_FIXTURES) {
        await page.evaluate(() => window.geothiteTui.save());
        const directory = resolve(process.env.TUI_EVAL_OVERWORLD_FIXTURES);
        for (const file of (await readdir(directory)).filter(file => file.endsWith('.crystalsave')).sort()) {
          const fixture = await readFile(resolve(directory, file));
          const map = file.replace('.crystalsave', '');
          await page.evaluate(base64 => {
            const key = Object.keys(localStorage).find(key => key.startsWith('crystal.save.v1.') && key.endsWith('-tui-local.crystalsave'));
            localStorage.setItem(key, base64);
          }, fixture.toString('base64'));
          await page.reload();
          await page.evaluate(() => window.geothiteTui.ready);
          await page.waitForFunction(() => !document.getElementById('screen').hidden, { timeout: 120000 });
          const state = await observe();
          assert(state.status_line.includes(map), `Actually loaded ${map}, not a mislabeled screenshot`);
          assert(!state.status_line.includes('Battle'), 'Overworld art scene is not a battle');
          const ink = await page.locator('#screen').textContent();
          assert(/[·•●]/.test(ink) && !ink.includes('▀'), `${map} has real tile/sprite halftone art`);
          await capture(`overworld-${map}`);
        }
      }
      assert.deepEqual(errors, [], `${name} has no browser exceptions`);
      console.log(`${name}: ${results.filter(result => result.name === name).length} real scenes captured`);
    } finally { await browser.close(); }
  }
  await writeFile(resolve(output, 'results.json'), JSON.stringify(results, null, 2));
} finally { server.closeAllConnections(); server.close(); }
