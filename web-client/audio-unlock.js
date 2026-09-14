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
