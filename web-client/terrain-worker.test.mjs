import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';
import vm from 'node:vm';

const source = (await readFile(new URL('./terrain-worker.js', import.meta.url), 'utf8'))
  .replace(/^import .+;$/m, '');
const turn = () => new Promise(resolve => setImmediate(resolve));
function fixture() {
  const messages = [];
  let builds = 0;
  let freed = 0;
  const self = { postMessage: message => messages.push({ ...message, freed }) };
  vm.runInNewContext(source, { self, performance, Uint8Array,
    init: () => Promise.resolve({ memory: { buffer: new ArrayBuffer(0) } }),
    install_scenery_sources() {},
    begin_terrain_job: () => {
      builds++;
      return { free: () => freed++, next_page: () => new Uint8Array(), finished: () => false,
        header: () => new Uint8Array(), raw_bytes: () => 0, trace_origin_epoch: () => 0 };
    } });
  return { messages, send: data => self.onmessage({ data }), builds: () => builds };
}

test('the worker advertises its cancellation protocol and frees a running transport before acknowledging', async () => {
  const { send, messages, builds } = fixture();
  send({ kind: 'initialize', id: 'boot' });
  await turn();
  assert.equal(messages[0].cancelAcknowledgements, true);
  send({ kind: 'terrain-pages', id: 1, request: new ArrayBuffer(0) });
  await turn();
  assert.equal(builds(), 1);
  assert.equal(messages.at(-1).kind, 'header');
  send({ kind: 'cancel', id: 1 });
  assert.equal(messages.at(-1).kind, 'cancelled');
  assert.equal(messages.at(-1).id, 1);
  assert.equal(messages.at(-1).freed, 1);
});

test('cancellation received before a legacy queued request prevents that request from building', async () => {
  const { send, messages, builds } = fixture();
  send({ kind: 'cancel', id: 9 });
  send({ kind: 'terrain-pages', id: 9, request: new ArrayBuffer(0) });
  await turn();
  assert.equal(builds(), 0);
  assert.equal(messages[0].kind, 'cancelled');
});
