# Substantial inland winter proposal

R12 Gunflint is a 64×56-metatile lake/portage region. R13 is a 32×64-metatile
granite ridge and Ely approach. Winter occupies both whole regions, with a
deliberate warm valley taper at R12's western Boundary Waters port. North Shore
R11 and Grand Marais stay warm. The coast enters Grand Marais from its west;
Root and the coast author own that geography and its separate inland exit.

The proposed winter overlay preserves the existing route design: asymmetric
enclosed lakes, portages, Raven Rock shelter, ranger trail, granite ridges,
Wind Tor drop/return circuit, required encounter grass and Cut-sealed rewards.
Native water stays water. Native caves, roofs/doors, trainer identities,
residents, item scripts, flags, encounters and story metadata stay exact.

The private `mn_winter_gunflint` art stage is in `gunflint-alias-v01/`. It copies
all installed cold forest art and inherits original Gunflint behavior. R13 uses
Root's exact narrow paired-pine `8b`, preserving the original Johto silhouette.
No broad forest canopy is substituted for the native narrow pines. No ordinary
snow tree introduces incidental Cut collision. Native Cut recipes remain
unchanged, including their original replacement quadrants.

The explicit terrain proposal contains three bounded ice runs amid broad snow
and icy encounter grass:

| Feature | ICE metatiles | Main entry/ordinary stop, core tiles |
|---|---|---|
| Loons north bank | x47–49, y6 | (93,13) Right → (100,13); reverse lane y12 |
| Bog portage approach | x16, y36–38 | (33,71) Down → (33,78); reverse lane x32 |
| Wind Tor frost run | x23, y4–7 | (46,7) Down → (46,16); reverse lane x47 |

Only ten metatiles / forty original Floor quadrants become ICE. Every other
collision token stays exact. Snow is ordinary Floor; icy habitat stays native
Tall Grass. Ports remain ordinary Floor. Actor bodies and conservative trainer
sight lines are outside all ice cells and intended stops.

There are 34 side/main entry courses to validate with production input. Ten
side entries intentionally stop on ICE against existing pine/granite walls:
four at the bog and six at Wind Tor. Each has explicit reverse/perpendicular
ordinary Floor exits. The six main traversals stop on Floor. Native controller
acceptance must observe mid-slide input ownership, every stop and return/turn,
and Save/Continue at both ordinary and wall stops. A static graph does not
establish those behaviors.

The source checker retains the existing four-step R12 and six-step R13 minimum
grass crossings, native Cut-negative isolation even allowing water adjacency,
the corresponding open-replacement reward paths, and Wind Tor's 19-step
ordinary return. These checks model source collision adjacency; they do not
prove field-move prerequisites, native battle scripts or earned progression.

`prepare_winter.py` reads an explicitly SHA-bound actual pack containing the
completed private alias and pine `8b`. It writes only fresh detached source and
render directories. It retains original authoring strokes and explicitly pins
the already fitted native shore cells before applying the seasonal points.
The candidate uses `explicit_source_edges`; it must never refit a fitted bank.
Each native chunk retains its actual cropped width/stride and actor geometry.
It emits exact module/cell preimages, raw block hashes, canonical `.before`
bytes, layouts/dossiers, whole before/candidate scenes, paired player views,
and static collision/course proof. Its in-memory candidate inverse must equal
the complete actual parent after restoring only the nine owned modules and
their attribute/block mirrors. It cannot write a pack or canonical source.

The final native contract is deliberately not frozen against the current R12
east connection. Its full-module preimages must bind Root's completed revised
coast geography; original terrain preimages must still match exactly. Root
owns the native publisher, shared layout-policy registration, controller/GPU
checks and eventual joint guarded canonical promotion. Existing source and
renders remain immutable history, including rejected candidates.

The actual art parent is now published at
`content-packs/minnesota-superior-inland-art-v01.crystalpack`, file SHA-256
`1c6ba5d59097546ff8b94e09c809d270727ce1272acb93759b89f3b3c7d9b90a`,
identity `317e9c039914d27c83dfed01ef55b4720023867fd9d8d5aef9ea1b1a452aec3e`.
It contains actual Ely, pine8b and the private alias. Root reports five alias
asset tests passed in 42.59s and three pine tests in 19.13s. Renderer/controller
acceptance of the new winter terrain is separate.

Current review candidate is `source-v02/`, after burying twenty purely
decorative flower tiles under existing Snow Floor. The immutable v01 remains.
V02 changes 3,033 R12 and 1,553 R13 metatiles, total 4,586; its forty ICE
quadrants are unchanged. Review contract SHA-256:
`7e345514059a5333fe383db501a0d82785fc294cb9f9b8d398b46f5ffeea2d9d`.
`terrain.freeze.json` freezes all nine exact old/new grids, candidate files and
canonical before bytes while permitting a later reviewed attribute-only
geography rebind. No source has been promoted.

The matching ignored render epoch has full before/candidate scenes, compact
whole-region comparison pairs, seventeen paired player positions and three
additional pairs showing the actual walkable snowline, mandatory R12 grass
saddle and ordinary lower R13 corridor. The author personally inspected both
complete regions and all twenty player positions. The native narrow pine and
forest canopy silhouettes, unfrozen lakes, icy grass, granite, stops and gates
read clearly. Retained native Cut/Headbutt ground pads and the original ledge
face remain green; Root is reviewing these rare interaction/ledge variants.
The author has not replaced them with visually similar but incorrect walls.

Static proof SHA-256 is
`0e1083cc7179e0c539529b9b5824660b87eac23328fd487c207de97ae7bd7d53`;
author sight receipt is
`49d5461df7e7058de2d7908b0fa5549fcd94360f344c095e09c79d19721e01c3`.
They establish the exact scoped source proposal and listed source views, with
explicit controller/GPU limits. Final geography binding, Root visual acceptance,
native publication and real winter traversal remain pending.

## Guarded native implementation (2026-10-27)

`crates/crystal-assets/src/inland_winter.rs` implements the frozen source-v02
review contract (`7e345514059a5333fe383db501a0d82785fc294cb9f9b8d398b46f5ffeea2d9d`).
The publication CLI is `build_inland_winter BASE CONTRACT EXPECTED_PARENT_FILE_SHA
NEW_OUTPUT`; the input must be the completed 694-map geography stage. The CLI
requires the actual parent file SHA and a fresh output, then verifies written
readback. This implementation entry is not a test or native publication receipt.

The geography rebind is deliberately exact: R12C01R01 east must target
`MnMaraisInlandNeck` at offset0. The guard temporarily reverses only that target
to `MnGrandMarais` to compare the entire module to the frozen original SHA.
Applying the seasonal terrain uses the same reverse normalization to compare
the entire result to the frozen candidate SHA. Every original/new block-array
hash, native artwork hash, collision quadrant and native block mirror also
remains guarded. No original module guard is dropped. All forty changed
collision quadrants are FLOOR→ICE, and all other collisions, lakes, programs,
actors, field-move tables, audio and unrelated parent data remain exact.

Asset tests use `CRYSTAL_INLAND_WINTER_TEST_PACK` for the geography parent and
filter `inland_winter_` (publication preservation is ignored unless explicitly
selected). Production ice tests use the completed winter result through
`CRYSTAL_INLAND_WINTER_CONTROLLER_PACK` and filter
`inland_winter_controller_all40_ice_quadrants_force_slides_and_stop_on_dry_land`.
They exercise both directions of all six lanes across all forty ICE quadrants,
including opposed input during sliding, dry stops and stable returns. Set
`CRYSTAL_INLAND_WINTER_CAPTURE_SAVES_DIR` to retain reached stop saves. Existing
`gunflint_cut_cache_denies_opens_collects_and_continues_native_tree_reset` and
`gunflint_controller_walks_six_clipped_maps_portages_and_original_rooms` now
read the actual warm/private winter tileset; run with
`CRYSTAL_GUNFLINT_TEST_PACK` pointing to the completed winter result for native
Cut denial, actual removal, cache access/reward, save/continue/tree reset,
clipped seams and original room returns. All fixtures declare local setup and
do not establish earned badges or regional campaign progression.

### Bog side-bay traversal under native sliding

The frozen bog rime spans core `(32..33,72..77)`. Native forced sliding prevents
turning east halfway down this lane. Consequently, the optional east pocket
containing the STONE BOG sign `(35,75)` and trainer `(37,79)` is reached from
the lake, rather than from an imagined walking stop on ICE. Its native Floor
landing `(37,78)` borders the same connected pool at `(38,78)`; the main-route
bank `(50,84)` mounts Surf west onto `(49,84)`. The local fixture
`gunflint_winter_bog_side_bay_surfs_from_main_bank_reads_sign_and_approaches_trainer`
checks main-road bank connectivity, native Bag Repel use, actual mount, the
connected water course, native dismount, the original sign, safe trainer
approach, and a real challenge. Its initial legal Mew60/Surf/badges and five
Max Repels are setup; it does not establish earned progression. The ordinary
six-map walking fixture remains empty-party and delegates this specific winter
side discovery to the water fixture. This description is the implemented
traversal contract, not a passing-test receipt; consult the final native logs.
