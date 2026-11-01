# Voltorb coherent orb skin

Voltorb now has one canonical body at
`crates/crystal-voxel-view/models/battle_species/voltorb.glb`, shared by the
battle renderer and world `icon_voltorb` route. Its old icon JSON is removed.
`tools/orb_sculpt.py` retains the original two closed hemispheres and fitted
stern eyes. `tools/voltorb_glb.py` adds the standard skin and source-cue clips.
Both builder paths publish this GLB without a second neutral icon export.

The verified local pack supplies height digits 108, or 0.508 m (1′8″). Normal,
front and back palettes agree: red RGB555 `(24,5,5)` becomes RGB8
`(198,41,41)`. The white lower shell and eyes use source white; borders and
pupils use source black. Voltorb's old pinkish red and tinted white/dark accents
were corrected. Before removal of the icon JSON, an executed comparison proved
all f32 positions, normals and triangle indices/order unchanged. Electrode's
stored file remains SHA256
`dda591e8e0560f22bda64e535bf4ae993bf41345013a05c40c2b5517a7f63d70`,
and its recipe still matches its stored asset exactly.

The GLB contains eight named parts, 4,660 vertices, 8,248 triangles, two joints,
35 accessors and 197,948 bytes. Authored reference height remains 0.82; the pack
owns physical scale. Neutral digest:
`560b40b398d50fcbeb2944890bd8e60733c1182c83c20b824e84b8a358f4e605`.
No runtime reader limit is widened.

The identity floor root owns the support cap. The coherent body bone owns the
upper shell, equator and complete fitted face. Lower-shell weights smoothly
join the body above the pinned band. Red and white equator vertices retain
identical owners and weights, including their posed positions. Idle rocks over
3.4 seconds with 85 keys. Attack/hit last one second with 65 keys, an early
body brace and a trailing turn. Rotation-only keys and identity endpoints use
the existing source battle cue clock.

Six independent stored-skin checks pass, covering topology, shading, materials,
face ownership, the welded equator through motion, support contacts, floor
clearance, winding/strain, articulation and endpoints. Six existing orb anatomy
checks plus two source/catalogue checks pass. Fresh Rust suites pass: 38 active
rig checks with one existing ignored check, 34 GPU-skinning checks and five
actor-prop checks. The pure orb CLI reproduced canonical bytes into a temporary
directory and emitted no duplicate Voltorb JSON. Builder branches were syntax
checked; full Blender batches were not run.

A rebuilt production paired Voltorb/default-wild battle at 1000×700 preserves
state: checksum 396955799, LCD animation frame 37, virtual time 0.921693208.
The deep red upper shell, white lower shell and stern near eye are readable at
the pack's physical size. The equator is continuous and the fitted eye clears
the shell. The comparison confirms this held native view; additional angles,
source-frame animation correspondence and browser visual acceptance remain
open. Temporary source sprites and the Rust reference helper were removed.

```sh
python3 tools/voltorb_glb.py
PYTHONPATH=tools python3 -m unittest test_voltorb_glb -v
PYTHONPATH=tools python3 -m unittest test_orb_sculpt \
  test_battle_model_assets.CanonicalSources.test_voltorb_resolves_one_new_canonical_skin \
  test_battle_model_assets.CanonicalSources.test_starters_resolve_production_actor_rigs -v
CARGO_TARGET_DIR=target/pokemon-models cargo test --locked -p crystal-voxel-view \
  species_rig::tests --lib -j2 \
  --config profile.dev.package.crystal-voxel-view.opt-level=0 \
  --config profile.dev.package.crystal-bevy.opt-level=0
```

There are now 35 registered standard skins and 244 dedicated battle bodies.
Seven battle-catalogue gaps and the remaining static-body rigging, motion and
visual requirements keep the all-Pokemon goal unfinished. Packs and audio were
not changed.
