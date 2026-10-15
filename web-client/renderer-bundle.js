// Bevy's WebGL layout is selected at compile time. Keep separate bundles so
// older browsers receive the correctly padded uniforms and texture views.
export async function selectRendererBundle({ fetch, navigator, url }) {
  let legacy = { module: './crystal-bevy.js', binary: './crystal-bevy_bg.wasm' };
  const forceGl = new URL(url).searchParams.get('renderer') === 'webgl';
  try {
    const response = await fetch('./renderer-bundles.json');
    if (!response.ok) return legacy;
    const bundles = await response.json();
    legacy = bundles.webgl ?? legacy;
    if (forceGl) return legacy;
    if (bundles.webgpu && await navigator.gpu?.requestAdapter()) return bundles.webgpu;
  } catch { /* Missing adapter or optional bundle uses the WebGL build. */ }
  return legacy;
}
