// Actual curl pipeline + controlling PTY, fresh install outside the checkout.
// Python supplies only the PTY; gameplay remains in the downloaded Rust client.
import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { mkdtemp, readFile, readdir, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { createInterface } from 'node:readline';
const root = await mkdtemp(join(tmpdir(), 'geothite-curl-smoke-'));
const base = process.env.GEOTHITE_DOWNLOAD_URL ?? 'https://geothite.ryanculligan.com';
const shell = process.env.GEOTHITE_TEST_INTEL === '1' ? ['arch', '-x86_64', 'sh'] : ['sh'];
const child = spawn('python3', ['-u', '-c', `
import os, pty, fcntl, termios, struct, subprocess, select, sys, signal
master, slave = pty.openpty()
fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack('HHHH', 24, 80, 0, 0))
def attach():
    os.setsid()
    fcntl.ioctl(slave, termios.TIOCSCTTY, 0)
game = subprocess.Popen(sys.argv[1:], stdin=slave, stdout=slave, stderr=slave, preexec_fn=attach)
signal.signal(signal.SIGTERM, lambda signum, frame: sys.exit(0))
os.close(slave)
try:
    while game.poll() is None:
        for fd in select.select([master, 0], [], [], .1)[0]:
            try: data = os.read(fd, 65536)
            except OSError: sys.exit(0)
            if not data: sys.exit()
            os.write(1 if fd == master else master, data)
finally:
    if game.poll() is None: os.killpg(game.pid, signal.SIGTERM)
    os.close(master)
    try: game.wait(timeout=2)
    except subprocess.TimeoutExpired:
        os.killpg(game.pid, signal.SIGKILL)
        game.wait(timeout=2)
`, ...shell, '-c', 'curl -fsSL "$GEOTHITE_DOWNLOAD_URL/install.sh" | sh'], {
  cwd: root, env: { ...process.env, GEOTHITE_HOME: join(root, 'installed'), GEOTHITE_DOWNLOAD_URL: base },
  stdio: ['pipe', 'pipe', 'pipe'],
});
let output = '', rawOutput = '', errors = '';
child.stdout.on('data', bytes => {
  rawOutput += bytes.toString();
  output += bytes.toString().replace(/\x1b\[[0-?]*[ -/]*[@-~]/g, '');
});
child.stderr.on('data', bytes => errors += bytes);
const pause = () => new Promise(resolve => setTimeout(resolve, 100));
const started = Date.now();
async function key(value) { child.stdin.write(value); await pause(); }
try {
  for (let i = 0; i < 1200 && !output.includes('PlayersHouse2F'); i++) await pause();
  assert(output.includes('PlayersHouse2F'), `Curl did not launch the actual game: ${errors}\n${output.slice(-1600)}`);
  assert(rawOutput.startsWith('\x1b[?1049h'), 'Curl opens directly into the game: no installer logs before the alternate screen');
  const seconds = ((Date.now() - started) / 1000).toFixed(1);
  await pause(); await pause();
  assert(/[\u2801-\u28ff]/u.test(output) && !output.includes('▀'), `Downloaded client must default to high-resolution dot illustration: ${output.slice(-1800)}`);
  const graphics = process.env.GEOTHITE_TUI_GRAPHICS === 'kitty';
  if (graphics) assert(rawOutput.includes('a=T,f=100,t=d,i='), 'The installed client paints the shared circle canvas too');
  const artWrites = bytes => (bytes.match(graphics ? /a=T,f=100,t=d,i=/g : /[\u2801-\u28ff]/gu) ?? []).length;
  let animationOffset = rawOutput.length;
  for (let i = 0; i < 7; i++) await pause();
  assert(artWrites(rawOutput.slice(animationOffset)) > 1, 'Actual downloaded dither animates while idle');
  const repeat = setInterval(() => child.stdin.write('p'), 15);
  try {
    for (let i = 0; i < 4; i++) await pause(); // Exclude the trailing idle redraw.
    animationOffset = rawOutput.length;
    for (let i = 0; i < 8; i++) await pause();
    assert(artWrites(rawOutput.slice(animationOffset)) > 1, 'Actual downloaded animation survives continuous unmapped input');
  } finally { clearInterval(repeat); }
  let offset = output.length;
  let rawOffset = rawOutput.length;
  await key('v');
  for (let i = 0; i < 30 && !output.slice(offset).includes('GAME BOY'); i++) await pause();
  assert(output.slice(offset).includes('GAME BOY'), 'V switches the installed client to the traditional text view');
  if (graphics) assert(rawOutput.slice(rawOffset).includes('a=d,d=I,i='), 'Installed V clears only owned canvas images');
  offset = output.length;
  rawOffset = rawOutput.length;
  await key('v');
  for (let i = 0; i < 30 && !/[\u2801-\u28ff]/u.test(output.slice(offset)); i++) await pause();
  assert(/[\u2801-\u28ff]/u.test(output.slice(offset)), 'V switches back to painted art without restarting the game');
  if (graphics) assert(rawOutput.slice(rawOffset).includes('a=T,f=100,t=d,i='), 'Installed V restores the shared canvas');
  await key('\r');
  for (let i = 0; i < 20 && !output.includes('SAVE'); i++) await pause();
  assert(output.includes('SAVE'), `Installed client exposes the production Start menu: ${output.slice(-1800)}`);
  await key('x'); await pause(); await pause(); await key('\x1b[15~'); // B, F5
  let saves = [];
  for (let i = 0; i < 50 && !saves.length; i++) {
    await pause(); saves = (await readdir(join(root, 'installed/saves'))).filter(name => name.endsWith('.crystalsave'));
  }
  assert(saves.length, `Installed client saves without needing checkout paths: ${output.slice(-1500)}`);
  assert.equal(errors, '');
  console.log(`Actual curl install launched painted CHRIS in ${seconds}s; V text/paint, production Start and save passed outside the checkout.`);
} finally {
  child.stdin.write('\x03');
  await pause(); await pause(); await pause();
  child.stdin.end(); child.kill('SIGTERM');
  if (child.exitCode === null && child.signalCode === null) await new Promise(resolve => child.once('close', resolve));
  const savedFiles = (await readdir(join(root, 'installed/saves')).catch(() => [])).filter(name => name.endsWith('.crystalsave'));
  if (savedFiles.length) {
    const savePath = join(root, 'installed/saves', savedFiles[0]);
    const savedBytes = await readFile(savePath);
    const reinstall = spawn(shell[0], [...shell.slice(1), '-c', 'curl -fsSL "$GEOTHITE_DOWNLOAD_URL/install.sh" | sh -s -- --no-play'], {
      cwd: root, env: { ...process.env, GEOTHITE_HOME: join(root, 'installed'), GEOTHITE_DOWNLOAD_URL: base },
      stdio: ['ignore', 'pipe', 'pipe'],
    });
    let reinstallOutput = '';
    reinstall.stdout.on('data', bytes => reinstallOutput += bytes);
    reinstall.stderr.on('data', bytes => reinstallOutput += bytes);
    const reinstallStatus = await new Promise((resolve, reject) => {
      reinstall.once('error', reject); reinstall.once('close', resolve);
    });
    assert.equal(reinstallStatus, 0, `Actual curl reinstall failed: ${reinstallOutput}`);
    assert.equal(reinstallOutput, '', 'Actual curl reinstall is silent');
    assert.deepEqual(await readFile(savePath), savedBytes, 'Reinstall preserves the actual player save byte for byte');
    const mcp = spawn(join(root, 'installed/bin/geothite'), ['mcp'], { cwd: root });
    let id = 0, stderr = '';
    const pending = new Map();
    mcp.stderr.on('data', bytes => stderr += bytes);
    createInterface({ input: mcp.stdout }).on('line', line => {
      const response = JSON.parse(line); pending.get(response.id)?.(response); pending.delete(response.id);
    });
    async function call(name, args = {}) {
      const request = ++id;
      const reply = new Promise((resolve, reject) => {
        const timeout = setTimeout(() => reject(new Error(`Installed MCP timeout: ${stderr}`)), 10000);
        pending.set(request, value => { clearTimeout(timeout); resolve(value); });
      });
      mcp.stdin.write(JSON.stringify({ jsonrpc: '2.0', id: request, method: 'tools/call', params: { name, arguments: args } }) + '\n');
      const response = await reply;
      assert(!response.error && !response.result?.isError, JSON.stringify(response));
      return response.result.structuredContent;
    }
    try {
      // Saving is also valid while Start is open; close any restored menu with
      // normal inputs before testing overworld movement.
      await call('press', { button: 'b' }); await call('press', { button: 'b' });
      const before = await call('observe'); assert.match(before.status_line, /PlayersHouse2F/);
      let moved = false;
      for (const button of ['right', 'down', 'left', 'up']) {
        await call('press', { button }); await call('press', { button });
        if (JSON.stringify((await call('observe')).marker) !== JSON.stringify(before.marker)) { moved = true; break; }
      }
      assert(moved, `Installed MCP changes authoritative player position: ${JSON.stringify(await call('observe'))}`);
      await call('press', { button: 'start' });
      assert((await call('observe')).menu.some(line => line.text.includes('SAVE')));
      assert.equal(stderr, '');
      console.log('Actual curl reinstall preserved the save; installed MCP resumed it, moved and opened production Start.');
    } finally { mcp.stdin.end(); mcp.kill('SIGTERM'); }
  }
  if (process.env.GEOTHITE_KEEP_INSTALL === '1') console.log(`Verified install retained at ${root}/installed`);
  else await rm(root, { recursive: true, force: true });
}
