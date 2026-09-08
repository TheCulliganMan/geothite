import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { JSDOM } from 'jsdom';
import { validateProfile, mountPlayerCustomization, loadKeyBindings } from './player-customization.js';

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
  const dom = new JSDOM(readFileSync(new URL('./index.html', import.meta.url), 'utf8'), { url: 'https://geothite.test/' });
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

test('profile polling resumes after browser back navigation', () => {
  const h = harness(); let timers = 0;
  h.window.setInterval = () => { timers++; return timers; };
  h.window.dispatchEvent(new h.window.Event('pagehide'));
  h.state.enabled = false;
  h.window.dispatchEvent(new h.window.Event('pageshow'));
  assert.equal(timers, 1);
  assert.equal(h.window.document.querySelector('#personalization').hidden, true);
  h.window.dispatchEvent(new h.window.Event('pageshow'));
  assert.equal(timers, 1, 'does not create duplicate polling loops');
  h.dom.window.close();
});

test('key bindings persist independently of profile edits, reject conflicts, and reset explicitly', () => {
  const h = harness(); const doc = h.window.document;
  doc.querySelector('#personalization').click();
  const chat = doc.querySelector('#chat-key'), start = doc.querySelector('#start-key');
  assert.equal(chat.value, 'Enter'); assert.equal(start.value, 'Space');
  chat.value = 'KeyT'; start.value = 'KeyT';
  doc.querySelector('[data-save-bindings]').click();
  assert.match(doc.querySelector('#key-bindings-status').textContent, /different/);
  assert.deepEqual(loadKeyBindings(h.window), { chat: 'Enter', start: 'Space', select: 'Backspace' });
  start.value = 'Enter'; doc.querySelector('[data-save-bindings]').click();
  assert.deepEqual(loadKeyBindings(h.window), { chat: 'KeyT', start: 'Enter', select: 'Backspace' });
  assert.equal(h.submitted(), undefined, 'key bindings do not submit the trainer profile');
  h.ui.close(); doc.querySelector('#personalization').click();
  assert.equal(chat.value, 'KeyT'); assert.equal(start.value, 'Enter');
  doc.querySelector('[data-reset-bindings]').click();
  assert.equal(chat.value, 'Enter'); assert.equal(start.value, 'Space');
  assert.deepEqual(loadKeyBindings(h.window), { chat: 'KeyT', start: 'Enter', select: 'Backspace' });
  doc.querySelector('[data-save-bindings]').click();
  assert.deepEqual(loadKeyBindings(h.window), { chat: 'Enter', start: 'Space', select: 'Backspace' });
  h.window.localStorage.setItem('geothite.key-bindings.v1', '{broken');
  assert.deepEqual(loadKeyBindings(h.window), { chat: 'Enter', start: 'Space', select: 'Backspace' });
  h.dom.window.close();
});

test('Select is saved and older preferences migrate without conflicting with a saved Start key', () => {
  const h = harness(); const doc = h.window.document;
  h.window.localStorage.setItem('geothite.key-bindings.v1', JSON.stringify({ chat: 'KeyT', start: 'Backspace' }));
  assert.deepEqual(loadKeyBindings(h.window), { chat: 'KeyT', start: 'Backspace', select: 'ShiftRight' });
  doc.querySelector('#personalization').click();
  const select = doc.querySelector('#select-key');
  assert.equal(select.value, 'ShiftRight');
  select.value = 'KeyQ'; doc.querySelector('[data-save-bindings]').click();
  h.ui.close(); doc.querySelector('#personalization').click();
  assert.equal(select.value, 'KeyQ');
  assert.equal(loadKeyBindings(h.window).select, 'KeyQ');
  h.dom.window.close();
});
