# Minneapolis exterior house pilot

**Current status: the user rejected the v03 shape/variety interpretation.**
The actual defect is incorrect native tile joins: side/end caps appear inside
wide houses and native ground-floor foundation strips are repeated on upper
storeys. V01–v03 renders and contracts are retained as rejected history. They
must not be published as the requested fix. The current correction is
`review_tile_joins.py` and `renders/minneapolis-houses-v01/join-correction-v04/`.
It keeps existing architecture and repairs the glyph-role grammar, with zero
new bitmap glyphs or palette changes. The correction is now published as
`content-packs/minnesota-house-joins-v04.crystalpack`, file SHA-256
`63117daf68439663d7632293f8bded7352bf8e39b96d30edcc14f035871d4d6b`,
content identity
`d2a3829ec3caa1f684a011b6004891b48aa65420a741c99795b0c89e1a4f984c`.
Strict written readback and three focused asset regressions pass (11.40s,
`.local/house-joins-assets-v04-exact.log`). The earlier zero-test log does not
count. Independent written-pack proof now passes 793 checks and 13 corruption
cases, including all four palette banks, all forty door images and all 5,120
collision quadrants. The actual-pack waterfall animation regression passes.
Initial GPU attempts failed the old renderer recipe guard and are retained as
failures; the narrowed original-glyph-only guard has four passing unit cases.
The renderer rebuild is complete. Root personally inspected three actual
headless GPU captures from the published pack and accepted the reported
tile-assembly corrections: Boat House, Supply Coop and Northeast Home.
This is manual visual review, not a zero-mismatch GPU comparison. Events and
collision are proved unchanged; fresh doorway input is not required for this
cosmetic change. No new earned progression is claimed. Scoped Rust formatting
passes; wider formatting has unrelated drift and all-target Clippy is blocked
by the existing multiplayer bad-bit-mask error. Acceptance covers the reported
assembly defects only; broader city design and all-route quality remain
unfinished. See [source-v04/README.md](source-v04/README.md).

This is a detached source candidate. It has not changed the installed game or
the canonical city layout. The user confirmed **2D exterior houses**.

The actual parent is `content-packs/minnesota-city-river-bend-v01.crystalpack`,
SHA-256 `57eefb675b9cb32ac66a01819f56961392e51678a6f9dd959888fbb9427e4e62`.
The city remains 40×32 metatiles. Existing doors, room identities, services,
streets, people, waterways, scripts and story are preserved.

## Rejected source pass v03: retained history

After review of v01/v02, v03 gives the four homes different architectural
identities. Northeast Home retains the blue diamond gable and River Naturalist
retains the lower pitched weatherboard cottage. Garden Flat becomes a low warm
native Ecruteak timber cottage, using unchanged Johto `26`/`27` metatiles; its
former solid upper roof row becomes ordinary solid pines. Pattern Studio gets
two glazed storeys of native Johto Modern cyan brick under a shortened blue
eave. Bookbinder retains its masonry frontage. The actual door and every old
collision quadrant stay unchanged in all four cases.

The latest complete source contract is
[pilot-v03/house-exteriors.contract.json](pilot-v03/house-exteriors.contract.json).
Thirteen appended metatiles occupy `9d`–`a9`. The exact all-four-bank pixel
study finds **27 new glyphs** (18 Kanto and 9 Johto timber), reusing 37 existing
alias glyph IDs. All 27 fit in appended logical slots `e0`–`fa`; the 224 old
palette entries can remain unchanged. See
[pilot-v03/glyph-feasibility.json](pilot-v03/glyph-feasibility.json).

Personally inspected latest images are in
`renders/minneapolis-houses-v01/pilot-v03/`: the whole city, northeast cluster,
three before/candidate native player cameras and all four native palette banks.
The wood cottage has the native warm timber/gold roof at day; the studio shows
both window bands and their native night lighting. Source render work after
pack loading took 0.780 seconds. An earlier integration-owner acceptance was
withdrawn when the user clarified that the actual problem was incorrect tile
joins and upper-storey foundations. No v03 native pack was published. These
shape/variety changes are rejected as the requested fix.

The rejected Rust implementation was archived under
`renders/minneapolis-houses-v01/rejected-pilot-v03-native-source/` and removed
from active source. Do not register or publish it. No canonical town source
has been changed by this author. The historical frozen publication contract is
[pilot-v03/native-publication.contract.json](pilot-v03/native-publication.contract.json),
SHA-256 `891e0030e1c5baed9d8fc4b6bb4941a8df33d7427597f3a6905c74f821a65600`.
Earlier formatting/source checks are historical evidence only; they do not
establish that the rejected v03 addresses the user's requirement.

## Complete-city inventory for the next pass

[exterior-inventory-v01.json](exterior-inventory-v01.json) records all forty
real frontages, their native footprint, real door/passive return and current
block identifiers. The complete native sheet and forty original crops are
saved in `renders/minneapolis-houses-v01/exterior-inventory-v02/`. The earlier
partial v01 inventory is retained; its panel-height assertion correctly exposed
the Department Store's fourteen-core-tile façade height.

The next coherent residential cluster is the west waterfront: Boat House,
Angler Home, Dispatcher House and Lake Cabin. They currently repeat the same
cyan-brick/blue-cap frontage despite their different functions. A later pass
should establish their individual silhouettes and relationships to the bank
after the current pilot has passed native review. No new template or art was
scattered across the city. The large repeated masonry masses remain a distinct
unresolved visual problem, especially Department Store, Grain Exchange and
Community Library.

The sections below preserve the earlier v01/v02 pilot history.

## What changed and why

Four houses in the northeast pilot replace repeated cyan rectangular roofs and
flat masonry fronts with two native pitched roof silhouettes and weatherboard
fronts. The tall diamond gable belongs to Northeast Home and Garden Flat. The
lower pitch belongs to River Naturalist and Pattern Studio. The Bookbinder's
masonry frontage remains beside them, so this street has a legible architectural
contrast. These are explicit placements, not randomly assigned colors.

The art uses unchanged 8×8 glyphs from native Kanto blocks `0c`, `0e`, `38`,
`39`, `3c` and `3d`. The complete native door quadrant is moved to each existing
warp; the donor doorway is replaced with a native window quadrant. No source
pixel is painted, recolored, interpolated, rotated or reflected.

| Building | Footprint in metatiles | Existing door in core tiles | Roof |
| --- | --- | --- | --- |
| Northeast Home | `(36,3)` through `(37,4)` | `(74,9)` | Diamond gable |
| River Naturalist | `(26,7)` through `(27,8)` | `(55,17)` | Shallow pitch |
| Garden Flat | `(29,7)` through `(30,8)` | `(60,17)` | Diamond gable |
| Pattern Studio | `(32,7)` through `(33,8)` | `(67,17)` | Shallow pitch |

The implementation contract is
[pilot-v01/house-exteriors.contract.json](pilot-v01/house-exteriors.contract.json).
It specifies all 16 map substitutions and all 112 source-glyph selections for
seven appended metatiles. Every old/new collision quadrant is identical; all
5,120 city collision quadrants are checked. All four existing door coordinates
are checked against the actual native warp records.

## Visual review and iteration

Actual native Johto, Johto Modern and Kanto metatile sheets were inspected in
`renders/minneapolis-houses-v01/inspection-v01/`. Whole native Goldenrod and
Saffron reference maps were inspected in `renders/source/`.

`renders/minneapolis-houses-v01/recipe-study-v01/` retains the native house
assembly study. `pilot-v01/` retains the first complete-city render, neighborhood
comparison and three native 160×144 camera comparisons. Their roof tops were
clipped by eight pixels at the selected approach positions. That camera choice
was corrected in `pilot-v02/` by using the real passive exit row; the map and art
contract did not change. Each complete gable and doorway is visible in the
revised player views. The four native palette banks were also rendered and
inspected in `pilot-v02/four-native-palette-banks.png`.

Personally inspected source result: the pilot has clear roof pitches, continuous
eaves, readable windows and one real door per building. The fronts share native
palette behavior at morning, day, night and dark. Night windows light using the
existing native palette. The complete town remains visible in
`pilot-v02/whole-city-candidate.png` and still contains the older large masonry
buildings and other repeated houses. This pilot does **not** establish that all
Minneapolis exteriors or all routes meet the requested quality.

## Fast loop and publication boundary

Run from the repository root:

```sh
.local/minnesota-review-env/bin/python \
  planning/minnesota/revisions/minneapolis-houses-v01/review_houses.py \
  --output planning/minnesota/renders/minneapolis-houses-v01/FRESH-EPOCH
```

The measured warm render work after pack loading was 0.551 seconds for v01 and
0.363 seconds for v02. Every epoch requires a fresh directory; all previous
originals and conversions remain saved. The script reads the pack and never
writes a pack or canonical source.

The all-bank glyph study in [pilot-v01/glyph-feasibility.json](pilot-v01/glyph-feasibility.json)
finds 18 new glyphs and reuses 15 existing alias glyph IDs. New logical glyph
slots `e0`–`f1` fit. The existing sheet contains 192 glyphs and its palette map
has 224 entries. A native Rust publisher must preserve old **logical** bank
resolution when expanding the sheet to 256 glyphs. Blind PNG padding would
change old banked glyphs. Strict old-art comparison across all four palette
banks, whole-pack preservation, native GPU captures and real doorway/room
return input checks are still required before publication acceptance.
