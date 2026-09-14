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
  assert.match(page, /import \{ mountAudioUnlock \} from '.\/audio-unlock.js\?v=20260907-mobile'/);
  assert.match(page, /mountAudioUnlock\(wasm, document/);
  for (const path of ['../Dockerfile']) {
    const build = await readFile(new URL(path, import.meta.url), 'utf8');
    assert.match(build, /web-client\/audio-unlock.js/);
  }
});

test('selects the iPhone playback session before creating or resuming Web Audio', () => {
  const listeners = new Map();
  const audioSession = { type: 'auto' };
  let calls = 0;
  mountAudioUnlock({ crystal_resume_audio() {
    assert.equal(audioSession.type, 'playback');
    calls++;
    return Promise.resolve();
  } }, {
    defaultView: { navigator: { audioSession } },
    addEventListener(type, handler) { listeners.set(type, handler); },
  });
  listeners.get('touchend')({ isTrusted: true });
  assert.equal(audioSession.type, 'playback');
  assert.equal(calls, 1);
});

test('reports activation success and failure so sound controls never imply silent playback is enabled', async () => {
  const listeners = new Map();
  const states = [];
  let fails = true;
  mountAudioUnlock({ crystal_resume_audio() {
    return fails ? Promise.reject(new Error('blocked')) : Promise.resolve();
  } }, { addEventListener(type, handler) { listeners.set(type, handler); } }, {
    onState: state => states.push(state),
  });
  listeners.get('touchend')({ isTrusted: true });
  await Promise.resolve();
  fails = false;
  listeners.get('touchend')({ isTrusted: true });
  await Promise.resolve();
  assert.deepEqual(states, ['blocked', 'ready']);
});

test('sound-button gestures are handled once by its click action, not first by pointerup', () => {
  const listeners = new Map();
  const target = {};
  let calls = 0;
  const activate = mountAudioUnlock({ crystal_resume_audio() { calls++; return Promise.resolve(); } }, {
    addEventListener(type, handler) { listeners.set(type, handler); },
  }, { control: { contains: node => node === target } });
  const event = { isTrusted: true, target };
  listeners.get('pointerup')(event);
  listeners.get('click')(event);
  assert.equal(calls, 0);
  activate(event);
  assert.equal(calls, 1);
});
