# Animated human model catalog

`crates/crystal-voxel-view/models/johto_characters/catalog.glb` is the canonical
human runtime asset: 75 distinct named scenes, each with the original sixteen
rigid joints, materials, and parent-relative bind translations. Its 977 shared
geometry accessor sets retain the exact runtime float32 positions and normals,
unsigned triangle indices, and RGBA colors. No welding, decimation, recentering,
normal repair, or arbitrary rescaling occurs. The characters remain rigidly
articulated; the file does not pretend to contain a deforming skin.

Each scene owns named `.walk` and `.run` animation clips with complete pelvis,
upper-body, arm, leg, and shoe movement. Clips use standard glTF translation and
quaternion channels, so a normal glTF viewer can play the full loops. Identical
animation sample accessors are shared across all characters. A normalized
one-second clip describes a complete distance-driven cycle; its duration is
not a prescription for game playback speed. The game keeps authoritative travel
and planted-foot IK in control. Its renderer selects and blends the clips using
traveled-distance phase; importing an asset never advances gameplay state.

`tools/human_locomotion.py` defines the production curves and includes checks
for exact loop endpoints, quaternion validity, foot contacts, and bounded
interpolation error. `tools/animated_glb.py` writes and reads the standard binary
format with Python's standard library. Runtime readers and tooling use the
checked-in GLB directly; a clean game build does not require Blender or
regenerated JSON. JSON rigs and a shared geometry table are authoring
intermediates, never a second tracked copy of the human catalog.

## Authoring and verification

```sh
# Original editable source generation; its temporary rig JSON is removed.
blender -b --python tools/build-johto-characters.py -- target/johto-character-kit --skip-preview
# The result contains catalog.glb and editable Blender source batches.

# Convert an existing transient authoring directory explicitly.
python3 tools/animated_glb.py --rig-directory /path/to/authored-rigs --output target/catalog.glb

# No Blender or game pack required for these checks.
python3 tools/test_animated_glb.py
python3 tools/human_locomotion.py
python3 tools/check-johto-characters.py
python3 tools/check-model-layout.py
python3 tools/check-room-furniture-bindings.py
python3 tools/check-facility-table-rigs.py
```

The ten current `character-families-01.blend` through `-10.blend` snapshots are
generated output, so they are not stored alongside the canonical catalog.
The recipe recreates separate named parts, local transforms, joint pivots,
materials and the studio layout. The unique older `characters.blend` is retained.
A complete Blender 4.3.2 replay reproduced all 917,024 triangle positions and
winding, materials and joint binds. Corner normals can vary by one six-decimal
export step (measured maximum 0.000001014); the checked-in GLB preserves the
original runtime normal bits. Regenerated authoring output therefore requires
review rather than blindly replacing the canonical file by hash.

The migration check compares every original JSON rig with its decoded GLB scene
while those inputs exist, including signed-zero float bits. Once retired, the
ongoing suite continues against the canonical GLB without JSON sidecars. It
checks all scenes, shared geometry, parent maps, materials, production clips,
and reader/exporter round trips. The Rust suite also retains compact per-rig
geometry digests captured from an exact comparison with the original Rust
loader, covering normalized normals, colors, indices, binds and UVs for all 75
looks. Optional STL previews read the GLB bind pose
and remain ignored output; see [local static previews](model-stl.md).

## Optional articulation proof

```sh
python3 tools/animated_glb.py --repo . --out target/animated-glb --human-catalog
```

This compatibility command exports separate trainer-wave and Pidgeotto-wing
proofs plus an optional human catalog with demo waves. Proof clips are clearly
marked as demonstrations and are not production locomotion or original attack
choreography. Pidgeotto's existing canonical JSON is unchanged by this command.
Proof outputs belong in ignored directories and are not additional tracked
geometry. The exporter does not migrate any other Pokémon.

## Native integration check

The native game runs imported locomotion with ordinary controller movement.
The verification route completed 64 movement frames across five grid origins
without an additional terrain rebuild. At rest, matching before/after captures
were pixel-identical outside the trainer's live breathing/blink region.
All 832 voxel tests pass (two existing ignored tests), including exact geometry
digests for all 75 rigs, loop seams, planted feet at 60/30/9 Hz, source-distance
phase, and entity/mesh reuse. All 35 asset/source Python checks pass.
The full browser client compiles for `wasm32-unknown-unknown`; browser rendering
was not exercised in this cloud environment.

Native timing used `--features location-tester` on both builds, a 32-tile terrain
margin, the 1180×812 viewport, New Bark `(13,6)`, orbit `-1`, zoom `3`, and Mesa
25.0.7 llvmpipe/OpenGL. Four warmed 15-second no-readback runs in baseline,
GLB, GLB, baseline order measured median/p95 milliseconds of 172.62/292.88,
150.56/241.65, 135.00/227.57, and 116.89/207.85. This noisy sample does not
establish a frame-rate improvement or a repeatable added cost. A separate
800×550 actual-timestamp recording averaged about 8 fps and remains a technical
check, not a fluid-performance demonstration. The earlier unmatched comparison
used a 48-tile fullscreen-scaling margin for only one build and is excluded.

The preview accepts `--size WIDTHxHEIGHT` for explicit capture dimensions.
Recording a scripted walk waits for its first ordinary input after warmup;
measurement-only runs keep their existing start condition. Normal play and
the authoritative movement clock are unchanged.
