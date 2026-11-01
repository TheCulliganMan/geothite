# Onix connected rock-chain skin

Onix now has one canonical battle/world body at
`crates/crystal-voxel-view/models/battle_species/onix.glb`.
The existing original `tools/onix_sculpt.py` geometry is retained: thirteen
irregular closed boulders, twelve buried connectors, a wedge face, fitted eyes,
rooted nose/jaw/mouth chamber and a long horn. The old `big_onix` JSON is removed;
the world alias and exact battle species share this GLB. Both builder branches
publish the same canonical source without a duplicate actor export.

The verified local pack reports height digits 2810: body length 8.7884 m
(28′10″). The renderer's existing length rule uses the authored tail-to-face
centerline, 2.867740619 units, rather than treating this as standing height.
The GLB records its physical datums; the stored-data check compares the movable
centers to inverse binds and verifies every datum lies within its rock or head.
The two fixed tail datums also match the stored boulders' vertical midpoints.
An initial check incorrectly equated an irregular rock's sampled-vertex centroid
with the authored center. That assumption was replaced with stored bind-center
and actual-volume checks, rather than widening the centroid tolerance.

Normal/front/back palettes agree. Base stone RGB555 `(17,15,20)` becomes RGB8
`(140,123,165)`; dark joints and jaw `(9,6,11)` become `(74,49,90)`.
Eye whites and dark details use source white/black. The horn and nose keep an
authored lighter accent of the same hue. The first native review exposed
unwanted alternating dark boulders. Every boulder now uses the base stone color;
face normals and lighting provide its facets. Dark buried joints/jaw remain.

Before the old actor JSON was removed, an executed comparison proved every
neutral f32 position, normal and triangle index/order unchanged. The subsequent
uniform-stone correction also preserved the complete GLB binary geometry,
weights, inverse binds, clip data and accessor schema. Final asset metadata:
36 named parts, 12,083 vertices, 9,848 triangles, twelve joints, 170 accessors,
458,756 bytes; neutral silhouette height 1.72. Final digest:
`c36c6f0a7b6d955e1f5937d4862220768229c6ac7e661abc7b00fd354622fa10`.
Existing reader limits remain unchanged.

The identity floor root owns the first three support rocks. Ten hierarchical
bones articulate the rising boulders; connectors blend the adjacent owners
along their actual centerline. The separate head bone owns the complete fitted
face, nose, jaw, mouth chamber and horn. Idle lasts four seconds with 85 keys.
Attack/hit last one second with 65 keys, an early chain brace and a trailing
head response. Rotation-only clips return to neutral and use the existing
source battle cue clock.

Eight final stored-skin checks pass, covering topology/normals, ownership,
source materials, scale datums, supports/floor clearance, winding/strain,
attachment overlap through motion, articulation and endpoints. Eleven source
anatomy checks and two source-routing checks pass. Before the final color-only
correction, fresh Rust suites passed 39 active rig checks with one existing
ignored check, 35 GPU-skinning checks and five actor-prop checks. Afterward,
both focused Onix registry/GPU-size regressions pass. The pure CLI reproduces
canonical bytes and emits no duplicate JSON. Builder branches were syntax
checked; full Blender batches were not executed.

The rebuilt final production paired view at 1000×700 preserves state, checksum
3502953395, LCD animation frame 36, virtual time 0.894852042. The consistent
purple-gray chain, wedge face and horn are readable. Its large size follows
the verified body-length rule beside the default wild Sudowoodo fixture.
The first striped view was inspected, corrected and captured again before this
held-view acceptance. Additional camera angles, detailed source-frame motion,
world-specific motion and browser fidelity remain open. Temporary source
sprites and the Rust reference helper were removed after review.

```sh
python3 tools/onix_glb.py
PYTHONPATH=tools python3 -m unittest test_onix_glb -v
PYTHONPATH=tools python3 -m unittest test_onix_sculpt \
  test_battle_model_assets.CanonicalSources.test_onix_resolves_one_new_canonical_skin \
  test_battle_model_assets.CanonicalSources.test_starters_resolve_production_actor_rigs -v
CARGO_TARGET_DIR=target/pokemon-models cargo test --locked -p crystal-voxel-view \
  species_rig::tests --lib -j2 \
  --config profile.dev.package.crystal-voxel-view.opt-level=0 \
  --config profile.dev.package.crystal-bevy.opt-level=0
```

There are now 36 registered standard skins and 245 dedicated battle bodies.
Six catalogue gaps and the remaining static-body rigging, motion and visual
requirements keep the full all-Pokemon goal unfinished. No pack/audio changes
were made for Onix.
