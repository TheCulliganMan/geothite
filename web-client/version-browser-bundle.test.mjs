import assert from 'node:assert/strict';
import { execFile } from 'node:child_process';
import { createHash } from 'node:crypto';
import { cp, mkdtemp, readFile, readdir, rm, writeFile } from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { promisify } from 'node:util';
import test from 'node:test';

const run = promisify(execFile);
const root = path.resolve(import.meta.dirname, '..');

for (const workers of [false, true]) {
  test(`versioned renderer bundles preserve WebGPU and matching workers=${workers}`, async t => {
    const directory = await mkdtemp(path.join(os.tmpdir(), 'geothite-bundle-'));
    t.after(() => rm(directory, { recursive: true, force: true }));
    for (const file of await readdir(path.join(root, 'web-client'))) {
      if (file.endsWith('.js') || file === 'index.html') {
        await cp(path.join(root, 'web-client', file), path.join(directory, file));
      }
    }
    const modules = ['crystal-bevy', 'crystal-bevy-webgpu', 'crystal-audio'];
    if (workers) modules.push('crystal-terrain-worker', 'crystal-actor-worker');
    for (const module of modules) {
      await writeFile(path.join(directory, `${module}.js`), `const binary = '${module}_bg.wasm';\n`);
      await writeFile(path.join(directory, `${module}_bg.wasm`), `test bytes for ${module}`);
    }
    await run('sh', [path.join(root, 'tools/version-browser-bundle.sh'), directory]);
    const files = await readdir(directory);
    const manifest = JSON.parse(await readFile(path.join(directory, 'renderer-bundles.json'), 'utf8'));
    assert.deepEqual(Object.keys(manifest), ['webgl', 'webgpu']);
    assert.notEqual(manifest.webgl.binary, manifest.webgpu.binary);
    for (const renderer of Object.values(manifest)) {
      for (const file of [renderer.module, renderer.binary]) {
        assert.ok(files.includes(file.slice(2)), `missing renderer asset ${file}`);
      }
      const glue = await readFile(path.join(directory, renderer.module), 'utf8');
      assert.ok(glue.includes(renderer.binary.slice(2)), 'glue must use its matching binary');
    }
    if (!workers) return;
    const page = await readFile(path.join(directory, 'index.html'), 'utf8');
    for (const name of ['terrain-worker-client', 'terrain-worker-bridge', 'actor-worker-bridge']) {
      const file = files.find(file => new RegExp(`^${name}-[a-f0-9]{64}\\.js$`).test(file));
      assert.ok(file && page.includes(`./${file}`), `page must load the versioned ${name}`);
    }
    for (const file of files.filter(file => /(?:worker|bridge|client|pool|lifecycle)-[a-f0-9]{64}\.js$/.test(file))) {
      const content = await readFile(path.join(directory, file), 'utf8');
      for (const [, reference] of content.matchAll(/['"]\.\/([^'"]+\.(?:js|wasm))['"]/g)) {
        assert.ok(files.includes(reference), `${file} refers to missing ${reference}`);
      }
      if (/^(?:cpu-worker-|terrain-worker-(?:client|bridge)-|actor-worker-bridge-)/.test(file)) {
        const digest = createHash('sha256').update(content).digest('hex');
        assert.ok(file.endsWith(`-${digest}.js`), `changed dependencies invalidated ${file}`);
      }
    }
  });
}
