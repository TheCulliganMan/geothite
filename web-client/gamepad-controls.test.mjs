import test from 'node:test';
import assert from 'node:assert/strict';
import { gamepadButtons, createGamepadInput } from './gamepad-controls.js';
const pad = (pressed = [], axes = [0, 0]) => ({ index: 0, connected: true, mapping: 'standard', axes, buttons: Array.from({ length: 17 }, (_, i) => ({ pressed: pressed.includes(i) })) });
test('standard Steam Deck and console layout maps face buttons, menu buttons and D-pad', () => {
  assert.deepEqual([...gamepadButtons(pad([0, 1, 8, 9, 12]))], ['a', 'b', 'select', 'start', 'up']);
});
test('stick deadzone rejects drift and chooses a single cardinal direction', () => {
  assert.deepEqual([...gamepadButtons(pad([], [.2, -.3]))], []);
  assert.deepEqual([...gamepadButtons(pad([], [.8, -.7]))], ['right']);
  assert.deepEqual([...gamepadButtons(pad([12], [.8, 0]))], ['up']);
});
test('disconnect, focus loss and repeated polling release input exactly once', () => {
  const events = [];
  const input = createGamepadInput((...args) => events.push(args));
  input.update([pad([0, 15])]); input.update([pad([0, 15])]); input.update([]);
  assert.deepEqual(events, [['a', true], ['right', true], ['a', false], ['right', false]]);
  input.update([pad([9])]); input.clear(); input.clear();
  assert.deepEqual(events.slice(-2), [['start', true], ['start', false]]);
});
test('nonstandard controllers are not silently assigned the wrong buttons', () => {
  assert.deepEqual([...gamepadButtons({ ...pad([0]), mapping: '' })], []);
});

import { mountGamepadControls } from './gamepad-controls.js';
test('browser polling hides touch controls on controller input and releases when chat takes focus', () => {
  const window = new EventTarget();
  let frame; let pads = [pad([0, 15])];
  window.requestAnimationFrame = callback => { frame = callback; };
  window.navigator = { getGamepads: () => pads };
  const document = new EventTarget();
  const classes = new Set();
  document.body = { classList: { add: value => classes.add(value), remove: value => classes.delete(value) } };
  document.hasFocus = () => true;
  document.querySelector = selector => selector === '#touch-controls' ? { disabled: false } : null;
  const events = [];
  mountGamepadControls({ document, window, canvas: { focus() {} }, onInput() {}, controls: { setButton: (...args) => events.push(args) } });
  frame(); frame();
  assert.equal(classes.has('controller-active'), true);
  assert.deepEqual(events, [['gamepad:a', 'a'], ['gamepad:right', 'right']]);
  document.activeElement = { tagName: 'INPUT' };
  document.dispatchEvent(new Event('focusin'));
  frame();
  assert.deepEqual(events.slice(-2), [['gamepad:a', null], ['gamepad:right', null]]);
  pads = [];
  window.dispatchEvent(new Event('gamepaddisconnected'));
  assert.equal(classes.has('controller-active'), false);
});

test('stick hysteresis prevents repeated presses around the deadzone and diagonal boundary', () => {
  const events = [];
  const input = createGamepadInput((...args) => events.push(args));
  for (const axes of [[.6, 0], [.49, 0], [.51, 0], [.7, .71], [.7, .8]]) input.update([pad([], axes)]);
  assert.deepEqual(events, [['right', true]]);
  input.update([pad([], [.4, .9])]);
  input.update([pad([], [.1, .2])]);
  assert.deepEqual(events.slice(1), [['right', false], ['down', true], ['down', false]]);
});

import { gamepadCamera } from './gamepad-controls.js';
test('camera stick is analog and independent of a continuously held walking stick', () => {
  const events = [];
  const input = createGamepadInput((...event) => events.push(event));
  for (let frame = 0; frame < 120; frame++) {
    const controller = pad([], [1, 0, .6, -.6]);
    input.update([controller]);
    const camera = gamepadCamera([controller]);
    assert.ok(Math.abs(camera.yaw - .5) < 1e-9);
    assert.ok(Math.abs(camera.zoom - .5) < 1e-9);
  }
  assert.deepEqual(events, [['right', true]]);
  input.clear();
  assert.deepEqual(events.at(-1), ['right', false]);
  assert.deepEqual(gamepadCamera([pad([], [1, 0, .19, NaN])]), { yaw: 0, zoom: 0 });
  assert.deepEqual(gamepadCamera([]), { yaw: 0, zoom: 0 });
});

test('camera polling uses elapsed time and stops immediately on blur or text focus', () => {
  const window = new EventTarget();
  let frame;
  window.requestAnimationFrame = callback => { frame = callback; };
  window.navigator = { getGamepads: () => [pad([], [0, 0, 1, 0])] };
  const document = new EventTarget();
  document.body = { classList: { add() {}, remove() {} } };
  document.hasFocus = () => true;
  document.querySelector = selector => selector === '#touch-controls' ? { disabled: false } : null;
  const calls = [];
  mountGamepadControls({ document, window, canvas: { focus() {} }, onInput() {},
    controls: { setButton() {} }, onCamera: (axes, dt) => { calls.push([axes, dt]); return Boolean(axes.yaw); } });
  frame(100); frame(120);
  assert.deepEqual(calls.at(-1), [{ yaw: 1, zoom: 0 }, .02]);
  window.dispatchEvent(new Event('blur'));
  assert.deepEqual(calls.at(-1), [{ yaw: 0, zoom: 0 }, 0]);
  document.activeElement = { tagName: 'INPUT' };
  frame(200);
  assert.deepEqual(calls.at(-1), [{ yaw: 0, zoom: 0 }, 0]);
  document.activeElement = null;
  frame(10000);
  assert.deepEqual(calls.at(-1), [{ yaw: 1, zoom: 0 }, 0]);
});

test('a gentle deliberate stick tilt starts walking and hysteresis sustains the hold', () => {
  const events = [];
  const input = createGamepadInput((...args) => events.push(args));
  input.update([pad([], [.4, 0])]);
  input.update([pad([], [.3, 0])]);
  assert.deepEqual(events, [['right', true]]);
  input.update([pad([], [.2, 0])]);
  assert.deepEqual(events.at(-1), ['right', false]);
});
