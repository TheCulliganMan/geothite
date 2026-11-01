import assert from 'node:assert/strict';
import test from 'node:test';
import { createCpuWorkerPool } from './cpu-worker-pool.js';

const turn = () => new Promise(resolve => setImmediate(resolve));
function fixture(eager = true) {
  const clients = [];
  const metrics = {};
  const pool = createCpuWorkerPool(index => {
    const client = { jobs: [], cancelled: 0, disposed: false,
      initialize: () => Promise.resolve(),
      installScenery: (_, json) => { client.source = json; return Promise.resolve(); },
      submit: id => client.jobs.push(id),
      cancel: () => client.cancelled++,
      dispose: () => { client.disposed = true; } };
    clients[index] = client;
    return client;
  }, metrics, { count: 2, eager });
  pool.sources('shared source');
  return { pool, clients, metrics };
}

test('two ready workers accept independent jobs without canceling either transport', async () => {
  const { pool, clients, metrics } = fixture();
  await turn();
  assert.equal(pool.capacity(), 2);
  assert.equal(metrics.initializations, 2);
  assert.equal(metrics.modelsInstalled, 2);
  assert.equal(metrics.retainedSourceCharacters, 0);
  assert.deepEqual(clients.map(client => client.source), ['shared source', 'shared source']);
  pool.submit(1, new Uint8Array());
  pool.submit(2, new Uint8Array());
  assert.deepEqual(clients.map(client => client.jobs), [[1], [2]]);
  assert.deepEqual(clients.map(client => client.cancelled), [0, 0]);
  assert.throws(() => pool.submit(3), /busy/);
});

test('canceling a ticket releases only its worker and completion reuses a free slot', async () => {
  const { pool, clients } = fixture();
  await turn();
  pool.submit(1); pool.submit(2);
  pool.cancel(1);
  assert.deepEqual(clients.map(client => client.cancelled), [1, 0]);
  pool.submit(3);
  assert.deepEqual(clients.map(client => client.jobs), [[1, 3], [2]]);
  pool.complete(2);
  pool.submit(4);
  assert.deepEqual(clients.map(client => client.jobs), [[1, 3], [2, 4]]);
  pool.cancel(99);
  assert.deepEqual(clients.map(client => client.cancelled), [1, 0]);
});

test('one failed worker leaves the other worker available', async () => {
  const { pool, clients } = fixture();
  await turn();
  pool.submit(1); pool.submit(2);
  pool.fail(0);
  assert.equal(pool.capacity(), 1);
  assert.equal(pool.ready(), 2);
  assert.throws(() => pool.submit(3), /busy/);
  pool.complete(2); pool.submit(3);
  assert.deepEqual(clients[1].jobs, [2, 3]);
});

test('a dormant 2D pool starts no workers and disposes without reactivation', async () => {
  const { pool, clients, metrics } = fixture(false);
  assert.equal(clients.length, 0);
  assert.equal(pool.ready(), 1);
  assert.equal(metrics.status, 'dormant');
  pool.ready(true);
  await turn();
  assert.equal(pool.capacity(), 2);
  pool.dispose();
  assert.equal(pool.ready(true), 0);
  assert.equal(metrics.status, 'disposed');
  assert.ok(clients.every(client => client.disposed));
});
