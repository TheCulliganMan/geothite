import test from 'node:test';
import assert from 'node:assert/strict';
import { mountAudioUnlock } from './audio-unlock.js';

test('resumes synchronously in trusted gestures, including repeated iPhone touchend', () => {
  const listeners = new Map();
  let calls = 0;
  mountAudioUnlock({ crystal_resume_audio() { calls++; return Promise.resolve(); } }, {
    addEventListener(type, handler, options) {
      assert.equal(options.capture, true);
      listeners.set(type, handler);
    },
  });
  for (const type of ['touchend', 'pointerup', 'click', 'keydown']) {
    listeners.get(type)({ isTrusted: false });
    assert.equal(calls, 0);
  }
  for (const type of ['touchend', 'touchend', 'pointerup', 'click', 'keydown']) {
    const before = calls;
    listeners.get(type)({ isTrusted: true });
    assert.equal(calls, before + 1);
  }
});

test('a rejected resume is handled and the next gesture retries', async () => {
  const listeners = new Map();
  let calls = 0;
  mountAudioUnlock({ crystal_resume_audio() { calls++; return Promise.reject(new Error('blocked')); } }, {
    addEventListener(type, handler) { listeners.set(type, handler); },
  });
  listeners.get('touchend')({ isTrusted: true });
  await new Promise(resolve => setImmediate(resolve));
  listeners.get('touchend')({ isTrusted: true });
  await new Promise(resolve => setImmediate(resolve));
  assert.equal(calls, 2);
});

test('the served page and Docker build include the gesture handler', async () => {
  const { readFile } = await import('node:fs/promises');
  const page = await readFile(new URL('./index.html', import.meta.url), 'utf8');
  assert.match(page, /import \{ mountAudioUnlock \} from '.\/audio-unlock.js'/);
  assert.match(page, /mountAudioUnlock\(wasm, document\)/);
  for (const path of ['../Dockerfile']) {
    const build = await readFile(new URL(path, import.meta.url), 'utf8');
    assert.match(build, /web-client\/audio-unlock.js/);
  }
});
