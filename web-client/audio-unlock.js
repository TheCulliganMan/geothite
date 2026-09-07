// Safari requires resume() in the DOM gesture itself, not the next game frame.
// Keep listening so touches also recover audio after a tab/phone interruption.
export function mountAudioUnlock(wasm, document) {
  const resume = event => {
    if (!event.isTrusted) return;
    try {
      wasm.crystal_resume_audio().catch(() => {});
    } catch {
      // A later trusted gesture can retry creation/resume.
    }
  };
  for (const type of ['touchend', 'pointerup', 'click', 'keydown']) {
    document.addEventListener(type, resume, { capture: true, passive: true });
  }
}
