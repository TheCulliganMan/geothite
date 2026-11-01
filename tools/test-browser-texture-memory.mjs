import assert from 'node:assert/strict';
import { installTextureMemoryAccounting } from './browser-texture-memory.mjs';

class FakeWebGL2 {
  constructor() { this.bound = new Map(); this.calls = []; }
  getParameter(parameter) { return this.bound.get(parameter) ?? null; }
}
Object.assign(FakeWebGL2.prototype, {
  TEXTURE_2D: 3553, TEXTURE_3D: 32879, TEXTURE_2D_ARRAY: 35866,
  TEXTURE_CUBE_MAP: 34067, TEXTURE_CUBE_MAP_POSITIVE_X: 34069,
  TEXTURE_CUBE_MAP_NEGATIVE_Z: 34074, TEXTURE_BINDING_2D: 32873,
  TEXTURE_BINDING_3D: 32874, TEXTURE_BINDING_2D_ARRAY: 35869,
  TEXTURE_BINDING_CUBE_MAP: 34068, RGBA8: 32856, R16F: 33325,
  RGBA: 6408, RGB: 6407, RED: 6403, RG: 33319,
  UNSIGNED_BYTE: 5121, FLOAT: 5126, HALF_FLOAT: 5131,
});
for (const method of ['texStorage2D', 'texStorage3D', 'texImage2D', 'texImage3D',
  'generateMipmap', 'deleteTexture', 'compressedTexImage2D', 'compressedTexImage3D']) {
  FakeWebGL2.prototype[method] = function (...args) { this.calls.push([method, args]); return 42; };
}
globalThis.window = { houseMemory: {} };
globalThis.WebGL2RenderingContext = FakeWebGL2;
installTextureMemoryAccounting();
const gl = new FakeWebGL2();
const meter = window.houseMemory;
const bind = (parameter) => { const texture = {}; gl.bound.set(parameter, texture); return texture; };
const initial = meter.liveTextureBytes;
const texture = bind(gl.TEXTURE_BINDING_2D);
assert.equal(gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA8, 8, 4, 0, gl.RGBA, gl.UNSIGNED_BYTE, null), 42);
assert.equal(meter.liveTextureBytes - initial, 128);
gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA8, 4, 4, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);
assert.equal(meter.liveTextureBytes - initial, 64, 'redefinition replaces storage');
gl.generateMipmap(gl.TEXTURE_2D);
assert.equal(meter.liveTextureBytes - initial, 84, '4x4, 2x2 and 1x1 RGBA levels');
gl.deleteTexture(texture);
assert.equal(meter.liveTextureBytes, initial, 'deletion releases every mip');
gl.deleteTexture(texture);
assert.equal(meter.liveTextureBytes, initial, 'repeated deletion does not double count');

const cube = bind(gl.TEXTURE_BINDING_CUBE_MAP);
gl.texStorage2D(gl.TEXTURE_CUBE_MAP, 3, gl.RGBA8, 4, 4);
assert.equal(meter.liveTextureBytes - initial, 504, 'six cube faces with three mip levels');
gl.deleteTexture(cube);
const array = bind(gl.TEXTURE_BINDING_2D_ARRAY);
gl.texStorage3D(gl.TEXTURE_2D_ARRAY, 3, gl.R16F, 4, 4, 7);
assert.equal(meter.liveTextureBytes - initial, 294, 'array mipmaps retain all seven layers');
gl.deleteTexture(array);
const volume = bind(gl.TEXTURE_BINDING_3D);
gl.texImage3D(gl.TEXTURE_3D, 0, gl.RGBA8, 4, 4, 4, 0, gl.RGBA, gl.UNSIGNED_BYTE, null);
gl.generateMipmap(gl.TEXTURE_3D);
assert.equal(meter.liveTextureBytes - initial, 292, 'volume depth also halves at each mip');
gl.deleteTexture(volume);

const image = bind(gl.TEXTURE_BINDING_2D);
gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, gl.RGBA, gl.UNSIGNED_BYTE,
  { naturalWidth: 512, naturalHeight: 512, width: 16, height: 16 });
assert.equal(meter.liveTextureBytes - initial, 1048576, 'DOM image uses native pixel dimensions');
assert.equal(meter.largestTextureBytes, 1048576);
gl.deleteTexture(image);
const unknown = bind(gl.TEXTURE_BINDING_2D);
gl.texStorage2D(gl.TEXTURE_2D, 1, 999999, 512, 512);
gl.compressedTexImage2D(gl.TEXTURE_2D, 0, 888888, 4, 4, 0, new Uint8Array(16));
assert.deepEqual(meter.unknownTextureFormats, [999999, 888888]);
assert.equal(meter.liveTextureBytes, initial, 'unsupported storage is explicit, never fabricated');
gl.deleteTexture(unknown);
assert.ok(gl.calls.length > 15, 'all original API calls were forwarded');
console.log('Texture accounting passed replacement, mipmaps, cube, array, volume, DOM-image and unknown-format checks.');
