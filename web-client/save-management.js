const MAX_SAVE = 4 * 1024 * 1024;
const MAX_LINK = 64000;
const PREFIX = '#geothite-save=1.';

async function transform(bytes, stream) {
  const reader = new Blob([bytes]).stream().pipeThrough(stream).getReader();
  const parts = []; let length = 0;
  try {
    for (;;) {
      const { value, done } = await reader.read();
      if (done) break;
      length += value.length;
      if (length > MAX_SAVE) throw new Error('Save exceeds the 4 MB limit.');
      parts.push(value);
    }
  } finally { await reader.cancel(); }
  const result = new Uint8Array(length); let offset = 0;
  for (const part of parts) { result.set(part, offset); offset += part.length; }
  return result;
}
export async function createSaveLink(bytes, href) {
  if (!bytes.length || bytes.length > MAX_SAVE) throw new Error('Invalid save size.');
  const compressed = await transform(bytes, new CompressionStream('gzip'));
  let binary = '';
  for (const byte of compressed) binary += String.fromCharCode(byte);
  const source = new URL(href);
  const url = new URL(source.pathname, source.origin);
  if (source.searchParams.get('multiplayer') === 'off') url.searchParams.set('multiplayer', 'off');
  url.hash = PREFIX + btoa(binary).replaceAll('+', '-').replaceAll('/', '_').replace(/=+$/, '');
  if (url.href.length > MAX_LINK) throw new Error('This save is too large for a bookmark. Download and share the save file instead.');
  return url.href;
}
export async function readSaveLink(hash) {
  if (!hash.startsWith(PREFIX) || hash.length > MAX_LINK) throw new Error('Invalid save link.');
  const encoded = hash.slice(PREFIX.length);
  if (!/^[A-Za-z0-9_-]+$/.test(encoded)) throw new Error('Invalid save link.');
  const bytes = Uint8Array.from(atob(encoded.replaceAll('-', '+').replaceAll('_', '/')), c => c.charCodeAt(0));
  return transform(bytes, new DecompressionStream('gzip'));
}

export function mountSaveManagement(wasm, { document, window }) {
  const dialog = document.querySelector('#save-games-dialog');
  const status = dialog.querySelector('[role="status"]');
  const summary = dialog.querySelector('[data-summary]');
  const confirm = dialog.querySelector('[data-confirmation]');
  const linkBox = dialog.querySelector('[data-link-box]');
  const link = dialog.querySelector('[data-link]');
  const file = dialog.querySelector('input[type=file]');
  const buttons = [...dialog.querySelectorAll('button')];
  let busy = false, pending = null, previousFocus, current = {};
  const message = error => String(error?.message ?? error);
  const render = () => {
    for (const button of buttons) button.disabled = busy;
    dialog.querySelector('[data-action=save]').disabled = busy || !current.can_save;
    for (const action of ['download','share']) dialog.querySelector(`[data-action=${action}]`).disabled = busy || !current.save;
    dialog.querySelector('[data-action=delete]').disabled = busy || !current.can_replace;
    file.disabled = busy;
  };
  const request = async (action, bytes = new Uint8Array()) => {
    wasm.crystal_save_manager_request(action, bytes);
    const deadline = Date.now() + 30000;
    for (;;) {
      const state = JSON.parse(wasm.crystal_save_manager_poll());
      if (state.result) {
        if (state.result.error) throw new Error(state.result.error);
        return state.result;
      }
      if (Date.now() > deadline) throw new Error('The game did not respond. Close and reopen Save games.');
      await new Promise(resolve => window.setTimeout(resolve, 50));
    }
  };
  const run = async action => {
    if (busy) return;
    busy = true; render(); status.textContent = 'Working…';
    try { await action(); } catch (error) { status.textContent = message(error); }
    finally { busy = false; render(); }
  };
  const refresh = async () => {
    current = await request('status');
    summary.textContent = current.save ? `${current.save.trainer} · Trainer ${current.save.trainer_id}` : 'No valid saved game for this player yet.';
    if (!current.can_save && !current.save) summary.textContent = 'Start your adventure to create a save.';
  };
  const open = () => {
    previousFocus = document.activeElement;
    document.querySelector('#player-options').open = false;
    wasm.crystal_save_manager_open(); dialog.showModal();
    return run(async () => { await refresh(); status.textContent = ''; });
  };
  const close = () => {
    if (busy) return;
    pending = null; confirm.hidden = true; linkBox.hidden = true; linkBox.open = false; file.value = '';
    wasm.crystal_save_manager_close(); dialog.close();
    (previousFocus?.isConnected ? previousFocus : document.querySelector('canvas'))?.focus();
  };
  const inspect = async bytes => {
    pending = null; confirm.hidden = true;
    if (!bytes.length || bytes.length > MAX_SAVE) throw new Error('Choose a save file smaller than 4 MB.');
    const preview = await request('inspect', bytes);
    pending = { action: 'import', bytes };
    confirm.querySelector('p').textContent = `Restore ${preview.save.trainer} (ID ${preview.save.trainer_id})? This replaces your current saved progress and restarts the game. Download a backup first if you want to keep it.`;
    confirm.hidden = false; confirm.scrollIntoView?.({ block: 'nearest' }); status.textContent = 'Ready to restore.';
  };
  document.querySelector('#save-games').disabled = false;
  document.querySelector('#save-games').addEventListener('click', open);
  dialog.querySelector('[data-close]').addEventListener('click', close);
  dialog.addEventListener('cancel', event => { event.preventDefault(); close(); });
  for (const type of ['keydown','keyup']) dialog.addEventListener(type, event => event.stopPropagation());
  dialog.querySelector('[data-action=save]').addEventListener('click', () => run(async () => {
    await request('save'); await refresh(); status.textContent = 'Progress saved in this browser.';
  }));
  dialog.querySelector('[data-action=download]').addEventListener('click', () => run(async () => {
    await request('export');
    const url = window.URL.createObjectURL(new Blob([wasm.crystal_save_manager_take_bytes()], { type: 'application/octet-stream' }));
    const anchor = document.createElement('a'); anchor.href = url; anchor.download = 'geothite.crystalsave';
    document.body.append(anchor); anchor.click(); anchor.remove(); window.setTimeout(() => window.URL.revokeObjectURL(url), 30000);
    status.textContent = 'Backup downloaded. It contains your last saved progress.';
  }));
  const copyLink = async () => {
    try { await window.navigator.clipboard.writeText(link.value); status.textContent = 'Link copied. Ready to share or bookmark.'; }
    catch { linkBox.open = true; link.focus(); link.select(); status.textContent = 'Copy the selected link to share it or save it as a bookmark.'; }
  };
  dialog.querySelector('[data-action=share]').addEventListener('click', () => run(async () => {
    await request('export'); link.value = await createSaveLink(wasm.crystal_save_manager_take_bytes(), window.location.href);
    linkBox.hidden = false; await copyLink();
  }));
  dialog.querySelector('[data-copy]').addEventListener('click', copyLink);
  file.addEventListener('change', () => run(async () => {
    const selected = file.files?.[0]; if (!selected) { status.textContent = ''; return; }
    if (selected.size > MAX_SAVE) throw new Error('Save files must be smaller than 4 MB.');
    await inspect(new Uint8Array(await selected.arrayBuffer())); file.value = '';
  }));
  dialog.querySelector('[data-action=delete]').addEventListener('click', () => {
    pending = { action: 'delete' }; confirm.hidden = false;
    confirm.querySelector('p').textContent = 'Delete this player’s saved game and its recovery copy? The game will restart. Download a backup first if you want to keep your progress.';
    confirm.scrollIntoView?.({ block: 'nearest' }); status.textContent = '';
  });
  dialog.querySelector('[data-cancel]').addEventListener('click', () => { pending = null; confirm.hidden = true; status.textContent = 'Cancelled. Your save is unchanged.'; });
  dialog.querySelector('[data-confirm]').addEventListener('click', () => run(async () => {
    if (!pending) return;
    const result = await request(pending.action, pending.bytes);
    if (result.reload) { status.textContent = 'Restarting…'; window.location.reload(); }
  }));
  window.addEventListener('pagehide', () => { wasm.crystal_save_manager_close(); if (dialog.open) dialog.close(); });
  const incoming = window.location.hash;
  if (incoming.startsWith('#geothite-save=')) {
    window.history.replaceState(null, '', window.location.pathname + window.location.search);
    open().then(() => run(async () => inspect(await readSaveLink(incoming))));
  }
  return { open, close };
}
