// Slot selection and page ownership only; checkpoints/gameplay remain Rust.
// A session name is NOT a credential and does not put a save on the server.
export function tuiSession(url) {
  url = new URL(url);
  const id = url.searchParams.get('session') ?? '';
  if (id.length > 64 || !/^[a-zA-Z0-9_-]*$/.test(id)) {
    throw new Error('Session must contain at most 64 ASCII letters, digits, hyphens or underscores.');
  }
  return Object.freeze({ id: id || 'default', slot: id, url: url.href, storage: 'this-browser' });
}

export async function acquireTuiSession(locks, slot) {
  if (!locks?.request) throw new Error('Safe session ownership needs HTTPS and a browser with Web Locks.');
  return new Promise((resolve, reject) => {
    locks.request(`geothite.tui.session.${slot ? `named.${slot}` : 'local'}`, { ifAvailable: true }, async lock => {
      if (!lock) throw new Error('This session is already playing in another tab. Close it before resuming, or use a different ?session= name.');
      await new Promise(release => { resolve(release); });
    }).catch(reject);
  });
}
