export const DEFAULT_KEY_BINDINGS = Object.freeze({ chat: 'Enter', start: 'Space', select: 'Backspace' });
const KEY_BINDINGS_STORAGE = 'geothite.key-bindings.v1';
export const BINDABLE_KEYS = ['Enter', 'Space', 'Backspace', 'ShiftRight', ...'BCEFGHIJKLMNOPQRTUVY'.split('').map(key => `Key${key}`), ...'0123456789'.split('').map(key => `Digit${key}`)];
export const keyLabel = code => code === 'ShiftRight' ? 'Right Shift' : code === 'Space' ? 'Space' : code.replace(/^(Key|Digit)/, '');
export function validateKeyBindings(bindings) {
  if (!bindings || !['chat', 'start', 'select'].every(action => BINDABLE_KEYS.includes(bindings[action]))) return 'Choose a key for Chat, Start, and Select.';
  if (new Set([bindings.chat, bindings.start, bindings.select]).size !== 3) return 'Chat, Start, and Select must use different keys.';
  return null;
}
export function loadKeyBindings(window) {
  try {
    const saved = JSON.parse(window.localStorage.getItem(KEY_BINDINGS_STORAGE));
    if (saved && saved.select === undefined) {
      // Preserve existing two-action preferences when adding Select.
      saved.select = ['Backspace', 'ShiftRight', ...BINDABLE_KEYS].find(code => code !== saved.chat && code !== saved.start);
    }
    if (saved) for (const action of ['chat', 'start', 'select']) {
      if (['KeyW', 'KeyA', 'KeyS', 'KeyD'].includes(saved[action])) {
        saved[action] = [DEFAULT_KEY_BINDINGS[action], ...BINDABLE_KEYS].find(code =>
          !Object.entries(saved).some(([other, value]) => other !== action && value === code));
      }
    }
    if (!validateKeyBindings(saved)) return saved;
  } catch {}
  return { ...DEFAULT_KEY_BINDINGS };
}
export function saveKeyBindings(window, bindings) {
  const error = validateKeyBindings(bindings);
  if (error) throw new Error(error);
  window.localStorage.setItem(KEY_BINDINGS_STORAGE, JSON.stringify({ chat: bindings.chat, start: bindings.start, select: bindings.select }));
  window.dispatchEvent(new window.Event('geothite-key-bindings-changed'));
}

export function validateProfile(profile) {
  if (!/^[A-Z0-9 .'-]{1,8}$/.test(profile.name) || profile.name.trim() !== profile.name)
    return 'Use 1–8 uppercase letters, numbers, spaces, periods, apostrophes or hyphens for your trainer name.';
  if (!/^[A-Za-z0-9_]{1,24}$/.test(profile.handle))
    return 'Use 1–24 letters, numbers or underscores for your handle.';
  if (![0, 1].includes(profile.sprite)) return 'Choose Chris or Kris.';
  return null;
}

export function mountPlayerCustomization(wasm, { document, window }) {
  const button = document.querySelector('#personalization');
  const dialog = document.querySelector('#personalization-dialog');
  const form = dialog.querySelector('form');
  const name = form.elements.namedItem('name');
  const handle = form.elements.namedItem('handle');
  const sprite = form.elements.namedItem('sprite');
  const save = dialog.querySelector('[type="submit"]');
  const status = dialog.querySelector('[role="status"]');
  const chatKey = form.elements.namedItem('chat-key');
  const startKey = form.elements.namedItem('start-key');
  const selectKey = form.elements.namedItem('select-key');
  const bindingStatus = dialog.querySelector('#key-bindings-status');
  for (const select of [chatKey, startKey, selectKey]) {
    for (const code of BINDABLE_KEYS) {
      const option = document.createElement('option');
      option.value = code; option.textContent = keyLabel(code); select.append(option);
    }
  }
  const fillBindings = bindings => { chatKey.value = bindings.chat; startKey.value = bindings.start; selectKey.value = bindings.select; };
  dialog.querySelector('[data-save-bindings]').addEventListener('click', () => {
    try {
      saveKeyBindings(window, { chat: chatKey.value, start: startKey.value, select: selectKey.value });
      bindingStatus.textContent = 'Key bindings saved on this browser.';
    } catch (error) { bindingStatus.textContent = error.message; }
  });
  dialog.querySelector('[data-reset-bindings]').addEventListener('click', () => {
    fillBindings(DEFAULT_KEY_BINDINGS);
    bindingStatus.textContent = 'Defaults selected. Save key bindings to apply.';
  });
  let waiting = false;
  let previousFocus;
  const close = () => { wasm.crystal_customization_close(); dialog.close(); };
  button.addEventListener('click', () => { wasm.crystal_customization_open(); poll(); });
  dialog.querySelector('[data-close]').addEventListener('click', close);
  dialog.addEventListener('cancel', event => { event.preventDefault(); close(); });
  dialog.addEventListener('close', () => {
    wasm.crystal_customization_close();
    waiting = false;
    (previousFocus?.isConnected ? previousFocus : document.querySelector('#crystal-canvas'))?.focus({ preventScroll: true });
  });
  // Keep text entry and Escape inside the modal, including game hotkeys.
  for (const type of ['keydown', 'keyup']) dialog.addEventListener(type, event => event.stopPropagation());
  name.addEventListener('input', () => { const pos = name.selectionStart; name.value = name.value.toUpperCase(); name.setSelectionRange(pos, pos); });
  form.addEventListener('submit', event => {
    event.preventDefault();
    const profile = { name: name.value, handle: handle.value, sprite: Number(sprite.value) };
    const error = validateProfile(profile);
    if (error) { status.textContent = error; return; }
    try {
      wasm.crystal_customization_save(JSON.stringify(profile));
      waiting = true;
      save.disabled = true;
      status.textContent = 'Saving…';
    } catch (error) { status.textContent = String(error?.message ?? error); }
  });
  const poll = () => {
    const state = JSON.parse(wasm.crystal_customization_poll());
    button.hidden = !state.enabled;
    button.disabled = !state.enabled;
    button.title = state.can_edit ? 'Personalize your trainer' : 'Available after returning to the overworld';
    if (state.open && !dialog.open) {
      previousFocus = document.activeElement;
      fillBindings(loadKeyBindings(window));
      bindingStatus.textContent = '';
      name.value = state.profile.name;
      handle.value = state.profile.handle;
      sprite.value = String(state.profile.sprite);
      status.textContent = '';
      save.disabled = false;
      document.querySelector('#player-options').open = false;
      dialog.showModal();
      name.focus();
    }
    if (state.error) {
      if (dialog.open) status.textContent = state.error;
      else document.querySelector('#personalization-notice').textContent = state.error;
      waiting = false; save.disabled = false;
    } else if (waiting && state.saved) {
      status.textContent = 'Saved. Your trainer and online profile are up to date.';
      waiting = false; save.disabled = false;
    }
    if (state.open) document.querySelector('#personalization-notice').textContent = '';
  };
  let timer = window.setInterval(poll, 100);
  window.addEventListener('pagehide', () => {
    window.clearInterval(timer); timer = null;
    if (dialog.open) close();
  });
  window.addEventListener('pageshow', () => {
    if (timer === null) { timer = window.setInterval(poll, 100); poll(); }
  });
  poll();
  return { poll, close };
}
