# Current installed review, 2026-10-28

The earlier grass pass is already present. [Current 2D/native review](../revisions/route-grass-v01/current-r07-audit-v01/README.md) verifies eight outward meadow grass steps and a dry sandbank return. Direct current collision count is192walking grass tiles; historical196below is superseded. Active selection is preserved. Earlier pack claims below are historical.

# Crystal-reference meadow and sandbank return, 2026-10-28

R07 now brings more encounter habitat onto the outward walking trail, using
Crystal Route42's lake-and-grass composition. The main meadow extends north and
south, including its lower bank corner; three smaller patches connect to paths.
Total encounter habitat increases from 120 to 196 walking tiles.

The outward journey requires eight grass steps through the expanded meadow,
before and after Cut. The existing sandbank ledge keeps a dry return that avoids
those bands. Its ordinary 119-step return, both crossings, fishing landing,
Cut-gated cache, actors and resources stay intact.

The named review pack passed exact preservation, native visual review, eight
trainer victories with spent PP, six wild battles, all pickups, native Cut/Surf,
audio and Save/Continue. See [the current runbook and explicit fixture / identity
limits](../revisions/route-grass-v01/R07.md). No default game pack is replaced.
Earlier source/native milestones below remain historical.

# Current garden/pine revision

This route now matches the471-map `minnesota-canopy-review.crystalpack`. See
[the current sight, preservation and native-input proof](../playable/garden-and-pine-revisions.md).
Earlier source/native milestones below are retained as history.

# Leech Lake Approach — hand-authored outlet trail

## Executed native populated journey — 2026-10-26

The actual warm run in `.local/leech-lake-populated-current.log` **passes** `leech_lake_controller_populated_round_trip_battles_resources_cut_and_continue`: one test passed in **2.44 seconds**. It executes a continuous south/north/return trip with all eight authored trainer parties, selected-move PP, money/defeat flags, all four pickups including the actual Cut-gated Ether, both crossings, required meadow, native seam directions, sandbank hop/reverse rejection, every resident/sign and full party/bag/money/flag Continue. Dialogue now reads every real LCD page through input. The explicit level-60 team, CUT and obedience badges are setup; this is not earned BADGE_6/Q04, regional blackout or balance acceptance. Existing independent Cut denial/reset and Surf/BUOY/seam tests remain separate passing mechanics evidence.

The route follows a wooded lake outlet from Brainerd to Leech Lake. The outward
trip crosses a broad southern causeway, follows a scalloped pine peninsula
through a short wet meadow, crosses a narrower northern footbridge, and reaches
the west lake bank. A low native sandbank gives the southbound return a different
route. A separate fishing landing and a sheltered pine spit provide destinations.

The published route is **32×48 metatiles**, two 32×24 exterior maps. It replaces
the repeated 64×120 district reserve. Its extent follows the two crossings and
return journey; this is not a requirement that every Minnesota route shrink.
There are no enterable facades or interiors on this route.

[Whole current source render](../renders/leech-lake/whole-route-before-d0bab63f.png),
[player-scale views](../renders/leech-lake/viewpoints.png),
[directed return overlay](../renders/leech-lake/ledges.png).

## Authoritative data and scope

- [MN_R07.json](MN_R07.json): exact actors, dialogue, parties, flags, pickups,
  viewpoints, encounter tables, ports and movement contracts.
- [MN_R07.layout.json](MN_R07.layout.json): explicit named terrain strokes and
  original source block placements. This contains the whole route, not a crop.
- [review_leech_lake.py](../tools/review_leech_lake.py): read-only source renderer
  with physical checks. It does not generate geography or write a crystalpack.

The original Johto 128-block art remains unchanged. The route reuses the bounded
`mn_headwaters` extension: `80` is native Kanto water below the pier edge, `81`
is its south rail, and `82` is the already verified reordered north rail. The
southern causeway and landing have ordinary native Johto `01` deck blocks. These
are deliberately light railed causeways; this map does not promise a new timber
boardwalk sprite or canoe-rack asset.

Only this route's files are authored by the R07 agent. The integration owner must
update shared `world.json`, route registrations, inventories and atlas labels.
Source checks and these renders do not establish native playability or final
visual acceptance.

## Journey and local discoveries

From the south arrival the road passes the rest pocket, then narrows between
pines. The first crossing leads east onto the peninsula. Its south flower point
and grass spur let the player investigate before following the turning trail
north. Opposing coves constrict its middle; the meadow at the neck is unavoidable
on the ordinary outward journey. A sign and local explain the dry route.

After the meadow, a short northern spur reaches Nell and the Great Ball between
sheltering pines. The northern crossing is only two gameplay tiles wide. It
returns to the west bank beside the bird hollow. The west road branches toward
the signed sandbank and toward the northern landing. The landing has Silas, a
closed end, and rails over actual water. Northern forest shoulders tighten the
last approach to Leech Lake.

All eight trainer identities and original parties remain, with individual names,
classes, source sprites and short place-specific dialogue. They stand in passing
bays or optional destinations, with one-tile sight. They do not accidentally
seal the walking route. Four once-only item identities retain their original
contents and flags. The native sandbank is the return shortcut. A single sealed bird hollow now
uses Cut to gate its Ether cache; this is optional and opens no route bypass.
No mandatory Cut or Surf is introduced. Seven incidental walk-around Cut-tree
quadrants were replaced with ordinary native canopy of identical initial
passability.

## Physical contract

Coordinates below are route-global **16px gameplay tiles**, except where marked
metatiles. Every actor's exact position and dialogue is in the JSON dossier.

| Feature | Exact contract |
| --- | --- |
| Brainerd gate | South metatile `(10,46)`; arrival `(10,44)`. In `MnR07C00R01`, local core departure `(20,44)`, arrival `(20,40)`. |
| Leech Lake gate | North metatile `(10,1)`; arrival `(10,3)`. In `MnR07C00R00`, core departure `(20,2)`, arrival `(20,6)`. |
| Story requirement | Preserve `BADGE_6` and quest `Q04`; actual hub departure checks and earned progression remain integration work. |
| Chunk seam | `MnR07C00R00` south ↔ `MnR07C00R01` north, offset zero. Walk both the west sandbank approach and the east meadow seam in both directions. |
| Southern causeway | Metatile columns 13–21, deck row34; north rail row33, south rail row35, water row36. Walking line core `(24,69)` through `(45,69)`. |
| Northern footbridge | Columns12–21: `82` at row18, `81` at row19, water at row20. No all-floor middle block. FLOOR rows37 and38 give exactly two-core-tile walking width between the native rails. Walking line `(22,37)` through `(45,37)`. |
| Fishing landing | Columns16–22, deck row7; north rail row6, south rail row8, water row9. Approach line `(28,15)` through `(45,15)`; end core `(47,15)` is WALL. Bank rail has its deliberate opening at metatile `(15,7)`. |
| Required meadow | Core rectangle `[44,44,56,48)`, all TALL_GRASS. At least four grass steps between endpoints; no dry bypass. |
| Sandbank drop | Native `56` at metatiles `(8,27)` and `(9,27)`: HOP_DOWN above WALL. Approach `(17,53)`, hop cell `(17,54)`, landing `(17,56)`, reverse obstruction `(17,55)`. Side walls `(15,55)` and `(20,55)` close the ends. |
| Ordinary return | Landing → approach has 119 ordinary walking steps in current static geometry, excluding hops. Southern crossing passage `[26,66,28,72)` is necessary. The drop saves116 walking steps. This geometry includes unfought trainer areas; production battle/pacing checks are separate. |
| Uphill bank access | Native ramp `74` at metatile `(8,21)`; walk it in both directions before testing the drop. |

Removing either crossing's actual walking quadrants disconnects the ordinary
south-to-north journey. The reverse trip may use the explicitly directed drop;
a reverse hop must fail. The upper footbridge's `deck_blocks` list is intentionally
empty: its two walkable rows belong to the adjoining native north/south rail
blocks. Do not replace that with another broad all-floor deck during integration.

## Actor and resource inventory

| Identity | Core position | Role |
| --- | --- | --- |
| T01 Sivren | `(13,83)` | Southern rest Fisher; WOOPER27/POLIWAG27 |
| T02 Dean | `(19,63)` | West return lane Camper; POLIWAG27/SKIPLOOM27 |
| T03 Fern | `(55,65)` | Southern peninsula Picnicker; SKIPLOOM28/PINECO28 |
| T04 Oscar | `(45,51)` | Middle inlet Bug Catcher; PINECO28/ODDISH28 |
| T05 Nell | `(51,31)` | Northern pine-spit Picnicker; ODDISH29/MARILL29 |
| T06 Paul | `(13,31)` | West bird-bank Bird Keeper; WOOPER29/YANMA29 |
| T07 Silas | `(41,15)` | Fishing-landing Fisher; POLIWAG30/WOOPER30 |
| T08 Wren | `(13,13)` | Northern picnic Picnicker; SKIPLOOM30/POLIWAG30 |
| ITEM01 | `(11,79)` | SUPER_POTION; southern rest pocket |
| ITEM02 | `(49,27)` | GREAT_BALL; northern pine spit |
| ITEM03 | `(25,13)` | REPEL; open northern bank |
| ITEM04 | `(13,23)` | ETHER; sealed Cut-gated bird hollow |

Trainer flags remain `EVENT_MN_R07_TRAINER_01` through `08`; pickup flags remain
`EVENT_MN_R07_ITEM_01` through `04`. Three local actors and seven readable signs
have reachable same-map approaches. NPCs are ordinary dialogue here; none is
claimed as a functioning healer, shop, badge gate or quest completion service.

## Fast source review and evidence

Run from the repository root, using the existing external/ignored pack:

```sh
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_leech_lake.py
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_leech_lake.py --serve
```

The watcher is at `http://127.0.0.1:8777/` in this session. A measured earlier warm source
revision rendered in197ms and appeared through `/state` in359ms. The new complete
pre/post-Cut review does both full renders and collision passes, currently
about468ms in a one-shot run. The complete watcher pass measured450ms with
source refresh observed in517ms. Pack
load/hash checking happens once in watcher mode. Generated images stay ignored.

Static checks currently pass: both endpoints, all resolved event approaches,
all thirteen player views, source classes/species/items, two
necessary protected crossings, one closed landing, grass constraint and directed
return. Current images are `whole-route-before-d0bab63f.png` and
`whole-route-after-7b6656ff.png`, with 38 distinct native block recipes in the
initial state. Counts establish coverage, not artistic acceptance.

Visual inspection used the original Johto metatile sheet, authentic Route42,
the complete route, the player-view sheet and enlarged source viewports. The
first pass exposed a broad western reserve, nearly identical white bridge
aprons and a rectangular green peninsula. The revised source closes unused
shoulders with forest, keeps larger turnouts only by activities, gives the
northern crossing adjoining narrow rail blocks, cuts real shoreline coves into
the peninsula, and moves an existing trainer/reward onto a sheltered pine spit.
An earlier rectangular sandbank/boulder slab was also replaced by an irregular
small native bank and one boulder. Re-review remains appropriate after native
publication; no source render alone approves the route.

## Sealed bird-hollow Cut gate

The sole initial `CUT_TREE` quadrant is core `(15,23)`, in native Johto block
`5f` at metatile `(7,11)`. The clear approach is `(16,23)`. Pine boundaries enclose
metatile pocket columns4–7, rows8–14. The existing Ether pickup `MN_R07_ITEM_04`
uses its original once-only flag at `(13,23)`; the existing Repel moves to the
previous open-bank Ether location `(25,13)`. Item contents and identities stay
unchanged. The tree blocks an optional resource area, rather than standing beside
an already open road.

The dossier's `cut_gates` entry explicitly lists every Ether interaction approach:
`(12,23)`, `(14,23)`, `(13,22)`, `(13,24)`, plus the gated item identity.
`shortcut` is null: opening this gate does not create a faster regional route.
Native permission is a party Pokémon knowing **CUT** plus Johto badge index1,
**BADGE_2/Hive**. Those values are read from the supplied pack's actual Cut rule.
The separate regional `BADGE_6`/Q04 condition is preserved; possession of badge6
alone must not silently grant the Cut permission.

The original Cut table maps Johto `5f` to `3d`, changing only the lower-right
quadrant from CUT_TREE to FLOOR while retaining the upper-right HEADBUTT_TREE.
`mn_headwaters` declares Johto as its behavior source. Before Cut, all four cache
approaches are unreachable from the route and the declared tree approach remains
reachable. A permissive graph that also treats every WATER quadrant as walkable
still cannot reach the pocket, so an unintended Surf landing cannot bypass this
pine enclosure. There are no other incidental CUT_TREE quadrants to remove.

After the exact native `5f→3d` replacement, all four cache approaches are reachable
and34 walking cells open. General route checks then pass for every actor,
encounter patch, viewpoint, road, seam and required meadow. Both bridges remain
necessary in both source states, the landing remains protected and closed, and
the 119-step ordinary ledge return remains unchanged.

Immutable inspected source-state renders are:

- [Before Cut — full route](../renders/leech-lake/whole-route-before-d0bab63f.png)
- [After Cut — full route](../renders/leech-lake/whole-route-after-7b6656ff.png)
- [Before Cut — gate viewport](../renders/leech-lake/cut-gate-before-d0bab63f.png)
- [After Cut — gate viewport](../renders/leech-lake/cut-gate-after-7b6656ff.png)

The review page defaults to the initial state and offers an **After Cut** button.
The second state is an explicit source simulation of the pack's native
replacement, not a claim that the controller has executed Cut. The tool separately
checks the initial main route, tree approach and sealed cache, then runs the
shared general physical checker on the exact post-Cut geometry.

Native denial without CUT, denial without BADGE_2, successful prompt/removal,
actual pocket traversal and Ether pickup/use remain production tests to run.
Confirm save overrides and the actual map re-entry/Continue/callback behavior
before claiming any permanence. This source work does not promise that a tree
stays removed across native reset rules; the integration owner must record and
verify the intended native reset contract. Saving cannot duplicate or restore the
once-only Ether reward. The local source checker also rejected three deliberately
invalid in-memory variants: an opening beside the tree, an extra decorative
Cut-tree quadrant, and an incorrect replacement block.

## Complete Surf boundary

The eastern lake previously reached the unconnected right map edge. The current
source closes it visibly with the original Johto blue-water rock recipes:
`32` at metatile `(31,12)`, `36` at column31 rows13–38, and `3a` at `(31,39)`.
The full outer core line `x63,y24..79` now has **BUOY** collision:56 blocked
quadrants. These are the authentic water-rock graphics, not brown land boulders
placed on water. The north/south caps fit the continuous east-facing rock chain.
The native movement code maps generic BUOY (`0x27`) to Wall terrain, including
Surf movement; actual blocked controller input is still a production test.

Only these three relevant source block IDs were added to
`additional_water_blocks`, so adjacent water keeps its water art instead of
acquiring a false brown shoreline. There is a continuous interior WATER lane
at `x62,y25..78`. Every outer edge—north, south, west and east—was checked at core
resolution; zero WATER/WHIRLPOOL/WATERFALL quadrants remain on any unconnected
outer edge. The closure adds no walking terrain and changes no Cut/cache,
necessary-crossing, forced-meadow or ledge-return contract.

Seventeen WATER-to-WATER column pairs remain across global rows47/48, including
central bay `x31` and the eastern water lane `x61`. These may remain open only
with the actual reciprocal `MnR07C00R00` south ↔ `MnR07C00R01` north, offset0,
connections. The checker inspects both native map-attribute connection records
when a published R07 pack is supplied; a source reservation is not reported as a
verified neighbor. Current base-pack review reports
`published_water_neighbor_verified:false` because R07 publication remains with
the integration owner. Reject a half-published route or missing reverse neighbor.

[The revised whole source map](../renders/leech-lake/whole-route-before-d0bab63f.png)
and [land-side boundary view](../renders/leech-lake/viewport-lake_boundary.png)
were visually inspected, including the unchanged lake/causeway composition.
The land viewpoint `(59,65)` shows the rock closure inside the ordinary player
camera. Also capture actual Surf views beside the rocks and both seam lanes
after native publication. Three deliberate invalid boundary variants—an open
WATER hole, a west-facing rock tile and a land boulder—are rejected by the
source checker.

## Native integration and remaining regional coverage

Current publication uses group52, `MINNESOTA_LEECH_LAKE`, `MusicRoute30`, native
`mn_headwaters`, with exactly `MnR07C00R00` and `MnR07C00R01`. Keep all base maps, graphics, palettes, audio, HOME
and unrelated content unchanged. There are no R07 room or warp-pair reservations.
The shared typed design/layout route allowlists must include these exact 32×48,
1-column/2-row dimensions. Every named source stroke already uses the existing
schema; no map-generator inference or new asset exporter is required.

The Rust integration owner published R07, and exact source/native comparison
passes for both maps and their 1,536 exterior blocks. Recheck future revisions with:

```sh
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_leech_lake.py \
  --pack content-packs/minnesota-leech-lake.crystalpack --verify-published
```

The populated journey and existing independent field-move tests now cover the
local route mechanics described below. Remaining acceptance work is earned
regional travel: real Brainerd/Leech Lake reciprocal hub arrivals, Q04/badge
departure authority, blackout/recovery, full pacing/balance and final user visual
judgment. The new journey collects resources but does not consume Ether through
Pack to demonstrate PP restoration. Its incidental-wild handler executes real
battle input when encounters occur; the successful captured log does not expose
a wild count, so no minimum wild-battle coverage is asserted. R07-specific native
MCP/TUI and source-matched headless captures remain separate from another zone's
passing frontend suite. Do not infer these results from a source/static check.

## Native publication evidence

Root published `content-packs/minnesota-leech-lake.crystalpack` with the Rust `pack_leech_lake` builder. `review_leech_lake.py --pack content-packs/minnesota-leech-lake.crystalpack --verify-published` passes: 2 authored maps and 1536 exterior metatiles match the native pack, along with outdoor objects and room rows where present. Open water seams have actual reciprocal offset-zero native neighbors. Independent controller Cut/Surf mechanics and the populated eight-trainer journey now pass; earned regional progression and final visual acceptance remain separate work.

## Production field-move results

The combined `content-packs/minnesota-northern-lakes.crystalpack` contains 401 maps (R05–R08 plus the preserved source). Six production-controller field-move regressions pass: all three Cut gates independently reject missing CUT and a missing Hive badge, perform the actual catalog replacement, allow walking to the sealed reward, and preserve the item/flag through Continue. Native temporary vegetation resets on Continue; cutting again from inside permits exit. R05/R06/R07 mount Surf through A/Yes, reject movement against real BUOY quadrants, and cross each actual internal river/lake seam in both directions. This explicit Mew 60/position/badge fixture proves mechanics; it does not establish route battle balance or earned regional progress.

```sh
CRYSTAL_CUT_ROUTES_TEST_PACK="$PWD/content-packs/minnesota-northern-lakes.crystalpack" \
CARGO_TARGET_DIR=target/runtime-fps cargo test --profile wasm-dev --locked \
  -p crystal-bevy --lib cut_gated_routes -- --ignored
```

The new route composition/source-preservation regression passes, including rejected fake gate targets, replacements, unknown actors and incidental decorative Cut trees. All 377 audio PCM hashes/frame counts/loop ranges match in the combined pack. R07’s complete populated local journey now passes as recorded above. R07-specific MCP/TUI, earned regional progression and final visual acceptance remain separate; unrelated routes and services have their own runbooks.

## Executed continuous populated controller journey

The focused leaf `crates/crystal-bevy/src/bevy_shell/tests/leech_lake.rs` adds `leech_lake_controller_populated_round_trip_battles_resources_cut_and_continue`. Root registered it as `#[path = "tests/leech_lake.rs"] mod leech_lake;` and the actual native warm rerun passes. `CRYSTAL_LEECH_LAKE_TEST_PACK` selects the published native composition. This author ran no competing Cargo build/export; scoped Rust formatting passes. The completed log reports one pass in 2.44 seconds.

The explicit initial fixture is CHRIS at southern arrival core `(20,88)`, with native-created level 60 Typhlosion, Feraligatr and Mew, a native-legal CUT in Mew's initial move set, and eight obedience badge bits. These are setup, not earned badges/HM/party or balance evidence. After setup, movement, text, all battle choices, Cut, pickups and menus use `VisibleShellController::press`; only the test path planner is updated after native Cut/pickup observations. No gameplay position, party, block, resource or flag is rewritten.

The executed continuous trip visits both actual exterior maps, crosses the southern causeway and separate northern footbridge, pays the unavoidable meadow grass and crosses the real y47/48 seam in both directions. It deliberately fights **all eight authored trainers**, verifies their exact two-opponent species/levels, selected-move PP mutation, actual money reward and individual native defeat flags. Incidental ordinary wild encounters are resolved through real battle input rather than disabled or bypassed. It collects Super Potion, Great Ball and Repel, actually cuts 5f→3d to collect the sealed Ether, interacts with all three residents/seven signs, reaches the north arrival and returns through the actual sandbank hop with blocked reverse. Saved party, bag, money, all eight trainer flags and all four pickup flags survive Continue; talking to defeated Sivren does not restart the battle.

The current source/native review CLI passes **two maps, 1,536 blocks and 15 exterior objects** against the canonical composition. The whole `d0bab63f` and player-camera sheet were inspected before authoring the fixture. **R07 has no interiors or doors**: the new test audits exactly two exterior modules with no invented room/warp, rather than fabricating room coverage. Existing separate Cut permission/reset and Surf/rock/seam tests remain their own evidence. Even a passing populated local round trip does not prove the free regional connections, earned BADGE_6/Q04 progression, balanced level curve or final visual acceptance.

```sh
CRYSTAL_LEECH_LAKE_TEST_PACK="$PWD/content-packs/minnesota-boundary-waters.crystalpack" \
  cargo test --locked -p crystal-bevy \
  leech_lake_controller_populated_round_trip_battles_resources_cut_and_continue \
  -- --ignored --nocapture
```

The first actual warm run compiled and reached Warden dialogue, then failed before any trainer battle: the test read only the controller's current LCD page, so the unchanged `railed crossing` assertion missed the following page. The owned dialogue reader now advances through real A presses, records each actual visible page, requires complete page-index coverage for every emitted text stream and normalizes only observed whitespace. It retains movement rejection while text owns input and the original source-wording assertions, with compact actual text/page diagnostics. Root’s actual repaired rerun passes. The earlier first-page-only failure remains historical; the pass comes from the completed native test, not from that failed run.
