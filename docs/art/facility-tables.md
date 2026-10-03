# Facility tables and square-backed chairs

The native facility source contains one square document table, five shallow
book-and-cup desks, four joined meeting tables and twenty-two square-backed
chairs. These are separate from the apparatus workbenches and round Radio Tower
stools. All four new meshes use original folded oak joinery, ochre writing
surfaces, recessed aprons, complete side/rear faces and low dark ferrules.
The book has raised cream leaves and binding; the cup has a true open faceted
interior and an open handle. Only the book's small printed face samples the
current native drawing. There are no copied textures inside model/source files.

## Complete source and native space

- Square table: full facility `$0f`, 4×4 source cells
- Side desk: full `$29` guard, with only the upper 4×3 drawing owned; the bottom
  checker floor row stays native. Its 32×18px body aligns with the existing
  instrument workbench's 6–24px physical depth and 9px tabletop
- Meeting table: complete joined `$10,$11 / $14,$15` 8×8 guard. Only its central
  6×4 table drawing is owned. Its four independent chairs and native 24 floor
  cells remain outside table ownership
- Chairs: complete 2×2 `$0e,$0f / $1e,$1f` drawings in the correct local phases of
  `$10,$11,$14,$15,$27,$35`; `$35` retains its unrelated upper wall course

Every guard checks atlas, block, both local coordinates and tile index. Exact
map scope supplements these checks. A custom live profile anywhere in an
assembly's guard wins; reserved ownership prevents double geometry. Runtime
append checks the guard and genuine checker donors again before writing.
Cropped, altered, differently phased or unsupported drawings retain the native
path. Both checker phases are sampled from actual facility `$07` source. The
red/black `$0b` stripe is unclaimed FLOOR, never an extruded pipe/bank.

Models add no collision, warp, script, actor-facing or support-height changes.
A chair is fitted inside x=2–14, z=0–8 of its 16×16px drawing, with its legs
recessed under the seat. The native actor root stays at (8,16), with no seating
translation. The clearance check includes actual occupied-chair rigs, Mr.
Pokémon's cane, and both player rigs rather than a substitute bounding capsule.

## Accounting

| Map | Square table | Meeting tables | Side desks | Chairs | Total cells |
|---|---:|---:|---:|---:|---:|
| MrPokemonsHouse | 16 | 0 | 0 | 8 | 24 |
| PowerPlant | 0 | 24 | 24 | 24 | 72 |
| RuinsOfAlphResearchCenter | 0 | 0 | 12 | 0 | 12 |
| SilphCo1F | 0 | 24 | 0 | 24 | 48 |
| TeamRocketBaseB3F | 0 | 48 | 24 | 32 | 104 |
| Total | 16 | 96 | 60 | 88 | 260 |

The table count is 172 cells; chairs add 88. Guard-only floor/chair cells are
not included in a table's count. Both relevant Rocket block-change script
states retain the exact expected set. These counts establish source coverage,
not native visual approval or complete gameplay verification.

## Rebuild and verification

`python tools/build-facility-tables.py OUTPUT --runtime-only` deterministically
reproduces the four compact gzip model documents (2,784 triangles). The same
part graph creates `facility-tables.blend` with four individually editable
asset collections and separately arranged preview instances:

`blender -b --threads 2 --python tools/build-facility-tables.py -- OUTPUT`

Pack that editable source using `tools/johto_art_sources.py --store
OUTPUT/facility-tables.blend`; retain existing manifest entries. No content pack
or extracted source image is committed.

`python tools/check-facility-tables.py --pack EXTERNAL_PACK --rustc RUSTC`
checks deterministic geometry, positive volume, manifold part shells, normals,
all native maps/script states and source/crop/custom/ground/actor/warp guards.
`python tools/check-facility-table-rigs.py` checks the original rig hierarchy
against all chair components at 72 yaw angles and 24 motion phases. Mr. Pokémon
uses the actual stationary idle behavior: the external-pack check confirms
that his source actor stands and his scripts never apply movement to him.
Other rigs exercise idle, walk and
run poses. The rig check uses NumPy only for local authoring validation.
Production mesher tests check support preservation, sparse ownership, live ink,
finished bounds, stale-source atomicity and custom profile priority.

Native front/back/side composition, player passage and the actual NPC
interactions remain required integration checks after staging. Standalone
source and geometry checks do not replace those checks.
