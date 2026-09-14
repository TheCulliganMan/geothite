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
  const dom = new JSDOM('<canvas tabindex="0"></canvas><input id="other">', { url: 'https://geothite.test/' });
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

import { saveKeyBindings } from './player-customization.js';
test('saved chat and Start bindings apply immediately and unbound Enter never leaks Start', async () => {
  const h = chatHarness();
  try {
    const canvas = h.document.querySelector('canvas');
    const game = [];
    canvas.addEventListener('keydown', event => game.push([event.code, !!event.crystalGameControl]));
    saveKeyBindings(h.window, { chat: 'KeyT', start: 'Space', select: 'Backspace' });
    canvas.focus(); h.key('Enter'); h.key('Enter', 'keyup');
    assert.deepEqual(game, []); assert.equal(h.document.querySelector('form').hidden, true);
    h.key('t', 'keydown', { code: 'KeyT' }); h.key('t', 'keyup', { code: 'KeyT' });
    assert.equal(h.document.activeElement.id, 'chat-message');
    h.key('Escape'); h.key('Escape', 'keyup');
    h.key(' ', 'keydown', { code: 'Space' }); h.key(' ', 'keyup', { code: 'Space' });
    assert.deepEqual(game, [['Enter', true]]);
    await new Promise(resolve => h.window.setTimeout(resolve, 80));
    saveKeyBindings(h.window, { chat: 'KeyM', start: 'Enter', select: 'Backspace' });
    h.key('Enter'); h.key('Enter', 'keyup');
    assert.deepEqual(game, [['Enter', true], ['Enter', true]]);
    assert.equal(h.document.querySelector('form').hidden, true);
    h.key('m', 'keydown', { code: 'KeyM' }); h.key('m', 'keyup', { code: 'KeyM' });
    assert.equal(h.document.activeElement.id, 'chat-message');
    assert.match(h.document.querySelector('.chat-toggle').title, /M/);
  } finally { h.cleanup(); }
});

test('Select defaults to Backspace, can be rebound, and old native Select does not leak', async () => {
  const h = chatHarness();
  try {
    const game = []; const canvas = h.document.querySelector('canvas'); canvas.focus();
    canvas.addEventListener('keydown', event => game.push([event.code, !!event.crystalGameControl]));
    h.key('Backspace'); h.key('Backspace', 'keyup');
    assert.deepEqual(game, [['ShiftRight', true]]);
    await new Promise(resolve => h.window.setTimeout(resolve, 80));
    h.key('Shift', 'keydown', { code: 'ShiftRight' }); h.key('Shift', 'keyup', { code: 'ShiftRight' });
    assert.equal(game.length, 1);
    saveKeyBindings(h.window, { chat: 'Enter', start: 'KeyM', select: 'KeyQ' });
    h.key('q', 'keydown', { code: 'KeyQ' }); h.key('q', 'keyup', { code: 'KeyQ' });
    assert.deepEqual(game, [['ShiftRight', true], ['ShiftRight', true]]);
    assert.throws(() => saveKeyBindings(h.window, { chat: 'Enter', start: 'KeyM', select: 'Enter' }), /different/);
  } finally { h.cleanup(); }
});

import { mountSpeechBubbles } from './social-chat.js';
test('speech bubbles follow visible speakers, escape text, expire, and exclude private/global messages', () => {
  const dom = new JSDOM('<canvas></canvas>', { pretendToBeVisual: true });
  const { window } = dom; const { document } = window;
  let now = 1000; window.Date.now = () => now;
  const canvas = document.querySelector('canvas');
  canvas.getBoundingClientRect = () => ({left:10, top:20, width:400, height:300});
  const speech = mountSpeechBubbles({ document, window, canvas, selfUserId: 'player-1' });
  const heads = [{user_id:'__self__',x:.5,y:.5},{user_id:'player-2',x:.7,y:.6}];
  const chat = (from, text, channel='say') => ({type:'chat',from_user_id:from,text,channel});
  speech.update({ heads, events:[chat('player-1','Hello!'),chat('player-2','<img src=x onerror=alert(1)>'),chat('player-3','secret','whisper'),chat('player-4','global','general')] });
  const bubbles = [...document.querySelectorAll('.speech-bubble')];
  assert.equal(bubbles.length,2); assert.equal(bubbles[0].style.left,'210px');
  assert.equal(bubbles[0].style.top,'162px'); assert.equal(document.querySelector('img'),null);
  speech.update({heads:[{user_id:'__self__',x:.6,y:.4}],events:[chat('player-1','Second message')]});
  assert.equal(document.querySelectorAll('.speech-bubble').length,2);
  assert.equal(bubbles[0].textContent,'Second message'); assert.equal(bubbles[0].style.left,'250px');
  assert.equal(bubbles[1].hidden,true);
  speech.update({heads:[],events:[]}); assert.equal(bubbles[0].hidden,true);
  now=20000; speech.update({heads,events:[]}); assert.equal(document.querySelectorAll('.speech-bubble').length,0);
  speech.destroy(); assert.equal(document.querySelector('#speech-bubbles'),null); window.close();
});

test('Social shows the server directory including offline users without mixing nearby actions', () => {
  const h = chatHarness();
  try {
    h.poll({ connected: true, events: [], players: [] });
    h.document.querySelector('.chat-toggle').click();
    h.document.querySelector('[data-tab="social"]').click();
    assert.deepEqual(h.sent[0], {type:'social_list', query:'', offset:0});
    h.poll({connected:true, players:[], events:[{type:'social_users',query:'',offset:0,total:3,users:[
      {user_id:'player-1',display_name:'CHRIS',online:true},
      {user_id:'player-2',display_name:'GOLD',online:true},
      {user_id:'player-3',display_name:'<img src=x>',online:false},
    ]}]});
    const rows = h.document.querySelectorAll('.directory-list .community-player');
    assert.equal(rows.length, 3); assert.equal(rows[0].disabled, true);
    assert.match(rows[2].textContent, /Offline/); assert.equal(rows[2].querySelector('img'), null);
    rows[2].click(); assert.equal(h.document.querySelector('[data-action="whisper"]').disabled, true);
    assert.equal(h.document.querySelector('[data-action="battle"]'), null);
    rows[1].click(); h.document.querySelector('[data-action="whisper"]').click();
    assert.equal(h.document.querySelector('#chat-message').value, '/w player-2 ');
    assert.equal(h.document.querySelector('[data-tab="chat"]').getAttribute('aria-selected'), 'true');
    h.document.querySelector('[data-tab="social"]').click();
    h.poll({connected:false,players:[],events:[]});
    assert.match(h.document.querySelector('.directory-list').textContent, /Unknown/);
  } finally {h.cleanup();}
});

test('Leaderboard tabs rank metrics, paginate, preserve drafts, and ignore stale responses', () => {
  const h = chatHarness();
  try {
    h.poll({connected:true,players:[],events:[]});
    h.document.querySelector('.chat-toggle').click();
    h.document.querySelector('#chat-message').value = 'draft';
    h.document.querySelector('[data-tab="leaderboard"]').click();
    assert.deepEqual(h.sent.at(-1), {type:'leaderboard',metric:'pvp_wins',offset:0});
    h.poll({connected:true,players:[],events:[{type:'leaderboard',metric:'pvp_wins',total:101,offset:0,entries:[{rank:1,user_id:'player-2',display_name:'GOLD',online:true,value:12}]}]});
    assert.match(h.document.querySelector('.leaderboard-list').textContent, /#1GOLDOnline12/);
    h.document.querySelector('.leaderboard-pages [data-page="next"]').click();
    assert.equal(h.sent.at(-1).offset,100);
    const metric = h.document.querySelector('#leaderboard-metric'); metric.value = 'trades';
    metric.dispatchEvent(new h.window.Event('change'));
    assert.deepEqual(h.sent.at(-1), {type:'leaderboard',metric:'trades',offset:0});
    h.poll({connected:true,players:[],events:[{type:'leaderboard',metric:'pvp_wins',total:1,offset:0,entries:[{rank:1,user_id:'player-2',display_name:'STALE',online:true,value:12}]}]});
    assert.doesNotMatch(h.document.querySelector('.leaderboard-list').textContent,/STALE/);
    h.document.querySelector('[data-tab="chat"]').click();
    assert.equal(h.document.querySelector('#chat-message').value,'draft');
  } finally {h.cleanup();}
});

test('Social search debounces requests and ignores old query results', async () => {
  const h = chatHarness();
  try {
    h.poll({connected:true,players:[],events:[]});
    h.document.querySelector('.chat-toggle').click();h.document.querySelector('[data-tab="social"]').click();
    const search=h.document.querySelector('#social-search');search.value='Gold';search.dispatchEvent(new h.window.Event('input'));
    await new Promise(resolve=>h.window.setTimeout(resolve,300));
    assert.deepEqual(h.sent.at(-1),{type:'social_list',query:'Gold',offset:0});
    h.poll({connected:true,players:[],events:[{type:'social_users',query:'',offset:0,total:1,users:[{user_id:'x',display_name:'STALE',online:true}]}]});
    assert.doesNotMatch(h.document.querySelector('.directory-list').textContent,/STALE/);
  } finally {h.cleanup();}
});

test('community tabs show a reconnect state before the first connection', () => {
  const h = chatHarness();
  try {
    h.poll({ connected: false, players: [], events: [] });
    h.document.querySelector('.chat-toggle').click();
    for (const [tab, list, pages] of [['social', '.directory-list', '.directory-pages'], ['leaderboard', '.leaderboard-list', '.leaderboard-pages']]) {
      h.document.querySelector(`[data-tab="${tab}"]`).click();
      assert.match(h.document.querySelector(list).textContent, /Reconnecting/);
      assert.equal(h.document.querySelector(pages).hidden, true);
      assert.ok([...h.document.querySelectorAll(`${pages} button`)].every(b => b.disabled));
    }
    assert.deepEqual(h.sent, []);
  } finally { h.cleanup(); }
});
