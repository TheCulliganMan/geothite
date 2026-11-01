# Expanded North Shore — frozen source handoff

The reviewed candidate is **56×96 metatiles**, 112×192 core tiles, with eight
exterior chunks. Lake Superior occupies the continuous eastern half; the
tributaries were narrowed rather than enlarged. This is authored source and
native-pixel preview evidence. Root/Brainerd own Rust integration, pack
publication and production-controller acceptance.

Final immutable directory:
`planning/minnesota/renders/north-shore/expanded-candidate/5deb5fba88dadc2c/`.

| Input | SHA256 |
|---|---|
| `MN_R11.json` | `9788dbc9f44466cde02bef8bc3a886c7a686ad6941c56687cbfe1467d95e5bb2` |
| `MN_R11.layout.json` | `7ebf908ee152597f27c240e800c8f9753df07da350f01bc42ab34e88eea631b0` |
| Source pack, `minnesota-cities-review.crystalpack` | `37a09cd52d95b7425711237a4014ec387555a27cc3d4df61e246581b4d108cf6` |

The candidate uses actual compiled tileset **`mn_north_shore`**. An earlier
preview's unnecessary `_expanded` alias was corrected in this new immutable
layout; all geometry, recipes and resulting pixels remain exact. Existing
`00`–`80` pixels must remain untouched. Source recipes append only `81` from
native cave `12` and `82` from native cave `30`, with their authentic Floor/Wall
quadrants, no palette override or row transformation. Original `80` is still
native cave `2c`/WATERFALL. Stairs are ordinary physical walking surfaces, not
warps, new HM rules or invented LADDER collision.

Native compilation rejected two diagonal cliff corners in the preceding `49a646cbe38415cd` source. The new immutable source inserts explicit inside orthogonal steps in the Pine headland rib and Pebble bend toe. All source raster blocks and rendered pixels are unchanged. The Python reviewer now rejects diagonal polygon, cliff-polygon and path edges before rasterization, including implicit closing edges; all 13 negative checks pass. Native compilation and exact block comparison remain required. The rejected source and every render are retained.

## Review images

The final directory contains `whole-route.png` at **1792×3072**, its half-size
overview, all original/RGBA/conversion stages, exact 160×144 crops and 4× views.
Start with:

- [Complete route](../renders/north-shore/expanded-candidate/5deb5fba88dadc2c/whole-route-half.png).
- [All decision cameras](../renders/north-shore/expanded-candidate/5deb5fba88dadc2c/decision-cameras.png).
- [Birch grass switchback](../renders/north-shore/expanded-candidate/5deb5fba88dadc2c/birch_grass_switchback-native160.png),
  [short ledge](../renders/north-shore/expanded-candidate/5deb5fba88dadc2c/birch_short_ledge-native160.png)
  and [ordinary stair return](../renders/north-shore/expanded-candidate/5deb5fba88dadc2c/birch_cave_stair_return-native160.png).
- [Stoney Point offset tiers](../renders/north-shore/expanded-candidate/5deb5fba88dadc2c/stoney_full_tiers-native160.png)
  and [Lester fall](../renders/north-shore/expanded-candidate/5deb5fba88dadc2c/lester_full_fall-native160.png).
- [Actual weakest routine frame](../renders/north-shore/expanded-candidate/5deb5fba88dadc2c/worst-corridor-native160.png),
  core `(48,152)`: a shore-road decision and lake-facing spur. The former large
  empty gorge shoulder at`(37,71)` was narrowed by an attached pine contour.
- Actual [east](../renders/north-shore/expanded-candidate/5deb5fba88dadc2c/superior_rock_boundary-native160.png),
  [south](../renders/north-shore/expanded-candidate/5deb5fba88dadc2c/southern_sea_boundary-native160.png)
  and [north](../renders/north-shore/expanded-candidate/5deb5fba88dadc2c/northern_sea_rock_boundary-native160.png)
  visible gray-white sea-rock boundaries.

The original two boundary-camera coordinates were inadvertently left inside
the expanded lake. Root caught their all-blue frames. They are corrected to
core `(107,81)`, `(90,187)` and new north`(90,5)`. Each legal Surf camera now
shows the native BUOY rock chain; individual 32px source blocks and enlarged
versions are retained in `boundary-source-art/`. No invisible-collision or
pure-water-art substitute was added. The checker now rejects that stale
camera regression as well as a physically open water edge.

The author personally inspected the full route, new grass/ledge/stair frames,
all new falls, worst frame and corrected boundary images. Root separately
reviewed the composition, Birch/Stoney revisions and corrected visible rocks.
These source renders are not relabeled as production-client captures.

## Concrete traversal changes

The old ten trainers, five residents, their parties/positions/text, four original
pickups, six original signs, encounter tables, Q07 requirements, lighthouse
facade/door/room/keeper and original summit contract remain exact. Added rewards
are NUGGET 05 and FULL_RESTORE 06; added signs explain Palisade, Stoney and Lester.

| Feature | Exact contract, zero-based core unless marked metatile |
|---|---|
| Palisade crown | Nugget approach requires **6 grass steps**; upper`(18,51)` → south lip`(18,52)` → landing`(18,54)`; **29-step** ordinary eastern cave-stair return. |
| Birch switchback above lighthouse | The former straight corridor is closed by joined pines. Winding road crosses at least **3 grass steps**; upper`(33,89)` → lip`(33,90)` → landing`(33,92)`; **11-step** stair return. Existing T04/T06 remain at their exact anchors. |
| Lester terrace | Full Restore approach requires **4 grass steps**; upper`(35,171)` → lip`(35,172)` → landing`(35,174)`; **15-step** ordinary stair return. |
| Narrow original mouths | Baptism, spring gorge and Temperance decks shrink from 4/7/2 metatiles to **one** each over actual narrow water. No pretend Surf-under-bridge connection. |
| Knife River | New native drop at meta `(12,59)`, compact pool, narrow mouth and required deck at`(17,61)`. |
| Stoney Point | Native upper faces at meta `(10,74)/(10,75)`, ordinary plunge at`(10,76)/(11,76)`, offset lower face`(11,77)`. Full tiered face is visible from`(25,152)`. |
| Lester gorge | New three-block native face at meta `(14,88)` through`(14,90)`, small ordinary-water source/plunge and walking bank at`(31,179)`. |

There are **15 WATERFALL metatiles across six distinct fall locations**,
including the preserved original spring-gorge fall. Most channels are one
metatile wide; small two/three-wide source or plunge pools provide context.
The main coastal road works without Surf or Waterfall. Optional cascades have
ordinary-Water approach/return stages and bank relaunches recorded explicitly.
Those HM transitions still require actual controller proof.

There is **no new cave entrance or cave room** in this surface candidate. Native
cave stairs do not imply a cave promise. The sole existing R11 interior is
still the complete lighthouse room. A future cave needs its own authored room,
honest facade, paired exits/arrivals and gameplay purpose before it is shown as
playable; no unbound cave glyph was placed here.

## Ports, chunks and integration

- Grand Marais north port remains meta `(20,1)`, landing`(20,3)`.
- Duluth's old meta `(17,70)` becomes an interior neck. New south port is
  **meta `(23,94)`**, landing **`(23,92)`**, on `MnR11C00R03` at local
  gate`(23,22)`/landing`(23,20)`. Two open south edge Floor lanes are global
  core `(46,191)/(47,191)`, local`(46,47)/(47,47)`.
- Preserve original native group41 IDs: C00R00=1, C00R01=2, C00R02=3,
  lighthouse=4. Append the five new chunk identities; never rebuild all IDs
  in row-major order and renumber existing maps/saves. First-column extents
  are 32×24; second-column extents are 24×24. Four chunks deliberately contain
  continuous lake, not compulsory walking islands.
- The old [Duluth seam proposal](duluth-seam-contract.md) describes the earlier
  32×72 R11. If this expansion is accepted, Duluth northcore `(32,0)/(33,0)`
  would align with the **new** R11 south lanes using city north offset **−7**
  and route south reciprocal **+7**, in metatiles. Its old −1/+1 offsets and
  old southern-pine-opening edit are superseded for this candidate. Regional
  connections/checkpoints remain Root's separate implementation work.

`expansion_contract.controller_probes` records exact native stair lanes/walls,
the three pairs of two-lane dry row seams, public sea bank`(51,57)` →
launch`(52,57)`, ordinary-Water input waypoints visiting **all eight chunks**,
and actual BUOY boundary probes. The earlier proposed bank`(51,58)` is a rail
wall and was rejected before freezing. Use the captured probes, not that
discarded coordinate. Surf crosses internal reciprocal water seams; it cannot
go around any outer edge.

## Executed source checks and replay

`review.json` reports `errors:[]`, all eight walking/Surf chunks reached, zero
outer Water, no incidental Cut, connected doors/actors/resources, three actual
hop lips/blocked reverses/ordinary returns, mandatory grass and measured cache
costs, real 06 decks and six complete fall cameras. Thirteen semantic negative
fixtures are rejected: open Surf boundary, lost original/offset fall, removed
or fake bridge, incidental Cut, erased directional face, free cache bypass,
missing stair, stale boundary camera and all three diagonal geometry kinds.
Final source render/check took **5.22 seconds**, excluding initial pack decoding.

```sh
.local/minnesota-review-env/bin/python \
  planning/minnesota/tools/review_expanded_north_shore.py \
  planning/minnesota/renders/north-shore/expanded-candidate/5deb5fba88dadc2c/MN_R11.json \
  planning/minnesota/renders/north-shore/expanded-candidate/5deb5fba88dadc2c/MN_R11.layout.json \
  --pack content-packs/minnesota-cities-review.crystalpack --self-test
```

The narrow wrapper reuses `SourceArt`, the existing explicit rasterizer,
source/common route/river/ledge checkers, native sprite handling and immutable
render archives. It never creates a pack or chooses random geography. Candidate
source changes produce a new content-addressed directory; originals, rejected
passes, crops and every conversion remain retained. Root's native checks,
controller cases, matching production renders and earned regional progression
must be reported separately.


## Actual native written-pack geometry and production images

After the publisher confirmed the 505-map immutable pack with file SHA256
`3949ad8dff81537f9b664f914f5358825da40512fbfdae52c2fffa1a049441b7`,
this worker compared the actual frozen 5deb source directly with the native
pack. All 5,376 blocks, authored actor placements and lighthouse room rows
match. Whole native/source terrain pixels are equal, all 128 original Johto
blocks/collisions and three native import recipes are exact, eight stable
chunk IDs and 20 connection records pass. Exact source snapshots and both
terrain originals are retained under
`renders/northwoods-native/3949ad8dff81537f/native-verify-1793064684803419000/`.

All seven R11 production 2D decision captures subsequently passed and were
personally viewed, alongside seven Safari frames. The [actual native sheet](../renders/northwoods-native/3949ad8dff81537f/production-1793064804698539000/production-decision-cameras.png)
includes the weakest shore fork as well as grass, ledges, stairs and waterfall
tiers. The [capture runbook](northwoods-production-capture.md) records exact
pack/executable hashes, native player pose, raw readbacks and derived crops.
No source promotion, regional seam installation or earned travel is implied
by these render fixtures; production-controller results remain separate.
