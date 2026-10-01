# Battle rendering and capture performance

This checkpoint improves the existing work-in-progress renderer and captures.
It does not certify complete attack fidelity. The original 2D reference governs
attack sequences; artistic world and battlefield styling remain. See the
[source-faithfulness matrix](original-2d-fidelity.md) for known gaps.

The renderer reuses translated source-effect textures, caches immutable battler
palettes, and avoids writing unchanged GPU materials and visibility. Known
software adapters select prelit vertex shading and a 0.75-scale arena. The arena
image shares the existing HUD pass; text and controls remain at native window
resolution. Hardware and unknown adapters retain full-resolution PBR.

These measurements use the cloud machine's software renderer, an optimized
development build, an 1180×812 window and the legal battle fixtures. Compilation
and asset-rendering jobs were stopped. These are not hardware-GPU or 60-fps claims.

| Trace | Median update | p95 update | Observations |
| --- | ---: | ---: | --- |
| Published renderer, no capture | 59.24 ms | 69.34 ms | 1,505 updates / 90.05 s |
| Current prelit/shared-HUD renderer, no capture | 34.27 ms | 46.74 ms | 1,696 updates / 60.02 s |
| Published renderer with capture | 64.34 ms | 74.55 ms | 179 real frames / 12 s capture window |
| Current Hyper Beam capture | 39.77 ms | 50.58 ms | 197 real frames / 8 s capture window |
| Current Psychic capture | 42.47 ms | 54.68 ms | 184 real frames / 8 s capture window |

Current captures retain about 25 and 23 actual frames per second. Their maximum
gaps are 88.29 ms and 75.07 ms; neither contains a gap over 100 ms. This does not
establish that every move, device or cold shader-cache state is free of stalls.
A separate shader experiment showed substantial first-use stalls and was
discarded. The current profile retains the original tonemapping and dithering.

The current arena renders at 885×609 beneath the native HUD. The comparison also
includes refined Raticate, Kadabra and Sudowoodo meshes; the published baseline
used earlier sculptures. No scene objects were removed. The scaled profile is
a quality/performance choice, not native-resolution 3D detail.

## Experimental actor-capture reuse

The physical-size/shared-camera integration has no new performance certification.
The measurements above describe the earlier renderer and cannot establish the
cost of the current framing or optional extracted-row path.

`CRYSTAL_BATTLE_ROW_PROTOTYPE=1` enables only the native Tackle/Water Gun pilot
listed in [immersive battles](immersive-battles.md). An additional
`CRYSTAL_BATTLE_ROW_CAPTURE_CACHE=1` enables static actor-capture reuse for that
pilot. Both switches are off by default and disabled in WebAssembly. Readiness
requires a matching successful draw, output blit and submitted frame; a fixed
frame delay never makes a capture valid. Appearance, camera, lighting, material,
image and target-size changes invalidate the relevant capture. Pending or
unsupported state retains the ordinary capture path.

A matched native Tackle pair at the current physical sizes, 1180×812 window and
885×609 arena captured 190 real frames each. Both cached actors were ready from
the first recorded frame. The uncached first-row gap was 205.763 ms (source
5→10); caching captured source 8 after 43.817 ms and the first nonzero row at
source 10 after another 45.741 ms. Shared source frames 0, 3, 5, 10 and 13 have
byte-identical actor and arena pixels; the only full-image differences are the
random opponent sex glyph in the HUD. Prewarm cost precedes these recordings.

Across all 189 timestamp intervals, median/p95 were 40.669/47.968 ms uncached
and 42.435/52.250 ms cached (linear quantiles). This demonstrates removing the
first row hitch in this pair, not a steady-state frame-rate improvement. Water
Gun, cold/changed-target lifecycle, alpha-edge, resize and interruption checks
remain open, so both flags stay opt-in. Source clocks and controller decisions
stay outside this cache.

## Source timing and recording

The verified Hyper Beam recording includes four source beam segments, the beam
tip, original flashes and subsequent visible HP loss. Source frames 0–59 span
0.989 seconds. Psychic reaches source frame 164 with its original wave objects,
hue changes and deformation. Misses retain the source miss sequence; they are
not substituted with a successful-hit animation for recording.

Capture deadlines are anchored to the recording start. The previous
`last_capture + interval` scheduler could turn a 40-Hz renderer into a 20-Hz
recording. The new scheduler admits at most one capture per actual update,
skips missed deadlines and keeps a bounded readback queue. It never creates
intermediate motion frames or changes the source animation clock.

`CRYSTAL_CAPTURE_FPS=30` is the default cap; `60` retains more observed frames
when available. A 60-Hz cap does not imply 60-fps video. CSV diagnostics retain
actual times, source frames, effects, asset counts, lighting and scene/window
dimensions. Encoding the generated `frames.ffconcat` with `-fps_mode vfr`
preserves observed timestamps. Verified MP4 times match the capture CSVs within
0.51 ms. The final held frame is an endpoint, not another rendered sample.

## Reproduce

Use an external, ignored pack and the production preview controller:

```sh
cargo build --locked -p crystal-bevy \
  --features voxel-view,location-tester,fullscreen-scaling \
  --example immersive_battle_3d
target/debug/examples/immersive_battle_3d \
  content-packs/core-modular.browser.crystalpack --hyper-beam \
  --measure target/battle-measure --seconds 60
CRYSTAL_CAPTURE_FPS=60 target/debug/examples/immersive_battle_3d \
  content-packs/core-modular.browser.crystalpack --hyper-beam \
  --record target/battle-record --record-on-move --seconds 8
```

Choose FIGHT and HYPER BEAM through the normal menu. Use `--psychic` for the
other fixture. `--reduced-flashes` or F4 limits contrast while retaining source
timing and gameplay. F3 restores classic presentation.

Native overrides are independent: `CRYSTAL_BATTLE_LIGHTING=pbr|vertex` and
`CRYSTAL_BATTLE_QUALITY=native|balanced|performance`. Scene scales are 1.0, 0.75
and 0.5 per dimension; HUD resolution is unchanged. Explicit PBR/native restores
the full-quality path on software adapters. Automatic selection also runs in
WebAssembly; browser compilation is checked, but browser hardware performance
has not been measured in this environment.

Regressions cover source-clock presentation at 30/60 Hz, unchanged Full/Reduced
semantics, translated texture reuse, stable materials, one HUD pass,
landscape/portrait resize, shake/depth alignment, capture caps at 30/40/60/120-Hz
render rates, stalls and readback backpressure. Captures and pack-derived reports
remain verification output outside the committed bundle.
