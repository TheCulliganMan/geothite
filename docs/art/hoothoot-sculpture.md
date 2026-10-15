# Hoothoot sculpture and standard skin

Hoothoot now has one original canonical body at
`models/battle_species/hoothoot.glb` under `crates/crystal-voxel-view`, authored
by `tools/hoothoot_sculpt.py` and `tools/hoothoot_glb.py`. It fills a previously
unsupported battle-catalogue species and uses the existing generic GPU skin
path. The same body is available to exact-species world presentation.

The round continuous owl body carries fitted navy eye rings, white lenses,
black pupils, clock-hand brows, folded wings, pale breast chevrons, a short
beak and a compact tail fan. There are two anatomical legs/feet: one supports
the authored rest stance, and the second is tucked behind the lower body.
This stance matches the single visible support in the inspected back sprite;
matching every pose in the six-frame front animation remains motion review.

The verified local pack supplies height digits 204, or 0.7112 m (2′4″). Its
normal/front/back palettes agree: RGB555 `(23,13,6)` produces RGB8
`(189,107,49)` for ochre, and `(4,6,16)` produces `(33,49,132)` for navy by
runtime bit replication. White and black complete the four authored colours.
The reference sprites were read from that pack into ignored temporary QA
files and removed after inspection; no game-content copy is shipped.

The final GLB has 38 named parts, 16,597 normal-split vertices, 25,884 triangles,
12 joints, 190 accessors and 733,832 bytes. All existing reader bounds are
unchanged. Authored full/body reference height is 1.0. Neutral digest:
`93fbea6a47abff0fff1f6ee6315f7ac09ea26bb1bf60a26b0d293a8ae7784555`.

The body and breast marks share a lower-root/upper-body field; the complete
face stays on the body bone. Each clock brow and folded wing has root/tip
joints, with spatial weight fields shared by their fitted parts. Tail feathers
blend into their fan joint; the lower beak has a jaw joint. Legs, pads, toes
and claws retain the identity floor root. Idle lasts 3.2 seconds with 81 keys;
attack/hit last one second with 65 keys. Clips use rotations only, identity
endpoints and the existing source battle cue clock, with an early body brace
and trailing brows, wings and tail.

The first native preview exposed gaps where coarse eye caps cut into the
curved body. Only eye-mask/lens cap subdivision was increased. An independent
stored-triangle ray check now samples front-cap interiors against the actual
parent surface and requires positive clearance. A freshly rebuilt second
preview shows continuous near-eye rings and lenses. It was inspected against
paired 2D at the same authoritative state/time: preserved state, checksum
3627163496, LCD frame 53, virtual time 1.27802025. Round silhouette, clock brows,
folded wings, beak and the planted foot remain readable at the pack scale.
Detailed source-cue motion, browser rendering and additional view angles remain
open; this is not final all-species visual acceptance.

All eight stored anatomy/skin checks and two canonical routing/catalogue checks
pass. The pure authoring CLI was executed and matches canonical bytes. The
Blender batch branch was syntax checked but not fully run. The final fresh Rust
suites pass 35 active rig tests (one existing audit ignored), 31 skinning tests
and 13 actor tests. Verification used local unoptimized crate builds to shorten
compilation, through the same production controller and rendering paths.

```sh
python3 tools/hoothoot_glb.py
PYTHONPATH=tools python3 -m unittest test_hoothoot_glb -v
PYTHONPATH=tools python3 -m unittest \
  test_battle_model_assets.CanonicalSources.test_hoothoot_resolves_one_new_canonical_skin \
  test_battle_model_assets.CanonicalSources.test_starters_resolve_production_actor_rigs -v
CARGO_TARGET_DIR=target/pokemon-models cargo test --locked -p crystal-voxel-view \
  species_rig::tests --lib -j2 \
  --config profile.dev.package.crystal-voxel-view.opt-level=0 \
  --config profile.dev.package.crystal-bevy.opt-level=0
CARGO_TARGET_DIR=target/pokemon-models cargo build --locked -p crystal-bevy \
  --features location-tester --example immersive_battle_3d -j2 \
  --config profile.dev.package.crystal-voxel-view.opt-level=0 \
  --config profile.dev.package.crystal-bevy.opt-level=0
CRYSTAL_PREVIEW_MUTE=1 CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image \
./target/pokemon-models/debug/examples/immersive_battle_3d \
content-packs/core-modular.browser.crystalpack --wild-species HOOTHOOT --size 1000x700 \
--screenshot target/pokemon-native-review/hoothoot-battle-3d.png \
--second target/pokemon-native-review/hoothoot-battle-2d.png
```

Registered skins now number 32 and dedicated battle bodies 241. Ten catalogue
body gaps and the other species' model/rig/visual requirements remain in the
full all-Pokemon goal.
