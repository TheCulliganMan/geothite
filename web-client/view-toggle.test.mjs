import test from 'node:test';
import assert from 'node:assert/strict';
import { mountViewToggle } from './view-toggle.js';

function mount(saved, savedCamera = {}) {
  const values = new Map(saved === undefined ? [] : [['crystal.display.voxel', saved]]);
  for (const [key, value] of Object.entries(savedCamera)) values.set(key, value);
  const calls = [];
  const cameraCalls = [];
  const control = () => ({ disabled: true, setAttribute() {}, addEventListener(type, handler) { this.click = handler; } });
  const cameraControls = { group: { hidden: true }, zoomOut: control(), zoomIn: control(), rotateLeft: control(), rotateRight: control(), reset: control() };
  const attributes = new Map();
  let click;
  let focused = false;
  const button = {
    disabled: true,
    setAttribute: (key, value) => attributes.set(key, value),
    addEventListener: (type, handler) => { assert.equal(type, 'click'); click = handler; },
  };
  mountViewToggle({ crystal_set_voxel_view: value => calls.push(value), crystal_set_voxel_camera: (...args) => cameraCalls.push(args) }, {
    cameraControls,
    button,
    canvas: { focus: options => { assert.deepEqual(options, { preventScroll: true }); focused = true; } },
    storage: { getItem: key => values.get(key) ?? null, setItem: (key, value) => values.set(key, value) },
  });
  return { cameraControls, cameraCalls, button, attributes, values, calls, click: () => click(), focused: () => focused };
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
  assert.equal(ui.button.title, 'Switch to 2.5D view');
  assert.equal(ui.values.get('crystal.display.voxel'), 'false');
});

test('restores the selected renderer on reload', () => {
  const ui = mount('true');
  assert.deepEqual(ui.calls, [true]);
  assert.equal(ui.attributes.get('aria-pressed'), 'true');
  assert.equal(ui.button.title, 'Switch to 2D view');
});


test('camera controls are shown only in 2.5D and clamp zoom, wrap rotation, and reset', () => {
  const ui = mount();
  const controls = ui.cameraControls;
  assert.equal(controls.group.hidden, true);
  assert.equal(controls.zoomIn.disabled, true);
  ui.click();
  assert.equal(controls.group.hidden, false);
  for (let i = 0; i < 10; i++) controls.zoomIn.click();
  assert.deepEqual(ui.cameraCalls.at(-1), [5, 0]);
  assert.equal(controls.zoomIn.disabled, true);
  assert.equal(controls.reset.textContent, '200%');
  for (let i = 0; i < 10; i++) controls.zoomOut.click();
  assert.deepEqual(ui.cameraCalls.at(-1), [0, 0]);
  assert.equal(controls.zoomOut.disabled, true);
  controls.rotateLeft.click();
  assert.deepEqual(ui.cameraCalls.at(-1), [0, 7]);
  controls.rotateRight.click();
  assert.deepEqual(ui.cameraCalls.at(-1), [0, 0]);
  for (let i = 0; i < 8; i++) controls.rotateRight.click();
  assert.deepEqual(ui.cameraCalls.at(-1), [0, 0]);
  controls.reset.click();
  assert.deepEqual(ui.cameraCalls.at(-1), [1, 0]);
  assert.equal(ui.values.get('crystal.display.zoom'), '1');
  assert.equal(ui.values.get('crystal.display.rotation'), '0');
  assert.equal(ui.focused(), true);
  ui.click();
  assert.equal(controls.group.hidden, true);
});

test('camera preferences survive reload and invalid saved values are sanitized', () => {
  const ui = mount('true', { 'crystal.display.zoom': '4', 'crystal.display.rotation': '6' });
  assert.deepEqual(ui.cameraCalls, [[4, 6]]);
  const invalid = mount('true', { 'crystal.display.zoom': 'NaN', 'crystal.display.rotation': '-1' });
  assert.deepEqual(invalid.cameraCalls, [[1, 0]]);
});
