import test from 'node:test';
import assert from 'node:assert/strict';
import { selectRendererBundle } from './renderer-bundle.js';
const legacy = { module: './crystal-bevy.js', binary: './crystal-bevy_bg.wasm' };
const modern = { module: './crystal-bevy-webgpu.js', binary: './crystal-bevy-webgpu_bg.wasm' };
const options = { url: 'http://localhost/', fetch: async () => ({ ok:true, json:async()=>({webgpu:modern}) }) };
test('selects separate modern artifact with available adapter', async () => {
  assert.deepEqual(await selectRendererBundle({...options,navigator:{gpu:{requestAdapter:async()=>({})}}}),modern);
});
test('missing, denied, and null adapters retain compile-time WebGL layout', async () => {
  for (const navigator of [{},{gpu:{requestAdapter:async()=>null}},{gpu:{requestAdapter:async()=>{throw Error('denied');}}}]) {
    assert.deepEqual(await selectRendererBundle({...options,navigator}),legacy);
  }
});
test('forced WebGL and absent optional artifact do not probe an adapter', async () => {
  const navigator = {gpu:{requestAdapter:()=>{throw Error('unexpected probe');}}};
  assert.deepEqual(await selectRendererBundle({...options,navigator,url:'http://localhost/?renderer=webgl'}),legacy);
  assert.deepEqual(await selectRendererBundle({...options,navigator,fetch:async()=>({ok:false})}),legacy);
});
test('versioned legacy artifact is used on forced, absent, and denied adapters', async () => {
  const webgl = {module:'./generation/crystal-bevy.js',binary:'./generation/crystal-bevy_bg.wasm'};
  const fetch = async()=>({ok:true,json:async()=>({webgpu:modern,webgl})});
  for (const scenario of [{url:'http://localhost/?renderer=webgl',navigator:{gpu:{requestAdapter:()=>{throw Error('must not probe');}}}},
    {url:'http://localhost/',navigator:{}},
    {url:'http://localhost/',navigator:{gpu:{requestAdapter:async()=>{throw Error('denied');}}}}]) {
    assert.deepEqual(await selectRendererBundle({fetch,...scenario}),webgl);
  }
});
