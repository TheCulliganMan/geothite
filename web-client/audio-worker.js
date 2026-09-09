// Keep synthesis off the UI thread; all audio execution and DSP live in Rust.
import init, { synthesize_crystal_midi, synthesize_modified_cry } from './crystal-audio.js';
const ready = init();
let queue = Promise.resolve();
self.onmessage = ({ data }) => {
  queue = queue.then(async () => {
    try {
      await ready;
      const result = data.cry == null ? synthesize_crystal_midi(data.midi) : synthesize_modified_cry(data.midi, data.cry);
      self.postMessage({ id: data.id, samples: result.samples, sampleRate: result.sampleRate }, [result.samples.buffer]);
    } catch (error) {
      self.postMessage({ id: data.id, error: String(error?.message ?? error) });
    }
  });
};
