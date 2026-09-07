import test from 'node:test';
import assert from 'node:assert/strict';
import { createMidiPreparation } from './audio-worker-client.js';
class WorkerStub {
  sent = [];
  postMessage(message) { this.sent.push(message); }
  reply(data) { this.onmessage({ data }); }
}
test('preparation is nonblocking, deduplicated and returns exact worker PCM', () => {
  const worker = new WorkerStub();
  const poll = createMidiPreparation(worker);
  assert.equal(poll('midi'), null);
  assert.equal(poll('midi'), null);
  assert.equal(worker.sent.length, 1);
  const samples = new Int16Array([-32768, 123, 0, 32767]);
  worker.reply({ id: worker.sent[0].id, samples, sampleRate: 22050 });
  assert.deepEqual(poll('midi'), { samples, sampleRate: 22050 });
});
test('worker failures reach the caller instead of synchronous fallback synthesis', () => {
  const worker = new WorkerStub();
  const poll = createMidiPreparation(worker);
  poll('bad');
  worker.reply({ id: worker.sent[0].id, error: 'invalid MIDI' });
  assert.throws(() => poll('bad'), /invalid MIDI/);
  worker.onerror({ message: 'worker failed', preventDefault() {} });
  assert.throws(() => poll('next'), /worker failed/);
});
