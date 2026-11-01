/** Keep shader normalization in Rust; forward browser shader descriptors. */
export function installGpuShaderNormalization(wasm) {
  if (typeof GPUDevice === 'undefined' || typeof wasm.crystal_normalize_webgpu_shader !== 'function') return;
  const create = GPUDevice.prototype.createShaderModule;
  GPUDevice.prototype.createShaderModule = function (descriptor) {
    return create.call(this, { ...descriptor,
      code: wasm.crystal_normalize_webgpu_shader(descriptor.code) });
  };
}
