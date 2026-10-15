import test from 'node:test';
import assert from 'node:assert/strict';
import { JSDOM } from 'jsdom';
import { mountViewToggle } from './view-toggle.js';

function mount(saved, savedCamera = {}, startIn2D = false, startIn3D = false, supportsPose = false) {
  const values = new Map(saved === undefined ? [] : [['crystal.display.voxel', saved]]);
  for (const [key, value] of Object.entries(savedCamera)) values.set(key, value);
  const calls = [];
  const cameraCalls = [];
  const attributes = new Map();
  let click;
  let focused = false;
  const button = {
    disabled: true,
    setAttribute: (key, value) => attributes.set(key, value),
    addEventListener: (type, handler) => { assert.equal(type, 'click'); click = handler; },
  };
  const dom = new JSDOM('<canvas></canvas>', { pretendToBeVisual: true });
  const canvas = dom.window.document.querySelector('canvas');
  canvas.focus = options => { assert.deepEqual(options, { preventScroll: true }); focused = true; };
  let captured;
  canvas.setPointerCapture = id => { captured = id; };
  canvas.hasPointerCapture = id => captured === id;
  canvas.releasePointerCapture = () => { captured = undefined; };
  canvas.getBoundingClientRect = () => ({ width: 400, height: 400 });
  const pointer = (type, x, y, id = 1, pointerType = 'touch') => {
    const event = new dom.window.Event(type, { cancelable: true });
    Object.assign(event, { clientX: x, clientY: y, pointerId: id, pointerType, button: 0 });
    canvas.dispatchEvent(event);
    return event;
  };
  const wasm = { crystal_set_voxel_view: value => calls.push(value), crystal_set_voxel_camera: (...args) => cameraCalls.push(args) };
  if (supportsPose) {
    wasm.crystal_set_voxel_camera_pose = (...args) => cameraCalls.push(args);
    wasm.crystal_cycle_voxel_camera = () => {
      const [zoom, rotation] = cameraCalls.at(-1);
      return JSON.stringify({ zoom, rotation, pitch: 0, firstPerson: true });
    };
  }
  const controller = mountViewToggle(wasm, {
    startIn2D,
    startIn3D,
    button,
    canvas,
    storage: { getItem: key => values.get(key) ?? null, setItem: (key, value) => values.set(key, value) },
  });
  return { pointer, dom, controller, cameraCalls, button, attributes, values, calls, click: () => click(), focused: () => focused };
}

test('starts in 2D, toggles the renderer both ways, and returns keyboard focus to the game', () => {
  const ui = mount();
  assert.deepEqual(ui.calls, [false]);
  assert.equal(ui.button.disabled, false);
  ui.click();
  assert.deepEqual(ui.calls, [false, true]);
  assert.equal(ui.attributes.get('aria-pressed'), 'true');
  assert.equal(ui.attributes.get('aria-label'), 'Switch to 2D view');
  assert.equal(ui.values.get('crystal.display.voxel'), 'true');
  assert.equal(ui.focused(), true);
  ui.click();
  assert.deepEqual(ui.calls, [false, true, false]);
  assert.equal(ui.attributes.get('aria-pressed'), 'false');
  assert.equal(ui.button.title, 'Switch to 3D view');
  assert.equal(ui.values.get('crystal.display.voxel'), 'false');
});

test('restores the selected renderer on reload', () => {
  const ui = mount('true');
  assert.deepEqual(ui.calls, [true]);
  assert.equal(ui.attributes.get('aria-pressed'), 'true');
  assert.equal(ui.button.title, 'Switch to 2D view');
});


test('camera preferences survive reload and invalid saved values are sanitized', () => {
  const ui = mount('true', { 'crystal.display.zoom': '4', 'crystal.display.rotation': '6' });
  assert.deepEqual(ui.cameraCalls, [[4, 6]]);
  const invalid = mount('true', { 'crystal.display.zoom': 'NaN', 'crystal.display.rotation': '-1' });
  assert.deepEqual(invalid.cameraCalls, [[1, 0]]);
});

test('analog camera integrates elapsed time and keeps fractional angles without stepping', () => {
  const ui = mount('true');
  ui.controller.moveCamera({ yaw: 1, zoom: 0 }, 1 / 60);
  const first = ui.cameraCalls.at(-1);
  assert.ok(first[1] > 0 && first[1] < 1);
  for (let i = 1; i < 60; i++) ui.controller.moveCamera({ yaw: 1, zoom: 0 }, 1 / 60);
  assert.ok(Math.abs(ui.cameraCalls.at(-1)[1] - 2) < 1e-9);
  ui.click();
  const count = ui.cameraCalls.length;
  assert.equal(ui.controller.moveCamera({ yaw: 1, zoom: 1 }, 1 / 60), false);
  assert.equal(ui.cameraCalls.length, count);
});


test('touch drag orbits and zooms only in 3D and persists on release', () => {
  const ui = mount();
  assert.equal(ui.pointer('pointerdown', 0, 100).defaultPrevented, false);
  ui.pointer('pointermove', 100, 0);
  assert.deepEqual(ui.cameraCalls.at(-1), [1, 0]);
  ui.click();
  assert.equal(ui.pointer('pointerdown', 0, 100).defaultPrevented, true);
  ui.pointer('pointermove', 100, 0);
  assert.deepEqual(ui.cameraCalls.at(-1), [2.25, 2]);
  ui.pointer('pointerup', 100, 0);
  assert.equal(ui.values.get('crystal.display.rotation'), '2');
  ui.pointer('pointermove', 200, 0);
  assert.deepEqual(ui.cameraCalls.at(-1), [2.25, 2]);
});

test('drag ignores secondary mouse buttons and stops on cancellation, blur, and mode switch', () => {
  const ui = mount('true');
  const rightClick = new ui.dom.window.Event('pointerdown');
  Object.assign(rightClick, { pointerId: 1, pointerType: 'mouse', button: 2 });
  ui.dom.window.document.querySelector('canvas').dispatchEvent(rightClick);
  ui.pointer('pointermove', 100, 0, 1, 'mouse');
  assert.deepEqual(ui.cameraCalls.at(-1), [1, 0]);
  for (const stop of [() => ui.pointer('pointercancel', 0, 0),
    () => ui.dom.window.dispatchEvent(new ui.dom.window.Event('blur')),
    () => ui.click()]) {
    ui.pointer('pointerdown', 0, 0);
    assert.deepEqual(ui.cameraCalls.at(-1), [1, 0]);
    stop();
    ui.pointer('pointermove', 100, 0);
    assert.deepEqual(ui.cameraCalls.at(-1), [1, 0]);
  }
});


test('Flygon starts in 2D despite saved 3D and still permits a manual switch', () => {
  const ui = mount('true', {}, true);
  assert.deepEqual(ui.calls, [false]);
  assert.equal(ui.values.get('crystal.display.voxel'), 'true');
  ui.click();
  assert.deepEqual(ui.calls, [false, true]);
  assert.deepEqual(mount('true', {}, true).calls, [false]);
  assert.deepEqual(mount('true').calls, [true]);
});


test('explicit local modeled preview starts in 3D without rewriting saved preference', () => {
  const ui = mount('false', {}, false, true);
  assert.deepEqual(ui.calls, [true]);
  assert.equal(ui.values.get('crystal.display.voxel'), 'false');
  assert.deepEqual(ui.cameraCalls[0], [0, 7.5]);
  assert.deepEqual(mount('false', {}, true, true).calls, [false]);
});


test('primary mouse drag looks, touch looks in first person, and wheel zoom respects overlays', () => {
  const ui = mount('true', {}, false, false, true);
  ui.pointer('pointerdown', 0, 100, 1, 'mouse');
  ui.pointer('pointermove', 100, 0, 1, 'mouse');
  assert.deepEqual(ui.cameraCalls.at(-1), [1, 6, 15, false]);
  ui.pointer('pointerup', 100, 0);
  ui.controller.cycleCamera();
  ui.pointer('pointerdown', 0, 100);
  ui.pointer('pointermove', 100, 0);
  assert.deepEqual(ui.cameraCalls.at(-1), [1, 4, -30, true]);
  ui.pointer('pointerup', 100, 0);
  const canvas = ui.dom.window.document.querySelector('canvas');
  const wheel = () => {
    const e = new ui.dom.window.WheelEvent('wheel', { deltaY: -100, cancelable: true });
    canvas.dispatchEvent(e);
    return e;
  };
  assert.equal(wheel().defaultPrevented, true);
  assert.deepEqual(ui.cameraCalls.at(-1), [1.5, 4, -30, true]);
  const dialog = ui.dom.window.document.createElement('dialog');
  dialog.setAttribute('open', '');
  ui.dom.window.document.body.append(dialog);
  const count = ui.cameraCalls.length;
  assert.equal(wheel().defaultPrevented, false);
  assert.equal(ui.controller.moveCamera({ yaw: 1 }, 0.05), false);
  assert.equal(ui.cameraCalls.length, count);
});


test('two-finger pinch and trackpad wheel gestures zoom and rotate without buttons', () => {
  const ui = mount('true', {}, false, false, true);
  ui.pointer('pointerdown', 0, 100, 1);
  ui.pointer('pointerdown', 100, 100, 2);
  ui.pointer('pointermove', 200, 100, 2);
  assert.ok(ui.cameraCalls.at(-1)[0] > 3);
  ui.pointer('pointerup', 200, 100, 2);
  const before = ui.cameraCalls.length;
  ui.pointer('pointermove', 250, 100, 1);
  assert.equal(ui.cameraCalls.length, before);
  const canvas = ui.dom.window.document.querySelector('canvas');
  const scroll = new ui.dom.window.WheelEvent('wheel', { deltaX: 100, deltaY: 0, cancelable: true });
  canvas.dispatchEvent(scroll);
  assert.equal(scroll.defaultPrevented, true);
  assert.equal(ui.cameraCalls.at(-1)[1], 7.5);
  const pinch = new ui.dom.window.WheelEvent('wheel', { deltaY: -20, ctrlKey: true, cancelable: true });
  const zoom = ui.cameraCalls.at(-1)[0];
  canvas.dispatchEvent(pinch);
  assert.equal(pinch.defaultPrevented, true);
  assert.ok(ui.cameraCalls.at(-1)[0] > zoom);
});
