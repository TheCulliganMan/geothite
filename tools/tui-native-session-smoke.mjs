// Real stdio MCP clients: twenty independent saves, single-writer ownership,
// abrupt disconnect and reconnect, using only Game Boy inputs/code mode.
import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { createInterface } from 'node:readline';
import { mkdtemp, mkdir, rm, readFile, stat, unlink } from 'node:fs/promises';
import { resolve, join } from 'node:path';
const bin = resolve(process.env.TUI_NATIVE_BIN ?? 'target/release/geothite');
const pack = resolve(process.env.TUI_NATIVE_PACK ?? 'content-packs/realtime-clock.browser.crystalpack');
await mkdir('target/tui-session-smoke', { recursive: true });
const root = await mkdtemp(resolve('target/tui-session-smoke/native-'));
const clients = new Set();
function connect(slot) {
  const child = spawn(bin, ['mcp', pack, '--save', join(root, `${slot}.crystalsave`)], { stdio: ['pipe', 'pipe', 'pipe'] });
  clients.add(child);
  const pending = new Map(); let id = 0, errors = '';
  child.stderr.on('data', data => errors += data);
  const closed = new Promise(done => child.once('close', code => { clients.delete(child); done({ code, errors }); }));
  createInterface({ input: child.stdout }).on('line', line => {
    const reply = JSON.parse(line); pending.get(reply.id)?.(reply); pending.delete(reply.id);
  });
  async function call(name, args = {}) {
    const request = ++id;
    const result = new Promise((resolve, reject) => {
      const timer = setTimeout(() => reject(new Error(`Client ${slot}: ${name} timed out: ${errors}`)), 120000);
      pending.set(request, reply => { clearTimeout(timer); resolve(reply); });
    });
    child.stdin.write(JSON.stringify({ jsonrpc: '2.0', id: request, method: 'tools/call', params: { name, arguments: args } }) + '\n');
    const reply = await result;
    assert(!reply.error && !reply.result?.isError, JSON.stringify(reply));
    return reply.result.structuredContent;
  }
  return { child, call, closed, path: join(root, `${slot}.crystalsave`) };
}
try {
  const sessions = [];
  for (let i = 0; i < 20; i++) {
    const client = connect(`player-${i}`); sessions.push(client);
    assert.match((await client.call('observe')).status_line, /PlayersHouse2F.*\(3, 3\)/);
  }
  const mtimes = await Promise.all(sessions.map(s => stat(s.path).then(s => s.mtimeMs)));
  await Promise.all(sessions.map(s => s.call('execute', { code: 'for(let i=0;i<10;i++) await tools.observe(); return null;' })));
  assert.deepEqual(await Promise.all(sessions.map(s => stat(s.path).then(s => s.mtimeMs))), mtimes, 'Read-only code mode never writes');
  await sessions[0].call('execute', { code: "await tools.press({button:'right'}); await tools.press({button:'right'}); return null;" });
  assert.match((await sessions[0].call('observe')).status_line, /\(4, 3\)/);
  for (let i = 1; i < 20; i++) assert.match((await sessions[i].call('observe')).status_line, /\(3, 3\)/);
  await Promise.all(sessions.slice(1).map((s, i) => s.call('execute', { code: `for(let i=0;i<${i % 2 ? 4 : 2};i++) await tools.press({button:'down'}); return null;` })));
  const expected = await Promise.all(sessions.map(s => s.call('observe')));
  for (let i = 1; i < 20; i++) assert(!expected[i].status_line.includes('(3, 3)'), `Player ${i}'s inputs must reach their own session`);
  const duplicate = connect('player-0'); duplicate.child.stdin.end();
  assert.match((await duplicate.closed).errors, /already open/, 'Second writer rejected before loading/overwriting');
  const sizes = await Promise.all(sessions.map(s => stat(s.path).then(s => s.size)));
  assert(sizes.every(n => n < 65536));
  for (const s of sessions) {
    assert.deepEqual(await readFile(`${s.path}.bak`), await readFile(s.path));
    s.child.kill('SIGKILL'); // Deliberate crash: no clean EOF/quit/save hook.
  }
  await Promise.all(sessions.map(s => s.closed));
  await unlink(sessions[19].path); // A valid backup-only slot must also resume.
  for (let i = 0; i < 20; i++) {
    const s = connect(`player-${i}`);
    assert.equal((await s.call('observe')).status_line, expected[i].status_line, 'Configured save resumes automatically');
    await s.call('press', { button: 'start' });
    assert((await s.call('observe')).menu.some(line => line.text.includes('PACK')));
    s.child.stdin.end(); assert.equal((await s.closed).errors, '');
  }
  console.log(`20 live native MCP sessions: isolation, writer lock, abrupt process death/reconnect, Start, no observe writes; saves ${Math.min(...sizes)}–${Math.max(...sizes)} bytes plus one recovery copy.`);
} finally {
  for (const child of clients) child.kill('SIGTERM');
  await Promise.all([...clients].map(child => new Promise(done => child.once('close', done))));
  await rm(root, { recursive: true, force: true });
}
