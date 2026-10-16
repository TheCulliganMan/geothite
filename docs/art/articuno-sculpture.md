# Articuno feather banks and curling streamer

`tools/articuno_sculpt.py` authors the original ice bird.
`tools/articuno_glb.py` exports the canonical skinned GLB; the battle-species
builder regenerates it through that same path. No third-party geometry is
included.

The slender blue breast has a fitted pale feather bib and throat collar.
Its calm head has fitted eyes and glints, a short grey bill and three
crystalline crown blades. Each bent wing arm carries nine primary vanes and
seven coverts, with separate closed front and rear layers. Grey shins lead
to feet with three forward digits, a rear digit and ivory talons. A continuous
blue ribbon sweeps backward into a broad curl and tapered tip. Three raised
longitudinal channels follow its actual curved surface.

The optimized source has 109 recoverable parts packed into 49 runtime
primitives, 19,074 vertices, 26,704 triangles and 11 joints. The GLB occupies
777,424 bytes and 210 accessors, within the unchanged renderer limits. The
former static source had 43,145 vertices and 63,816 triangles. Excess cap
subdivisions, duplicated feather corners and oversampled channel paths were
reduced. All 32 core anatomy parts, including the complete main tail ribbon,
retain their original float32 positions, normals, colors and triangle order.
Their geometry SHA-256 is
`3e1053c83be263e340ea67b0dd98bae4a1ade7ad9f9a4eaa6ce40cdc76fef350`.
The other 74 feather and fitted-marking outlines were compared in both
directions against their original projected boundaries; maximum displacement
is 0.000000112 authoring units. The three channel paths remain fitted to the
unchanged ribbon.

The canonical source geometry SHA-256 is
`26e353890f4a03ddfdc599cbeb0a77f14f474505f2c060627ac25ebf4b926160`;
the packed runtime geometry pin is
`e38af1c0bb7982e268d4de9841a4722458a42d6d8f23e5abb82d697d7f0b377c`.
Strict decoding restores all source parts in their original order and rejects
noncanonical data. Same-color flight feathers with a shared carrier are
packed together without changing their individual geometry.

The identity root fixes every foot, digit and talon. Blended shins bridge to
the yawing body. The pale bib follows the breast, the collar follows the neck,
and the complete gaze, bill and crown follow the head. Each complete wing
assembly has one coherent carrier. Three tail joints use continuous
arc-length weights along the authored ribbon; the channels use the same
field. Tail rotations preserve vertical clearance while the curl articulates.
Rotation-only idle, attack and recoil clips return exactly to neutral.

Authoring height stays 1.28 with its lowest point at zero. The local verified
pack supplies a 5 ft 7 in production height of 1.7018 metres.

Eleven anatomy tests and nine decoded-skin tests passed, covering occupied
anatomical joins, supported fitted surfaces, winding, physical edge strain,
fixed floor contacts, independent wing and tail motion, posed ribbon
self-intersection, channel-to-ribbon distance, strict round trips and budgets.
Two focused Rust tests passed for production GPU activation at pack height,
planted talons, tail clearance, articulation and culling bounds. The shared
point-to-triangle distance helper also passed its two existing dragon-ribbon
checks after moving to common test support. Blender regeneration is
byte-identical. Fixed-camera front and rear neutral, idle, attack and recoil
software views were inspected. Shiny checks are skipped at the user's request.

The production native client built successfully with:

```sh
CARGO_TARGET_DIR=target/pokemon-models cargo build --locked -p crystal-bevy \
  --features location-tester,bevy/zstd_rust --example immersive_battle_3d -j1 \
  --config profile.dev.package.crystal-voxel-view.opt-level=0 \
  --config profile.dev.package.crystal-bevy.opt-level=0
```

The wider exact-species grounding/volume and elapsed-animation timing tests
passed. Normal paired 3D/2D inspection against Sentret shows the explicit
Articuno label, fitted pale chest, layered wings, complete tail curl and its
three clear channels. Grey feet and ivory talons meet visible terrain. The
camera accepted the shot and preserved game state: core frame 3950/hash
123451209, LCD animation frame 4754, virtual time 88.511306625. Final ignored
QA prefix is `target/pokemon-native-review/articuno-normal`. This accepts the
current Articuno pass; the full catalog and remaining animation work stay open.
