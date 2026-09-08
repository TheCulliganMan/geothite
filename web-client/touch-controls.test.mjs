import test from 'node:test';
import assert from 'node:assert/strict';
import { directionAt, createButtonState } from './touch-controls.js';

test('D-pad slides across cardinal directions with a neutral center and outside release', () => {
  const rect = { left: 10, top: 20, width: 180, height: 180 };
  assert.equal(directionAt(rect, 100, 30), 'up');
  assert.equal(directionAt(rect, 180, 110), 'right');
  assert.equal(directionAt(rect, 100, 190), 'down');
  assert.equal(directionAt(rect, 20, 110), 'left');
  assert.equal(directionAt(rect, 100, 110), null);
  assert.equal(directionAt(rect, 220, 110), null);
});

test('multiple fingers hold independent buttons and sliding releases the old direction first', () => {
  const events = [];
  const state = createButtonState((button, down) => events.push([button, down]));
  state.set(1, 'up'); state.set(2, 'a'); state.set(1, 'right');
  state.set(2, null); state.clear();
  assert.deepEqual(events, [['up', true], ['a', true], ['up', false], ['right', true], ['a', false], ['right', false]]);
});

test('lifting one of two fingers on A does not release the remaining finger', () => {
  const events = [];
  const state = createButtonState((...event) => events.push(event));
  state.set(1, 'a'); state.set(2, 'a'); state.set(1, null);
  assert.deepEqual(events, [['a', true]]);
  state.clear(); state.clear();
  assert.deepEqual(events, [['a', true], ['a', false]]);
});

test('touch cancellation does not drop a controller holding the same button', () => {
  const events = [];
  const state = createButtonState((...event) => events.push(event));
  state.set('gamepad:a', 'a'); state.set(1, 'a');
  state.clear(id => typeof id !== 'string');
  assert.deepEqual(events, [['a', true]]);
  state.set('gamepad:a', null);
  assert.deepEqual(events, [['a', true], ['a', false]]);
});

import { mountTouchControls } from './touch-controls.js';
function mountedPad() {
  class Element extends EventTarget {
    constructor(button) { super(); this.dataset = { gameButton: button }; this.classList = { toggle() {} }; }
    setPointerCapture() {}
    focus() {}
    contains(element) { return ['up', 'down', 'left', 'right'].includes(element.dataset.gameButton); }
    getBoundingClientRect() { return { left: 0, top: 0, width: 180, height: 180 }; }
  }
  const buttons = ['up', 'down', 'left', 'right', 'a', 'b', 'start', 'select'].map(button => new Element(button));
  const pad = new Element(); pad.disabled = false;
  pad.querySelectorAll = selector => selector === '[data-game-button]' ? buttons : [];
  const dpad = new Element();
  const help = new Element();
  const document = new EventTarget();
  document.querySelector = selector => ({ '#touch-controls': pad, '#dpad': dpad, '#help': help })[selector] ?? null;
  const window = new EventTarget();
  const timers = new Map(); let timerId = 0;
  window.performance = { now: () => 100 };
  window.setTimeout = callback => { timers.set(++timerId, callback); return timerId; };
  window.clearTimeout = id => timers.delete(id);
  window.KeyboardEvent = class extends Event { constructor(type, options) { super(type, options); this.code = options.code; } };
  const canvas = new Element(); const events = [];
  for (const type of ['keydown', 'keyup']) canvas.addEventListener(type, event => events.push([type, event.code]));
  mountTouchControls({ document, window, canvas });
  return { window, document, events, buttons, dpad, flush: () => { for (const fn of timers.values()) fn(); timers.clear(); },
    fire(element, type, id = 1, x = 90, y = 10) {
      const event = new Event(type, { cancelable: true });
      Object.assign(event, { pointerId: id, button: 0, clientX: x, clientY: y, detail: 1 });
      element.dispatchEvent(event);
    } };
}

test('quick taps survive pointer capture release long enough to reach a game frame', () => {
  const pad = mountedPad();
  pad.fire(pad.buttons[4], 'pointerdown');
  pad.fire(pad.buttons[4], 'pointerup');
  pad.fire(pad.buttons[4], 'lostpointercapture');
  assert.deepEqual(pad.events, [['keydown', 'KeyZ']]);
  pad.flush();
  assert.deepEqual(pad.events, [['keydown', 'KeyZ'], ['keyup', 'KeyZ']]);
});

test('touch cancellation and window blur immediately release all held keys', () => {
  const pad = mountedPad();
  pad.fire(pad.dpad, 'pointerdown');
  pad.fire(pad.buttons[4], 'pointerdown', 2);
  pad.fire(pad.dpad, 'pointercancel');
  pad.window.dispatchEvent(new Event('blur'));
  pad.flush();
  assert.deepEqual(pad.events, [['keydown', 'ArrowUp'], ['keydown', 'KeyZ'], ['keyup', 'ArrowUp'], ['keyup', 'KeyZ']]);
});

function keyboard(pad, type, code, extra = {}) {
  const event = new Event(type, { cancelable: true });
  Object.assign(event, { code, ...extra }); pad.window.dispatchEvent(event);
}
test('WASD and arrows share held directions without early releases or repeat presses', () => {
  const pad = mountedPad();
  for (const [letter, arrow] of [['KeyW','ArrowUp'],['KeyA','ArrowLeft'],['KeyS','ArrowDown'],['KeyD','ArrowRight']]) {
    const before = pad.events.length;
    keyboard(pad, 'keydown', letter); keyboard(pad, 'keydown', letter, { repeat: true });
    keyboard(pad, 'keydown', arrow); keyboard(pad, 'keyup', letter);
    assert.deepEqual(pad.events.slice(before), [['keydown', arrow]]);
    keyboard(pad, 'keyup', arrow);
    assert.deepEqual(pad.events.slice(before), [['keydown', arrow], ['keyup', arrow]]);
  }
});
test('typing does not move, blur releases WASD, and browser shortcuts remain available', () => {
  const pad = mountedPad();
  pad.document.activeElement = { tagName: 'INPUT' };
  keyboard(pad, 'keydown', 'KeyW'); keyboard(pad, 'keyup', 'KeyW');
  assert.deepEqual(pad.events, []);
  pad.document.activeElement = null;
  keyboard(pad, 'keydown', 'KeyW', { ctrlKey: true });
  assert.deepEqual(pad.events, []);
  keyboard(pad, 'keydown', 'KeyW');
  pad.document.activeElement = { tagName: 'INPUT' };
  pad.document.dispatchEvent(new Event('focusin'));
  assert.deepEqual(pad.events, [['keydown','ArrowUp'],['keyup','ArrowUp']]);
  pad.document.activeElement = null;
  keyboard(pad, 'keydown', 'KeyD'); pad.window.dispatchEvent(new Event('blur'));
  assert.deepEqual(pad.events.slice(-2), [['keydown','ArrowRight'],['keyup','ArrowRight']]);
});
