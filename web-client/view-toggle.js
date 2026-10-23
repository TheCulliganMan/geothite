const VIEW_KEY = 'crystal.display.voxel';
const ZOOM_KEY = 'crystal.display.zoom';
const ROTATION_KEY = 'crystal.display.rotation';

function savedStep(storage, key, defaultValue, maximum) {
  const raw = storage.getItem(key);
  const value = raw === null ? defaultValue : Number(raw);
  return Number.isFinite(value) && value >= 0 && value <= maximum ? value : defaultValue;
}

export function mountViewToggle(wasm, { button, canvas, storage, startIn2D = false, startIn3D = false }) {
  let enabled = !startIn2D && (startIn3D || storage.getItem(VIEW_KEY) === 'true');
  let zoom = savedStep(storage, ZOOM_KEY, startIn3D ? 0 : 1, 5);
  let rotation = savedStep(storage, ROTATION_KEY, startIn3D ? 7.5 : 0, 8);
  let firstPerson = false;
  let pitch = 45;
  const supportsPose = typeof wasm.crystal_set_voxel_camera_pose === 'function';
  const updateCamera = () => {
    if (supportsPose) wasm.crystal_set_voxel_camera_pose(zoom, rotation, pitch, firstPerson);
    else wasm.crystal_set_voxel_camera(zoom, rotation);
  };
  const saveCamera = () => {
    storage.setItem(ZOOM_KEY, String(zoom));
    storage.setItem(ROTATION_KEY, String(rotation));
  };
  const tilt = amount => { pitch = Math.max(firstPerson ? -85 : 5, Math.min(85, pitch + amount)); };
  let drag = null;
  let pinch = null;
  const endDrag = () => {
    if (!drag) return;
    const ids = pinch ? pinch.points.map(point => point.id) : [drag.id];
    drag = null;
    pinch = null;
    for (const id of ids) if (canvas.hasPointerCapture(id)) canvas.releasePointerCapture(id);
    saveCamera();
  };
  const update = () => {
    endDrag();
    wasm.crystal_set_voxel_view(enabled);
    updateCamera();
    button.setAttribute('aria-pressed', String(enabled));
    button.setAttribute('aria-label', enabled ? 'Switch to 2D view' : 'Switch to 3D view');
    button.title = enabled ? 'Switch to 2D view' : 'Switch to 3D view';
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
    if (blocked()) return;
    if (event.pointerType === 'touch' && drag?.touch && !pinch) {
      const points = [{ id: drag.id, x: drag.x, y: drag.y }, { id: event.pointerId, x: event.clientX, y: event.clientY }];
      pinch = { points, distance: Math.hypot(points[1].x - points[0].x, points[1].y - points[0].y) };
      canvas.setPointerCapture(event.pointerId);
      event.preventDefault();
      return;
    }
    if ((event.pointerType !== 'touch' && event.button !== 0 && event.button !== 2) || drag) return;
    drag = { id: event.pointerId, x: event.clientX, y: event.clientY, touch: event.pointerType === 'touch' };
    canvas.setPointerCapture(event.pointerId);
    canvas.focus({ preventScroll: true });
    event.preventDefault();
  });
  canvas.addEventListener('pointermove', event => {
    if (pinch) {
      if (blocked()) { endDrag(); return; }
      const point = pinch.points.find(point => point.id === event.pointerId);
      if (!point) return;
      point.x = event.clientX; point.y = event.clientY;
      const [a, b] = pinch.points;
      const distance = Math.hypot(b.x - a.x, b.y - a.y);
      if (pinch.distance > 0 && distance > 0) {
        zoom = Math.max(0, Math.min(5, zoom + Math.log(distance / pinch.distance) * 3));
        updateCamera();
      }
      pinch.distance = distance;
      event.preventDefault();
      return;
    }
    if (!drag || event.pointerId !== drag.id) return;
    if (blocked()) { endDrag(); return; }
    const { width, height } = canvas.getBoundingClientRect();
    const dx = event.clientX - drag.x;
    const dy = event.clientY - drag.y;
    drag.x = event.clientX;
    drag.y = event.clientY;
    if (width > 0 && height > 0) {
      rotation = ((rotation + (supportsPose && (firstPerson || !drag.touch) ? -dx : dx) / width * 8) % 8 + 8) % 8;
      if (supportsPose && (firstPerson || !drag.touch)) tilt(dy / height * 120);
      else zoom = Math.max(0, Math.min(5, zoom - dy / height * 5));
      updateCamera();
    }
    event.preventDefault();
  });
  for (const type of ['pointerup', 'pointercancel', 'lostpointercapture']) {
    canvas.addEventListener(type, event => {
      if (event.pointerId === drag?.id || pinch?.points.some(point => point.id === event.pointerId)) endDrag();
    });
  }
  canvas.addEventListener('wheel', event => {
    if (blocked() || !Number.isFinite(event.deltaY) || !Number.isFinite(event.deltaX)) return;
    event.preventDefault();
    const unit = event.deltaMode === 1 ? 16 : event.deltaMode === 2 ? canvas.getBoundingClientRect().height : 1;
    zoom = Math.max(0, Math.min(5, zoom - Math.max(-200, Math.min(200, event.deltaY * unit)) / (event.ctrlKey ? 80 : 200)));
    if (!event.ctrlKey) rotation = ((rotation - event.deltaX * unit / 200) % 8 + 8) % 8;
    updateCamera();
    saveCamera();
  }, { passive: false });
  canvas.addEventListener('contextmenu', event => { if (enabled) event.preventDefault(); });
  window.addEventListener('blur', endDrag);
  window.addEventListener('pagehide', endDrag);
  document.addEventListener('visibilitychange', () => { if (document.hidden) endDrag(); });
  let moving = false;
  return {
    setEnabled(value) {
      enabled = Boolean(value); update(); storage.setItem(VIEW_KEY, String(enabled));
    },
    cycleCamera() {
      if (blocked() || typeof wasm.crystal_cycle_voxel_camera !== 'function') return false;
      const state = wasm.crystal_cycle_voxel_camera();
      if (!state) return false;
      endDrag();
      ({ zoom, rotation, pitch, firstPerson } = JSON.parse(state));
      updateCamera();
      saveCamera();
      return true;
    },
    moveCamera({ yaw = 0, zoom: zoomAxis = 0 }, seconds) {
      if (blocked() || !Number.isFinite(yaw) || !Number.isFinite(zoomAxis) || (!yaw && !zoomAxis)) {
        if (moving) saveCamera();
        moving = false;
        return false;
      }
      const dt = Number.isFinite(seconds) ? Math.max(0, Math.min(seconds, 0.05)) : 0;
      rotation = (rotation + (supportsPose ? -yaw : yaw) * dt * 2 + 8) % 8;
      if (firstPerson && supportsPose) tilt(-zoomAxis * dt * 100);
      else zoom = Math.max(0, Math.min(5, zoom + zoomAxis * dt * 2));
      moving = true;
      updateCamera();
      return true;
    },
  };
}
