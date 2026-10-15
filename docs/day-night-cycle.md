# Outdoor 3D day and night lighting

The optional 3D renderer samples the game's local RTC seconds on outdoor maps
with an automatic palette. Dawn and sunset use warm, low directional light;
noon retains the authored regional light; night uses cool sky fill and weaker
moonlight without large moon-cast silhouettes. Morning/evening sun angles
limit cast-shadow length to 1.25 times the caster height (less than one caster
height at 06:00), and low-energy shadows
fade in contrast while retaining the surface pigment and scene light tint.
The sun direction, color, energy, and horizon interpolate with
smoothstep between clock stops, including across midnight. Lighting updates
every second, without movement or terrain reconstruction. The real game clock
sets the pace: dawn, daylight, dusk, and night unfold across the 24-hour day.

Classic 2D rendering and its morning/day/night texture palettes remain
source-authored. Those discrete palette changes are preserved; the continuous
3D lighting does not change encounter periods or advance the game clock.
Indoor, cave, and explicitly fixed map palettes retain their existing lighting.

## Reproduce the comparison

Use the ignored local browser pack and the production render-at-location
client. These verification commands leave GPU dependencies and mesh generation
optimized while reducing compile optimization for the asset/runtime/shell crates.
Keep the map, tile, facing, party, and camera unchanged across captures.
The paired capture records the game state and renders both views from that state.

```sh
CARGO_TARGET_DIR=target/day-night-review cargo build --locked \
  -p crystal-bevy --features location-tester \
  --example render_at_location --profile wasm-dev -j2 \
  --config 'profile.wasm-dev.package.crystal-assets.opt-level=0' \
  --config 'profile.wasm-dev.package.crystal-runtime.opt-level=0' \
  --config 'profile.wasm-dev.package.crystal-bevy.opt-level=0'

for hour in 0 6 12 18; do
  CRYSTAL_RENDER_TEST_CAPTURE_TIMEOUT_SECS=300 \
  CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image \
    target/day-night-review/wasm-dev/examples/render_at_location \
      --pack content-packs/core-modular.browser.crystalpack \
      --map NewBarkTown --x 8 --y 8 --view both --hour "$hour" \
      --screenshot "target/day-night-review/after-$hour.png"
done
```

The generated screenshots and state witnesses are verification output, not
tracked game content. In each pair, inspect character silhouettes, facade and
path readability, tree shading, shadow direction/length, and source palette
agreement. Compare before and after at the same hour, rather than comparing a
daytime baseline to an updated night view.

```sh
RUST_MIN_STACK=33554432 \
CRYSTAL_RENDER_TEST_PACK="$PWD/content-packs/text-tui.crystalpack" \
CARGO_TARGET_DIR=target/day-night-review cargo test --locked \
  -p crystal-bevy -p crystal-voxel-view --features crystal-bevy/location-tester \
  --profile wasm-dev --lib day_cycle -j2 \
  --config 'profile.wasm-dev.package.crystal-assets.opt-level=0' \
  --config 'profile.wasm-dev.package.crystal-runtime.opt-level=0' \
  --config 'profile.wasm-dev.package.crystal-bevy.opt-level=0' \
  --config 'profile.wasm-dev.package.crystal-voxel-view.opt-level=0' -- --test-threads=2
```

The regressions cover clock publication while idle, retained classic textures,
light energy and sun transforms, camera horizon, continuity at source palette
boundaries, every second of the full day, midnight wrap, and indoor/cave isolation.

## Verified on 2026-10-13

Native production captures passed at 06:00, 12:00, 18:00, and 00:00 in
NewBarkTown at `(8, 8)`. Every paired state witness reports
`state_preserved=true`. Composition, actor/scenery scale, facing, and source
palette identity were inspected against the paired 2D view.

The final revision replaces oversized cyan/gray dusk shadows with shorter,
low-contrast shadows tinted by the scene light. Night retains the source-like
blue temperature without large cast silhouettes. Morning no longer stacks its
yellow temperature with a second regional golden tint. Noon retains its original
light direction and energy, with corrected shadow fill color.

The progressive-clock follow-up also checks 06:00 and the interpolated 08:00
lighting. At 06:00, raising the sun reduces the geometric shadow length by about
16%; the lower-energy shadow blend preserves more path color and reduces the
visual hardness of building and actor silhouettes. This is a contrast change,
not a change to the shadow-map filtering kernel.

The subsequent morning color correction removes the cooler ambient tint from
outdoor cast-shadow fill. Cast shade now follows the lit surface's color tint;
shadow geometry and contrast remain unchanged. In paired native 07:00 captures
at NewBarkTown `(8, 8)`, a fixed road-shadow patch changed from mean sRGB
`(164.55, 161.15, 130.88)` to `(165.49, 161.06, 128.47)`. The nearby exposed
path remained `(170.21, 166.15, 131.73)`: the subtle correction removes excess
blue in the shade rather than recoloring the exposed scene. Both paired state
witnesses report `state_preserved=true`.

The midnight screenshot's mean weighted sRGB image luma decreased from 109.7 to
94.3 (about 14%). This is a measurement of this captured scene, not a physical
illuminance or performance measurement.

All six new clock/cycle regressions and four existing atmosphere, fixed-palette,
cave-energy, and cascade-restoration regressions passed with the external pack.
The native mesh-optimized build and final GPU readbacks passed. Comparison images
are local verification artifacts outside tracked source; content packs remain
external and ignored.

## Procedural sky dome

The outdoor camera uses an unlit, camera-centered sky dome, with horizon haze,
a blue zenith, slowly drifting three-octave clouds, directional sun glow, a
crescent moon, and fixed stars that fade through twilight. Colors and star
visibility follow the same local RTC as the outdoor lights, continuously across
midnight. The horizon color also supplies the outdoor atmosphere's fog color.
The dome casts no shadows, writes no depth, and is hidden in caves, interiors,
and inactive 3D views. It adds no texture assets or external content dependencies.

Use a low-angle paired capture to inspect the sky in the production renderer:

```sh
CRYSTAL_RENDER_TEST_CAMERA='1 0 5 0' \
CRYSTAL_RENDER_TEST_CAPTURE_TIMEOUT_SECS=300 \
CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image \
  target/day-night-review/wasm-dev/examples/render_at_location \
    --pack content-packs/core-modular.browser.crystalpack \
    --map NewBarkTown --x 8 --y 8 --view both --hour 12 \
    --screenshot target/day-night-review/sky-day.png
```

`CRYSTAL_RENDER_TEST_CAMERA` accepts zoom, orbit (45-degree units), optional
pitch in degrees, and optional first-person (`0` or `1`). It affects render-test
presentation only. For sky-facing first-person reviews use `1 5 -25 1` at
06:00 for the sun, or `1 3 -25 1` at 00:00 for the moon.

The native sky review passed at 06:00, 12:00, 18:00, and 00:00, plus sun-facing
and moon-facing first-person captures. All six paired 2D/3D state witnesses
report `state_preserved=true`. The two sky regressions and all ten existing
clock/lighting regressions passed. Browser GPU output was not exercised in
this native review; the shader uses ordinary fragment operations and no textures,
compute features, or platform-specific branches.
