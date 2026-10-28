# Short Gunflint winter route

The latest direction shortens Gunflint and removes the long east–west inland
connector. This detached route is **32×32 metatiles**, down from64×56 (71% less
terrain). Dimensions follow the lake, shelter, frost basin, ranger hut and bog
activities; they are not a template for other Minnesota routes.

The full route now shares one irregular Loons Lake. The main journey walks its
visible bank rather than passing behind a broad forest moat. Raven Rock is a
compact native granite brow with a clear, real cave entrance beside a substantial
frost basin. The frosted basin has deliberate dry/wall stops and a dry rim.
The southern bog neck, hammock resource and ranger forecourt make a short
secondary loop; the sign and trainer previously stranded behind sliding ICE
are reachable by ordinary Foot input. The original Cut herb clearing remains
actually sealed. No Cut or ledge count is imposed as decoration.

Native snowy pine recipe8b is imported as one declared block49 into the private
Gunflint tileset. Existing native snow, encounter grass, water, cliff, hut and
Cut recipes retain their collision/field-move behavior. The native green Cut
pad remains a known art compromise. No new raster artwork or game catalog is
invented. R12 has no newly promised one-way ledge; the short Cedar approach owns
its useful ledge and ordinary return.

## Frozen source

Use **source-v09**, not an earlier candidate. The root visually accepted the
v06 terrain and player cameras subject to native proof. Subsequent bounded
corrections preserve its exact1024 terrain cells:

- v07 assigns the ranger hut exterior warp2; both rooms now share upper chunk.
- v08 explicitly declares `reject_cliff_cuts:false`, matching the authored layer
  order and Python source raster. Later native cliff strokes repair the brow.
- v09 moves the original REVIVE/item04 from inaccessible Raven roof `(34,9)` to
  existing Floor `(35,17)` beside the cave. Its identity, item and event remain
  exact. The strict source graph treats doors as terminal warps, not shortcuts
  through the roof. No stairs or climbing mechanic is invented.

The frozen native contract SHA is
`c2fc7a8d3326c934fb87444c60b122237f878e4fb3181716e95138fd1da83e28`.
Layout SHA:
`760537313a4fe083ec62ffea1e7ed6142c4cd837bc7ab67dc09c0d2b5e856b59`.
Dossier SHA:
`53f040424a2a6bbd43da660eb850532524a74443f29ae7103a4bb31d6ec7477f`.
Exact flattened native terrain SHA:
`d3623b44e5c69e5a2bddc938832e916db94c562346ab5e0be3d0fc858cf8a426`.

The input is the actual65cf parent pack and its six full R12 module preimages.
Canonical layout/dossier originals are retained byte-for-byte beside the
candidate. The contract declares all actor/room relocations, native art import,
new chunk rows and original camp flag2387 reuse. Earlier source and render
epochs remain unchanged history.

## Exact anchors

Root owns the regional placement and registration. Proposed origin is
`[190,-432]`. Only these outer gameplay quadrants are open:

| Passage | Whole-route core | Native chunk/local core |
| --- | --- | --- |
| East to short Cedar | `(63,54/55)` | lower `MnR12C00R01` `(63,6/7)` |
| West to Boundary Waters | `(0,28/29)` | upper `MnR12C00R00` `(0,28/29)` |

The superseded south port is visibly enclosed by native pine forest. All lake
water is internal; no open water border creates an unmatched Surf escape.

Cave exterior warp1 is `(40,15)` with clear return `(40,16)`. Ranger hut exterior
warp2 is `(49,47)`. Its clear return `(49,48)` is **lower chunk `(49,0)`**, so
native passive movement must cross the upper/lower seam. Both original hut
carpets must pair to that exterior warp2. Original interiors, rooms, actors,
programs, flags, encounters, ten trainer parties and four original rewards
remain unchanged except their declared exterior locations.

The retired connector's camp supply moves to `(51,53)`: BERRY×3 using the same
`EVENT_MN_GUNFLINT_CAMP_SUPPLY`/2387. Its original three-line sign moves to
`(55,53)`. Previously collected stock must stay collected after migration.

## Evidence and remaining native work

`source-v09/physical-proof.json` is a static source graph, not a controller
receipt. It uses straight native ICE transitions and terminal door quadrants.
All27 ordinary actor approaches, both native door/clear-return pairs and all
four port quadrants are reachable. The herb reward approach `(14,11)` is
unreachable before Cut and reachable after native0f→17 replacement. Its gate is
`(14,17)`, approached from `(14,18)`. These checks do not prove earned badges,
regional story progression, random encounters or actual native field actions.

Whole-route/source-native player renders are retained under
`planning/minnesota/renders/gunflint-quality-v01/`. v06 is the reviewed terrain;
v09 refreshes the whole actor scene and twelve cameras, including the reachable
REVIVE forecourt. Every older original, crop and rejected epoch remains intact.
Authentic Ilex Forest, Route42, Route44 and Lake of Rage source references are
retained under `reference-v01`.

The production fixture leaf is
`crates/crystal-bevy/src/bevy_shell/tests/compact_gunflint.rs`, using
`CRYSTAL_COMPACT_SUPERIOR_TEST_PACK`. It covers both real seam lanes, ordinary
actor approaches/dry bog sign, original rooms and actual Max Ether PP use,
both hut carpets and cross-chunk passive return, native Cut denial/opening/
reward/Continue/map reset, camp stock persistence, meaningful ice stops/returns,
and native required-grass encounters. It declares local initial fixtures and
uses production inputs afterward. Set `CRYSTAL_MAP_RETAIN_SAVES_DIR` to retain
every reached save. Publication, written readback, production controller runs,
audio preservation and actual native capture acceptance remain separate gates.
