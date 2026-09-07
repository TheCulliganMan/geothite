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
  const timer = window.setInterval(poll, 100);
  window.addEventListener('pagehide', () => { window.clearInterval(timer); close(); }, { once: true });
  poll();
  return { poll, close };
}
