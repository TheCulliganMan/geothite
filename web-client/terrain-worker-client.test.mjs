import assert from 'node:assert/strict';
import test from 'node:test';
import { createTerrainWorkerClient } from './terrain-worker-client.js';

function fixture(run, options = {}) {
  const previous = { Worker: globalThis.Worker, MessageChannel: globalThis.MessageChannel };
  let worker;
  class Worker {
    messages = [];
    constructor() { worker = this; }
    postMessage(message, transfer = []) {
      this.messages.push(structuredClone(message, { transfer }));
    }
    terminate() {}
    reply(data) { this.onmessage({ data }); }
  }
  class MessageChannel {
    port1 = { close() {} };
    port2 = { postMessage() {}, close() {} };
  }
  Object.assign(globalThis, { Worker, MessageChannel });
  const cancelled = [];
  const client = createTerrainWorkerClient({ onCancel: id => cancelled.push(id) },
    { cancelAcknowledgements: true, ...options });
  client.initialize('boot');
  worker.reply({ kind: 'ready', id: 'boot', cancelAcknowledgements: options.capability ?? true });
  try { run({ client, worker, cancelled }); }
  finally { client.dispose(); Object.assign(globalThis, previous); }
}

const submitted = worker => worker.messages.filter(message => message.kind === 'terrain-pages');

test('repeated cancellation dispatches only the latest replacement after the CPU worker releases its job', () => {
  fixture(({ client, worker, cancelled }) => {
    client.submit(1, new Uint8Array([1]));
    client.cancel();
    client.submit(2, new Uint8Array([2]));
    client.cancel();
    client.submit(3, new Uint8Array([3]));
    assert.deepEqual(submitted(worker).map(message => message.id), [1]);
    assert.deepEqual(worker.messages.filter(message => message.kind === 'cancel').map(message => message.id), [1]);
    assert.deepEqual(cancelled, [1, 2]);
    worker.reply({ kind: 'header', id: 1, header: new ArrayBuffer(0) });
    assert.deepEqual(submitted(worker).map(message => message.id), [1]);
    worker.reply({ kind: 'cancelled', id: 1 });
    assert.deepEqual(submitted(worker).map(message => message.id), [1, 3]);
    assert.deepEqual([...new Uint8Array(submitted(worker)[1].request)], [3]);
  });
});

test('deferred requests own their bytes and stale acknowledgements cannot release a newer job', () => {
  fixture(({ client, worker }) => {
    client.submit(1, new Uint8Array([1]));
    client.cancel();
    const borrowed = new Uint8Array([5, 6, 7, 8]);
    client.submit(2, borrowed.subarray(1, 3));
    borrowed.fill(0);
    worker.reply({ kind: 'cancelled', id: 99 });
    assert.equal(submitted(worker).length, 1);
    worker.reply({ kind: 'cancelled', id: 1 });
    assert.deepEqual([...new Uint8Array(submitted(worker)[1].request)], [6, 7]);
    client.cancel();
    client.submit(3, new Uint8Array([3]));
    worker.reply({ kind: 'cancelled', id: 1 });
    assert.equal(submitted(worker).length, 2);
    worker.reply({ kind: 'cancelled', id: 2 });
    assert.deepEqual(submitted(worker).map(message => message.id), [1, 2, 3]);
  });
});

test('transports without the cancellation protocol continue dispatching immediately', () => {
  fixture(({ client, worker }) => {
    client.submit(1, new Uint8Array([1]));
    client.cancel();
    client.submit(2, new Uint8Array([2]));
    assert.deepEqual(submitted(worker).map(message => message.id), [1, 2]);
  }, { cancelAcknowledgements: false });
});


test('an older worker without cancellation acknowledgement support uses the compatible transport', () => {
  fixture(({ client, worker }) => {
    client.submit(1, new Uint8Array([1]));
    client.cancel();
    client.submit(2, new Uint8Array([2]));
    assert.deepEqual(submitted(worker).map(message => message.id), [1, 2]);
  }, { capability: false });
});
