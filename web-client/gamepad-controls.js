const buttonMap = [[0, 'a'], [1, 'b'], [8, 'select'], [9, 'start'], [12, 'up'], [13, 'down'], [14, 'left'], [15, 'right']];

export function stickDirection(x, y, previous = null) {
  const magnitude = Math.max(Math.abs(x), Math.abs(y));
  if (magnitude < (previous ? 0.25 : 0.35)) return null;
  // Keep the chosen axis near a diagonal so a resting thumb cannot chatter
  // between directions; let deliberate turns cross a small hysteresis band.
  let horizontal = Math.abs(x) > Math.abs(y);
  if (previous && Math.abs(Math.abs(x) - Math.abs(y)) < 0.15) {
    horizontal = previous === 'left' || previous === 'right';
  }
  return horizontal ? (x < 0 ? 'left' : 'right') : (y < 0 ? 'up' : 'down');
}

export function gamepadButtons(pad, previousStick = null) {
  const held = new Set();
  if (!pad?.connected || pad.mapping !== 'standard') return held;
  for (const [index, button] of buttonMap) if (pad.buttons[index]?.pressed) held.add(button);
  const dpad = ['up', 'down', 'left', 'right'].some(button => held.has(button));
  const [x = 0, y = 0] = pad.axes;
  const direction = stickDirection(x, y, previousStick);
  if (!dpad && direction) held.add(direction);
  return held;
}

export function createGamepadInput(emit) {
  let previous = new Set();
  const sticks = new Map();
  const update = pads => {
    const next = new Set();
    const connected = new Set();
    for (const pad of Array.from(pads)) {
      if (!pad?.connected || pad.mapping !== 'standard') continue;
      connected.add(pad.index);
      const prior = sticks.get(pad.index);
      for (const button of gamepadButtons(pad, prior)) next.add(button);
      sticks.set(pad.index, stickDirection(pad.axes[0] ?? 0, pad.axes[1] ?? 0, prior));
    }
    for (const index of sticks.keys()) if (!connected.has(index)) sticks.delete(index);
    for (const button of previous) if (!next.has(button)) emit(button, false);
    for (const button of next) if (!previous.has(button)) emit(button, true);
    previous = next;
  };
  return { update, clear: () => update([]) };
}

// Remap the deadzone so camera velocity starts at zero and reaches full speed.
export function gamepadCamera(pads) {
  const axis = value => Number.isFinite(value) && Math.abs(value) > 0.2
    ? Math.sign(value) * (Math.min(1, Math.abs(value)) - 0.2) / 0.8 : 0;
  for (const pad of Array.from(pads)) {
    if (!pad?.connected || pad.mapping !== 'standard') continue;
    const yaw = axis(pad.axes[2]);
    const zoom = axis(-pad.axes[3]);
    if (yaw || zoom) return { yaw, zoom };
  }
  return { yaw: 0, zoom: 0 };
}

export function mountGamepadControls({ document, window, canvas, controls, onInput, onCamera = () => false }) {
  if (!window.navigator.getGamepads) return;
  const input = createGamepadInput((button, down) => {
    if (down) {
      onInput();
      canvas.focus({ preventScroll: true });
      document.body.classList.add('controller-active');
    }
    controls.setButton(`gamepad:${button}`, down ? button : null);
  });
  const blocked = () => document.hidden || !document.hasFocus() ||
    document.querySelector('#touch-controls').disabled ||
    document.querySelector('dialog[open], #player-options[open]') ||
    /^(INPUT|TEXTAREA|SELECT)$/.test(document.activeElement?.tagName);
  let lastFrame;
  let cameraActive = false;
  const clear = () => {
    input.clear();
    lastFrame = undefined;
    cameraActive = false;
    onCamera({ yaw: 0, zoom: 0 }, 0);
  };
  const poll = timestamp => {
    const seconds = lastFrame === undefined ? 0 : (timestamp - lastFrame) / 1000;
    lastFrame = timestamp;
    if (blocked()) clear();
    else {
      const pads = window.navigator.getGamepads();
      input.update(pads);
      const movingCamera = Boolean(onCamera(gamepadCamera(pads), seconds));
      if (movingCamera && !cameraActive) {
        onInput();
        canvas.focus({ preventScroll: true });
        document.body.classList.add('controller-active');
      }
      cameraActive = movingCamera;
    }
    window.requestAnimationFrame(poll);
  };
  window.requestAnimationFrame(poll);
  window.addEventListener('blur', () => clear());
  window.addEventListener('pagehide', () => clear());
  window.addEventListener('gamepaddisconnected', () => {
    clear();
    if (!Array.from(window.navigator.getGamepads()).some(pad => pad?.connected && pad.mapping === 'standard')) {
      document.body.classList.remove('controller-active');
    }
  });
  document.addEventListener('visibilitychange', () => { if (document.hidden) clear(); });
  document.addEventListener('focusin', () => { if (blocked()) clear(); });
  window.addEventListener('keydown', event => {
    if (event.isTrusted && !blocked() && ['ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight', 'KeyZ', 'KeyX', 'Enter', 'ShiftRight'].includes(event.code)) {
      document.body.classList.add('controller-active');
    }
  });
  document.addEventListener('pointerdown', event => {
    if (event.pointerType === 'touch') document.body.classList.remove('controller-active');
  }, { capture: true });
}
