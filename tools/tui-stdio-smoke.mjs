// Actual executable regression: production stdio MCP and controller only.
import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { createInterface } from 'node:readline';
import { resolve } from 'node:path';
import { checkMenus } from './tui-menu-checks.mjs';
const child = spawn(resolve(process.env.TUI_NATIVE_BIN ?? 'target/release/geothite'), [
  'mcp', resolve(process.env.TUI_NATIVE_PACK ?? 'content-packs/realtime-clock.browser.crystalpack'),
  '--load', resolve(process.env.TUI_NATIVE_MENU_FIXTURE ?? process.env.TUI_NATIVE_FIXTURE ?? 'target/tui-smoke/lowlevel.crystalsave'),
], { stdio: ['pipe', 'pipe', 'pipe'] });
const pending = new Map();
let id = 0, stderr = '';
child.stderr.on('data', data => { stderr += data; });
createInterface({ input: child.stdout }).on('line', line => {
  const reply = JSON.parse(line);
  pending.get(reply.id)?.(reply); pending.delete(reply.id);
});
async function call(name, args = {}) {
  const requestId = ++id;
  const response = new Promise((resolve, reject) => {
    const timeout = setTimeout(() => { pending.delete(requestId); reject(new Error(`MCP timed out: ${name}; stderr=${stderr}`)); }, 10000);
    pending.set(requestId, reply => { clearTimeout(timeout); resolve(reply); });
  });
  child.stdin.write(`${JSON.stringify({ jsonrpc: '2.0', id: requestId, method: 'tools/call', params: { name, arguments: args } })}\n`);
  const reply = await response;
  assert(!reply.error && !reply.result?.isError, JSON.stringify(reply));
  return reply.result.structuredContent;
}
try {
  const observe = () => call('observe');
  const press = button => call('press', { button });
  if (process.env.TUI_NATIVE_MENU_FIXTURE) {
    await checkMenus(observe, press);
    assert.equal(stderr, '', 'Menus/audio cannot corrupt terminal output');
    console.log('Native MCP: Potion HP/count, all Pack pockets, Itemfinder, Dex entry/area/cry/printer/options/search, Gear map/phone/radio and movement passed.');
  } else {
  assert.match((await observe()).status_line, process.env.TUI_NATIVE_TRAINER ? /Battle/ : /Route29.*\(46, 12\)/);
  for (let step = 0; step < 256 && !(await observe()).status_line.includes('Battle'); step++) {
    await press(Math.floor(step / 2) % 2 ? 'right' : 'left');
  }
  assert.match((await observe()).status_line, /Battle/);
  let turns = 0, repeats = 0;
  for (let step = 0; step < 256; step++) {
    const before = await observe();
    if (before.status_line.startsWith('Overworld')) break;
    if (before.menu.some(line => /TACKLE|EMBER|SCRATCH|RAGE|BITE|WATER GUN|LEER|SCARY FACE/.test(line.text))) turns++;
    await press('a');
    const after = await observe();
    const visible = view => JSON.stringify([view.status_line, view.viewport, view.menu, view.dialogue, view.info]);
    repeats = visible(before) === visible(after) ? repeats + 1 : 0;
    assert(repeats <= 1, `Native battle froze after ${turns} turns: ${visible(after)}`);
  }
  const before = await observe();
  assert.match(before.status_line, /^Overworld/);
  assert(turns >= Number(process.env.TUI_NATIVE_MIN_TURNS ?? 2), `Expected multiple real turns, got ${turns}`);
  await press('up'); await press('up');
  assert.notDeepEqual((await observe()).marker, before.marker, 'Native movement after battle');
  await press('start');
  assert((await observe()).menu.some(line => line.text.includes('SAVE')));
  assert.equal(stderr, '', 'Native gameplay must not corrupt terminal output');
  console.log(`Native executable: ${turns} real battle turns, no animation stall, post-battle movement and Start passed.`);
  }
} finally { child.stdin.end(); child.kill('SIGTERM'); }
