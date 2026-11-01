// Exercise the actual raw terminal with lowercase keyboard A, not the MCP alias.
// Python's stdlib supplies the PTY only; no game code or file extraction.
import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { resolve } from 'node:path';
import { StringDecoder } from 'node:string_decoder';
import { PtyScreen } from './tui-pty-screen.mjs';
import { checkHome } from './tui-home-checks.mjs';
const binary = resolve(process.env.TUI_NATIVE_BIN ?? 'target/release/geothite');
const pack = resolve(process.env.TUI_NATIVE_PACK ?? 'content-packs/realtime-clock.browser.crystalpack');
const fixture = resolve(process.env.TUI_NATIVE_FIXTURE ?? 'target/tui-smoke/lowlevel.crystalsave');
const child = spawn('python3', ['-u', '-c', `
import os, pty, fcntl, termios, struct, subprocess, select, sys, signal
master, slave = pty.openpty()
fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack('HHHH', 40 if os.environ.get('TUI_NATIVE_HOME') else 24, 120 if os.environ.get('TUI_NATIVE_HOME') else 80, 0, 0))
def attach():
    os.setsid()
    fcntl.ioctl(slave, termios.TIOCSCTTY, 0)
game = subprocess.Popen(sys.argv[1:], stdin=slave, stdout=slave, stderr=slave, preexec_fn=attach)
os.close(slave)
try:
    while game.poll() is None:
        for fd in select.select([master, 0], [], [], .1)[0]:
            data = os.read(fd, 65536)
            if not data: sys.exit()
            os.write(1 if fd == master else master, data)
finally:
    if game.poll() is None: os.killpg(game.pid, signal.SIGTERM)
    game.wait()
    os.close(master)
`, binary, 'play', pack, ...(process.env.TUI_NATIVE_HOME ? [] : ['--load', fixture])], { stdio: ['pipe', 'pipe', 'pipe'] });
let output = '', error = '';
const decoder = new StringDecoder('utf8');
const screen = new PtyScreen(process.env.TUI_NATIVE_HOME ? 120 : 80, process.env.TUI_NATIVE_HOME ? 40 : 24);
const ansi = /\x1b\[[0-?]*[ -/]*[@-~]/g;
child.stdout.on('data', bytes => { const text = decoder.write(bytes); screen.feed(text); output += text.replace(ansi, ''); });
child.stderr.on('data', bytes => { error += bytes; });
const pause = () => new Promise(resolve => setTimeout(resolve, process.env.TUI_NATIVE_HOME ? 200 : 80));
async function key(value) { child.stdin.write(value); await pause(); }
try {
  if (process.env.TUI_NATIVE_HOME) {
    for (let i = 0; i < 100 && !screen.text().includes('PlayersHouse2F'); i++) await pause();
    assert(screen.text().includes('PlayersHouse2F'), `Native home did not load: ${error}`);
    await key('v');
    assert(screen.text().includes('GAME BOY'), 'Native home verification reads the actual text screen');
    const observe = async () => {
      const text = screen.text();
      const status = text.match(/\b[A-Za-z]+\s+PlayersHouse[12]F\s+\(\d+,\s*\d+\)\s+facing\s+\w+/)?.[0];
      assert(status, `Missing current home status: ${text}`);
      const marker = status.match(/\((\d+),\s*(\d+)\)/).slice(1).map(Number);
      const rows = text.split('\n').map(text => ({ text }));
      const info = rows.map(row => ({ text: row.text.split('││')[1]?.replace(/│$/, '').trim() ?? '' }));
      const dialogue = info.slice(info.findIndex(row => row.text === 'DIALOGUE') + 1,
        info.findIndex(row => row.text === 'TRAINER')).filter(row => row.text);
      return { status_line: status, marker, viewport_title: status.match(/PlayersHouse[12]F/)[0], dialogue, menu: status.startsWith('Menu') ? rows : [] };
    };
    const keys = { up: '\x1b[A', down: '\x1b[B', right: '\x1b[C', left: '\x1b[D', a: 'a', b: 'x', start: '\r' };
    await checkHome(observe, async button => key(button === 'a' && (await observe()).status_line.startsWith('Overworld') ? 'z' : keys[button]));
    assert.equal(error, '');
    assert(!/crystal-bevy (?:movement|dialogue)|input_trace/.test(output), 'No debug logs in fullscreen');
    console.log('Native PTY: real lowercase A, Mom repeat dialogue, immediate Start and doormat movement passed.');
  } else {
  for (let i = 0; i < 100 && !output.includes('Route29'); i++) await pause();
  assert(output.includes('Route29'), `Terminal did not load: ${error}`);
  assert(/[\u2801-\u28ff]/u.test(output) && !output.includes('▀'), 'Native terminal packs the shared high-resolution dot field');
  await key('v');
  assert(output.includes('GAME BOY'), 'Native V exposes the traditional text view');
  await key('v');
  for (let i = 0; i < 256 && !output.includes('WildBattle'); i++) await key(Math.floor(i / 2) % 2 ? 'a' : 'd');
  assert(output.includes('WildBattle'), 'Native terminal enters a real encounter');
  const entry = output.indexOf('WildBattle');
  // A final grass-walking key can land on the newly opened command menu before
  // the PTY paint is delivered. Close a possible submenu and select FIGHT using
  // actual Game Boy inputs instead of assuming its cursor stayed at zero.
  // Use unambiguous B keys: adjacent bare ESC bytes can be combined by a PTY's
  // escape-sequence parser on Linux, leaving the party submenu open.
  for (let i = 0; i < 4; i++) await key('x');
  await key('\x1b[A'); await key('\x1b[D');
  for (let i = 0; i < 256 && !output.slice(entry).includes('Overworld'); i++) await key('a');
  assert(output.slice(entry).includes('Overworld'), `Native lowercase A froze battle: ${output.slice(-1800)}`);
  await key('\x1b[A'); await key('\x1b[A'); await key('\r');
  await pause();
  // The Start menu may already be drawn by the first pause; inspect retained
  // post-battle output rather than a dump made by a different session.
  assert(output.slice(entry).includes('SAVE'), `Native Start remains blocked: ${output.slice(-1800)}`);
  assert.equal(error, '');
  console.log('Native PTY: lowercase A completed a real battle and Start reopened afterward.');
  }
} finally { child.stdin.write('\x03'); child.stdin.end(); child.kill('SIGTERM'); }
