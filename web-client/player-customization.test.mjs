import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { JSDOM } from 'jsdom';
import { validateProfile, mountPlayerCustomization } from './player-customization.js';

test('profile validation matches the Rust field limits', () => {
  assert.equal(validateProfile({ name: 'KRIS', handle: 'Kris_22', sprite: 1 }), null);
  for (const profile of [
    { name: 'NINECHARS', handle: 'ok', sprite: 0 },
    { name: 'KRIS', handle: 'bad name', sprite: 0 },
    { name: 'KRIS', handle: 'a'.repeat(25), sprite: 0 },
    { name: 'KRIS', handle: 'ok', sprite: 2 },
    { name: ' KRIS', handle: 'ok', sprite: 0 },
  ]) assert.ok(validateProfile(profile));
});

function harness() {
  const dom = new JSDOM(readFileSync(new URL('./index.html', import.meta.url), 'utf8'));
  const { window } = dom;
  const dialog = window.document.querySelector('#personalization-dialog');
  dialog.showModal = () => { dialog.open = true; };
  dialog.close = () => { dialog.open = false; dialog.dispatchEvent(new window.Event('close')); };
  window.setInterval = () => 1; window.clearInterval = () => {};
  const state = { enabled: true, open: false, can_edit: true, profile: { name: 'CHRIS', handle: 'PLAYER1234', sprite: 0 } };
  let submitted;
  const wasm = {
    crystal_customization_poll: () => JSON.stringify(state),
    crystal_customization_open: () => { state.open = true; },
    crystal_customization_close: () => { state.open = false; },
    crystal_customization_save: json => { submitted = JSON.parse(json); },
  };
  const ui = mountPlayerCustomization(wasm, { document: window.document, window });
  return { dom, window, dialog, state, ui, submitted: () => submitted };
}

test('menu opens prefilled profile, submits edits, reports completion and returns focus', () => {
  const h = harness(); const doc = h.window.document;
  const button = doc.querySelector('#personalization'); button.focus(); button.click();
  assert.equal(h.dialog.open, true);
  const form = h.dialog.querySelector('form');
  assert.equal(form.elements.namedItem('name').value, 'CHRIS');
  form.elements.namedItem('name').value = 'KRIS';
  form.elements.namedItem('handle').value = 'Kris_22';
  form.elements.namedItem('sprite').value = '1';
  form.dispatchEvent(new h.window.Event('submit', { cancelable: true }));
  assert.deepEqual(h.submitted(), { name: 'KRIS', handle: 'Kris_22', sprite: 1 });
  assert.equal(h.dialog.querySelector('[type="submit"]').disabled, true);
  h.state.saved = true; h.ui.poll();
  assert.match(h.dialog.querySelector('[role="status"]').textContent, /Saved/);
  assert.equal(h.dialog.querySelector('[type="submit"]').disabled, false);
  h.ui.close(); assert.equal(h.state.open, false); assert.equal(doc.activeElement, button);
  h.dom.window.close();
});

test('invalid edits stay in the dialog; cancel does not save; pack gating hides the entry', () => {
  const h = harness(); const doc = h.window.document;
  doc.querySelector('#personalization').click();
  const form = h.dialog.querySelector('form');
  form.elements.namedItem('handle').value = 'bad handle';
  form.dispatchEvent(new h.window.Event('submit', { cancelable: true }));
  assert.equal(h.submitted(), undefined);
  assert.match(h.dialog.querySelector('[role="status"]').textContent, /underscores/);
  const cancel = new h.window.Event('cancel', { cancelable: true }); h.dialog.dispatchEvent(cancel);
  assert.equal(h.dialog.open, false); assert.equal(h.submitted(), undefined);
  h.state.enabled = false; h.ui.poll(); assert.equal(doc.querySelector('#personalization').hidden, true);
  h.dom.window.close();
});
