import test from 'node:test';
import assert from 'node:assert/strict';
import { parseChat, channelName } from './social-chat.js';

test('WoW-style channel aliases and normal messages', () => {
  for (const [command, channel] of [['s', 'say'], ['1', 'general'], ['2', 'trade'], ['3', 'lfg']]) {
    assert.deepEqual(parseChat(`/${command} Hello!`), { type: 'chat', channel, target_user_id: null, text: 'Hello!' });
    assert.deepEqual(parseChat(`/${command}`), { select: channel });
  }
  assert.equal(parseChat(' Hello! ', 'custom:friends').channel, 'custom:friends');
});

test('whisper, reply, join, leave, and validation', () => {
  assert.deepEqual(parseChat('/w player-2 Hello!'), { type: 'chat', channel: 'whisper', target_user_id: 'player-2', text: 'Hello!' });
  assert.equal(parseChat('/r Hello!', 'say', 'player-2').target_user_id, 'player-2');
  assert.throws(() => parseChat('/r Hello!'), /No whisper/);
  assert.deepEqual(parseChat('/join Friends'), { type: 'chat_join', channel: 'custom:friends' });
  assert.deepEqual(parseChat('/leave 2'), { type: 'chat_leave', channel: 'trade' });
  assert.equal(channelName('GENERAL'), 'general');
  for (const invalid of ['', 'a'.repeat(281), 'Hello\nworld', '/unknown hi', '/w player-2', '/join bad name']) assert.throws(() => parseChat(invalid));
  assert.equal(parseChat('😀'.repeat(280)).text.length, 560);
});

test('battle and trade requests are explicit and do not collide with the trade channel', () => {
  assert.deepEqual(parseChat('/battle player-2'), { type: 'interaction_request', target_user_id: 'player-2', kind: 'battle' });
  assert.deepEqual(parseChat('/tradewith player-2'), { type: 'interaction_request', target_user_id: 'player-2', kind: 'trade' });
  assert.equal(parseChat('/trade Hello').type, 'chat');
  assert.deepEqual(parseChat('/cancel'), { type: 'interaction_cancel' });
  assert.throws(() => parseChat('/battle'), /trainer/);
});

import { JSDOM } from 'jsdom';
import { readFileSync } from 'node:fs';
const chatStyles = readFileSync(new URL('./social-chat.css', import.meta.url), 'utf8');
import { mountSocialChat } from './social-chat.js';
function chatHarness() {
  const dom = new JSDOM('<canvas tabindex="0"></canvas><input id="other">');
  const { window } = dom;
  const style = window.document.createElement('style');
  style.textContent = chatStyles;
  window.document.head.append(style);
  let state = { connected: true, events: [], players: [] };
  let poll;
  window.setInterval = fn => { poll = fn; return 1; };
  window.clearInterval = () => {};
  const sent = [];
  const cleanup = mountSocialChat({ crystal_social_focus() {}, crystal_social_send: json => sent.push(JSON.parse(json)), crystal_social_poll: () => JSON.stringify(state) }, { window, document: window.document, playerId: 1 });
  const key = (key, type = 'keydown', extra = {}) => window.document.activeElement.dispatchEvent(new window.KeyboardEvent(type, { key, code: key, bubbles: true, cancelable: true, ...extra }));
  return { window, document: window.document, sent, key, poll: next => { state = next; poll(); }, cleanup: () => { cleanup(); window.close(); } };
}

test('Enter opens and sends without closing chat or leaking into gameplay', () => {
  const h = chatHarness();
  try {
    const leaked = [];
    h.window.addEventListener('keydown', e => leaked.push(e.key));
    h.window.addEventListener('keyup', e => leaked.push(e.key));
    h.poll({ connected: true, events: [], players: [] });
    h.document.querySelector('canvas').focus();
    h.key('Enter'); h.key('Enter', 'keyup');
    assert.equal(h.document.activeElement.id, 'chat-message');
    assert.doesNotMatch(h.document.body.textContent, /World chat|Connected to world chat|Welcome!/);
    h.document.activeElement.value = 'hello';
    h.key('Enter'); h.key('Enter', 'keyup');
    assert.equal(h.sent[0].text, 'hello');
    assert.equal(h.document.activeElement.id, 'chat-message');
    assert.equal(h.document.activeElement.value, '');
    assert.equal(h.document.querySelector('form').hidden, false);
    h.key('Enter'); h.key('Enter', 'keyup');
    h.document.activeElement.value = '   ';
    h.key('Enter'); h.key('Enter', 'keyup');
    assert.equal(h.sent.length, 1);
    assert.equal(h.document.activeElement.id, 'chat-message');
    assert.equal(h.document.querySelector('form').hidden, false);
    h.key('Escape'); h.key('Escape', 'keyup');
    assert.equal(h.document.activeElement.tagName, 'CANVAS');
    assert.deepEqual(leaked, []);
    h.document.querySelector('#other').focus();
    h.key('Enter');
    assert.equal(h.document.activeElement.id, 'other');
  } finally { h.cleanup(); }
});

test('player selection sends requests and incoming requests have working response buttons', () => {
  const h = chatHarness();
  try {
    h.poll({ connected: true, players: [{ user_id: 'player-2', display_name: 'GOLD' }], selected_player: 'player-2', events: [] });
    h.document.querySelector('[data-action="battle"]').click();
    assert.deepEqual(h.sent[0], { type: 'interaction_request', target_user_id: 'player-2', kind: 'battle' });
    h.poll({ connected: true, players: [], events: [{ type: 'interaction_request', request_id: 'request-1', from_user_id: 'player-2', from_display_name: 'GOLD', kind: 'trade' }] });
    h.document.querySelector('[data-action="accept"]').click();
    assert.deepEqual(h.sent[1], { type: 'interaction_response', request_id: 'request-1', target_user_id: 'player-2', accepted: true });
  } finally { h.cleanup(); }
});

test('closed chat exposes only an accessible bubble, with no visible label', () => {
  const h = chatHarness();
  try {
    const toggle = h.document.querySelector('.chat-toggle');
    assert.equal(toggle.textContent.trim(), '');
    assert.ok(toggle.querySelector('svg[aria-hidden="true"]'));
    assert.equal(toggle.getAttribute('aria-label'), 'Open chat');
    assert.equal(h.document.querySelector('form').hidden, true);
    toggle.click();
    assert.equal(h.document.activeElement.id, 'chat-message');
    h.key('Escape');
    assert.equal(h.document.querySelector('form').hidden, true);
  } finally { h.cleanup(); }
});

test('Escape keeps drafts, touch Start bypasses chat, and failed accepts can be retried', () => {
  const h = chatHarness();
  try {
    h.document.querySelector('canvas').focus();
    const start = new h.window.KeyboardEvent('keydown', { key: 'Enter', code: 'Enter', bubbles: true });
    Object.defineProperty(start, 'crystalGameControl', { value: true });
    h.document.activeElement.dispatchEvent(start);
    assert.equal(h.document.activeElement.tagName, 'CANVAS');
    h.key('Enter');
    h.document.activeElement.value = 'draft';
    h.key('Escape'); h.key('Escape', 'keyup');
    h.key('Enter');
    assert.equal(h.document.activeElement.value, 'draft');
    h.poll({ connected: true, players: [], events: [{ type: 'interaction_request', request_id: 'req', from_user_id: 'player-2', from_display_name: 'GOLD', kind: 'trade' }] });
    const accept = h.document.querySelector('[data-action="accept"]');
    accept.click();
    assert.equal(accept.disabled, true);
    h.poll({ connected: true, players: [], events: [{ type: 'error', code: 'social_error', message: 'Close the current dialogue or menu first.' }] });
    assert.equal(accept.disabled, false);
    h.poll({ connected: true, players: [], events: [{ type: 'interaction_response', request_id: 'req', accepted: false }] });
    assert.equal(h.document.querySelector('[data-action="accept"]'), null);
  } finally { h.cleanup(); }
});


test('Close tab returns to gameplay while retaining drafts and passive messages', () => {
  const h = chatHarness();
  try {
    h.document.querySelector('.chat-toggle').click();
    h.document.querySelector('input#chat-message').value = 'draft';
    const close = h.document.querySelector('.chat-close');
    assert.ok(close, 'expanded chat needs a visible close control');
    assert.equal(close.hidden, false);
    close.click();
    assert.equal(h.document.activeElement.tagName, 'CANVAS');
    assert.equal(h.document.querySelector('form').hidden, true);
    h.poll({ connected: true, players: [], events: [{ type: 'chat', channel: 'say', from_user_id: 'player-2', from_display_name: 'GOLD', text: 'Hello!' }] });
    const line = h.document.querySelector('.chat-line');
    assert.match(line.textContent, /Hello!/);
    assert.equal(h.window.getComputedStyle(h.document.querySelector('.chat-log')).display === 'none', false);
    assert.equal(line.querySelector('button').tabIndex, -1);
    h.document.querySelector('.chat-toggle').click();
    assert.equal(h.document.querySelector('input#chat-message').value, 'draft');
    assert.equal(line.querySelector('button').tabIndex, 0);
  } finally { h.cleanup(); }
});


test('passive messages fade with age and reopening reveals their history', () => {
  const h = chatHarness();
  const originalNow = h.window.Date.now;
  let now = 1000;
  h.window.Date.now = () => now;
  try {
    const state = { connected: true, players: [], events: [] };
    h.poll({ ...state, events: [{ type: 'chat', channel: 'say', text: 'Hello!' }] });
    const line = h.document.querySelector('.chat-line');
    assert.equal(h.window.getComputedStyle(line).opacity, '1');
    now += 10000;
    h.poll(state);
    assert.equal(h.window.getComputedStyle(line).opacity, '0');
    h.document.querySelector('.chat-toggle').click();
    assert.equal(h.window.getComputedStyle(line).opacity, '1');
    h.document.querySelector('.chat-close').click();
    h.poll({ ...state, events: [{ type: 'chat', channel: 'say', text: 'New message' }] });
    assert.equal(h.window.getComputedStyle(h.document.querySelector('.chat-line:last-child')).opacity, '1');
  } finally { h.window.Date.now = originalNow; h.cleanup(); }
});

test('mobile chat tracks the visible viewport above the keyboard and clears its open state', () => {
  const dom = new JSDOM('<canvas tabindex="0"></canvas>');
  const { window } = dom;
  const viewport = new window.EventTarget();
  viewport.height = 350;
  viewport.offsetTop = 50;
  Object.defineProperty(window, 'visualViewport', { value: viewport });
  const cleanup = mountSocialChat({
    crystal_social_focus() {}, crystal_social_poll: () => '{"connected":true,"events":[],"players":[]}',
  }, { window, document: window.document, playerId: 1 });
  try {
    const panel = window.document.querySelector('#social-chat');
    panel.querySelector('.chat-toggle').click();
    assert.equal(window.document.body.classList.contains('chat-open'), true);
    assert.equal(panel.style.getPropertyValue('--chat-viewport-height'), '350px');
    assert.equal(panel.style.getPropertyValue('--chat-viewport-bottom'), (window.innerHeight - 400) + 'px');
    viewport.height = 280;
    viewport.dispatchEvent(new window.Event('resize'));
    assert.equal(panel.style.getPropertyValue('--chat-viewport-height'), '280px');
    panel.querySelector('.chat-close').click();
    assert.equal(window.document.body.classList.contains('chat-open'), false);
    cleanup();
    viewport.height = 700;
    viewport.dispatchEvent(new window.Event('resize'));
    assert.equal(panel.style.getPropertyValue('--chat-viewport-height'), '280px');
  } finally { window.close(); }
});

test('connection status recovers visibly without duplicating reconnect errors in the log', () => {
  const h = chatHarness();
  try {
    const status = h.document.querySelector('.chat-status');
    assert.ok(status);
    h.poll({ connected: true, events: [], players: [] });
    assert.equal(status.textContent, 'Connected');
    h.poll({ connected: false, events: [], players: [] });
    assert.equal(status.textContent, 'Reconnecting…');
    h.poll({ connected: true, events: [], players: [] });
    assert.equal(status.textContent, 'Connected');
    assert.equal(h.document.querySelector('.chat-log').textContent.includes('Reconnecting'), false);
  } finally { h.cleanup(); }
});

test('closing chat while its opener is held cannot leak repeated Start presses', () => {
  const h = chatHarness();
  try {
    const leaked = [];
    h.window.addEventListener('keydown', event => leaked.push(event.code));
    h.window.addEventListener('keyup', event => leaked.push(event.code));
    h.document.querySelector('canvas').focus();
    h.key('Enter');
    h.document.querySelector('.chat-close').click();
    h.key('Enter', 'keydown', { repeat: true });
    h.key('Enter', 'keyup');
    assert.deepEqual(leaked, [], 'chat owns the entire press through release');
    assert.equal(h.document.querySelector('form').hidden, true);
  } finally { h.cleanup(); }
});
