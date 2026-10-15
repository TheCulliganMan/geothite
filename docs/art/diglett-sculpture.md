# Diglett capsule and fixed-soil skin

Diglett fills a battle-body gap with one canonical GLB at
`crates/crystal-voxel-view/models/battle_species/diglett.glb`.
`tools/diglett_sculpt.py` replaces the old small ellipsoid icon proxy with a
continuous capsule, rounded elliptical dome, fitted black eyes and small white
glints, an overlapping oval nose and eight separate soil clods. The world
`icon_diglett` alias shares the same body; its old JSON is removed. Actor and
battle builder branches publish the canonical GLB without a second icon mesh.
Archived Blender scenes remain unchanged.

The verified local pack supplies height digits 8: 0.2032 m (8 inches).
Normal/front/back palettes agree. Body RGB555 `(19,11,4)` becomes RGB8
`(156,90,33)`; nose and soil `(24,6,4)` become `(198,49,33)`. Eyes/glints use
source black/white. Temporary source sprites and the Rust reader helper were
removed after review.

The first front geometry review showed an overly pointed cap. An explicit
elliptical dome corrected it; the stored-shape test checks its vertices against
the ellipse instead of retaining a narrow-tip expectation. A subsequent native
view showed a smooth regular soil ring resembling petals. The clods now have
varied footprints and flat facets, while body geometry, palette and scale are
unchanged by that revision. Both defects were corrected and inspected again.

The final GLB contains 15 named parts, 10,646 normal-split vertices, 14,064
triangles, two joints, 65 accessors and 389,680 bytes. Full authoring height is
0.82; the measured capsule span is 0.8126839, excluding soil. Final digest:
`f81fd70948ece5d6ade091c74c000f7a9b03ee6aab22981524543f4a07fc38c4`.
Existing runtime reader limits remain unchanged.

The identity root owns all soil and the buried body base. A coherent body bone
owns the visible capsule, face, nose and cap glint, with a short lower-body
weight transition. Idle surveys over 3.4 seconds with 85 keys. Attack/hit last
one second with 65 keys, an early lean and trailing turn. Rotation-only clips
return to neutral and use the existing source battle cue clock. Soil never
follows the body motion.

Both the physical-scale rule and the source-projection height datum use the
capsule span. A new Rust check measures actual stored capsule vertices and
verifies both paths, as well as the resulting 0.2032 m world scale. This avoids
measuring the disturbed soil as part of the animal.

Nine final stored-model checks pass (the eight core checks plus the final
soil-profile/facet check). They cover topology/normals, capsule shape, reference
span, materials, fitted-feature clearance against stored parent triangles,
soil ownership, varied clod footprints/facets, floor clearance, winding/strain,
articulation and endpoints. Two canonical-routing checks pass. The pure CLI
reproduces canonical bytes. Builder branches were syntax checked; full Blender
batches were not executed.

Before the cap/clod corrections, fresh Rust suites passed 40 active rig checks
with one existing ignored check, 36 GPU-skinning checks, five actor-prop checks
and 21 layout checks. After the final correction, all three focused Diglett
registry/GPU-scale/capsule-projection checks pass. Temporary test output and
build products stay ignored.

The rebuilt repeated native pair at 1000×700 preserves state, checksum
3361697257, LCD animation frame 55, virtual time 1.391161. The rounded capsule,
nose, near eye and less regular clods remain visible at the pack's small size
beside the default wild Sudowoodo. This verifies the held native view; detailed
source-frame motion, additional angles, world-specific motion and browser
fidelity remain open.

```sh
python3 tools/diglett_glb.py
PYTHONPATH=tools python3 -m unittest test_diglett_glb -v
PYTHONPATH=tools python3 -m unittest \
  test_battle_model_assets.CanonicalSources.test_diglett_resolves_one_new_canonical_skin \
  test_battle_model_assets.CanonicalSources.test_starters_resolve_production_actor_rigs -v
CARGO_TARGET_DIR=target/pokemon-models cargo test --locked -p crystal-voxel-view \
  species_rig::tests --lib -j2 \
  --config profile.dev.package.crystal-voxel-view.opt-level=0 \
  --config profile.dev.package.crystal-bevy.opt-level=0
```

There are now 37 standard skins and 246 dedicated battle bodies. Five catalogue
gaps and the remaining static-body rigging, motion and visual requirements keep
the full all-Pokemon goal unfinished. Packs and audio were not changed.
