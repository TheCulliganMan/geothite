/** Poll preparation without running the canonical synthesizer on the UI thread. */
export function createMidiPreparation(worker) {
  let nextId = 0;
  let failure = null;
  const byMidi = new Map();
  const byId = new Map();
  worker.onmessage = ({ data }) => {
    const entry = byId.get(data.id);
    if (!entry) return;
    byId.delete(data.id);
    if (data.error) entry.error = new Error(data.error);
    else entry.result = { samples: data.samples, sampleRate: data.sampleRate };
  };
  worker.onerror = event => {
    event.preventDefault();
    failure = new Error(event.message || 'Audio worker failed');
  };
  worker.onmessageerror = () => { failure = new Error('Invalid audio worker message'); };
  return (midi, cry = null) => {
    const key = cry == null ? midi : JSON.stringify([midi, cry]);
    if (failure) throw failure;
    let entry = byMidi.get(key);
    if (!entry) {
      entry = { id: ++nextId };
      byMidi.set(key, entry);
      byId.set(entry.id, entry);
      worker.postMessage({ id: entry.id, midi, ...(cry == null ? {} : { cry }) });
    }
    if (entry.error) { byMidi.delete(key); throw entry.error; }
    if (!entry.result) return null;
    byMidi.delete(key); // Rust owns the validated PCM cache after this poll.
    return entry.result;
  };
}
