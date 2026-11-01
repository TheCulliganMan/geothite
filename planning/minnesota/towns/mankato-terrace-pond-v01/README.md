# Mankato terrace and pond garden

This whole-town revision replaces the broad, mostly empty clearing with an
upper Gym/clinic court and a southern shop/garden loop beside a contained pond.
The notebook, gardener and coach remain at their exact original positions.
The original low ledge, landing and ordinary uphill-return rectangle remain
exact. The new water gives the existing native Pond rod table a usable habitat;
two ordinary dry casting banks stay open without Surf or Cut.

This town was selected in the local review pack
`content-packs/minnesota-reviewed-maps-v06.crystalpack`, byte-identical to the
standalone `minnesota-mankato-terrace-pond-v02.crystalpack`, SHA256
`f4cd87d34fba9696d67c9658052976b9f7ba2d9689371603fa358685b5592547`,
content identity `b1de4770a0c928a48ded99396a762b09bef835d3d948ce6660a6b0de493f6610`.
The subsequent Brainerd v07 selection preserves this complete town and all its
programs exactly. Resolve `world.json` for the current local review. This
document does not claim an earned campaign, browser release or save migration.

- [Whole installed town](../../renders/mankato-terrace-pond-installed-v1375/after-whole-original.png)
- [Whole before/after](../../renders/mankato-terrace-pond-installed-v1375/before-after-original.png)
- [Current immutable source](source-v04.json)

## Whole-town design and original gameplay

The native footprint remains 24×18 metatiles. All154 changed metatiles use
original Johto pixels and palettes; no custom tileset, recolor, ice rectangle,
Cut tree or new progress gate is added. The inhabited upper court puts the
clinic beside the Gym. The shop serves the notebook garden and pond rather
than a distant empty lawn. Pines close unused shoulders. Wider forecourt space
serves the two original facades and coach; ordinary through lanes remain two
walking tiles wide.

| Useful native FLOOR walk with actor bodies | Before | After |
| --- | ---: | ---: |
| Clinic → Gym | 23 | 11 |
| Gym → Mart | 43 | 33 |
| Clinic → Mart | 20 | 22 |
| East road arrival → clinic | 19 | 31 |
| East road arrival → Mart | 23 | 17 |

The east-arrival care walk is twelve steps longer, while repeated Gym/care
trips are shorter and share one useful court. This is a deliberate placement
tradeoff, not a claim that every trip improves. The original native arrival at
(40,18) stays clear and connected. Terrain alone does not prove enjoyable pace;
the complete town and native player views require visual inspection.

The original ten HOP_DOWN quadrants at x10–19/y18 remain exact, with clear
landings at y20. Metatile rectangle (4,8,9,4) is preserved byte for byte,
including the existing ordinary uphill return and side closures. There is no
ledge added to a cliff. Keep the original coach's advice and Gym2 terrace course.

## Exact anchors, connections and story

Coordinates below are16px walking tiles. Native room identities, interior
arrivals, return indices, all six active room carpets and service programs stay
unchanged. Only the clinic/Mart exterior positions move.

| Destination | Active door | Passive exterior return | Town warp indices |
| --- | --- | --- | --- |
| Gym, unchanged | (10,11) | (10,12) | 5 / 6 |
| Clinic | (21,11) | (21,12) | 1 / 2 |
| Mart | (33,21) | (33,22) | 3 / 4 |

The clinic's complete original2×2 assembly moves from metatile (15,5) to (10,4).
The Mart's complete2×2 assembly moves from (18,12) to (16,9). Gym art is exact.
The Gym post moves (16,12)→(16,10), clinic post (34,12)→(26,10), and Mart post
(40,26)→(38,20). Garden post stays (24,28). All occupy native solid post
quadrants with public reading approaches.

Coach `MN_MK_COACH` remains (16,14), gardener `MN_MK_GARDENER` (29,25),
notebook `MN_MANKATO_NOTEBOOK` (26,25), and the two northern wardens
(24,2)/(25,2). Their exact native programs and event identities are cloned.
The notebook requires the Care Badge and only records
`EVENT_MN_PRAIRIE_NOTEBOOK` after its text is acknowledged; it grants no badge.
The northern departure still requires the native Hive/Prairie Badge. The new
pond does not connect to another map or bypass that gate. Free inbound road
behavior remains unchanged.

East connects to `MnR02C00R00`, offset−6: town (47,18)/(47,19) pairs to route
(0,30)/(0,31). North connects to the8-metatile-high `MnR03C00R01`, offset3:
town (24,0)/(25,0) pairs to route (18,15)/(19,15). All mouths and offsets stay
exact. The public graph excludes only the four intentional warden-pocket floor
tiles above the northern bodies; it does not silently remove a real street.

## Pond banks, rails and passing room

The52 WATER quadrants form one connected pond entirely inside x32–41/y24–29.
It has no outside water edge or seam; visible shore and surrounding land contain
it. The native `FISHGROUP_MN_POND`, all three rod tables and their original
Minnesota fish remain exact. No rod, field move, fishing permission or catch is
granted by publication.

| Dry casting bank | Water faced | Direction |
| --- | --- | --- |
| (31,26) | (32,26) | Right |
| (42,26) | (41,26) | Left |

Selected retail/south margins have native rails; the western and eastern banks
stay open. The south rail run is **40,41,41,41,42** at metatiles x16–20/y15: a
left corner, three straight spans and a right corner. Do not repeat corner40
across the span. The original rejected pack/render remains retained.

Public rectangles checked with all five actor bodies present are: upper court
x6–25/y12–13; west pond lane x30–31/y26–33; east pond lane x42–43/y22–33;
south walk x30–43/y32–33; retail rail passing lane x36–41/y21–22. These are
walking coordinates, not metatile widths. The player can pass the notebook and
gardener along the clear lower promenade.

## Fast source loop and Rust publication

Parent is selected v05, SHA256
`79e431d36fd47c02904b8c9a92d7871a4d9cead4c3e557ed6a73c98d27837e30`.
Complete native Mankato preimage, canonical JSON SHA256:
`212295edfdba36ba45dfa090a1045a512c94aa0a44b87537fbb9793d31edf9f1`.
Source-v04 SHA256:
`9d2b4bbc5aa30521eb1724a4a6e66d6f39ee86c716a547c48e3db00f9b2ec9ea`.
Earlier frozen source versions and every original/crop/conversion stay retained.
`proposal.json` is the editable working copy; it is not the publisher input.

```sh
PYTHONPATH=.local/ely-backroute-python python3 \
  planning/minnesota/tools/review_mankato_terrace_pond.py \
  --output planning/minnesota/renders/mankato-terrace-pond-FRESH
```

The source loop checks the complete preimage, original facade copies, protected
ledge/road planes, native arrival, all actors/posts/doors, two clear promenade
lanes, both dry casting banks and the contained connected water. It renders the
complete town with original art. Measured warm source composition/render takes
0.107 seconds after loading; installed mode includes loading the child. Add
`--installed content-packs/minnesota-mankato-terrace-pond-v02.crystalpack` for
the actual published rows. Each invocation uses a fresh ignored output.

```sh
CARGO_TARGET_DIR=target/verdigrim-scene RUST_MIN_STACK=67108864 \
  cargo build --locked -p crystal-assets --bin revise_mankato_terrace_pond
target/verdigrim-scene/debug/revise_mankato_terrace_pond \
  content-packs/minnesota-reviewed-maps-v05.crystalpack \
  79e431d36fd47c02904b8c9a92d7871a4d9cead4c3e557ed6a73c98d27837e30 \
  planning/minnesota/towns/mankato-terrace-pond-v01/source-v04.json \
  content-packs/minnesota-mankato-terrace-pond-FRESH.crystalpack
```

Serialize Cargo/publication and keep outputs ignored. Rust changes only the
native town, corresponding encoded/event mirrors, marker and derived identity.
An inverse replacement must recover the complete parent. Publication strictly
reads the result back. The independent complete-root audit in
`renders/mankato-terrace-pond-preservation-v1376/` preserves all742 other maps,
all original programs, rooms, actors, species, encounters, fishing tables, art
and audio. All602 audio PCM/hash/frame/loop and desktop checks pass on the new
candidate. Source versions1/2 were rejected before publication for changed
protected return terrain and a blocked native arrival. Version3's repeated rail
corner was caught by actual native walking; version4 fits the complete rail.
No story condition or collision assertion was weakened to accept those mistakes.

## Current native acceptance and combined build

The [seventeen-stage recipe](../../revisions/reviewed-maps-v01/recipe-v06.json)
reproduces the exact standalone bytes; the new master also reproduces v05
unchanged. Its [source index](../../revisions/reviewed-maps-v01/sources-v06.json)
binds29 native layouts, six route groups,22 sources and24 town anchors.
Ten corrupt-index and thirteen unsafe publisher-input/output cases are rejected.
All602 audio checks, affected assets/Bevy all-target Clippy and scoped Rust
formatting pass. Workspace formatting still reports pre-existing differences,
beginning at `crates/offset-allocator/src/lib.rs:408`.

Six current controller cases pass: three doors/six carpets/four posts; clear
promenades and both registered rod banks; notebook acknowledgment/repeat;
care/PC/priced shop; the original Gym/ledge course; and native Surf over all52
water quadrants, both rail refusals, Surf-mode Continue and native dismount.
Local partners, the registered rod and the Surf case's upstream permission are
explicit setup. Subsequent movement, dialogue, battles and persistence use
production input without position, progress, party or RNG rewrites.

- `renders/mankato-terrace-pond-current-controller-1793515166376780000/receipt.json`:
  five successful town/service/story cases.
- `renders/mankato-pond-surf-current-controller-1793516047812102000/receipt.json`:
  the sixth, full-water case. Its earlier one-press dismount failure is retained;
  the native travel helper now settles to the exact bank and still asserts Normal mode.
- `renders/mankato-terrace-pond-mcp-v1379/receipt.json`: three actual stdio
  processes,403 calls, byte-empty stderr; public loop, native rod encounter/Run
  and fresh-process Continue, using the unedited played saves.
- `renders/mankato-terrace-pond-native-1793515500451056000/receipt.json`:
  five inspected native views from those actual night/rain saves.
- `renders/mankato-terrace-pond-camera-pixels-1793516557482030000/capture-progress.json`:
  five complete640×576 clear daytime fixture views, zero differing pixels.
  The whole town, this camera sheet and108-map atlas were inspected.
- `renders/selected-mankato-terrace-pond-v1380/receipt.json`: local selection,
  exact prior world and complete proof hashes. Its visual review explains the
  retained rainy-source mismatch, occupied coach fixture and camera-clamp correction.

`views-v01.json` is retained. `views-v02.json` puts the upper-court fixture one
walking tile north of the coach instead of overlapping that actor. The source
comparison mirrors the native connection-composite camera clamp at the south
edge. Its explicit10am fixture uses naturally clear scheduled weather; rainfall
remains enabled, and actual saved night/rain captures remain separate.

The full root audit preserves Winona, R01 and all guru rooms/tables. Fresh native
Old Rod gift/casting/encounter/Run/Continue and three-guru gift/repeat cases pass
on v06 in `renders/early-fishing-reviewed-v06-1793516344553674000/`.
An independent dry flood still reaches ten Winona and six R01 banks, excluding
water, ice, Cut trees and actor bodies. No badge or field move is needed to cast.
The [early-fishing contract](../../connections/early-fishing-v01/README.md)
remains a requirement for future shoreline work. The broader map/cave quality
and earned regional progression goal remains active.
