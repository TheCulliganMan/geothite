const VIEW_KEY = 'crystal.display.voxel';
const ZOOM_KEY = 'crystal.display.zoom';
const ROTATION_KEY = 'crystal.display.rotation';

function savedStep(storage, key, defaultValue, maximum) {
  const raw = storage.getItem(key);
  const value = raw === null ? defaultValue : Number(raw);
  return Number.isInteger(value) && value >= 0 && value <= maximum ? value : defaultValue;
}

export function mountViewToggle(wasm, { button, canvas, storage, cameraControls }) {
  let enabled = storage.getItem(VIEW_KEY) === 'true';
  let zoom = savedStep(storage, ZOOM_KEY, 1, 5);
  let rotation = savedStep(storage, ROTATION_KEY, 0, 7);
  const updateCamera = () => {
    wasm.crystal_set_voxel_camera(zoom, rotation);
    cameraControls.group.hidden = !enabled;
    cameraControls.zoomOut.disabled = !enabled || zoom === 0;
    cameraControls.zoomIn.disabled = !enabled || zoom === 5;
    cameraControls.rotateLeft.disabled = !enabled;
    cameraControls.rotateRight.disabled = !enabled;
    cameraControls.reset.disabled = !enabled;
    cameraControls.reset.textContent = `${75 + zoom * 25}%`;
    cameraControls.reset.setAttribute('aria-label', `Reset camera (zoom ${75 + zoom * 25}%)`);
  };
  const cameraAction = (control, change) => control.addEventListener('click', () => {
    change();
    updateCamera();
    storage.setItem(ZOOM_KEY, String(zoom));
    storage.setItem(ROTATION_KEY, String(rotation));
    canvas.focus({ preventScroll: true });
  });
  cameraAction(cameraControls.zoomOut, () => { zoom = Math.max(0, zoom - 1); });
  cameraAction(cameraControls.zoomIn, () => { zoom = Math.min(5, zoom + 1); });
  cameraAction(cameraControls.rotateLeft, () => { rotation = (rotation + 7) % 8; });
  cameraAction(cameraControls.rotateRight, () => { rotation = (rotation + 1) % 8; });
  cameraAction(cameraControls.reset, () => { zoom = 1; rotation = 0; });
  const update = () => {
    wasm.crystal_set_voxel_view(enabled);
    updateCamera();
    button.setAttribute('aria-pressed', String(enabled));
    button.setAttribute('aria-label', enabled ? 'Switch to 2D view' : 'Switch to 2.5D view');
    button.title = enabled ? 'Switch to 2D view' : 'Switch to 2.5D view';
  };
  update();
  button.disabled = false;
  button.addEventListener('click', () => {
    enabled = !enabled;
    update();
    storage.setItem(VIEW_KEY, String(enabled));
    canvas.focus({ preventScroll: true });
  });
}
