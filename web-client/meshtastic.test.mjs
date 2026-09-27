import assert from 'node:assert/strict';
import test from 'node:test';
import {
  encodeSerialFrame,
  extractSerialFrames,
  openBluetoothMeshtastic,
  openSerialMeshtastic,
  prepareMeshtasticBridge,
} from './meshtastic.js';

test('serial framing handles split, adjacent, and leading-noise frames', () => {
  const first = encodeSerialFrame(Uint8Array.of(1, 2, 3));
  const second = encodeSerialFrame(Uint8Array.of(4));
  const bytes = Uint8Array.from([0, ...first, ...second]);
  const partial = extractSerialFrames(bytes.slice(0, 6));
  assert.equal(partial.frames.length, 0);
  const joined = Uint8Array.from([...partial.remainder, ...bytes.slice(6)]);
  const complete = extractSerialFrames(joined);
  assert.deepEqual(complete.frames.map(frame => Array.from(frame)), [[1, 2, 3], [4]]);
  assert.equal(complete.remainder.byteLength, 0);
});

test('bridge reports that it is inactive outside Meshtastic mode', () => {
  assert.equal(prepareMeshtasticBridge({ global: {}, navigator: {}, document: {}, location: { search: '' } }), null);
});

test('browser adapters clearly report unsupported APIs and permission denial', async () => {
  await assert.rejects(openSerialMeshtastic({}, () => {}, () => {}), /Web Serial is unavailable/);
  await assert.rejects(openBluetoothMeshtastic({}, () => {}, () => {}), /Web Bluetooth is unavailable/);
  await assert.rejects(
    openSerialMeshtastic({ serial: { requestPort: async () => { throw new Error('permission denied'); } } }, () => {}, () => {}),
    /permission denied/,
  );
});
