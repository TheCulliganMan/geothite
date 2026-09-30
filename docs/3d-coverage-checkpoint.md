# 3D conversion checkpoint

This is a working conversion, not a claim that every original scene is finished.
The authored models are original procedural sculptures with editable Blender
sources. All 251 normal-palette species have a first-pass model; their silhouettes,
faces, proportions and animation still need individual art direction and gameplay
review. The private inspection gallery is a review tool, not a quality certification.

The original 2D reference now governs all further changes. Current appearance
is not certified faithful; see the [fidelity matrix](original-2d-fidelity.md).

## Included and checked

- 542 runtime model documents and 48 complete editable Blender sources
- 75 articulated human rigs, 73 creature/prop actor assets, 68 interior models,
  59 exterior models, 19 dungeon models, 11 special-environment models, and 220 additional exact battle species
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
371 maps consume at least one authored model, totaling 226,454 source cells. This
means partial coverage, not 371 completed maps. Flat floors, water planes, source
facades and fallback objects are reported separately. Dynamic decorations and
changed block states require their own scene fixtures.

## Still open

Seventeen base maps still have no modeled static scenery: Battle Tower battle/
elevator rooms, Blackthorn Gym 2F, both department-store elevators, Goldenrod's
store roof, the Celadon prize room and Mansion roof, both mobile-link rooms,
Colosseum/Time Capsule/Trade Center, Dragon Shrine, Fighting Dojo, the Bike Shop
and Rocket Base B2F. Those kits are under active authoring. The other 371 maps
also need remaining-family and visual checks; a positive count is not completion.


- Special rooms and structures: elite rooms, shrines, ruins chambers, ship/port
  details, link rooms, department-store roof/elevators, special gyms, tower roof,
  underground passage and selected remaining building facades
- Dungeon topology and bespoke set dressing beyond the nineteen-model first kit
- Per-map native review, including eliminating the remaining source-art strips in
  interiors and assessing outdoor silhouette/readability at the gameplay camera
- Species art refinement; shiny, Substitute, Minimize and other special
  presentations currently use faithful source art, not the normal-palette mesh
- Move effects now follow the existing source interpreter's live objects,
  palettes and timing, with additional 3D styling for Psychic and Hyper Beam.
  Source-matched pose/effect corrections and individual review of every move remain open
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

The battle performance and refined-sculpture increment is documented in
[battle performance checks](battle-3d-performance.md). The source-animation
checkpoint verification below remains the baseline for the ongoing conversion.

- 569 voxel tests passed, two pre-existing benchmarks ignored
- Twenty-one immersive-battle presentation/recording bridge tests and twenty-one
  render API tests passed; the independent BGP/OBP palette cadence regression passed
- Native build and the normal browser-feature Wasm check passed after the source
  animation compiler was moved verbatim into its own file
- Real production controller battle regression passed: commands, selected move,
  authoritative PP mutation and retained battle dialogue
- Six connected-world regressions passed
- Production host-frame motion regression passed at 60, 30 and 9 Hz without
  inventing interpolation progress or changing the authoritative movement rate
- Character storage equivalence tests and all model/source validators passed
- Native examples built and representative exterior/interior/cave/ice scenes were
  inspected; twelve further native scenes verified the new structural and special
  environment families. Remaining pixel floors/wall edges were identified for
  refinement; this is representative review, not every-map visual approval
- Native FIGHT → TACKLE produced the move dialogue, visible damage and effectiveness
  text. F3 restored source battlers/HUD/text and returned to modeled presentation.
  Party/cancel and Pack/cancel restored the original battle state and compact HUD
- A real 12-second Tackle capture recorded 232 native frames with its original
  timestamps, beginning at presented move progress 0.0. Median update time during
  capture was 50.75 ms on software GL; this is not a hardware benchmark
- Actual Psychic and Hyper Beam captures begin at presented move progress zero
  and retain their frame timestamps. Source waves, deformation, palette changes,
  beam segments and visible damage were inspected. The legal Hyper Beam fixture
  also passes the real controller's recharge/PP regression. Native Shadow Ball
  confirms the original single inversion and moving projectile; reduced-mode
  timing/contrast/pose tests pass, with native comparison still in progress

The software-rendered native preview is not a hardware performance benchmark.
Recordings keep their actual timestamps; no interpolated frames or high-FPS claim
is used. The renderer's `--measure` mode excludes PNG/GPU readback, while
`--record` measures the capture workload separately.
