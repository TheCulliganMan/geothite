import { test } from 'node:test';
import assert from 'node:assert/strict';
import { keyButton, modalInput, tuiTools, registerTuiTools } from './tui/bridge.js';
import { tuiSession, acquireTuiSession } from './tui/session.js';
import { mkdtemp, readFile, writeFile, readdir, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { execFileSync } from 'node:child_process';

test('browser keys preserve native modal, Game Boy and vim controls', () => {
  assert.equal(keyButton('a'), 'left');
  assert.equal(keyButton('a', true), 'a');
  for (const key of ['ArrowUp', 'w', 'k']) assert.equal(keyButton(key), 'up');
  for (const key of ['z', ' ', 'j']) assert.equal(keyButton(key), 'a');
  assert.equal(keyButton('Enter'), 'start');
  assert.equal(keyButton('Tab'), 'select');
  assert.equal(keyButton('Escape'), 'b');
  assert.equal(keyButton('q'), null);
});

test('A confirms battle and dialogue even when there is no menu', () => {
  const view = { menu: [], prompt: [], dialogue: [], status_line: 'Overworld Route29' };
  assert.equal(keyButton('a', modalInput(view)), 'left');
  for (const phase of ['WildBattle', 'TrainerBattle', 'StaticWildBattle', 'Text']) {
    assert.equal(keyButton('a', modalInput({ ...view, status_line: phase })), 'a');
  }
  assert.equal(keyButton('a', modalInput({ ...view, dialogue: [{ text: 'Wild RATTATA appeared!' }] })), 'a');
});

test('WebMCP tools call the live input bridge and reject invalid inputs', async () => {
  const calls = [];
  const bridge = { observe: () => ({ viewport_title: 'Route29' }), press: button => { calls.push(button); return { status_line: 'moved' }; } };
  const tools = tuiTools(bridge);
  const observe = tools.find(tool => tool.name === 'geothite_tui_observe');
  const press = tools.find(tool => tool.name === 'geothite_tui_press');
  assert.deepEqual(await observe.execute({}), { viewport_title: 'Route29' });
  assert.deepEqual(await press.execute({ button: 'right' }), { status_line: 'moved' });
  assert.deepEqual(calls, ['right']);
  assert.throws(() => press.execute({ button: 'teleport' }));
  assert.throws(() => press.execute({ button: 'a', debug: true }));
  assert.throws(() => observe.execute({ hidden: true }));
});

test('WebMCP registers on arrival, awaits readiness, cancels and cleans up failures', async () => {
  let finish;
  let pressed = 0;
  const bridge = { ready: new Promise(resolve => { finish = resolve; }), press: () => { pressed++; return {}; } };
  const registrations = [];
  const document = { modelContext: { registerTool: async (tool, options) => { registrations.push({ tool, options }); } } };
  const registration = await registerTuiTools(document, bridge);
  assert.equal(registration.names.length, 6);
  const controller = new AbortController();
  const pending = registrations.find(({ tool }) => tool.name === 'geothite_tui_press').tool.execute({ button: 'a' }, { signal: controller.signal });
  controller.abort(); finish();
  await assert.rejects(pending, { name: 'AbortError' });
  assert.equal(pressed, 0);
  registration.dispose();
  assert(registrations.every(({ options }) => options.signal.aborted));
  const unsupported = {};
  assert.equal((await registerTuiTools(unsupported, bridge)).supported, false);
  assert.equal(unsupported.modelContext, undefined);
  let signal;
  await assert.rejects(registerTuiTools({ modelContext: { registerTool: async (_, options) => { signal = options.signal; throw new Error('denied'); } } }, bridge));
  assert(signal.aborted);
});

test('code-mode discovery and execution await readiness, validate and cancel', async () => {
  let ready;
  const calls = [];
  const bridge = {
    ready: new Promise(resolve => { ready = resolve; }),
    search: query => ({ query }),
    execute: code => { calls.push(code); return { value: 7, calls: 1 }; },
  };
  const tools = tuiTools(bridge);
  const search = tools.find(t => t.name === 'geothite_tui_search');
  const execute = tools.find(t => t.name === 'geothite_tui_execute');
  const pending = execute.execute({ code: 'return 7;' });
  assert.equal(calls.length, 0);
  ready();
  assert.deepEqual(await pending, { value: 7, calls: 1 });
  assert.deepEqual(await search.execute({ query: 'press' }), { query: 'press' });
  assert.throws(() => search.execute({ query: 3 }));
  assert.throws(() => execute.execute({ code: 'return 1;', path: '/tmp' }));
  assert.throws(() => execute.execute({ code: ' '.repeat(16385) }));
  const controller = new AbortController(); controller.abort();
  await assert.rejects(execute.execute({ code: 'return 8;' }, { signal: controller.signal }), { name: 'AbortError' });
  assert.equal(calls.length, 1);
});

test('TUI glue, WASM, adapters and CSS deploy as one versioned bundle', async () => {
  const dir = await mkdtemp(join(tmpdir(), 'geothite-tui-version-'));
  try {
    await writeFile(join(dir, 'geothite.js'), "new URL('geothite_bg.wasm', import.meta.url)");
    await writeFile(join(dir, 'geothite_bg.wasm'), 'test WASM bytes');
    await writeFile(join(dir, 'ascii-mount.js'), 'test pinned ASCII painter');
    for (const name of ['index.html', 'browser.js', 'bridge.js', 'session.js', 'browser.css', 'ascii-frame.js']) {
      await writeFile(join(dir, name), await readFile(resolve('web-client/tui', name)));
    }
    execFileSync('sh', [resolve('tools/version-tui-bundle.sh'), dir]);
    const files = await readdir(dir);
    const glue = files.find(name => /^geothite-[a-f0-9]{64}\.js$/.test(name));
    assert(glue);
    const hash = glue.slice('geothite-'.length, -3);
    const page = await readFile(join(dir, 'index.html'), 'utf8');
    assert(page.includes(`/tui/browser-${hash}.js`));
    assert(page.includes(`/tui/browser-${hash}.css`));
    const browser = await readFile(join(dir, `browser-${hash}.js`), 'utf8');
    assert(browser.includes(`./${glue}`));
    assert(browser.includes(`./bridge-${hash}.js`));
    assert(browser.includes(`./session-${hash}.js`));
    assert(browser.includes(`./ascii-frame-${hash}.js`));
    assert((await readFile(join(dir, `ascii-frame-${hash}.js`), 'utf8')).includes(`./ascii-mount-${hash}.js`));
    assert((await readFile(join(dir, glue), 'utf8')).includes(`geothite-${hash}.wasm`));
    assert(files.includes(`geothite-${hash}.wasm.gz`));
    assert(!files.includes('geothite.js') && !files.includes('geothite_bg.wasm'));
  } finally { await rm(dir, { recursive: true, force: true }); }
});

test('named sessions retain the old default and reject path/URL injection', () => {
  assert.equal(tuiSession('https://game.test/tui').slot, '');
  assert.equal(tuiSession('https://game.test/tui?session=chris-20').slot, 'chris-20');
  for (const id of ['../someone', 'a/b', 'a.b', 'a'.repeat(65), '🎮']) {
    assert.throws(() => tuiSession(`https://game.test/tui?session=${encodeURIComponent(id)}`));
  }
});

test('one writer per browser session, independent slots, release then reconnect', async () => {
  const held = new Set();
  const locks = { async request(name, _, run) {
    if (held.has(name)) return run(null);
    held.add(name);
    try { return await run({ name }); } finally { held.delete(name); }
  } };
  const release = await acquireTuiSession(locks, 'chris');
  await assert.rejects(acquireTuiSession(locks, 'chris'), /another tab/);
  const releaseOther = await acquireTuiSession(locks, 'silver');
  release(); await new Promise(resolve => setImmediate(resolve));
  const reconnect = await acquireTuiSession(locks, 'chris');
  reconnect(); releaseOther();
  await assert.rejects(acquireTuiSession(undefined, ''), /HTTPS/);
});
