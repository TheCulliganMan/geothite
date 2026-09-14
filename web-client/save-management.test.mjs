import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import { JSDOM } from 'jsdom';
import { createSaveLink, readSaveLink, mountSaveManagement } from './save-management.js';

const bytes = new TextEncoder().encode('current Rust save snapshot '.repeat(200));
test('bookmark links round-trip and strip authentication and remote server parameters', async () => {
  const link = await createSaveLink(bytes, 'https://game.example/?multiplayer_token=SECRET&multiplayer_player_id=42&multiplayer_server=wss://other.example#old');
  const url = new URL(link);
  assert.equal(url.search, '');
  assert.equal(link.includes('SECRET'), false);
  assert.deepEqual(await readSaveLink(url.hash), bytes);
  assert.equal(new URL(await createSaveLink(bytes, 'https://game.example/?multiplayer=off')).search, '?multiplayer=off');
});
test('bad, unsupported and excessive share payloads are rejected', async () => {
  for (const hash of ['#geothite-save=2.abc', '#geothite-save=1.!!!', '#geothite-save=1.abc', '#geothite-save=1.' + 'a'.repeat(64000)]) {
    await assert.rejects(readSaveLink(hash));
  }
  await assert.rejects(createSaveLink(new Uint8Array(4 * 1024 * 1024 + 1), 'https://game.example'));
  const huge = new Uint8Array(4 * 1024 * 1024 + 1);
  const compressed = new Uint8Array(await new Response(new Blob([huge]).stream().pipeThrough(new CompressionStream('gzip'))).arrayBuffer());
  await assert.rejects(readSaveLink('#geothite-save=1.' + Buffer.from(compressed).toString('base64url')), /4 MB/);
});
function harness(url = 'https://game.example') {
  const dom = new JSDOM(fs.readFileSync(new URL('./index.html', import.meta.url), 'utf8'), { url });
  const {window} = dom, {document} = window;
  const dialog = document.querySelector('#save-games-dialog');
  dialog.showModal = () => { dialog.open = true; };
  dialog.close = () => { dialog.open = false; };
  let result, imported = 0, deleted = 0, invalid = false;
  const actions = [];
  const wasm = {
    crystal_save_manager_open() {}, crystal_save_manager_close() {},
    crystal_save_manager_request(action) {
      actions.push(action);
      result = action === 'status' ? { save: {trainer:'KRIS',trainer_id:24},can_save:true,can_replace:true }
        : action === 'inspect' ? invalid ? {error:'Different pack'} : {save:{trainer:'NOVA',trainer_id:77}}
        : {};
      if (action === 'import') imported++;
      if (action === 'delete') deleted++;
    },
    crystal_save_manager_poll: () => JSON.stringify({result}),
    crystal_save_manager_take_bytes: () => bytes,
  };
  const manager = mountSaveManagement(wasm,{document,window});
  return { dom, document, dialog, manager, actions, get imported(){return imported},get deleted(){return deleted},set invalid(value){invalid=value} };
}
const settle = () => new Promise(resolve => setTimeout(resolve, 30));
test('save UI supports saving and requires explicit deletion confirmation', async () => {
  const h = harness();
  try {
    await h.manager.open(); assert.equal(h.dialog.open, true);
    assert.match(h.document.querySelector('[data-summary]').textContent,/KRIS/);
    h.document.querySelector('[data-action=save]').click(); await settle();
    assert.ok(h.actions.includes('save'));
    h.document.querySelector('[data-action=delete]').click();
    assert.equal(h.deleted,0);
    h.document.querySelector('[data-cancel]').click(); assert.equal(h.deleted,0);
    h.document.querySelector('[data-action=delete]').click();
    h.document.querySelector('[data-confirm]').click(); await settle(); assert.equal(h.deleted,1);
    h.manager.close(); assert.equal(h.dialog.open,false);
  } finally { h.dom.window.close(); }
});
test('opening a shared bookmark previews it without overwriting a save', async () => {
  const h = harness(await createSaveLink(bytes,'https://game.example'));
  try {
    for(let i=0;i<50&&!h.actions.includes('inspect');i++) await settle();
    assert.ok(h.actions.includes('inspect'));
    assert.equal(h.imported,0);
    assert.equal(h.dom.window.location.hash,'');
    assert.match(h.document.querySelector('[data-confirmation] p').textContent,/NOVA/);
    h.document.querySelector('[data-confirm]').click(); await settle();
    assert.equal(h.imported,1);
  } finally { h.dom.window.close(); }
});
