# 3D conversion checkpoint

This is a working conversion, not a claim that every original scene is finished.
The authored models are original procedural sculptures with editable Blender
sources. All 251 normal-palette species have a first-pass model; their silhouettes,
faces, proportions and animation still need individual art direction and gameplay
review. The private inspection gallery is a review tool, not a quality certification.

## Included and checked

- 514 runtime model documents and 44 complete editable Blender sources
- 75 articulated human rigs, 73 creature/prop actor assets, 68 interior models,
  42 exterior models, 19 dungeon models, and 220 additional exact battle species
  alongside retained source models
- All 140 packed sprite/icon source identities resolve to a modeled actor
- All 251 normal-palette battle species resolve to an exact species mesh;
  generic icon meshes are not counted as species models
- Source-aware, complete-object placement preserves original spacing, authoritative
  collision, warps, NPC source identity, facing and sampled ground height
- Plank, tile, tatami, wallpaper and cutaway interior finishes plus source-scoped
  outdoor/cave ground treatments; a surface finish is not counted as a prop model
- Six battle arena families, visible-event effects, contact shadows, compact
  source HUD placement, party/pack modals and reversible classic/3D presentation
- Shared human geometry reduces the human catalog from 33,650,186 to 6,444,167
  bytes without changing reconstructed f32 positions, normals, indices, colors,
  joints or runtime meshes

The pack-based headless audit processed all 388 base maps without a mesher error.
345 maps consume at least one authored model, totaling 208,892 source cells. This
means partial coverage, not 345 completed maps. Flat floors, water planes, source
facades and fallback objects are reported separately. Dynamic decorations and
changed block states require their own scene fixtures.

## Still open

- Special rooms and structures: elite rooms, shrines, ruins chambers, ship/port
  details, link rooms, department-store roof/elevators, special gyms, tower roof,
  underground passage and selected remaining building facades
- Dungeon topology and bespoke set dressing beyond the nineteen-model first kit
- Per-map native review, including eliminating the remaining source-art strips in
  interiors and assessing outdoor silhouette/readability at the gameplay camera
- Species art refinement; shiny, Substitute, Minimize and other special
  presentations currently use faithful source art, not the normal-palette mesh
- Battle choreography is elemental presentation classes with a bounded particle
  pool, not custom skeletal choreography for every move or species
- Native battle interruption/transition matrix: switching, fainting, capture,
  Pokédex and returning to overworld require continued end-to-end checks

## Reproduce the coverage checklist

Use an external, ignored content pack. The audit output remains a local verification
artifact, not a shipped copy of game content:

```sh
cargo run --locked -p crystal-bevy --features location-tester \
  --example audit_authored_assets -- \
  content-packs/core-modular.browser.crystalpack \
  target/authored-model-coverage.json
```

Each map entry lists authored model families, actual consumed source cells,
remaining source classifications and mesh/footing validation. It is deliberately
based on the production mesher rather than a map-name allowlist.

## Verified on this checkpoint

- 551 voxel tests passed, two pre-existing benchmarks ignored
- Fourteen immersive-battle presentation/recording bridge tests passed
- Real production controller battle regression passed: commands, selected move,
  authoritative PP mutation and retained battle dialogue
- Six connected-world regressions passed
- Production host-frame motion regression passed at 60, 30 and 9 Hz without
  inventing interpolation progress or changing the authoritative movement rate
- Character storage equivalence tests and all model/source validators passed
- Native examples built and representative exterior/interior/cave/ice scenes were
  inspected; this is representative review, not every-map visual approval
- Native FIGHT → TACKLE produced the move dialogue, visible damage and effectiveness
  text. F3 restored source battlers/HUD/text and returned to modeled presentation.
  Party/cancel and Pack/cancel restored the original battle state and compact HUD
- A real 12-second Tackle capture recorded 232 native frames with its original
  timestamps, beginning at presented move progress 0.0. Median update time during
  capture was 50.75 ms on software GL; this is not a hardware benchmark

The software-rendered native preview is not a hardware performance benchmark.
Recordings keep their actual timestamps; no interpolated frames or high-FPS claim
is used. The renderer's `--measure` mode excludes PNG/GPU readback, while
`--record` measures the capture workload separately.
