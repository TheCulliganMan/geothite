import test from 'node:test';
import assert from 'node:assert/strict';
import { mountAudioUnlock, mountGameAudio } from './audio-unlock.js';

test('prepares the device during loading but resumes only from a trusted gesture', () => {
  const calls = [];
  const listeners = new Map();
  const audioSession = { type: 'auto' };
  mountAudioUnlock({
    crystal_prepare_audio_context() { assert.equal(audioSession.type, 'playback'); calls.push('prepare'); },
    crystal_resume_audio() { calls.push('resume'); return Promise.resolve(); },
  }, {
    defaultView: { navigator: { audioSession } },
    addEventListener(type, handler) { listeners.set(type, handler); },
  });
  assert.deepEqual(calls, ['prepare']);
  listeners.get('keydown')({ isTrusted: false });
  assert.deepEqual(calls, ['prepare']);
  listeners.get('keydown')({ isTrusted: true });
  assert.deepEqual(calls, ['prepare', 'resume']);
});

test('a failed device preparation still permits gesture-driven recovery', () => {
  let resumes = 0;
  const resume = mountAudioUnlock({
    crystal_prepare_audio_context() { throw Error('device not ready'); },
    crystal_resume_audio() { resumes++; return Promise.resolve(); },
  }, { addEventListener() {} });
  resume({ isTrusted: true });
  assert.equal(resumes, 1);
});

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
  assert.match(page, /import \{ mountGameAudio \} from '.\/audio-unlock.js\?v=20261018-playback'/);
  assert.match(page, /mountGameAudio\(wasm, document/);
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

function gameAudio({ query = '', embedded = false, values = new Map(), blocked = false } = {}) {
  const listeners = new Map();
  const attributes = new Map();
  const mutes = [];
  let resumes = 0, focuses = 0, click;
  const button = {
    dataset: {},
    setAttribute: (key, value) => attributes.set(key, value),
    getAttribute: key => attributes.get(key),
    contains: target => target === button,
    addEventListener(type, handler) { assert.equal(type, 'click'); click = handler; },
  };
  const resume = mountGameAudio({
    crystal_set_muted: muted => mutes.push(muted),
    crystal_resume_audio() { resumes++; return blocked ? Promise.reject(Error('blocked')) : Promise.resolve(); },
  }, { addEventListener: (type, handler) => listeners.set(type, handler) }, {
    button, canvas: { focus() { focuses++; } },
    storage: { getItem: key => values.get(key) ?? null, setItem: (key, value) => values.set(key, value) },
    params: new URLSearchParams(query), embedded,
  });
  return { button, attributes, listeners, mutes, values, resume,
    click: () => click?.({ isTrusted: true, target: button }),
    resumes: () => resumes, focuses: () => focuses };
}

test('normal play defaults to sound and unlocks on the first game gesture despite the old forced mute', async () => {
  const values = new Map([['crystal.audio.muted', 'true']]);
  const ui = gameAudio({ values });
  assert.deepEqual(ui.mutes, [false]);
  assert.equal(ui.resumes(), 0);
  assert.equal(ui.button.dataset.audible, 'false');
  assert.equal(values.has('crystal.audio.preference'), false);
  ui.listeners.get('keydown')({ isTrusted: true });
  await Promise.resolve();
  assert.equal(ui.resumes(), 1);
  assert.equal(ui.button.dataset.audible, 'true');
  assert.equal(ui.attributes.get('aria-label'), 'Mute sound');
});

test('normal play remembers deliberate mute and unmute choices across reloads', async () => {
  const values = new Map();
  const ui = gameAudio({ values });
  ui.listeners.get('pointerup')({ isTrusted: true });
  await Promise.resolve();
  ui.click();
  await Promise.resolve();
  assert.equal(values.get('crystal.audio.preference'), 'muted');
  assert.equal(ui.button.dataset.audible, 'false');
  const reloaded = gameAudio({ values });
  assert.deepEqual(reloaded.mutes, [true]);
  reloaded.click();
  await Promise.resolve();
  assert.equal(values.get('crystal.audio.preference'), 'audible');
  assert.equal(reloaded.button.dataset.audible, 'true');
  assert.equal(reloaded.focuses(), 1);
});

test('development previews keep silent audio timing after gestures and cannot be unmuted', async () => {
  for (const options of [{ query: '?preview=bedroom' }, { query: '?preview=unknown' },
    { query: '?development=1' }, { embedded: true }]) {
    const values = new Map([['crystal.audio.preference', 'audible']]);
    const ui = gameAudio({ ...options, values });
    assert.deepEqual(ui.mutes, [true]);
    assert.equal(ui.button.disabled, true);
    assert.equal(ui.listeners.size, 4);
    ui.resume({ isTrusted: true });
    await Promise.resolve();
    ui.click();
    assert.equal(ui.resumes(), 1);
    assert.deepEqual(ui.mutes, [true, true]);
    assert.equal(ui.button.dataset.audible, 'false');
    assert.equal(values.get('crystal.audio.preference'), 'audible');
  }
});

test('blocked browser autoplay keeps the sound control available for retry', async () => {
  const ui = gameAudio({ blocked: true });
  ui.listeners.get('keydown')({ isTrusted: true });
  await Promise.resolve();
  assert.equal(ui.button.dataset.audible, 'false');
  assert.equal(ui.attributes.get('aria-label'), 'Enable sound');
  assert.deepEqual(ui.mutes, [false, false]);
});
