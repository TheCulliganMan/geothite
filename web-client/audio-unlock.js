// Safari requires resume() in the DOM gesture itself, not the next game frame.
// Keep listening so touches also recover audio after a tab/phone interruption.
export function mountAudioUnlock(wasm, document, { onState = () => {}, control } = {}) {
  const resume = event => {
    if (!event.isTrusted) return;
    // The default iOS session follows the ringer switch even with a running
    // AudioContext. Select media playback before Web Audio is activated.
    const session = document.defaultView?.navigator.audioSession;
    if (session) {
      try { session.type = 'playback'; } catch { /* Resume still works when the browser disallows session changes. */ }
    }
    try {
      wasm.crystal_resume_audio().then(() => onState('ready'), () => onState('blocked'));
    } catch {
      // A later trusted gesture can retry creation/resume.
      onState('blocked');
    }
  };
  for (const type of ['touchend', 'pointerup', 'click', 'keydown']) {
    document.addEventListener(type, event => {
      if (!control?.contains(event.target)) resume(event);
    }, { capture: true, passive: true });
  }
  return resume;
}

// The old page wrote a forced mute on every visit. Store only deliberate user
// choices under a new key, so that obsolete default cannot silence normal play.
export function mountGameAudio(wasm, document, { button, canvas, storage, params, embedded = false }) {
  const development = embedded || params.has('preview') || params.get('development') === '1';
  let muted = development || storage.getItem('crystal.audio.preference') === 'muted';
  let audioState = 'idle';
  const update = () => {
    wasm.crystal_set_muted(muted);
    button.dataset.audible = String(!muted && audioState === 'ready');
    button.setAttribute('aria-pressed', String(muted));
    button.setAttribute('aria-label', development ? 'Sound muted in development'
      : muted || audioState !== 'ready' ? 'Enable sound' : 'Mute sound');
    button.title = button.getAttribute('aria-label');
  };
  button.disabled = development;
  update();
  const resume = mountAudioUnlock(wasm, document, {
    control: button,
    onState: state => { audioState = state; update(); },
  });
  // Muted playback still advances source WaitSFX timing in development.
  // Keep gesture-driven activation, with master gain fixed at zero.
  if (development) return resume;
  button.addEventListener('click', event => {
    muted = muted || audioState !== 'ready' ? false : true;
    update();
    storage.setItem('crystal.audio.preference', muted ? 'muted' : 'audible');
    resume(event);
    canvas.focus({ preventScroll: true });
  });
  return resume;
}
