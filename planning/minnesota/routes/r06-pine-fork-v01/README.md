# Pine Country Road: wooded downhill fork

The complete 32×44 route keeps its lake, cabins, Super Rod guru, pier, bridge,
five trainers, original northern hop and Cut-gated Ether lookout. The northern
lakeside fork now has a real two-lane downhill ledge, closed pine flanks and two
grassy uphill returns. Pine cover divides the broad approach to the shore, and
the lookout's boulder-built corner becomes a continuous native rim.

[Whole route](../../renders/r06-pine-fork-source-v1471/after-whole-original.png)
· [whole before/after](../../renders/r06-pine-fork-source-v1471/before-after-whole-original.png)
· [ten native cameras](../../renders/r06-pine-fork-native-v1473/production-decisions.png)
· [four actual played views](../../renders/r06-pine-fork-played-native-v1479/actual-played-cameras.png)
· [connected state](../../renders/r06-pine-fork-state-v1477/full-state-native-current.png).

## Source and publication

[`source-v04.json`](source-v04.json), SHA256
`28a41698794d63140e17326b2f0c52e2e1bbc137e6fd6733278ae68089581c48`,
binds both complete native chunks to the selected v10 parent. Thirty-nine
metatiles change in `MnR06C00R00`; the complete southern chunk remains exact.
Source pixels, palettes, every actor/program, regional fauna, fishing, rooms,
resources, attributes and all 742 other maps are preserved exactly.

The verified standalone child is
`content-packs/minnesota-r06-pine-fork-v01.crystalpack`, SHA256
`d3c16929aade3df2e4b26406dd1bd1c0ac61f05b05be5f569c41e1657c3899d9`,
content identity
`834d4be07c45079fdc81f1777856a37058d064b4188f548e31616684436da5e3`.
The twenty-two-stage master reproduces those bytes as
`content-packs/minnesota-reviewed-maps-v11.crystalpack` and reproduces v10 exactly.
`world.json` now selects `r06_pine_fork_revision`. The exact preceding world
and evidence hashes are retained in
`renders/selected-r06-pine-fork-v1485/receipt.json`. Broader campaign work and
user visual approval remain open.

The historical `MN_R06.json` dossier and populated `pine_country.rs` fixture are
not regeneration authority for current fauna. The old fixture uses a supplied
nonregional test partner and is not used for this revision. All current route
parties and encounters remain the selected Minnesota originals. The new tests
use an original Borealynx; the existing Cut test uses a legally compatible
Voyagear. Johto/Kanto outside Minnesota stays unchanged.

## Gameplay contract

All coordinates below are 16px walking tiles. Route y is global: chunk row1
starts at y48. Source `metatile` coordinates are local 32px cells.

| Feature | Exact contract |
| --- | --- |
| New downward lanes | x42 and x43: upper y25, native hop lip y26, reverse wall y27, landing y28. The downhill path crosses zero grass. |
| Ordinary return | Both lanes need at least six grass steps. The short ordinary return is 11/9 movement steps for x42/x43. The six-step price remains with the original lookout tree intact or cut; normal native grass-cutting behavior is unchanged. |
| Pine flanks | Native 05 pines at metatiles (20,13)/(22,13) close the sides. Down from (41,25)/(44,25) and up from (41,28)/(44,28) are blocked; the lip cannot be entered sideways from a free bypass. |
| East return | The existing grass pocket at metatiles x22–25, y12–13 becomes the short uphill choice around the right pine. Its lower-left cell is the flank pine; the open field wraps around it. |
| West return | Metatiles x16/17, y10–12 form a six-walking-row band beside the lookout cliff. It connects the longer woodland approach without a dry parallel lane. |
| Original northern hop | x44 and x45 retain upper y9, lip y10, reverse wall y11 and landing y12. Both downward lanes, reverse denial and the original return of at least eight grass steps remain intact. |
| Ledge versus cliff | The new drop is on ordinary ground, several metatiles from the lookout cliff. No ledge is placed on a cliff face or rim. The lookout uses its original stair; a road is not painted through a side face. |
| Existing Cut gate | Core (29,32), metatile (14,16), native 5b→3c replacement. Approach (29,33), stair (29,31), upper tread (29,30), Ether approach (13,29), reward actor at (13,27). Neither the new rim nor the pine cover opens a pre-Cut entrance. |
| Ether and reset | Original `MN_R06_ITEM_04` / `EVENT_MN_R06_ITEM_04`, one ETHER. Missing move/badge denies Cut; native Cut opens the stair and resource. The claim survives Continue while native temporary vegetation rebuilds, and the player can cut from inside to return without a duplicate item. |
| Lookout composition | Native caps/corners straighten the northwest rim. The existing interior boulder moves away from the right rim and clear of the trainer. Pines and grass add cover inside the existing gated clearing. No new actor or quest is invented. |
| Needle Cabin | Active/passive exterior (9,61)/(9,62), local row1 (9,13)/(9,14). Its full assembly, room, treatment program and resource remain exact. |
| Super Rod guru | Active/passive exterior (37,39)/(37,40). Original house, gift, repeat behavior and bank access remain intact before Brainerd's northern-lake progression. |
| Bridge and pier | Both bridge lanes y66/67, with clear approaches x25/x38, remain walkable. The shore pier, rails, angler and all original water/shore collision are exact. |
| North road | Actual R00 north link to `MnBrainerd`, offset +5; route lanes (42,0)/(43,0) arrive at (32,47)/(33,47). |
| South road | Actual R01 south link to `MnStCloud`, offset −4; global (16,87)/(17,87), local (16,39)/(17,39), arrive at (24,0)/(25,0). |
| Regional trip | Static native movement prices change from 14 to 17 grass steps northbound; southbound remains six. The new hop makes the return trip different. These are collision costs, not an encounter or balance guarantee. |

The new source retains every original Cut quadrant, stair, building cell,
warp, river/lake tile, rail, external edge and regional script except the explicit
lookout rim/boulder repairs. Pine barriers contain no incidental Cut collision.
The 108-map connection/water report is identical to v10 outside pack identity:
three direct-seam groups, zero overlaps or inconsistent cycles, 1,796 reciprocal
water lanes and the same previously triaged 110 raw edge mismatches. Neither
shoreline barriers nor ice close any early casting bank.

## Evidence and limits

- `renders/r06-pine-fork-preservation-v1472/receipt.json` checks the complete pack,
  restoring the one terrain payload, encoded mirror, marker and identity to
  recover the entire v10 parent exactly.
- `renders/r06-pine-fork-controller-v1474/` contains three successful new native
  courses: both new hops/closed sides/grassy returns/Continue, original northern
  hops/both bridge lanes/cabin approaches/chunk seam, and a real encounter/Run
  in the newly planted west grass. Its later seam harness failure is excluded.
- `renders/r06-pine-fork-cut-fishing-v1475/receipt.json` contains three passing
  courses: original Cut permissions/Ether/native reset/inside return plus actual
  awarded-rod fishing and all three guru door/gift/repeat journeys. Early fishing
  uses zero badges and no field moves.
- `renders/r06-pine-fork-road-input-v1483/receipt.json` passes the fourth new
  native course: both lanes of both actual town seams, both directions, stable
  64-frame arrivals and Continue. The supplied eight badges and matching engine
  flags are initial QA qualifications, not earned progress. Earlier v1480/v1482
  fixture/input failures are retained and excluded.
- `renders/r06-pine-fork-stdio-v1484/receipt.json` passes three real processes
  and 938 calls with byte-empty stderr: new/original hops, grassy returns,
  fresh-process Continue and both lanes of both actual town seams. Inputs and
  source saves stay unchanged. That completes seven distinct native courses
  across v1474's first three, v1475's three and v1483's one.
- `renders/r06-pine-fork-native-v1473/capture-progress.json` checks ten complete
  640×576 frames with zero differing pixels. All camera placements are on
  unoccupied Floor/Tall Grass. The whole route, all native frames and four
  actual saved night/rain views were personally inspected.
- `renders/r06-pine-fork-early-banks-v1478/receipt.json` retains all ten Winona
  and six R01 public dry-bank tiles. Fishing maps, tables, gifts and shorelines
  are unchanged.
- Six standalone and five master publisher refusal cases reject; seven source-
  index refusal cases reject and v10 source compatibility passes. The current
  index binds 38 native layouts, ten route groups, 27 sources and both new ledge
  lanes while checking the original Cut gate.
- The master and compatibility publication, affected-crate all-target Clippy
  and all 602 audio PCM/hash/frame/loop checks pass. Owned formatting passes;
  workspace fmt retains unrelated offset-allocator differences.

Geometry fixtures declare prior trainer flags and local positions. Wild tests
supply one original Minnesota companion and Smoke Ball; the Cut course declares
a legally learned move and badge prerequisites. Road QA declares its own eight
badges with consistent native engine flags. All later movement, dialogue,
resources, encounters, field moves and Continue use production input. These
fixtures do not earn route access or prove unboosted campaign balance. Existing
saves are not migrated; broader map quality and user visual approval remain open.

## Fast iteration

```sh
PYTHONPATH=.local/ely-backroute-python python3 \
  planning/minnesota/tools/review_r06_pine_fork.py \
  --source planning/minnesota/routes/r06-pine-fork-v01/source-v04.json \
  --output planning/minnesota/renders/r06-pine-fork-next-v01
```

Use a fresh output each time. The complete source composition/check/render took
0.33 seconds after pack loading. [`native-cameras-v01.json`](native-cameras-v01.json)
records explicit valid player placements for the production camera check.
Publication uses Rust `revise_r06_pine_fork`; pack generation and Cargo remain
serialized by `.local/minnesota-cargo.slot.lock` with the pinned toolchain,
`CARGO_TARGET_DIR=target/verdigrim-scene` and `RUST_MIN_STACK=67108864`.

All prior source variants, produced images, conversions, raw native frames,
failed harness logs and standalone packs remain retained. The first inspection's
1344px image cropped the last two metatile rows; `whole-current-full.png` in
v1469 corrects it, and every authoritative source render is the full 1408px.
The v01/v02 designs failed the return-cost check before rendering; v03's whole
render exposed the bad inward cliff corner, corrected in v04 without a new tile
or palette. No image has been edited to hide a terrain or collision defect.

The executed standalone publisher is
`.local/r06-pine-fork-build-20261101T042823/publisher`; the master and its logs are
in `.local/r06-reviewed-master-build-20261101T043507/`. The final road input
controller and affected-crate Clippy are retained in
`.local/r06-road-input-controller-20261101T045056/`. To repeat native gameplay
with a fresh ignored save directory:

```sh
CRYSTAL_WINONA_TEST_PACK="$PWD/content-packs/minnesota-reviewed-maps-v11.crystalpack" \
CRYSTAL_R06_PINE_FORK_REVIEW_DIR="$PWD/planning/minnesota/renders/r06-next-controller/native-saves" \
RUST_MIN_STACK=67108864 CARGO_TARGET_DIR=target/verdigrim-scene \
  cargo test --locked -p crystal-bevy --lib --features location-tester \
  pine_fork_controller_ -- --ignored --nocapture --test-threads=1
```

Acquiring the shared Cargo lock and choosing fresh paths remain required.
The first road harness assumed a single pulse after a seam and used a route-only
position helper while the player was still in town. Its qualified cold-save
fixture also omitted the engine-flag mirrors for its supplied badge bits;
Save/Continue correctly normalized them. The final fixture initializes both
representations before input and uses bounded native travel with exact arrival
assertions. No controller, collision or story gate was changed to satisfy it.
