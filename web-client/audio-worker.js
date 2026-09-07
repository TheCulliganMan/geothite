import { initializeCrystalAudioSynth, synthesizeCrystalMidi } from './audio-runtime/browser-synth.js';
self.onmessage = ({ data }) => {
  if ('context' in data) {
    initializeCrystalAudioSynth(data.context);
    return;
  }
  try {
    const result = synthesizeCrystalMidi(data.midi);
    self.postMessage({ id: data.id, ...result }, [result.samples.buffer]);
  } catch (error) {
    self.postMessage({ id: data.id, error: String(error?.message ?? error) });
  }
};
