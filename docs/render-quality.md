# 3D rendering quality

The renderer uses the existing source palettes and shared assets. Authored
material roles now select roughness and reflectance for ceramic, plaster,
clay, mortar, timber, glass, stone, ice and water. The surface shader adds
bounded specular highlights and samples Bevy's environment lighting without
applying another full diffuse sun response over painted face shading.

Two small procedural cubemaps follow the visible sky's day/night palette.
The specular map has six roughness levels. Their asset handles remain stable;
the renderer refreshes their pixels at five-minute clock boundaries. Fixed
interiors and dark maps do not receive outdoor environment light. These are
sky reflections, not reflections of nearby buildings or actors.

Painted terrain detail and authored scenery textures have complete mip chains,
linear-light downsampling, premultiplied-alpha filtering and 4x anisotropic
sampling. Masked scenery preserves approximate alpha-test coverage at each mip.
Source tile/sprite atlases keep their original pixels and sampling. Terrain
detail uses explicit gradients computed before material branches or discards.

Shadow coverage follows camera distance, zoom and viewport scale in stable depth
bins, with Gaussian filtering and a smaller normal bias. Browser rendering uses
one cascade; native low/balanced/high use one/two/three respectively. This policy
applies across maps instead of selecting a few towns by name. Existing nighttime
and software-renderer shadow rules still apply.

## Presets

Select a native preset at launch with `CRYSTAL_RENDER_QUALITY`:

| Preset | Anti-aliasing | Contact shading | Dynamic range |
| --- | --- | --- | --- |
| `high` | SMAA High | Native SSAO on supported hardware, plus authored contacts | HDR, neutral tonemapping, subtle highlight bloom |
| `balanced` | SMAA High | Authored contacts | Existing display-linear output |
| `low` | Existing FXAA policy | Authored contacts | Existing display-linear output |
| `legacy` | Existing FXAA policy | Existing opt-in SSAO | Previous material, texture and shadow settings for comparison |

Native hardware defaults to high. Recognized software adapters fall back to
low. Browsers default to balanced; Bevy 0.20 SSAO is unsupported on both WebGL
and WebGPU. `CRYSTAL_MODELED_OCCLUSION=off` disables native SSAO independently;
`on` requests it on supported native hardware in other presets.

Post-processing applies only to the world camera. While that HDR world is
active, the marked UI camera shares its floating-point main-buffer format so
Bevy composites both cameras into the same buffer. Its own tone curve and
effects remain unchanged. Switching to 2D restores the original UI buffer
format; battle actor capture targets keep their own camera settings.
Bloom extracts highlights above display white, including luminous
night windows, instead of blurring daylight paint. SMAA uses Bevy's bundled
lookup textures with the Rust zstd decoder. TAA is not enabled: animated grass,
skinning and camera apertures need a separate motion-vector and ghosting review.

The SSAO normal prepass uses the visible surface shader's source cutouts,
coverage fades and cave apertures. It writes geometry normals for unlit painted
materials, which Bevy's standard unlit prepass intentionally omits. Shadow and
depth-only shader selection retain their existing behavior.

Modeled overworld Pokémon use the same shadow-readability policy as painted
people. Their bodies retain PBR contour lighting and cast ground shadows, while
cast-shadow reception is disabled so small followers such as Totodile do not
become silhouettes. Their canonical colors, animation and emission are unchanged.
Elm's lab also continues its existing clinical wall panels behind the complete
rear bookcase run; the freestanding bookcases do not acquire walls.

## Verification

The native location tester renders the production cameras and can capture a
2D reference and 3D view with the same authoritative state:

```sh
cargo build --locked -p crystal-bevy --example render_at_location --features location-tester
CRYSTAL_PREVIEW_MUTE=1 CRYSTAL_RENDER_QUALITY=high \
CRYSTAL_RENDER_TEST_CAMERA='1 0' CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image \
target/debug/examples/render_at_location \
  --pack content-packs/core-modular.browser.crystalpack \
  --map ElmsLab --x 4 --y 3 --hour 12 --view both \
  --screenshot /tmp/geothite-lab-review.png
```

The outputs include a comparison image and a paired-state receipt. Keep capture
outputs outside tracked source. The browser/WebGPU compilation check is:

```sh
cargo check --locked --target wasm32-unknown-unknown -p crystal-bevy \
  --features voxel-view,browser-webgpu
```
