/** Install passive WebGL2 texture accounting in the measured page. */
export function installTextureMemoryAccounting() {
  const meter = window.houseMemory;
  Object.assign(meter, { liveTextureBytes: 0, peakTextureBytes: 0,
    largestTextureBytes: 0, unknownTextureFormats: [] });
  const textures = new WeakMap();
  const proto = WebGL2RenderingContext.prototype;
  const formats = [
    [1, ['R8', 'R8_SNORM', 'R8UI', 'R8I']],
    [2, ['RG8', 'RG8_SNORM', 'RG8UI', 'RG8I', 'R16F', 'R16UI', 'R16I',
      'RGB565', 'RGBA4', 'RGB5_A1', 'DEPTH_COMPONENT16']],
    [3, ['RGB8', 'SRGB8', 'RGB8_SNORM', 'RGB8UI', 'RGB8I']],
    [4, ['RGBA8', 'SRGB8_ALPHA8', 'RGBA8_SNORM', 'RGBA8UI', 'RGBA8I',
      'RG16F', 'RG16UI', 'RG16I', 'R32F', 'R32UI', 'R32I', 'RGB10_A2',
      'RGB10_A2UI', 'R11F_G11F_B10F', 'RGB9_E5', 'DEPTH_COMPONENT24',
      'DEPTH_COMPONENT32F', 'DEPTH24_STENCIL8']],
    [6, ['RGB16F', 'RGB16UI', 'RGB16I']],
    [8, ['RGBA16F', 'RGBA16UI', 'RGBA16I', 'RG32F', 'RG32UI', 'RG32I',
      'DEPTH32F_STENCIL8']],
    [12, ['RGB32F', 'RGB32UI', 'RGB32I']],
    [16, ['RGBA32F', 'RGBA32UI', 'RGBA32I']],
  ];
  const bytesPerPixel = (gl, format, type) => {
    for (const [bytes, names] of formats) {
      if (names.some(name => gl[name] !== undefined && gl[name] === format)) return bytes;
    }
    // Unsized texture formats use the supplied scalar or packed pixel type.
    const packed = [['UNSIGNED_SHORT_5_6_5', 2], ['UNSIGNED_SHORT_4_4_4_4', 2],
      ['UNSIGNED_SHORT_5_5_5_1', 2], ['UNSIGNED_INT_2_10_10_10_REV', 4]];
    for (const [name, bytes] of packed) if (gl[name] !== undefined && type === gl[name]) return bytes;
    const components = format === gl.RGBA ? 4 : format === gl.RGB ? 3 :
      format === gl.RG ? 2 : format === gl.RED ? 1 : 0;
    const scalar = type === gl.UNSIGNED_BYTE || type === gl.BYTE ? 1 :
      type === gl.HALF_FLOAT || type === gl.UNSIGNED_SHORT || type === gl.SHORT ? 2 :
      type === gl.FLOAT || type === gl.UNSIGNED_INT || type === gl.INT ? 4 : 0;
    if (components && scalar) return components * scalar;
    if (!meter.unknownTextureFormats.includes(format)) meter.unknownTextureFormats.push(format);
    return null;
  };
  const binding = (gl, target) => {
    const cube = target === gl.TEXTURE_CUBE_MAP ||
      (target >= gl.TEXTURE_CUBE_MAP_POSITIVE_X && target <= gl.TEXTURE_CUBE_MAP_NEGATIVE_Z);
    const parameter = cube ? gl.TEXTURE_BINDING_CUBE_MAP :
      target === gl.TEXTURE_2D ? gl.TEXTURE_BINDING_2D :
      target === gl.TEXTURE_3D ? gl.TEXTURE_BINDING_3D :
      target === gl.TEXTURE_2D_ARRAY ? gl.TEXTURE_BINDING_2D_ARRAY : undefined;
    return parameter === undefined ? null : gl.getParameter(parameter);
  };
  const account = (texture, levels) => {
    if (!texture) return;
    const previous = textures.get(texture) ?? new Map();
    const before = [...previous.values()].reduce((sum, level) => sum + level.bytes, 0);
    for (const [key, level] of levels) previous.set(key, level);
    const after = [...previous.values()].reduce((sum, level) => sum + level.bytes, 0);
    textures.set(texture, previous);
    meter.liveTextureBytes += after - before;
    meter.peakTextureBytes = Math.max(meter.peakTextureBytes, meter.liveTextureBytes);
    meter.largestTextureBytes = Math.max(meter.largestTextureBytes, after);
  };
  const key = (target, level) => `${target}:${level}`;
  const levels = (gl, target, count, width, height, depth, bpp) => {
    const result = [];
    const faces = target === gl.TEXTURE_CUBE_MAP ?
      Array.from({ length: 6 }, (_, i) => gl.TEXTURE_CUBE_MAP_POSITIVE_X + i) : [target];
    for (let level = 0; level < count; level++) {
      for (const face of faces) result.push([key(face, level),
        { width, height, depth, bpp, bytes: width * height * depth * bpp }]);
      width = Math.max(1, Math.floor(width / 2));
      height = Math.max(1, Math.floor(height / 2));
      if (target === gl.TEXTURE_3D) depth = Math.max(1, Math.floor(depth / 2));
    }
    return result;
  };
  for (const method of ['texStorage2D', 'texStorage3D']) {
    const original = proto[method];
    proto[method] = function (target, count, format, width, height, depth = 1) {
      const result = original.apply(this, arguments);
      const bpp = bytesPerPixel(this, format);
      if (bpp !== null) account(binding(this, target), levels(this, target, count, width, height, depth, bpp));
      return result;
    };
  }
  for (const method of ['texImage2D', 'texImage3D']) {
    const original = proto[method];
    proto[method] = function (...args) {
      const result = original.apply(this, args);
      const [target, level, format] = args;
      const source = method === 'texImage2D' && args.length === 6 ? args[5] : null;
      const width = source ? source.videoWidth || source.naturalWidth || source.width : args[3];
      const height = source ? source.videoHeight || source.naturalHeight || source.height : args[4];
      const depth = method === 'texImage3D' ? args[5] : 1;
      const type = args[source ? 4 : method === 'texImage3D' ? 8 : 7];
      const bpp = bytesPerPixel(this, format, type);
      if (bpp !== null) account(binding(this, target), [[key(target, level),
        { width, height, depth, bpp, bytes: width * height * depth * bpp }]]);
      return result;
    };
  }
  const generate = proto.generateMipmap;
  proto.generateMipmap = function (target) {
    const result = generate.call(this, target);
    const texture = binding(this, target);
    const baseTarget = target === this.TEXTURE_CUBE_MAP ? this.TEXTURE_CUBE_MAP_POSITIVE_X : target;
    const base = textures.get(texture)?.get(key(baseTarget, 0));
    if (base) {
      const extent = Math.max(base.width, base.height, target === this.TEXTURE_3D ? base.depth : 1);
      account(texture, levels(this, target, Math.floor(Math.log2(extent)) + 1,
        base.width, base.height, base.depth, base.bpp));
    }
    return result;
  };
  const remove = proto.deleteTexture;
  proto.deleteTexture = function (texture) {
    const result = remove.call(this, texture);
    const recorded = textures.get(texture);
    if (recorded) meter.liveTextureBytes -= [...recorded.values()].reduce((sum, level) => sum + level.bytes, 0);
    textures.delete(texture);
    return result;
  };
  for (const method of ['compressedTexImage2D', 'compressedTexImage3D']) {
    const original = proto[method];
    proto[method] = function (...args) {
      const result = original.apply(this, args);
      if (!meter.unknownTextureFormats.includes(args[2])) meter.unknownTextureFormats.push(args[2]);
      return result;
    };
  }
}
