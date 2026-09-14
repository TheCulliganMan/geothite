const keys = {
  up: ['ArrowUp', 'ArrowUp'], down: ['ArrowDown', 'ArrowDown'],
  left: ['ArrowLeft', 'ArrowLeft'], right: ['ArrowRight', 'ArrowRight'],
  a: ['z', 'KeyZ'], b: ['x', 'KeyX'], start: ['Enter', 'Enter'], select: ['Shift', 'ShiftRight'],
};

export function directionAt(rect, x, y) {
  const dx = (x - rect.left) / rect.width * 2 - 1;
  const dy = (y - rect.top) / rect.height * 2 - 1;
  if (Math.max(Math.abs(dx), Math.abs(dy)) > 1.15 || Math.hypot(dx, dy) < 0.22) return null;
  return Math.abs(dx) > Math.abs(dy) ? (dx < 0 ? 'left' : 'right') : (dy < 0 ? 'up' : 'down');
}

export function createButtonState(emit) {
  const pointers = new Map();
  return {
    set(id, button) {
      const previous = pointers.get(id);
      if (previous === button) return;
      pointers.delete(id);
      if (previous && ![...pointers.values()].includes(previous)) emit(previous, false);
      if (button) {
        const held = [...pointers.values()].includes(button);
        pointers.set(id, button);
        if (!held) emit(button, true);
      }
    },
    clear(predicate = () => true) {
      for (const id of [...pointers.keys()]) if (predicate(id)) this.set(id, null);
    },
  };
}

// Use the same canvas keyboard path as physical controls, so every game surface
// (including names, menus and battles) receives normal held/released input.
export function mountTouchControls({ document, window, canvas, onInput = () => {} }) {
  const pad = document.querySelector('#touch-controls');
  const dpad = document.querySelector('#dpad');
  const movement = { KeyW: 'up', KeyA: 'left', KeyS: 'down', KeyD: 'right', ArrowUp: 'up', ArrowLeft: 'left', ArrowDown: 'down', ArrowRight: 'right' };
  const heldKeyboard = new Set();
  const heldPointers = new Set();
  const pressedAt = new Map();
  const pendingReleases = new Map();
  const state = createButtonState((button, down) => {
    const [key, code] = keys[button];
    const event = new window.KeyboardEvent(down ? 'keydown' : 'keyup', {
      key, code, bubbles: true, cancelable: true, location: button === 'select' ? 2 : 0,
      shiftKey: button === 'select' && down,
    });
    Object.defineProperty(event, 'crystalGameControl', { value: true });
    canvas.dispatchEvent(event);
    for (const control of pad.querySelectorAll(`[data-game-button="${button}"]`)) {
      control.classList.toggle('held', down);
    }
  });
  const releaseAll = () => {
    heldPointers.clear(); pressedAt.clear();
    for (const timer of pendingReleases.values()) window.clearTimeout(timer);
    pendingReleases.clear();
    state.clear(id => typeof id !== 'string' || id.startsWith('keyboard:'));
    heldKeyboard.clear();
  };
  const blocked = () => Boolean(document.querySelector('dialog[open]')) ||
    /^(INPUT|TEXTAREA|SELECT)$/.test(document.activeElement?.tagName);
  const keyboardBlocked = () => blocked() || document.activeElement?.isContentEditable ||
    /^(BUTTON|A)$/.test(document.activeElement?.tagName) || document.activeElement?.closest?.('#social-chat');
  const keyboard = event => {
    if (event.crystalGameControl || !movement[event.code]) return;
    const id = `keyboard:${event.code}`;
    if (event.type === 'keyup' && heldKeyboard.delete(event.code)) {
      event.preventDefault(); event.stopImmediatePropagation(); state.set(id, null); return;
    }
    if (pad.disabled || keyboardBlocked() || event.ctrlKey || event.metaKey || event.altKey) return;
    event.preventDefault(); event.stopImmediatePropagation();
    if (event.type === 'keydown' && !event.repeat && !heldKeyboard.has(event.code)) {
      if (event.isTrusted) document.body?.classList.add('controller-active');
      onInput(); heldKeyboard.add(event.code); state.set(id, movement[event.code]);
    }
  };
  window.addEventListener('keydown', keyboard, { capture: true });
  window.addEventListener('keyup', keyboard, { capture: true });
  document.addEventListener('focusin', () => { if (keyboardBlocked()) releaseAll(); });
  const bind = (element, resolve) => {
    element.addEventListener('pointerdown', event => {
      if (event.button !== 0 || pad.disabled || blocked()) return;
      event.preventDefault();
      onInput();
      canvas.focus({ preventScroll: true });
      element.setPointerCapture(event.pointerId);
      window.clearTimeout(pendingReleases.get(event.pointerId));
      pendingReleases.delete(event.pointerId);
      state.set(event.pointerId, null);
      pressedAt.set(event.pointerId, window.performance.now());
      heldPointers.add(event.pointerId);
      state.set(event.pointerId, resolve(event));
    });
    element.addEventListener('pointermove', event => {
      if (heldPointers.has(event.pointerId)) state.set(event.pointerId, resolve(event));
    });
    for (const type of ['pointerup', 'pointercancel', 'lostpointercapture']) {
      element.addEventListener(type, event => {
        const wasHeld = heldPointers.delete(event.pointerId);
        if (type === 'lostpointercapture' && !wasHeld) return;
        const remaining = 50 - (window.performance.now() - (pressedAt.get(event.pointerId) ?? 0));
        pressedAt.delete(event.pointerId);
        // Keep a quick tap visible across a game frame. Cancellation is immediate.
        if (type === 'pointerup' && wasHeld && remaining > 0) {
          pendingReleases.set(event.pointerId, window.setTimeout(() => {
            pendingReleases.delete(event.pointerId);
            state.set(event.pointerId, null);
          }, remaining));
        } else {
          window.clearTimeout(pendingReleases.get(event.pointerId));
          pendingReleases.delete(event.pointerId);
          state.set(event.pointerId, null);
        }
      });
    }
  };
  bind(dpad, event => directionAt(dpad.getBoundingClientRect(), event.clientX, event.clientY));
  for (const button of pad.querySelectorAll('[data-game-button]')) {
    if (!dpad.contains(button)) bind(button, () => button.dataset.gameButton);
    // Keyboard / assistive activation emits a short press; pointer input uses holds.
    button.addEventListener('click', event => {
      if (event.detail !== 0 || pad.disabled || blocked()) return;
      onInput();
      canvas.focus({ preventScroll: true });
      const id = Symbol('activation');
      state.set(id, button.dataset.gameButton);
      window.setTimeout(() => state.set(id, null), 80);
    });
  }
  pad.addEventListener('contextmenu', event => event.preventDefault());
  window.addEventListener('blur', releaseAll);
  window.addEventListener('pagehide', releaseAll);
  window.addEventListener('resize', releaseAll);
  document.addEventListener('visibilitychange', () => { if (document.hidden) releaseAll(); });
  document.addEventListener('focusin', () => { if (blocked()) releaseAll(); });
  document.querySelector('#help').addEventListener('click', releaseAll);
  return { releaseAll, setButton: (id, button) => state.set(id, button) };
}
