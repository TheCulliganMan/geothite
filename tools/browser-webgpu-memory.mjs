/** Passive WebGPU accounting and test-only drawing-buffer readback. */
export function installWebGpuMemoryAccounting() {
  if (typeof GPUDevice === 'undefined') return;
  const meter = window.houseMemory;
  meter.deviceLosses = [];
  meter.gpuErrors = [];
  const requestDevice = GPUAdapter.prototype.requestDevice;
  GPUAdapter.prototype.requestDevice = function (...args) {
    return requestDevice.apply(this, args).then(device => {
      device.lost.then(info => {
        meter.contextLost++;
        meter.deviceLosses.push({ reason:info.reason, message:info.message });
      });
      device.addEventListener('uncapturederror', event => {
        meter.gpuErrors.push({ type:event.error.constructor.name, message:event.error.message });
      });
      return device;
    });
  };
  meter.bufferLabels = Object.create(null);
  const releaseBuffer = token => {
    if (!token.active) return;
    meter.liveBufferBytes -= token.size;
    meter.bufferLabels[token.label].liveBytes -= token.size;
    token.active = false;
  };
  const sizes = new WeakMap(), textureSizes = new WeakMap();
  // WGPU drops JS references rather than explicitly destroying every object.
  // Count those releases as well; the driver footprint is measured separately.
  const buffersCollected = new FinalizationRegistry(token => {
    releaseBuffer(token);
  });
  const texturesCollected = new FinalizationRegistry(token => {
    if (token.active) { meter.liveTextureBytes -= token.size; token.active=false; }
  });
  const contexts = new WeakMap(), canvasTextures = new WeakSet(), canvasViews = new WeakSet();
  const passes = new WeakMap();
  let current;
  const createShader = GPUDevice.prototype.createShaderModule;
  GPUDevice.prototype.createShaderModule = function (descriptor) {
    const module = createShader.call(this, descriptor);
    module.getCompilationInfo().then(info => {
      for (const message of info.messages) if (message.type === 'error') {
        const detail = { message:message.message, lineNum:message.lineNum, linePos:message.linePos,
          context:descriptor.code.split('\n').slice(Math.max(0,message.lineNum-4),message.lineNum+3).join('\n') };
        (window.houseShaderErrors ??= []).push(detail);
        console.error(JSON.stringify({ shaderCompilationError:detail }));
      }
    });
    return module;
  };
  const createBuffer = GPUDevice.prototype.createBuffer;
  GPUDevice.prototype.createBuffer = function (descriptor) {
    meter.backend = 'webgpu';
    const buffer = createBuffer.call(this, descriptor);
    const label = descriptor.label || '(unlabelled)';
    const group = meter.bufferLabels[label] ??= { created:0, liveBytes:0, uploadedBytes:0 };
    group.created++;
    group.liveBytes += descriptor.size;
    const token={size:descriptor.size,active:true,label}; sizes.set(buffer,token);
    buffersCollected.register(buffer,token,token);
    if (descriptor.mappedAtCreation) {
      meter.bufferUploadBytes += descriptor.size;
      group.uploadedBytes += descriptor.size;
    }
    meter.largestBuffer = Math.max(meter.largestBuffer, descriptor.size);
    meter.liveBufferBytes += descriptor.size;
    meter.peakBufferBytes = Math.max(meter.peakBufferBytes, meter.liveBufferBytes);
    return buffer;
  };
  const destroyBuffer = GPUBuffer.prototype.destroy;
  GPUBuffer.prototype.destroy = function () {
    const token=sizes.get(this);
    if (token?.active) { releaseBuffer(token); buffersCollected.unregister(token); }
    sizes.delete(this);
    return destroyBuffer.call(this);
  };
  const writeBuffer = GPUQueue.prototype.writeBuffer;
  GPUQueue.prototype.writeBuffer = function (buffer, offset, data, dataOffset, size) {
    meter.bufferUploadBytes += size ?? data.byteLength;
    const token = sizes.get(buffer);
    if (token) meter.bufferLabels[token.label].uploadedBytes += size ?? data.byteLength;
    return writeBuffer.apply(this, arguments);
  };
  const formatBytes = { r8unorm:1, rg8unorm:2, r16float:2, rg16float:4,
    r16uint:2, r16sint:2, rg16uint:4, r32uint:4, r32sint:4,
    r32float:4, rg32float:8, rgba32float:16, rgba16float:8,
    rgba8unorm:4, 'rgba8unorm-srgb':4, bgra8unorm:4, 'bgra8unorm-srgb':4,
    depth32float:4, depth24plus:4, 'depth24plus-stencil8':4, depth16unorm:2,
    rgb10a2unorm:4, rg11b10ufloat:4 };
  const createTexture = GPUDevice.prototype.createTexture;
  GPUDevice.prototype.createTexture = function (descriptor) {
    const texture = createTexture.call(this, descriptor);
    const dimensions = descriptor.size;
    let [width, height, depth] = Array.isArray(dimensions) ? dimensions :
      [dimensions.width, dimensions.height ?? 1, dimensions.depthOrArrayLayers ?? 1];
    height ??= 1; depth ??= 1;
    const bytes = formatBytes[descriptor.format];
    if (!bytes) {
      if (!meter.unknownTextureFormats.includes(descriptor.format)) meter.unknownTextureFormats.push(descriptor.format);
      return texture;
    }
    let total = 0;
    for (let level = 0; level < (descriptor.mipLevelCount ?? 1); level++) {
      total += width * height * depth * bytes * (descriptor.sampleCount ?? 1);
      width = Math.max(1, width >> 1); height = Math.max(1, height >> 1);
      if (descriptor.dimension === '3d') depth = Math.max(1, depth >> 1);
    }
    const token={size:total,active:true};textureSizes.set(texture,token);
    texturesCollected.register(texture,token,token);meter.liveTextureBytes += total;
    meter.peakTextureBytes = Math.max(meter.peakTextureBytes, meter.liveTextureBytes);
    meter.largestTextureBytes = Math.max(meter.largestTextureBytes, total);
    return texture;
  };
  const destroyTexture = GPUTexture.prototype.destroy;
  GPUTexture.prototype.destroy = function () {
    const token=textureSizes.get(this);
    if (token?.active) { meter.liveTextureBytes -= token.size;token.active=false;texturesCollected.unregister(token); }
    textureSizes.delete(this);
    return destroyTexture.call(this);
  };
  const configure = GPUCanvasContext.prototype.configure;
  GPUCanvasContext.prototype.configure = function (descriptor) {
    contexts.set(this, descriptor);
    return configure.call(this, { ...descriptor,
      usage: (descriptor.usage ?? GPUTextureUsage.RENDER_ATTACHMENT) | GPUTextureUsage.COPY_SRC });
  };
  const getTexture = GPUCanvasContext.prototype.getCurrentTexture;
  GPUCanvasContext.prototype.getCurrentTexture = function () {
    const texture = getTexture.call(this); canvasTextures.add(texture);
    current = { texture, canvas:this.canvas, ...contexts.get(this) };
    return texture;
  };
  const createView = GPUTexture.prototype.createView;
  GPUTexture.prototype.createView = function () {
    const view = createView.apply(this, arguments);
    if (canvasTextures.has(this)) canvasViews.add(view);
    return view;
  };
  const beginPass = GPUCommandEncoder.prototype.beginRenderPass;
  GPUCommandEncoder.prototype.beginRenderPass = function (descriptor) {
    const pass = beginPass.call(this, descriptor);
    passes.set(pass, descriptor.colorAttachments.some(attachment =>
      attachment && (canvasViews.has(attachment.view) || canvasViews.has(attachment.resolveTarget))));
    return pass;
  };
  for (const name of ['draw', 'drawIndexed', 'drawIndirect', 'drawIndexedIndirect']) {
    const original = GPURenderPassEncoder.prototype[name];
    GPURenderPassEncoder.prototype[name] = function () {
      meter.drawCalls++;
      if (passes.get(this)) meter.defaultFramebufferDrawCalls++;
      return original.apply(this, arguments);
    };
  }
  const submit = GPUQueue.prototype.submit;
  GPUQueue.prototype.submit = function () {
    const result = submit.apply(this, arguments);
    if (current && (meter.requestDefaultCapture || meter.requestDefaultPixel) && !meter.captureInFlight) {
      const capture = meter.requestDefaultCapture, pixel = meter.requestDefaultPixel;
      meter.requestDefaultCapture = meter.requestDefaultPixel = false;
      meter.captureInFlight = true;
      const target = current;
      const { width, height } = target.canvas;
      const stride = Math.ceil(width * 4 / 256) * 256;
      const buffer = target.device.createBuffer({ size:stride * height,
        usage:GPUBufferUsage.COPY_DST | GPUBufferUsage.MAP_READ });
      const encoder = target.device.createCommandEncoder();
      encoder.copyTextureToBuffer({ texture:target.texture }, { buffer, bytesPerRow:stride },
        { width, height, depthOrArrayLayers:1 });
      submit.call(this, [encoder.finish()]);
      (async () => {
        try {
          await buffer.mapAsync(GPUMapMode.READ);
          const source = new Uint8Array(buffer.getMappedRange());
          const rgba = new Uint8ClampedArray(width * height * 4);
          for (let row = 0; row < height; row++) rgba.set(source.subarray(row * stride, row * stride + width * 4), row * width * 4);
          if (target.format.startsWith('bgra')) for (let i=0; i<rgba.length; i+=4) {
            const blue=rgba[i]; rgba[i]=rgba[i+2]; rgba[i+2]=blue;
          }
          if (pixel) {
            const offset=(Math.floor(height/2)*width+Math.floor(width/2))*4;
            meter.defaultFramebufferPixel=[...rgba.subarray(offset,offset+4)];
          }
          if (capture) {
            const canvas=document.createElement('canvas'); canvas.width=width; canvas.height=height;
            canvas.getContext('2d').putImageData(new ImageData(rgba,width,height),0,0);
            window.houseCapturedFrame=canvas.toDataURL('image/png');
          }
          buffer.unmap();
        } catch (error) { window.houseCaptureError=String(error); }
        finally { buffer.destroy(); meter.captureInFlight=false; }
      })();
    }
    return result;
  };
}
