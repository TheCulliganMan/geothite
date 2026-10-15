# Slowpoke sleepy quadruped and standard skin

Slowpoke fills a battle-body gap with one canonical GLB at
`crates/crystal-voxel-view/models/battle_species/slowpoke.glb`.
`tools/slowpoke_sculpt.py` replaces the sphere proxy with a long low body,
a broad sleepy head, rounded ears, fitted blank eyes with tiny pupils,
a broad upper muzzle, an open lower jaw, two small canines, four grounded
feet with twelve claws, and a continuous raised pale-tipped tail.
Both `slowpoke` and `icon_slowpoke` world aliases share that body. Their old
JSON assets are removed; both builder branches publish the canonical skin.
Archived Blender scenes remain unchanged.

The verified local pack supplies height digits 311: 1.1938 m (3 feet 11 inches).
Normal/front/back palettes agree: body RGB555 `(31,10,17)` becomes RGB8
`(255,82,140)`, shadow `(21,5,8)` becomes `(173,41,66)`. Muzzle, claws,
canines and tail end use source white; pupils and oral chamber use black.
The pink shadow is restricted to ear insets and the tongue.

The GLB contains 40 named parts, 19,574 normal-split vertices, 32,984 triangles,
11 joints, 170 accessors and 791,604 bytes. Authored height is 1.0. Neutral digest:
`e6b60180c57a253da129803d01cb7856b76599b67a51e6ac3f8d06b4e1e38c64`.
Existing runtime reader limits remain unchanged.

The identity root owns the feet and claws. Upper legs blend into four leg
bones; the torso, coherent head and facial pieces, paired ears, and tail have
separate owners. Tail weights follow the nearest actual centerline chord,
including the upward turn, so the tail end stays coherent. Idle lasts four
seconds with 85 keys; attack/hit last one second with 65 keys and early head
motion followed by tail/ear response. Rotation-only clips return to neutral
and consume the existing source battle cue clock.

Eight stored-model checks pass: canonical bytes, closed outward geometry and
normals, low body/raised tail proportions, fitted-feature clearance against
actual stored parent triangles, pack colors, hierarchy/owners, planted feet,
floor clearance, winding/strain, articulation and neutral endpoints.
The pure CLI reproduces canonical bytes. Builder branches were syntax checked;
full Blender batches were not executed.

All 41 active rig checks pass, with one existing ignored check. All 37
GPU-skinning checks and five actor-prop checks pass. The focused Rust registry
check proves head/tail articulation and planted support across all three clips.
Both canonical source-routing and full catalogue-count checks pass.

The rebuilt native battle pair at 1000×700 (image target at scale 2) preserves
state, checksum 1352458233, LCD animation frame 48, virtual time 1.25993175.
The oblique production camera shows a low quadruped, broad round head, blank
stare, open pale muzzle, short clawed feet and a continuous upturned pale-tipped
tail. The pack pink agrees with the paired 2D back sprite; white surfaces pick
up the scene lighting. The body is fully in frame and grounded beside the wild
Sudowoodo. This verifies this held native view; source-frame motion, additional
angles, world-specific motion and browser fidelity remain open.

```sh
python3 tools/slowpoke_glb.py
PYTHONPATH=tools python3 -m unittest test_slowpoke_glb -v
PYTHONPATH=tools python3 -m unittest \
  test_battle_model_assets.CanonicalSources.test_slowpoke_resolves_one_canonical_skin_for_battle_and_world \
  test_battle_model_assets.CanonicalSources.test_starters_resolve_production_actor_rigs -v
CARGO_TARGET_DIR=target/pokemon-models cargo test --locked -p crystal-voxel-view \
  species_rig::tests --lib -j2 \
  --config profile.dev.package.crystal-voxel-view.opt-level=0 \
  --config profile.dev.package.crystal-bevy.opt-level=0
```

There are now 38 standard skins and 247 dedicated battle bodies. The all-Pokemon
objective remains unfinished: four battle catalogue gaps (Entei, Raikou, Suicune
and Unown), other static rigs,
source-frame motion, further native angles and browser fidelity remain open.
Packs and audio are unchanged.
