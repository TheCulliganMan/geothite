# 3D conversion checkpoint

This is a working conversion, not a claim that every original scene is finished.
The authored models are original procedural sculptures with editable Blender
sources. All 251 normal-palette species have a first-pass model; their silhouettes,
faces, proportions and animation still need individual art direction and gameplay
review. The private inspection gallery is a review tool, not a quality certification.

## Included and checked

- 652 runtime model documents, one shared human-geometry library and
  69 complete editable Blender sources
- The preceding 19 model documents cover six Facility/Radio, six ship-room,
  three traditional-room, one stationary train and three park prototypes.
  They add six editable Blender scenes; existing shared stools, beds and
  lighthouse tea furniture are reused without inflating the asset count
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

The latest production audit processes all 388 base maps without mesher errors
and counts 256,191 source cells consumed by authored geometry. The preceding
institutional checkpoint had 255,919; this increment adds 172 facility table,
88 chair and 12 Rocket plant cells. The 2,424-cell ship floor treatment is a
material finish and adds no object-coverage credit. This base-map audit now has
zero unmodeled cutout cells; other native-art classifications, dynamic states
and individual scene review remain open.

Four new original furniture meshes and one editable source bring the catalog
to 652 model documents plus one shared library, 69 Blender scenes and 571 chunks.
The two Rocket plants reuse the existing radial-frond model/cache. This is
partial scene coverage, not 388 completed maps.

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
The latest five kits address the remaining ten entries from that selected
checklist, with native water/door/floor roles explicitly retained. The selected
list was not exhaustive: additional source-specific furniture and architecture
packages remain open.
See [the remaining-family checklist](3d-remaining-families.md).

- Remaining source furniture and architecture, including 192 Power Plant
  equipment/track plot cells, 120 Fast Ship B1F front-wall/cap cells,
  café/game-corner fittings, divider networks, sofas, wall edges and puzzle fixtures
- Latest kit volumes, corrected stool/block36 bed bindings and close park
  placements have representative static native review. The train controller
  route is still being verified; static views do not approve whole rooms,
  idle poses or every approach
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

- All 797 voxel tests pass, with two existing benchmarks ignored, including
  source guards, custom/cropped fallback, native grounding, open-door geometry,
  face lighting, player reveal and the corrected stool/bed bindings
- All 26 Python model/source CI commands pass. All 69 editable source archives
  reconstruct successfully from 571 chunks, with verified chunk, compressed and
  decoded SHA-256 hashes (35,178,139 compressed and 324,828,620 decoded bytes).
  Source reconstruction does not certify every scene was opened or approved
  in Blender
- All five external-pack source scans pass the production matcher assertions.
  The park scanner used a lower-optimization temporary Rust harness after the
  optimized harness was killed under shared memory pressure; source and
  assertions were unchanged. A separate actual-pack production ship-selector
  scan verifies all 65 furniture/bulkhead assemblies and 850 owned cells.
  The player's two exact sleep/heal background reads are deliberately retained
  on its joined berth; all other background overlaps and all warp/coordinate
  overlaps are rejected
- The final optimized native examples build, the complete Bevy test target
  type-checks, and the normal browser-feature WebAssembly check passes.
  All 22 render-API regressions pass. The full monolithic Bevy test suite was
  not rerun; its prior relink exceeded this preview machine's memory limits
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

- Forty-eight native captures screen Facility/Radio, all four Fast Ship room
  maps, traditional rooms, both stations and the three park maps. High-risk
  captain/chair, bulkhead, stage, counter and reverse silhouettes were also
  inspected at native size. No additional blocking defect was observed in
  the five new kits; those captures predate the stool/bed correction
- The new stage retains the dancers' 8px datum, Wise Trio/barn passages remain
  visibly open, and the train shell keeps its live door recesses. Static images
  cannot establish idle/spin clearance, moving-camera reveal or executed travel
- Twenty-eight additional native captures inspect the corrected stools and
  block36 beds, all ship cabin variants and close park fixtures from opposite
  sides. No new clipping, actor intersection or bad joins were observed in the
  implemented correction. Eight subsequent native views verify all four joined
  block38/39 berths from opposite sides, including closed foot ends and clean
  wall/trashcan joins. These static checks do not execute the bed's sleep/heal script
- Four runtime pack-mount tests and three real-pack file integration tests pass.
  An ordinary native screenshot session also observes its completed extracted
  pack mount while alive and confirms no mount remains after normal AppExit.
  The real battle controller regression verifies menus, an executed move and its
  PP change. The New Bark controller test skipped because its required TUI pack
  was absent; that skip is not a pass
- The train controller example is diagnostic. Its mesh, power/PASS-denial and
  door-approach checks pass; full travel remains an open gameplay blocker.
  The published controller does not finish the retained train animation clock.
  A separate experimental repair exposed map-load ordering and dropped
  post-load source commands, including a 120-frame arrival hold. Those partial
  engine fixes are excluded from this scenery checkpoint; a successful trip
  without the complete authored tail is not accepted as a faithful repair

- Thirty-one further native views inspect the four facility furniture forms,
  occupied chairs, live book faces and finished ship floors from opposite and
  side angles. Source guards cover five maps and two scripted Rocket states;
  32,832 sampled actor poses clear the new chairs
- Native front/back/side and classic 2D views verify both Rocket plants, their
  floor and spacing. Ordinary held-Down input advances one tile to the next
  native wall with no extra terrain rebuild. This local approach check does
  not prove unrelated Rocket trainer/sensor sequences

These are representative checks, not every-map or every-species art approval.

A paired reveal-aware follow-up measured Viridian at 69.23→60.62 ms median,
while Goldenrod measured 89.82→95.03 ms on llvmpipe. These single paired samples
compare against the optimized authored kit, not the older source-art baseline.
The earlier source-art regression and Goldenrod cost remain open; this is not
a fluid-performance certification. See [the timings and scope](johto-3d-performance.md).

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
