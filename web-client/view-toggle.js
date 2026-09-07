const VIEW_KEY = 'crystal.display.voxel';
const ZOOM_KEY = 'crystal.display.zoom';
const ROTATION_KEY = 'crystal.display.rotation';

function savedStep(storage, key, defaultValue, maximum) {
  const raw = storage.getItem(key);
  const value = raw === null ? defaultValue : Number(raw);
  return Number.isFinite(value) && value >= 0 && value <= maximum ? value : defaultValue;
}

export function mountViewToggle(wasm, { button, canvas, storage, cameraControls }) {
  let enabled = storage.getItem(VIEW_KEY) === 'true';
  let zoom = savedStep(storage, ZOOM_KEY, 1, 5);
  let rotation = savedStep(storage, ROTATION_KEY, 0, 8);
  const updateCamera = () => {
    wasm.crystal_set_voxel_camera(zoom, rotation);
    cameraControls.group.hidden = !enabled;
    cameraControls.zoomOut.disabled = !enabled || zoom === 0;
    cameraControls.zoomIn.disabled = !enabled || zoom === 5;
    cameraControls.rotateLeft.disabled = !enabled;
    cameraControls.rotateRight.disabled = !enabled;
    cameraControls.reset.disabled = !enabled;
    cameraControls.reset.textContent = `${Math.round(75 + zoom * 25)}%`;
    cameraControls.reset.setAttribute('aria-label', `Reset camera (zoom ${Math.round(75 + zoom * 25)}%)`);
  };
  const saveCamera = () => {
    storage.setItem(ZOOM_KEY, String(zoom));
    storage.setItem(ROTATION_KEY, String(rotation));
  };
  const cameraAction = (control, change) => control.addEventListener('click', () => {
    change();
    updateCamera();
    saveCamera();
    canvas.focus({ preventScroll: true });
  });
  cameraAction(cameraControls.zoomOut, () => { zoom = Math.max(0, zoom - 1); });
  cameraAction(cameraControls.zoomIn, () => { zoom = Math.min(5, zoom + 1); });
  cameraAction(cameraControls.rotateLeft, () => { rotation = (rotation + 7) % 8; });
  cameraAction(cameraControls.rotateRight, () => { rotation = (rotation + 1) % 8; });
  cameraAction(cameraControls.reset, () => { zoom = 1; rotation = 0; });
  let drag = null;
  const endDrag = () => {
    if (!drag) return;
    const id = drag.id;
    drag = null;
    if (canvas.hasPointerCapture(id)) canvas.releasePointerCapture(id);
    saveCamera();
  };
  const update = () => {
    endDrag();
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
  // Use displacement rather than frame time so the world follows the finger.
  const document = canvas.ownerDocument;
  const window = document.defaultView;
  const blocked = () => !enabled || document.hidden ||
    document.querySelector('dialog[open], #player-options[open]') ||
    document.querySelector('#touch-controls')?.disabled;
  canvas.addEventListener('pointerdown', event => {
    if (event.pointerType !== 'touch' || drag || blocked()) return;
    drag = { id: event.pointerId, x: event.clientX, y: event.clientY };
    canvas.setPointerCapture(event.pointerId);
    canvas.focus({ preventScroll: true });
    event.preventDefault();
  });
  canvas.addEventListener('pointermove', event => {
    if (!drag || event.pointerId !== drag.id) return;
    if (blocked()) { endDrag(); return; }
    const { width, height } = canvas.getBoundingClientRect();
    const dx = event.clientX - drag.x;
    const dy = event.clientY - drag.y;
    drag.x = event.clientX;
    drag.y = event.clientY;
    if (width > 0 && height > 0) {
      rotation = ((rotation + dx / width * 8) % 8 + 8) % 8;
      zoom = Math.max(0, Math.min(5, zoom - dy / height * 5));
      updateCamera();
    }
    event.preventDefault();
  });
  for (const type of ['pointerup', 'pointercancel', 'lostpointercapture']) {
    canvas.addEventListener(type, event => {
      if (event.pointerId === drag?.id) endDrag();
    });
  }
  window.addEventListener('blur', endDrag);
  window.addEventListener('pagehide', endDrag);
  document.addEventListener('visibilitychange', () => { if (document.hidden) endDrag(); });
  let moving = false;
  return {
    moveCamera({ yaw = 0, zoom: zoomAxis = 0 }, seconds) {
      if (!enabled || (!yaw && !zoomAxis)) {
        if (moving) saveCamera();
        moving = false;
        return false;
      }
      const dt = Number.isFinite(seconds) ? Math.max(0, Math.min(seconds, 0.05)) : 0;
      rotation = (rotation + yaw * dt * 2 + 8) % 8;
      zoom = Math.max(0, Math.min(5, zoom + zoomAxis * dt * 2));
      moving = true;
      updateCamera();
      return true;
    },
  };
}
