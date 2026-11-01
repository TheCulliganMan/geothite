# Warm North Shore restoration

Restore the full North Shore to its actual native warm terrain. The user's
current instruction places Grand Marais and North Shore on Lake Superior and
puts winter inland near Ely. Root approved the exact 828-cell reversal as the
first native part of that instruction. This is not acceptance of the final
direct coastal connection or new Ely geography.

The immutable publisher input is [source-v01/native.contract.json](source-v01/native.contract.json),
SHA256 `ad3d97771d1bd0c2e4bcba253a3dba7e060ee7996f8e723527796644fdff0976`.
It binds all eight current R11 modules, every changed block and collision
quadrant, source preimages and the two detached warm authoring candidates.
Canonical promotion remains unapplied. Root has now published the bounded
native warm restoration; the direct coastal geography remains pending.

The inspected current native baseline is
`content-packs/minnesota-river-long-v02.crystalpack`, file SHA256
`3ddbad16c6538e52f219c12497e4fbe6e41a7f18af5c42589b7b92357cfad243`,
identity `3ead916d12c705f2c0559c98b983f5af4760c9a8f557d8e31f62b36ea197e29a`.
The actual prewinter reference is the completed mixed-art pack
`content-packs/minnesota-winter-mixed-art-v01.crystalpack`, file SHA256
`2b2887fd056d507d85ac5d2b8861e92559d8679cf11902f63c42f270d617842d`,
identity `bed021120fb15c2e2106b1ec8b4f030af252bc0f405572c6e7cf83f42f0275e4`.
The restoration was compared against that pack's real native modules, rather
than inferred from an old layout or a general snow-to-grass substitution.

## Exact reversal

| Installed winter recipe | Warm original | Metatiles |
|---|---|---:|
| Snow canopy 86 | Native pine canopy 05 | 580 |
| Snow Floor 83 | Native lawn 02 | 117 |
| Snow Floor 83 | Native path 01 | 63 |
| Snow Floor 83 | Native rock terrace top 71 | 27 |
| Frosted encounter grass 84 | Native encounter grass 03 | 20 |
| Sliding ice 85 | Native path 01 | 8 |
| Sliding ice 85 | Native lawn 02 | 5 |
| Snow-ground sign 88 | Native sign 45 | 6 |
| Snowy roof-canopy 89 | Native roof-canopy 2c | 1 |
| Snowy roof-canopy 8a | Native roof-canopy 2d | 1 |

MnR11C00R00 restores499 cells, including the two roof halves; MnR11C00R01
restores329. Both map tileset names return from `mn_winter_north_shore` to
`mn_north_shore`. The remaining six chunks and Split Rock interior already
match the actual warm native pack. Every restored whole R11 module matches
that warm reference, including the approved native cap at meta20,0.

Only52 collision quadrants change semantics, from ICE to FLOOR. Every other
quadrant is exact, including the route's11,790 WATER,60 WATERFALL,310 BUOY,
82 UP_WALL,32 LADDER,14 HOP_DOWN and2 DOOR quadrants. Grass footprints,
Headbutt shapes, roads, short ledges, cave stairs, rock faces, bridges, falls,
current seam records, actors, trainer parties, flags, events, story, room
returns and scripts are retained. No snowy cliff-face recipe was installed;
the27 rock-top Floor cells are the precise cliff appearance restoration.

The cold alias's private snow-tree87 is installed but unused on R11. The
inventory in the proof enumerates every active winter recipe's native glyph
addresses, palette attributes, foreground RGB colors and art file hashes.
All131 original warm recipes remain pixel-exact in all four time banks,
including their counterparts in the cold alias's unchanged prefix. Retargeting
the maps restores ordinary grass rustle selection while retaining native
water/flower/waterfall behavior. No Rust renderer, palette, animation, art or
flag change is needed. Leave the shared winter art and its report history
installed for inland regions.

Grand Marais itself uses `johto`, its cedar approach is warm, and Gunflint R12
uses `mn_gunflint`; none received this winter overlay. The proof inventories
the entire current R11/Grand Marais/R12 stretch. Rainy River and Border Pines
winter remain outside this restoration. Existing R11-to-Cedar geometry stays
guarded until the coordinated direct coastal connection is reviewed; this
intermediate is not a claim that Grand Marais already sits correctly on the
coast.

## Authoring and read-only proof

The warm layout candidate retains the original218 authored strokes and285
explicit native fitted shore/cap points, removing only the828 winter/roof
tail points. Its503 layers compile to the complete56×96 actual warm native
grid. Keep `shore_fitting: explicit_source_edges`; refitting the shoreline
would change approved native geometry. The warm dossier preserves every
nonwinter field and replaces the superseded winter acceptance annotation
with a proposal-only warm review. No stale dossier is lowered into scripts.

Current canonical preimages are layout
`022353fdc855967a02a8f99dd9664624d6b7dfb0a9273f9a98051d36dca64813`
and dossier
`27cd3a73d84564fc87f38fc3666c339e54a2c94b380f53bf5c43c635fcbbacfd`.
The checker has no apply or pack-write mode:

```sh
PYTHONDONTWRITEBYTECODE=1 .local/minnesota-review-env/bin/python \
  planning/minnesota/revisions/north-shore-ely-geography-v01/r11-warm/prepare_review.py \
  --output planning/minnesota/renders/north-shore-ely-geography-v01/r11-warm/NEW_REVIEW
```

An explicitly approved later parent can be checked using `--pack` and
`--expected-parent-sha256`, provided all eight R11 preimages remain exact.
The full inverse residual preserves that actual parent's unrelated changes.
The first proposal freeze is already retained; do not use `--freeze-proposal`
again on it. That flag can only create the detached owned source folder.

Review epochs under `renders/north-shore-ely-geography-v01/r11-warm/` retain
the complete current cold route, actual prewinter warm route, candidate warm
route, original source sprites, all intermediate composites and player crops.
Source-v01 used an ICE pose at40,2, and source-v02 used the legal seam tile40,0
but cropped the player against the top edge. Both remain historical views.
Source-v03 uses the ordinary Floor turn40,8 for a readable common player pose.
The other ten views cover the house/falls, ordinary bank, required habitat,
Superior cove, Palisade ledge, rocky terrace, second stream, cave stair return,
Stoney falls and Duluth approach. These source cameras do not execute a
controller, GPU, native border neighborhood or earned campaign journey.

The static proof checks the two block/attribute mirrors, all eight exact warm
modules, all other native collision quadrants, immutable source hashes and a
whole-parent residual. Eight deliberately corrupt candidates are rejected,
including synchronized wrong terrain, leftover ICE/roof snow, moved actors,
changed programs, guarded southern terrain, unrelated city terrain and a stale
owned parent. No source image or graph proves gameplay input.

## Native integration handoff

The focused, initially unregistered Rust leaves are `north_shore_warm.rs`,
`north_shore_warm_tests.rs` and `bin/build_north_shore_warm.rs` under
`crates/crystal-assets/src/`. API: `build_north_shore_warm_pack(&CompiledGamePack, &str)`.
Root owns registration, Cargo and publication. No Cargo or native publication
was run by this author. The intended output is the ignored external
`content-packs/minnesota-superior-warm-coast-v01.crystalpack`.

The CLI accepts `BASE CONTRACT EXPECTED_PARENT_FILE_SHA NEW_OUTPUT`, requires
a fresh path and checks full verified written readback. The constructor checks
the actual parent's derived identity,683 maps, all eight R11 module preimages,
two cold aliases, warm native definition/art preimages and block isolation.
It can compose after the approved park update without resetting that map or
its art/program changes. It changes only828 cells, two encoded mirrors and two
tileset/attribute mirrors, then derives the new identity. Cached programs and
uncached programs are each compared to their own inherited representation;
the old wire and typed caches are not assumed equal.

Five authored Rust cases cover source hashes/whole layout, actual828/2/2
counts and all eight warm grids, stale/current-identity owned parents,
contract/art drift, ten full residual corruptions, and composition with a
valid unrelated park revision. Four require the approved external parent in
`CRYSTAL_NORTH_SHORE_WARM_TEST_PACK`. All five passed in45.03s in Root's
`.local/north-shore-warm-assets-v01.log`. Publication passed strict full written
equality in `.local/north-shore-warm-publication-v01.log`.

The actual published file is
`content-packs/minnesota-superior-warm-coast-v01.crystalpack`, SHA256
`ebea77ccb9b9dbe54aa5101e32590c1b075ed8130f6e7fcfe2d8c8451dab11d8`,
identity `5009f2898563ecf699d7036b19d988ef5f4d43a8b39cbd48f8616cc9c45eca32`.
It composes with the approved R04 parent. This author independently read the
actual native file, verified its hash/identity and checked the controller
courses against all eight warm chunks. The focused controller leaf and
reached-save/GPU guidance are in [controller-v01/README.md](controller-v01/README.md).
Controller traversal, Save/Continue and actual GPU review still require their
own execution receipts; asset publication does not complete those checks.
Reversibility is the exact
contract inverse, not replacement of the actual parent with an old pack.
