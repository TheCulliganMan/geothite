# Fast Ship room furniture and lower bulkheads

Read-only inspection of the native lighthouse atlas, complete Fast Ship room
layouts, collision quadrants and script events identifies four lengths of tea
table and one open-logbook captain's writing desk. Five wall-side cup tables and
the captain's high-backed chair are additional complete source drawings. The source tables are not
interchangeable full-block props: their headers and feet cross metatile seams,
and the lower half of `$2c` and east half of `$33` are genuine checker floor.

The original tea furniture shares the lighthouse keeper's walnut joinery,
linen runner, faceted blue-green glaze, open cup/teapot handles and rising
pouring spouts. The 48-pixel table uses that existing authored mesh directly
because its complete source drawing is identical. Three additional lengths
retain the native cup/teapot counts: 32px has one cup; 64px has two cups and
one teapot; 80px has two cups and two teapots. The 48px version has one of each.
The captain's 48×32px desk has four legs, recessed aprons, brass deck fasteners,
a quiet writing blotter and a thick open logbook with a raised fabric binding,
sloping cream leaves and restrained ruled markings. The wall-side tables reuse the short cup table in their genuine 32×24px
pictures. A walnut high-backed captain chair has a crest rail, navy cushion
faces, seat supports and brass foot ferrules. No asset contains copied source
textures or pixels.

## Whole drawings and sparse source ownership

`ship_room_sources.rs` expresses small explicit source motifs. All fields of
every guarded cell must match: tileset, metatile, tile index and both subtile
coordinates. There is no location-only conversion or collision extrusion.
The four Fast Ship maps are an additional scope guard. Clipped or changed
pictures and masks overlapping a custom live profile retain their native path.

- Short tea table: south two rows of `$06` plus north two rows of `$2c`, 4×4
- Medium tea table: south two rows of `$06` plus all `$2d`, 4×6
- Long tea table: south `$06`, all `$2a`, north `$2c`, 4×8
- Mess table: south `$06`, all `$2a`, all `$2d`, 4×10
- Captain desk: complete `$32` joined to west half of `$33`, 6×4
- Wall-side cup tables: rows 1–3 of `$37`, 4×3; the porthole row stays separate
- Captain chair: east half of `$30`, rows 1–3, 2×3; the upper wall row stays separate
- Round stools: southeast `$07` or southwest `$08`, 2×2; both source phases
  reuse the canonical original ship stool model and cache
- Complete berths: east half of `$36`, 2×4, or the lower east half of `$38`
  joined to the upper east half of `$39`, also 2×4. Both include the complete
  `$83–$86` foot artwork; `$38` alone is an incomplete berth
- Basement room U: three `$20`/`$21` side courses followed by
  `$22,$25,$25,$25,$25,$23`; every involved block is verified completely,
  while claims contain only 60 vertical and 44 horizontal source cells per U

The lower-deck U kit preserves the source corner narrowing. Its side wall
limbs occupy the original 16px strips, narrow to 8px at the last corner course,
and join a 3px-deep transverse wall at the native south facade seam. The open
north edge, both rooms, berths, perimeter passages and north entrances remain
empty where the native source is empty. The continuous recessed steel shell,
deck skirting and ivory top rail are separately closed, editable components.
Inset navy enamel courses, outer gray-green panels and small brass rivets
provide structure across joined courses. Panel faces stand clear of the core,
avoiding coplanar hidden decoration.

The 208 claimed divider cells retain the original 16px height. Only those wall
vertices join the existing local player reveal ranges; native floors and all
furniture stay opaque. The complete sparse mask is checked again on append.
Each claimed cell receives its matching parity of the verified zero-height
checker donor. Geometry never changes the native support-height vector.

## Exact accounting and script exclusions

There are 63 complete furniture/fixture assemblies (642 cells) and two
connected bulkhead assemblies (208 cells), totaling 850 source cells across
four maps:

| Map | Furniture | Bulkhead | Total |
|---|---:|---:|---:|
| FastShipB1F | 168 | 208 | 376 |
| FastShipCabins_NNW_NNE_NE | 120 | 0 | 120 |
| FastShipCabins_SE_SSE_CaptainsCabin | 194 | 0 | 194 |
| FastShipCabins_SW_SSW_NW | 160 | 0 | 160 |

The former 272-cell table/desk plot queue contains 232 actual furniture cells
and 40 native floor cells. All 232 furniture cells are consumed; all 40 floor
cells remain at native elevation. Complete table headers add 88 source cells
outside that plot-only queue. The 40 retained floor cells are intentional,
not unfinished furniture. This accounting is independent of the basement's
120 vertical plus 88 horizontal divider cells. The additional `$37` tables
and `$30` chair add 66 cells outside the prior plot queue.

Both supplied core and realtime packs give the same result. The original table, desk, chair, bulkhead and bed drawings have WALL collision.
The 104 round-stool source cells retain native FLOOR collision. Claims overlap
no warp or coordinate event. The player's joined berth in
`FastShipCabins_SW_SSW_NW` intentionally contains two native `BGEVENT_READ`
interactions at gameplay `(7,1)` and `(7,2)`, both invoking `FastShipBed`.
These cover the mattress and foot quadrants respectively; every other ship-room
claim excludes background events. Fitted volumes remain within their original
drawings and preserve all native support heights. Four chair source cells intentionally share the
captain's native sprite footprint. The captain remains at gameplay `(3,25)`,
with source-relative foot anchor `(8,24)` at the chair's south edge. The chair
stays behind that anchor and does not add actor support or change the pose.
Using the actual captain rig, component AABB checks at all four cardinal
facings prove the static geometry clears the fitted chair; idle-animation and
camera readability still need native review. The two blocker-sailor
side-steps, both lazy-sailor exit routes, granddaughter entrance, grandpa step
and the player's faded relocation are clear. These maps contain no scripted
block changes; voyage flags change actor visibility and scenes. Source/native
scripts remain authoritative, including the sleeping berth and captain scene.

## Rebuild and verification

```sh
python3 tools/build-ship-rooms.py target/ship-rooms --runtime-only
python3 tools/check-ship-rooms.py target/ship-rooms
blender -b --threads 2 --python tools/build-ship-rooms.py -- target/ship-rooms
cargo test -p crystal-voxel-view ship_rooms
```

The generator uses 182 named, individually editable closed components. Runtime
and Blender generation share the same geometry. Six compact runtime files
are added, and the medium table reuses the existing lighthouse asset. The
runtime importer is the existing `interior_models::Model`, including its
normal transformation and required face-light bake for the terrain shader.
The editable source is stored in bounded, hash-verified gzip/base64 chunks
under the existing art-source format.

The checker validates finite coordinates, opaque material colors, unit normals,
positive component volumes, consistently wound closed topology and original
bounds. For bulkheads it checks every vertex, triangle edge midpoint and face
centroid against the physical sparse footprint. Focused Rust tests exercise
source-field mutations, cropping, exact counts, ownership conflicts, missing
floor donors, append-time validation, repeat append, untouched support heights
and cutaway scope. Lightweight tests are not an integrated renderer pass.

Native review remains the integration gate: inspect all four furniture lengths,
the logbook desk, captain/chair clearance, both sides of the joined bulkheads and their narrow corners.
Check table spacing, portal/berth passages and player reveal under camera motion.
Walk the B1F sailor blocker scene, lazy sailor exit, captain/granddaughter scene
and cabin entrances. The shared 48px table and unrelated lighthouse rooms must
retain their existing behavior and appearance.

## Complete stools and berth feet

The 26 stools add 104 source cells to this pass, including 56 previously flat
cells. Fifteen complete `$36` berths own 120 cells, correcting 60 formerly flat
bed-foot cells. Four wall-side berths span `$38` and `$39`, owning another 32
cells and correcting 16 more flat foot cells. The previous half-berth model
matcher no longer consumes either upper mattress when its lower drawing is
cropped, edited or owned by a custom live profile. Native fallback remains
available for the whole drawing. The `$38` wall rows and `$39` floor rows stay
outside the berth claim; a 2×2 `$38` mattress by itself is never a full model.

The SW cabin player's berth starts at source `(14,2)`. Its two native read
interactions occupy source `(14–15,2–3)` and `(14–15,4–5)`. `FastShipBed`
shows the sleep text, fades out, heals the party, restores display and music,
and checks the existing voyage-arrival flags. Both source quadrants remain
WALL, with unchanged support and adjacent approaches. The source checker
requires these exact two read events on this exact compound berth, verifies
the sleep/heal script identity, and rejects all other background overlaps as
well as every warp or coordinate-event overlap. Gameplay interaction testing
remains a native integration check; matching the event does not execute it.

Both stool phases reuse the same authored blue padded seat, timber rim, four
splayed legs and stretchers through the existing dungeon model cache. Their
12×12px round footprint is fitted to local x=2–14, z=0–12 at 7px high. This
stays north of the unchanged actor foot anchor (8,16); the former south-aligned
10px fit would intersect standing hands during rotation. Seven actual authored
NPC rigs, including all spinning and alternate-voyage occupants, clear the
stool components in 72 sampled yaw orientations. This is a conservative rest
geometry AABB check, not proof for animation or a collision modification.

The existing complete berth mesh already contains a timber frame and legs,
rounded mattress, marine blanket, woven stripes, pillow and headboard. It now
uses the complete 16×32px source plot at its unchanged 7px rise, instead of
squeezing into the upper 16×16px half. It uses the canonical mesh cache and the
existing editable dungeon source; no duplicate or new mesh is needed.

The analogous six `$27` northeast Radio Tower stools use the canonical round
studio seat at the same actor-safe 12×12×7px fit. All four source cells guard
each stool, and any live custom profile wins ownership. Their 24 source cells
remain walkable and the two Radio Tower 3F actors keep their native feet.

Run `python3 tools/check-room-furniture-bindings.py --pack <external-pack>`
for exact native ship counts, every identity-field mutation, crop and warp
exclusion, canonical mesh fitting and sampled rig clearance. Run
`python3 tools/check-facility-radio.py --pack <external-pack>` for the radio
source counts and live/custom/ground guards. Native review must still check
all actor approaches, idle/spin poses and the full bed silhouette from both
camera sides; these lightweight checks do not claim that review passed.
