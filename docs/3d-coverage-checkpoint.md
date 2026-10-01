# 3D conversion checkpoint

This is a working conversion, not a claim that every original scene is finished.
The authored models are original procedural sculptures with editable Blender
sources. All 251 normal-palette species have a first-pass model; their silhouettes,
faces, proportions and animation still need individual art direction and gameplay
review. The private inspection gallery is a review tool, not a quality certification.

## Included and checked

- 629 runtime model documents and 62 complete editable Blender sources
- 75 articulated human rigs, 73 creature/prop actor assets, 69 interior models,
  63 exterior models, 19 dungeon models, 11 special-environment models,
  23 special-room models, 17 joined-counter modules, three lighthouse shells,
  four Cable Club room prototypes, twenty Gym scenery models, eight department-store
  models, four outdoor-sign models, three lighthouse furniture models, and
  220 additional exact battle species
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
mesher error and counts 254,371 source cells consumed by authored geometry,
including 9,176 Kanto boundary-rock, 1,904 capped-post, 1,630 joined-counter/phone,
4,816 lighthouse masonry/window cells, 968 Cable Club partition cells and 88
Time Capsule cells. This increment adds 1,253 Gym scenery, 632 store, 348 sign,
36 keeper-room and 88 missing-ground cells, plus a net 462 Cable rear-fixture
cells. The 374 Cable ceramic-floor, 108 lighthouse checker and 2,566 Gym floor
cells receive no object credit.
Cable Club replaces 48 earlier generic partition cells, for a net gain of 920.
This is partial coverage, not 388 completed maps. Flat floors, water planes, source facades and fallback objects
are reported separately. Dynamic decorations and changed block states require
their own scene fixtures.

## Still open

The 23-model special-room kit now reaches the 17 previously uncovered maps,
including Rocket Base B2F. It includes explicitly labeled surface finishes;
their cells are not a count of complete furniture or architecture. Representative native roof, Rocket Base and three link-room reviews pass. The source-family audit initially identified 29 entries:
two existing-model backing gaps are fixed, two describe legitimate animated or
water-datum treatments, and 25 actionable geometry/refinement families were tracked. Three Kanto rock
families now pass geometry/source-selection tests and native Route17, Route20
and SilverCaveOutside review. Capped posts and gate counters/phones now pass
native review. Cable Club partitions are installed across eleven upstairs maps.
Lighthouse masonry now has a camera-aware player reveal and source-scoped slate
floors, with native 4F and 6F checks. Seven more source families now have exact installed kits and reviewed native
views: the four Gym groups, outdoor signs, store walls and store display islands.
Ten entries from the earlier checklist and additional source-specific architecture
packages remain; the earlier checklist was not exhaustive.
See [the remaining-family checklist](3d-remaining-families.md).

- Remaining source families and structures, including facility/radio furniture,
  cabin fittings, traditional screens/stage, station trains, and park props
- Cable Club live door/warp artwork, mobile-entry source bands and two negative
  corner cells per room remain intentional surfaces; original open doorway
  frames and console assemblies now preserve their distinct roles
- Lighthouse 6F furniture and its central checker floor are modeled/finished, with
  a reviewed native chamber. Additional movement and camera arrangements remain
  in the per-map review queue
- Gym flowerbed surfaces and small source strips beneath older statues remain
  a material/detail review queue; quieter floors do not complete those rooms
- Dungeon topology and bespoke set dressing beyond the nineteen-model first kit
- Per-map native review, including eliminating the remaining source-art strips in
  interiors and assessing outdoor silhouette/readability at the gameplay camera
- Species art refinement; shiny, Substitute, Minimize and other special
  presentations currently use faithful source art, not the normal-palette mesh
- Move effects now follow the existing source interpreter's live objects,
  palettes and timing. Unverified companion volumes are suppressed; the artistic
  battlefield remains. Extracted-row/reveal phases and individual native review
  of every move remain open
- The native-only Tackle/Water Gun row prototype and its separately gated capture
  cache remain disabled by default; their integration does not close the general
  extracted-row gap or establish a performance improvement
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

- 755 voxel tests pass (two existing benchmarks ignored), including exact source
  guards, custom/cropped fallback, native grounding, open-door geometry, face
  lighting and exterior player-reveal eligibility.
  The native build succeeds. Prior 22 render-API and 19 focused source-row/bridge
  tests remain applicable to unchanged attack logic; they were not all rerun
- All 20 CI model/source validator programs pass. All 62 editable sources decode
  in memory with verified chunk, compressed-file and expanded-file hashes, Blender
  headers and 552 verified chunks with no orphans; this does not establish that every scene was
  opened or visually approved in Blender
- The complete Bevy test target type-checks and the normal browser-feature
  WebAssembly check passes. The full monolithic Bevy test suite was not rerun;
  its prior relink exceeded this preview machine's memory limits
- Retained from the prior sound checkpoint: 33 immersive battle bridge/controller
  tests, four channel-mask tests, 24 audio
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
- Refined Cyndaquil and both Sudowoodo identities pass closed-volume and byte-exact
  rebuild checks and appear in the actual arena. These replace earlier meshes
- Native Route2, Route35/National Park, Route29/46 and Ilex gates exposed a
  padded-frame source-boundary bug. The corrected source-origin mapping now has
  explicit base/padded/scrolled, crop and counterfeit-layout regression fixtures
- Prior captures cover all six lighthouse floors and Route4/Route13/Lavender
  posts. Current native 4F review verifies the player cutaway and slate; 6F review
  verifies slate around the retained central checker/furniture zone
- Current native Cable Club and Celadon beta-room captures verify the joined
  partitions, capsule, new consoles, PCs, sign frames, open doorway frames and
  coherent floor strips. Door art, mobile entrances and source-negative corners
  remain intact
- The exact lighthouse Amphy map object/script now publishes Ampharos identity;
  other shared monster sprites retain their current model. Native 6F review
  verifies the yellow Ampharos beside Jasmine, with original footing and facing
- Battle bodies now use pack-supplied physical dimensions with uniform scaling
  and one shared camera fit. Native Cyndaquil/Sudowoodo and Onix/Diglett encounters
  verify representative ordinary and extreme size differences; this does not
  approve every species or attack composition
- Kanto shallow ledges use coherent lawn/path caps and bank sides while retaining
  identical vertices, normals and footing; deeper cliffs and edge strips remain

- Eight final native Gym views cover the four rooms from both camera directions.
  Rounded shrubs now receive fitted-normal face lighting, and three rooms have
  quiet continuous floor materials; Viridian's native stone treatment is retained
- Ten store views cover both store families and the closed 5F display/staff inset;
  ten sign views cover Kanto, modern Johto, park and forest frame/backing variants
- Native Azalea default/side/reverse views verify the player remains visible
  through foreground building fragments while backing, props and rear buildings
  stay opaque. The reveal reuses the existing player-only camera capsule

- Six native controller routes cross the actual four Gym exits, beta Cable
  stairs and a department-store stair. Movement traces record both source and
  destination maps; collision and warp logic are not replaced by a preview

These are representative checks, not every-map or every-species art approval.

The optimized Gym kit still has a measured software-renderer cost versus the
prior source-art baseline: Viridian 55.45→68.09 ms and Goldenrod 85.73→94.06 ms
median frame time. This remains a priority optimization gap; see the detailed
[comparison](johto-3d-performance.md).

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
