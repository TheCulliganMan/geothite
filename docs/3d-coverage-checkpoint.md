# 3D conversion checkpoint

This is a working conversion, not a claim that every original scene is finished.
The authored models are original procedural sculptures with editable Blender
sources. All 251 normal-palette species have a first-pass model; their silhouettes,
faces, proportions and animation still need individual art direction and gameplay
review. The private inspection gallery is a review tool, not a quality certification.

## Included and checked

- 569 runtime model documents and 52 complete editable Blender sources
- 75 articulated human rigs, 73 creature/prop actor assets, 69 interior models,
  62 exterior models, 19 dungeon models, 11 special-environment models,
  23 special-room models, and 220 additional exact battle species
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

The current pack-based headless audit processes all 388 base maps without a
mesher error and counts 243,314 source cells consumed by authored geometry,
including 9,136 Kanto boundary-rock cells. This is partial coverage, not 388
completed maps. Flat floors, water planes, source facades and fallback objects
are reported separately. Dynamic decorations and changed block states require
their own scene fixtures.

## Still open

The 23-model special-room kit now reaches the 17 previously uncovered maps,
including Rocket Base B2F. It includes explicitly labeled surface finishes;
their cells are not a count of complete furniture or architecture. Representative native roof, Rocket Base and three link-room reviews pass. The source-family audit initially identified 29 entries:
two existing-model backing gaps are fixed, two describe legitimate animated or
water-datum treatments, and 25 actionable geometry/refinement families were tracked. Three Kanto rock
families now pass geometry/source-selection tests and native Route17, Route20
and SilverCaveOutside review; 22 tracked families remain.
See [the remaining-family checklist](3d-remaining-families.md).

- Remaining source families and structures, including Kanto posts,
  lighthouse masonry, gate counters, link partitions, gym greenery and apparatus
- Dungeon topology and bespoke set dressing beyond the nineteen-model first kit
- Per-map native review, including eliminating the remaining source-art strips in
  interiors and assessing outdoor silhouette/readability at the gameplay camera
- Species art refinement; shiny, Substitute, Minimize and other special
  presentations currently use faithful source art, not the normal-palette mesh
- Move effects now follow the existing source interpreter's live objects,
  palettes and timing. Unverified companion volumes are suppressed; the artistic
  battlefield remains. Extracted-row/reveal phases and individual native review
  of every move remain open
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

- 628 voxel tests pass (two existing benchmarks ignored), all 12 model/source
  validator programs pass, and native examples plus the normal browser-feature
  WebAssembly check build successfully
- 33 immersive battle bridge/controller tests, four channel-mask tests, 24 audio
  synthesis tests and 12 audio/UI regressions pass; all use the current sound path
- Three wrapped-attack identity/OAM/render regressions pass with the external pack
- The Blackthorn Gym callback regression validates every installed scoped stone
  table reference from the real pack; synthetic tests reject forged targets
- Native roof, Rocket Base, link rooms, Kanto boundaries, bedroom and Blackthorn
  Gym captures verify representative geometry and source identity. Unfinished
  source terrain strips and room detailing remain visible and are tracked
- Prior native Surf/Hyper Beam captures and frame-pacing results remain in the
  attack-fidelity document. They predate this checkpoint's Surf sound-tail wait
- The rebuilt Totodile is verified in the actual immersive arena with its
  connected crocodile silhouette, angular eyes, splayed feet and dorsal plates;
  native review also caught and corrected water-material seams around shore rocks

These are representative checks, not every-map or every-species art approval.

The software-rendered native preview is not a hardware performance benchmark.
Recordings keep their actual timestamps; no interpolated frames or high-FPS claim
is used. The renderer's `--measure` mode excludes PNG/GPU readback, while
`--record` measures the capture workload separately.


## Current attack and art direction

The immersive biome arena, perspective camera, compact HUD and artistic models
are retained. Original-source fidelity applies to attack choreography, colors,
flashes, object motion, sound and timing. See [the attack matrix](original-2d-fidelity.md)
for source-art fallbacks and pending wrapper/audio work.

The refined Gengar sculpture is the visual benchmark for further Pokémon
refinement: broad connected volumes, controlled low-poly planes, inset facial
features and clear silhouette. Registry completeness does not establish that
every species has reached that standard.
