// Installer safety tests use tiny synthetic downloads, not gameplay fixtures.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { mkdtemp, readFile, readlink, writeFile, rm } from 'node:fs/promises';
import { createServer } from 'node:http';
import { tmpdir, platform, arch } from 'node:os';
import { join, resolve } from 'node:path';
import { spawn } from 'node:child_process';
import { gzipSync } from 'node:zlib';
import test from 'node:test';
const sha = bytes => createHash('sha256').update(bytes).digest('hex');
const target = `${arch() === 'arm64' ? 'aarch64' : 'x86_64'}-${platform() === 'darwin' ? 'apple-darwin' : 'unknown-linux-gnu'}`;
const run = (command, args, env) => new Promise(resolve => {
  const child = spawn(command, args, { env: { ...process.env, ...env } });
  child.stdin.end();
  let output = '';
  child.stdout.on('data', bytes => output += bytes); child.stderr.on('data', bytes => output += bytes);
  child.on('error', error => resolve({ code: 127, output: String(error) }));
  child.on('close', code => resolve({ code, output }));
});
test('verified install, launcher args, reinstall preservation and corrupt downloads', async () => {
  const root = await mkdtemp(join(tmpdir(), 'geothite-installer-test-'));
  const pack = Buffer.from('synthetic installer-only pack');
  const binary = gzipSync('#!/bin/sh\nprintf "argument=%s\\n" "$@"\n');
  const launcher = await readFile(new URL('./terminal-play.sh', import.meta.url));
  let corruptBinary = false, corruptPack = false, release = 'test-1';
  const manifest = () => `release ${release}\npack ${sha(pack)}\nlauncher ${sha(launcher)}\n${target} ${sha(binary)}\n`;
  const requests = [];
  const server = createServer((req, res) => {
    requests.push(req.url);
    const bytes = req.url.endsWith('current.txt') ? manifest() : req.url.endsWith('.gz') ? (corruptBinary ? Buffer.from('bad') : binary) : req.url.endsWith('play.sh') ? launcher : req.url.endsWith('.crystalpack') ? (corruptPack ? Buffer.from('bad') : pack) : null;
    res.writeHead(bytes === null ? 404 : 200); res.end(bytes);
  });
  await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
  const env = { GEOTHITE_HOME: join(root, 'install with spaces'), GEOTHITE_DOWNLOAD_URL: `http://127.0.0.1:${server.address().port}` };
  const install = () => run('sh', [resolve('web-client/install.sh'), '--no-play'], env);
  try {
    let result = await install(); assert.equal(result.code, 0, result.output);
    assert.equal(result.output, '', 'Successful installation is silent');
    const executable = join(env.GEOTHITE_HOME, 'bin/geothite');
    result = await run(executable, ['mcp', '--name', 'CHRIS'], env);
    assert.equal(result.code, 0, result.output); assert.match(result.output, /argument=mcp/);
    assert.match(result.output, /argument=CHRIS/);
    const save = join(env.GEOTHITE_HOME, 'saves', `${sha(pack)}.crystalsave`);
    await writeFile(save, 'USER SAVE'); requests.length = 0;
    release = 'test-2';
    result = await install(); assert.equal(result.code, 0, result.output);
    assert.equal(result.output, '', 'Successful upgrade is silent');
    assert.match(await readlink(executable), /releases\/test-2-/);
    assert.equal(await readFile(save, 'utf8'), 'USER SAVE');
    assert(!requests.some(url => url.endsWith('.crystalpack')), 'Reinstall should reuse verified pack');
    result = await run(executable, ['dump'], env); assert.match(result.output, /argument=--load/);
    result = await run(executable, ['mcp', '--session', 'alice', '--name', 'CHRIS'], env);
    assert.equal(result.code, 0, result.output);
    assert.match(result.output, /-session-alice\.crystalsave/);
    assert(!result.output.includes('argument=--session'), 'Launcher selects a real save path, not a second dispatcher');
    const named = join(env.GEOTHITE_HOME, 'saves', `${sha(pack)}-session-alice.crystalsave`);
    await writeFile(`${named}.bak`, 'RECOVERY SAVE');
    result = await run(executable, ['mcp', '--session', 'alice'], env);
    assert.match(result.output, /argument=--load/, 'Recovery-only slot must resume too');
    assert.equal(await readFile(save, 'utf8'), 'USER SAVE');
    for (const name of ['../alice', 'alice/bob', 'a'.repeat(65)]) {
      result = await run(executable, ['mcp', '--session', name], env); assert.notEqual(result.code, 0);
    }
    corruptBinary = true;
    result = await install(); assert.notEqual(result.code, 0); assert.match(result.output, /Executable checksum mismatch/);
    assert.equal(await readFile(save, 'utf8'), 'USER SAVE');
    corruptBinary = false; corruptPack = true;
    const clean = { ...env, GEOTHITE_HOME: join(root, 'corrupt-pack') };
    result = await run('sh', [resolve('web-client/install.sh'), '--no-play'], clean);
    assert.notEqual(result.code, 0); assert.match(result.output, /Content checksum mismatch/);
    result = await run('sh', [resolve('web-client/install.sh'), '--no-play'], { ...env, GEOTHITE_HOME: 'relative' });
    assert.notEqual(result.code, 0); assert.match(result.output, /absolute path/);
  } finally {
    server.closeAllConnections();
    await new Promise(resolve => server.close(resolve));
    await rm(root, { recursive: true, force: true });
  }
});
