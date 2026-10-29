# Current installed grass review, 2026-10-28

[Current 2D/native audit](../revisions/route-grass-v01/current-r08-audit-v01/README.md) verifies the installed grass pass on the704-map R03 cave child, retaining the ancient mire. Both local ledge loops have4grass steps uphill and0downhill; the longer dry lake circuit remains. Four native cases and15clean stdio calls pass. Terrain and selector are unchanged. Older isolated pack claims below are historical.

# Northern Lake Circuit — current native review

## Grass and dry ledge returns — 2026-10-28

Accepted in the isolated `minnesota-route-grass-r08-v01.crystalpack`, identity
prefix `75f7b910a3f7`. Twelve additional grass metatiles connect useful trails;
the Birch climb requires four grass steps and its downhill hop stays dry. The
Cedar bank has the same local pattern, while the whole-lake road and original
dry shore walk remain available. Native movement, Cut, two trainer battles,
pickups, Save/Continue, stdio and audio checks passed. See
[R08 review](../revisions/route-grass-v01/R08.md) for exact scope and evidence.
The shared game pack has not been replaced. Older reports below are historical.

## Eastern cedar peninsula — published and checked, 2026-10-26

The current **40×48-metatile** R08 revision is published as four clipped exterior maps in `content-packs/minnesota-quality-review.crystalpack` (**471 maps**). Root's strict native read passes; pack SHA256 is `5fe3071c6242e932096912bb3195982ff5ce7e5bd4f821a94985e9d652e50424`. All377 audio PCM hashes, frame counts and loop ranges match (`.local/quality-review-audio.log`). Earlier401-map captures and the469-map populated result below are explicitly historical.

Both current production-controller cases pass in `.local/quality-r08-controller-current.log` (**2 passed,0 failed**): actual Cut move/badge denial, native removal, sealed reward collection and reload/reset; and the populated journey through two actual trainer victories, selected move PP, three ordinary resources, money and Continue. Initial Mew60/Ampharos54, badge bits, divider stream and trailhead placement are explicit fixture setup. No post-setup party, map, flag, movement-mode or RNG rewriting supplies the journey. This does not establish all eight trainer battles or earned `BADGE_6`/Q05/connected regional progression.

The bounded hand edit is one named orthogonal forest polygon: `[[25,22],[32,22],[32,21],[37,21],[37,24],[25,24]]`. Exactly22 native lawn `02` blocks become forest `05`, connecting the eastern cedar mass and separating the north/south shore corridors. Earlier layers, dimensions, ports, story, actors/parties, pickups/signs, Cut/grass/ledge/water and bridge anchors are retained. R08 has no interiors or healing service. Root accepted the source whole and paired frames before publishing.

- [Current wholeab83cf76](../renders/northern-lakes/whole-route-ab83cf76.png) and [after-Cut source viewf987e70e](../renders/northern-lakes/after-cut/whole-route-f987e70e.png).
- [Retained six-pair source comparison](../renders/northern-lakes/pine-peninsula-review/before-after-ab83cf76.png) and [corrected east-return Floor-centered67,51 comparison](../renders/northern-lakes/pine-peninsula-review/east-return-walking-before-after-ab83cf76.png).
- Actual headless640×576 engine captures: [main49,47](../renders/northern-lakes/pine-peninsula-review/native-main.png), [north50,43](../renders/northern-lakes/pine-peninsula-review/native-north.png), [south55,51](../renders/northern-lakes/pine-peninsula-review/native-south.png), [east return67,51](../renders/northern-lakes/pine-peninsula-review/native-east-return.png). All four are actual Floor positions and were inspected by sight against the source frames. The pine edge frames the two shore lanes; the northern flowers/lake edge and southern/eastern bends remain readable. No further geometry change is proposed. The640 desktop captures show a wider field than a160×144 handheld frame; Root's [derived native-camera panel](../renders/terrain-revisions/engine-review/native-camera-review.png) preserves that framing distinction.

The retained center55,47 is now forest, so it is spatial comparison rather than a legal new walking placement. East-edge67,49 was02/Floor in the exact source before snapshot and is05/Wall now; its old/new crops remain scenery context and are excluded from current walking evidence. The corrected67,51 frame is Floor in both layouts. The [separate four-route sight audit](../reviews/route-visual-audit-northern.md) records why this specific change was selected; R11/R12/R14 were not changed by this author.

Full source/static checks also pass: all thirteen declared viewpoints and ordinary actor/sign/resource approaches; required endpoints; native63→3f island gate with CUT/Hive Badge and32 unlocked tiles; Surf cannot bypass Avrin/Great Ball; both shoreline habitat choices still cost two grass steps; ledge/return, three railed crossings and all1,712 enclosed water cells remain intact. Five named valid-schema negatives reject missing Cut, a real Surf shore bypass, dry habitat, removed ledge and exposed outer water. These are static evidence alongside, not substitutes for, the current native input tests.

Frozen layout SHA256: `f7e24ee36e5f174929dd834483bcc7df52d18779d0fd37274cf8b78aa43ec9ca`. Current dossier SHA256 after status-only evidence sync: `3c8bf84a6958fc28f4df0fc5f4afb68481e2d52e241ea1f102acba9955a7a721`. Exact pre-edit source remains at ignored `.local/r08-visual-before.layout.json`; all117 previous R08 renders, new originals, conversions, crops, previews, native captures and rejected passes remain archived. No Cargo/export/publication was run by this author. Final human visual approval and the full earned regional journey remain open.

## Previous native evidence — preceding terrain

### Native review and populated fixture scope — 2026-10-26

The frozen **40×48** layout is already published as four clipped exterior maps
in the **401-map** `content-packs/minnesota-northern-lakes.crystalpack`. Native
Cut prerequisites/removal/reward/reset and shared Surf boundary tests have
passed as recorded below. There are **no interiors, doors or HealParty service**
on R08; no room/rest behavior is invented by the new coverage. Earlier
publication-pending paragraphs below are historical source handoffs.

The focused leaf
`crates/crystal-bevy/src/bevy_shell/tests/northern_lakes.rs` executes
`northern_lakes_populated_journey_spends_selected_pp_collects_open_resources_and_continues`.
It **passed against the preceding terrain in the469-map composition**: `.local/northern-lakes-populated-current.log` records one passed test, zero failures. This executed result does not validate the new22-block source revision until Root republishes and reruns it.
Initial setup supplies native-created Mew60 with legal Psychic/Surf,
Ampharos54 with legal Thunderpunch, badge bits, a varied divider stream and
trailhead placement `(26,88)`. All later gameplay uses production inputs. No
party/status, map, flag, movement mode, clock or divider is rewritten after setup.

The journey challenges Norek `(21,85)` and Nell `(69,85)` through their actual
native scripts, observes their Pineco/Weepinbell and Pidgeotto/Weepinbell teams,
requires selected `PSYCHIC_M` PP consumption and money/defeat flags for each, and
revisits both defeated actors without another battle. All three ordinary
resources are collected: Ether `(59,89)`, Repel `(65,27)` and Super Potion
`(31,15)`. It visits the trail warden, fishing-pier resident and birch walker,
crosses all four native chunks and reaches the Bemidji landing `(40,6)`.
Native grass encounters remain enabled and resolve through RUN menus/text.
The passing save/Continue assertions preserve exact party, bag, money, position and the five
earned trainer/item flags. Other six trainer flags and Cut-island Great Ball
stay unset. Two victories do not establish all eight battles, battle balance,
earned BADGE_6/Q05 or connected hub progression.

Read-only source preflight checked the complete planned actor/resource paths
against actual authored collision and trainer sight; its grass-bearing legs
remain real encounter destinations. That preflight is separate from runtime
success. Root's subsequent warm execution passed as recorded above. The focused repeat command is:

```sh
CRYSTAL_NORTHERN_LAKES_TEST_PACK="$PWD/content-packs/minnesota-northern-lakes.crystalpack" \
CARGO_TARGET_DIR=target/runtime-fps cargo test --profile wasm-dev --locked \
  -p crystal-bevy --lib northern_lakes_populated_journey_ -- --ignored --test-threads=1
```

`CRYSTAL_NORTHERN_LAKES_JOURNEY_SAVE` optionally exports the actual completed
local fixture. Disposable Continue saves are removed after a successful test.

Actual production 2D captures were made with the existing Rust render binary,
the published 401-map pack, noon and headless image capture:
[fishing pier](../renders/northern-lakes/native-fishing-pier.png)
(`MnR08C00R01`, local `(63,24)`) and the widest
[Long Lake saddle](../renders/northern-lakes/native-long-lake-saddle.png)
(`MnR08C00R00`, `(41,43)`). Both were inspected by sight against the frozen whole
map. The pier reads as a closed rail-protected landing attached to encounter
grass; the saddle has legible western bridge, northern bank and eastern shore
choices. These captures support keeping the terrain frozen for this input
regression. They do not establish complete native visual or user acceptance.

The former 96×96 repeated district lattice is replaced by one hand-authored
40×48 route. All eight trainer identities/parties and four persistent pickup
identities are retained. Leech Lake, Bemidji, `BADGE_6` and quest `Q05` remain
its regional contract. The old unplaced “optional Cut return loop” is replaced
by an actual **Cut-gated island excursion** with named inaccessible targets.

[Whole route](../renders/northern-lakes/whole-route-acf04716.png),
[after native Cut source override](../renders/northern-lakes/after-cut/whole-route-dab78c2f.png),
[player-scale source views](../renders/northern-lakes/viewpoints.png),
[anchors/chunks](../renders/northern-lakes/anchors.png), and
[one-way return overlay](../renders/northern-lakes/ledges.png).
These images use the unchanged local pack source pixels and palettes. They are
source previews, not production captures or proof of native playability.

## Shape and play

- **Cedar Lake** is the broad southern water body. The fast road crosses its
  narrow inlet on a real two-row railed bridge. A longer shore circuit offers
  Finn/Nell, a marsh neck, the fishing pier and the Ether spit. The east shore
  narrows to about three core tiles outside activity bays. A boulder and forest
  close the marsh sides: the long shore has two unavoidable habitat steps.
- **Long Lake** is the northern finger lake with two eastern bays and a dry
  alder spit between them. Its short western bank crosses a two-step habitat
  neck; its longer northern/eastern shore offers Sivren and the Repel spit.
  These choices are distinct from Cedar's bridge-versus-long-marsh circuit.
- **Birch Pond** has a small accessible island and a short protected bridge.
  A native Cut tree seals the western approach, gating Avrin and the Great Ball.
  Native shore rails, a northern abutment rail and a visible water rock prevent
  Surf from landing behind it.
  Removing the island bridge still disconnects the island after Cut, proving
  there is no accidental dry dam or alternate walking entry.
- **Birch bank** is a stepped optional terrace with Reed, Super Potion and a
  real south-facing return ledge. Its eastern native ramp is the uphill path.
  The hop saves fourteen ordinary walking steps; the reverse is blocked.
- **Leech trailhead** has a mandatory four-step grass entrance, a local warden
  and flowers at the first clearing. Forest fingers remove the former broad
  empty mainland; wider areas are retained at actual encounters and rewards.

Authentic `Route42`, the Johto metatile atlas and the complete map/player-scale
views were inspected. Visible revisions removed the repeated room lattice,
reduced the open mainland, narrowed west/east travel lanes, reshaped the brown
terrace, turned the second large rectangle into a two-bay finger lake, removed
an isolated grass cell, closed dry rail-strip and cliff-corner bypasses, moved
NPCs off rail walls/seams, and moved the Cut approach inward so its camera shows
source forest rather than a black crop outside the authored map.

## Exact native registration

| Property | Contract |
| --- | --- |
| Exterior dimensions | 40×48 metatiles = 80×96 core tiles |
| Native maps | `MnR08C00R00` 32×24, `MnR08C01R00` 8×24, `MnR08C00R01` 32×24, `MnR08C01R01` 8×24 |
| South port | Leech Lake: global gate meta `(13,47)`, landing `(13,44)`; `MnR08C00R01` local meta gate `(13,23)`, landing `(13,20)` |
| North port | Bemidji: global/local gate meta `(20,0)`, landing `(20,3)` in `MnR08C00R00` |
| Regional prerequisite | `BADGE_6`, quest `Q05`; actual regional gate scripts remain integration work |
| Native tileset | Existing `mn_headwaters`: unchanged 128 Johto blocks + existing `80`–`82` source recipes |
| Bridge/pier blocks | Cedar bridge/pier use `82` north and `81` south; island bridge uses matching native Johto `41`/`49` rails; real WATER remains below except its explicit west-abutment rock |
| Interiors | None; no false enterable facades |
| Encounter catalog | Original R08 morning/day/night/water tables retained |
| Cut catalog | Copy the unchanged Johto Cut replacement table to the `mn_headwaters` tileset key; never treat art inheritance as field-move rule inheritance |

No new source graphic, palette, animation, audio, game catalog or content pack
was produced by this source author. Root owns native publication and the shared
world/port contracts. Retire the old 3×4 R08 chunk reservations rather than
filling them with blank maps.

## Cut gate: exact before/after contract

The gate is native Johto `63` at **metatile `(3,10)`**, with `CUT_TREE` at core
`(6,21)` and adjacent `HEADBUTT_TREE` at `(7,21)`. The safe southern approach is
`(6,22)`. Native forest blocks close the west side at `(5,21)`; a visible native `0a` water rock
closes the east side at `(8,21)`.

The supplied pack's actual field-move table says decimal block `99` (`63` hex)
replaces with decimal `63` (**`3f` hex**), tree variant. **It does not replace
with `02`.** The source `3f` keeps the adjacent Headbutt tree and opens the left
Cut quadrant. The rule requires a party Pokémon that knows `CUT` and Johto
badge **index 1**, `ENGINE_HIVEBADGE`. The route's regional `BADGE_6` is a
separate requirement and is not the native Cut badge.

| Condition | Expected whole-map result |
| --- | --- |
| Tree present | Both regional endpoints reachable; all other actors/rewards/signs reachable; island Avrin and Great Ball unreachable |
| Actual `63`→`3f` override | Both endpoints still reachable; Avrin and Great Ball become reachable |
| Bridge walking cells removed after Cut | Island targets unreachable again |
| Missing move or Hive Badge | Native denial, no block mutation; requires production controller test |
| Cut success | Native prompt/removal, walk through and interact with targets; requires production controller test |
| Map return/save/Continue | Test native reset/override lifetime and record actual behavior; no permanent-clearing flag is invented or claimed |

The authoritative cut_gates entry retains ID MN_R08_SHORTCUT, with targets
[[14,17],[14,19]], gated_actor_ids as listed below, and shortcut:null.
The complete source CUT_TREE inventory is exactly [[6,21]]; an incidental
5b canopy was replaced with ordinary 5d, so there is no decorative Cut tree.

The exact gated actors are `MN_R08_T04` (Avrin, `(15,19)`) and
`MN_R08_ITEM_02` (Great Ball, `(15,17)`); their tested walking approaches are
`(14,19)` and `(14,17)`. Only `BIRCH_ISLAND` is a gated viewpoint. No sign or
required main-route actor is gated. Static flood evidence: **2,418** reachable
core tiles before Cut; **2,450** after; **32** newly reachable. These walking counts exclude occupied actor tiles and WATER travel. A stronger
whole-map graph admitting both walking and Surf reaches **4,276** core tiles
before Cut and **4,308** after: the same **32** newly accessible tiles, with
both named targets sealed before and opened after. Every other actor and both
regional endpoints remain accessible before Cut. This proves source collision
geometry, not that native Cut/Surf has been published or exercised.

The source rail ring uses metatiles `(6,8):41`, `(7,8):41`, `(8,8):42`,
`(6,9):49`, `(8,9):46`, `(6,10):48`, `(7,10):49`, `(8,10):4a`. Its only west
mouth aligns with bridge footway core y17/18. The north abutment rail is `41`
at meta `(3,8)`; a real all-WALL `0a` water rock at `(4,10)` closes the otherwise
landable Floor quadrant beside the Cut canopy. The bridge's original rail
collision is retained using Johto `41` north/`49` south for a coherent native
metal-rail join. Removing the island north rail, the abutment rail, or that rock
separately makes the walk+Surf before-Cut flood reach the sealed targets; all
three negative mutations are rejected by the checker.

## Other physical contracts

| Feature | Exact source/core contract |
| --- | --- |
| Southern habitat | Core `[22,90,32,94)`, minimum four grass steps between regional endpoints |
| Cedar inlet bridge | Meta x17–20; north rail y24, south rail y25; WATER y26 |
| Island bridge | Meta x4–5; native `41` north rail y8, `49` south rail y9; WATER at `(5,10)` and explicit blocking rock at `(4,10)` |
| Fishing pier | Meta x30–35; north rail y35, south rail y36; footway core y71/72; closed west end core x58; angler `(65,72)` leaves a passing tile |
| Long Lake habitat neck | Core `[44,26,46,28)`, minimum two grass steps within west-bank region `[40,16,48,36)` |
| Cedar marsh neck | Core `[70,68,74,70)`, minimum two grass steps within east-shore region `[70,64,74,80)` |
| Birch return hop | Native `57`, core `[28,26,30,28)`; approach `(29,25)`, hop cell `(29,26)`, landing `(29,28)`, rejected reverse `(29,27)` |
| Ledge side walls | `(27,27)` and `(30,27)` both WALL |
| Walking climb | Native ramp `74` at meta `(17,13)`; required return passage core `[34,26,36,28)`; shortest ordinary return 17 steps |

The two habitat-choice proofs deliberately inspect the stated lane regions;
their other complete shore/bridge alternatives remain legitimate global routes.
They do not claim that all grass is globally mandatory.

## Walking and Surf boundary audit

The new water-edge requirement was checked against every collision quadrant,
not only the visible shoreline. The outer lakes were already enclosed; the
subsequent Cut correction adds island shore rails and one rock, preserving
those outer limits and intentional Surf seams. All 1,712
WATER core tiles lie inside the route's outer perimeter. The nearest WATER is
eight core coordinate steps inside the west, east and south borders, and ten
inside the north border. The global outer perimeter has **zero WATER tiles**.
A permissive graph admitting FLOOR, TALL_GRASS and WATER reaches all water
before and after Cut and still has no outer water exit; this deliberately
admits more travel than native Surf, so an unvisited water pocket cannot hide
an off-map route.

| Native WATER component | Core bounds, exclusive maximum | Core tiles |
| --- | --- | --- |
| Birch Pond around its island | `[8,10,24,30)` | 192 |
| Cedar Lake below the inlet bridge | `[30,52,72,88)` | 1,036 |
| Inlet head above the bridge | `[40,46,42,48)` | 4 |
| Long Lake and its two bays | `[46,12,72,40)` | 480 |

There are three named lakes and four WATER collision components: the inlet
bridge separates its tiny northern head from Cedar Lake for Surf movement.
There is no outer lake or river exit to close with a buoy/rock row, so adding
one would be arbitrary scenery rather than fixing a boundary.

The two clipped eastern chunks are **8×24** metatiles, rather than full-width
32×24 maps. Their actual east edge is global core x79. Neither eastern edge
has WATER. The internal `C00R00`↔`C01R00` seam has fourteen WATER-to-WATER tile
pairs; `C00R01`↔`C01R01` has twenty-two. The two horizontal internal joins have
zero WATER-to-WATER pairs; the inlet ends at the neighboring rail rather than
an open ocean edge. The shared Rust builder authors all four neighboring-map
pairs reciprocally at offset zero. No published R08 pack was available during
this source audit, so those actual native connection records remain unverified.
The dedicated checker now checks the native dimensions and both directional
connection records automatically when all four R08 maps exist in its input
pack; it reports source-only status otherwise.

The earlier boundary-only audit exposed a natural Surf landing on the island.
That bypass is now closed with the visible rail ring, abutment rail and water
rock specified above. This is an actual Cut-exclusive optional excursion even
in the permissive walking+Surf graph. Required main traversal and all other
intentional Surf access/seams remain unchanged. No arbitrary outer buoy line
was added to lakes that do not approach an outer map boundary.

## Actors and resources

Full legal parties, source sprites, facing, sight ranges and dialogue are in
`MN_R08.json`. All existing trainer flags `EVENT_MN_R08_TRAINER_01` through
`08` and pickup flags `EVENT_MN_R08_ITEM_01` through `04` remain unchanged.

| ID | Person/resource | Global core tile |
| --- | --- | --- |
| T01 | Norek, Camper | `(21,85)` |
| T02 | Willow, Picnicker | `(27,53)` |
| T03 | Finn, Fisher | `(73,63)` |
| T04 | Avrin, Bug Catcher, Cut-gated island | `(15,19)` |
| T05 | Eli, Bird Keeper | `(37,35)` |
| T06 | Sivren, Fisher | `(67,11)` |
| T07 | Reed, Camper | `(27,17)` |
| T08 | Nell, Pokémaniac | `(69,85)` |
| ITEM_01 | Super Potion, upper terrace | `(31,15)` |
| ITEM_02 | Great Ball, Cut-gated island | `(15,17)` |
| ITEM_03 | Repel, alder spit | `(65,27)` |
| ITEM_04 | Ether, Cedar shore | `(59,89)` |

Three locals occupy `(65,72)`, `(39,39)` and `(29,89)`. Six signs use exact
source sign WALL quadrants at `(28,88)`, `(22,54)`, `(30,28)`, `(8,32)`,
`(70,80)` and `(38,4)`. All actors have same-chunk approaches and stay off native
chunk seams; the pier angler stands on the floor quadrant rather than its rail.

## Fast review loop and verified evidence

```sh
.local/minnesota-review-env/bin/python \
  planning/minnesota/tools/review_northern_lakes.py
.local/minnesota-review-env/bin/python \
  planning/minnesota/tools/review_northern_lakes.py --serve
```

The live source review uses `http://127.0.0.1:8778/`, watches the dossier/layout,
and preserves the full 1,280×1,536 source image. The measured full warm review
was about **490–515 ms** with the current before/after Cut images, including
the complete walking+Surf proofs, actor/grass/ledge checks, two whole maps and
thirteen player-scale views for each state. Loading the pack is
outside that warm measurement. The source checker uses the actual pre-Cut
pixels, explicitly labels gated-anchor validation as post-Cut, and never writes
a pack or changes runtime state.

The current source pass has no errors, uses 37 native metatile varieties, checks
all four exterior chunks, eight trainers, three locals, four pickups, six
signs, three railed crossings, the actual Cut rule and two priced habitat lanes.
The immutable reviewed images are `whole-route-acf04716.png` (uncut) and
`after-cut/whole-route-dab78c2f.png` (exact source `63`→`3f` preview). Their
before/after gate crops are `viewport-island_cut_gate.png` and
`after-cut/viewport-island_cut_gate.png`; their island crops use
`viewport-birch_island.png` in each directory. Both whole maps and those
player-scale changes were inspected.

The authoring layout uses native `schema_version:2` and
`reject_cliff_cuts:true`. All twelve polygon/cliff outlines have explicit
orthogonal closing edges. The old one-cell diagonal shoulder was transcribed
to an orthogonal stair without changing the rendered map. Deliberate interior
wear paths use explicit native ground rectangles; the terrace border still
uses its actual `74` ramp and `57` hop, rather than an ordinary floor cut.

Native work still required: Rust route registration and typed gated-target
validation; source-preservation/negative regressions; actual native captures;
controller input across all seams, encounters/trainers/PP, Cut denial/success/
return semantics, ledge/reverse/climb, pier collision and all rewards; saves and
Continue/MCP/TUI; audio verification; regional hub ports and earned Q05/BADGE_6
progression. Source previews and green geometry do not close those requirements
or establish final visual acceptance.

## Production field-move results

The combined `content-packs/minnesota-northern-lakes.crystalpack` contains 401 maps (R05–R08 plus the preserved source). Six production-controller field-move regressions pass: all three Cut gates independently reject missing CUT and a missing Hive badge, perform the actual catalog replacement, allow walking to the sealed reward, and preserve the item/flag through Continue. Native temporary vegetation resets on Continue; cutting again from inside permits exit. R05/R06/R07 mount Surf through A/Yes, reject movement against real BUOY quadrants, and cross each actual internal river/lake seam in both directions. This explicit Mew 60/position/badge fixture proves mechanics; it does not establish route battle balance or earned regional progress.

```sh
CRYSTAL_CUT_ROUTES_TEST_PACK="$PWD/content-packs/minnesota-northern-lakes.crystalpack" \
CARGO_TARGET_DIR=target/runtime-fps cargo test --profile wasm-dev --locked \
  -p crystal-bevy --lib cut_gated_routes -- --ignored
```

The new route composition/source-preservation regression passes, including rejected fake gate targets, replacements, unknown actors and incidental decorative Cut trees. All 377 audio PCM hashes/frame counts/loop ranges match in the combined pack. The later R08 two-battle/three-resource populated journey passes in the469-map composition as recorded above. All-eight-battle route completion, earned regional progression, broader MCP/TUI coverage and final user visual acceptance remain open. The new peninsula source revision still requires Root's native republication and rerun.
