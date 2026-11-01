> Current grass review: [R20 verification](../revisions/route-grass-v01/current-r20-v01/README.md).
> Five metatiles add a six-step grassy camp climb with dry ledge descent, plus a
> three-step western meadow crossing each way. Eastern neck and bridges remain.
> Four native cases, 15 stdio, audio and actual 2D pass. Other regional links and
> the separate Grand Marais cedar warmth correction remain open.

# Voyageurs Border Loop — source review and integration contract

## Current executed native evidence — 2026-10-26

Root published `content-packs/minnesota-voyageurs-border-loop.crystalpack` with
**427 maps**: four clipped exteriors and the two-mouth cave. Exact comparison
passed **2,048 exterior metatiles**, all exterior events and the room/paired
returns. All three native asset suites pass. The integration owner inspected
the actual native granite-notch capture; this is not final whole-route/user
visual approval.

Both actual production-controller tests **PASS**
(`.local/voyageurs-controller-current.log`): the empty-party terrain/bridge/habitat/
ledge tour, both directions and same-mouth cave returns, all ordinary resources,
Start/Continue; and the real Officer Parker battle with selected PP, money,
defeat flag, post-victory movement and persisted party/flags through Continue.
The fixture explicitly supplies its party, badges and starting position.
One Officer victory is not the remaining nine battles, an earned lighthouse
signal, regional travel, Falls check-in or SQ03's once-only PP Up reward. Free
route traversal remains separate from Safari admission/runtime.

Current integration audio evidence is the shared **443-map** composition with
all **377** PCM hash/frame-count/loop-range checks passing. No redundant audio
run is claimed. Older pending implementation/publication statements below are
historical handoff snapshots, superseded by these executed local results.


This is a free endgame shore route between International Falls and Boundary Waters. A connected northern lake, crooked pine headlands, native granite shoals, two bridged outlets and an inhabited granite peninsula define its geography. The mainland remains walkable without Surf or preserve admission. It does not author a second Safari preserve, capture expedition, fee, boat dispatcher or counter renewal.

Current review candidate: [complete route](../renders/voyageurs-border-loop/whole-route-602813e1.png), [all sixteen player views](../renders/voyageurs-border-loop/viewpoints-3ea426e4.png), [actual emptiest walking camera](../renders/voyageurs-border-loop/viewport-emptiest_ground-1602fef4.png), [two-chamber cave](../renders/voyageurs-border-loop/interiors-2ddb9030.png). These ignored images show actual native source art. Root accepted this source integration candidate after reviewing its revised granite notch and weakest camera. Native publication and both focused controller fixtures passed as recorded above. Final whole-route/user visual acceptance and earned regional progression remain pending; source previews do not establish those outcomes.

## Exterior and regional binding

| Property | Exact source contract |
| --- | --- |
| Exterior | 64×32 metatiles; 128×64 gameplay tiles; 2048×1024 source pixels |
| Native chunks | `MnR20C00R00`, `MnR20C01R00`: 32×24; `MnR20C00R01`, `MnR20C01R01`: 32×8 metatiles |
| Art | `mn_headwaters`: unchanged Johto blocks plus existing native Kanto source-row recipes80–82; no new graphics or copied catalog |
| International Falls | west gate meta `(1,22)`, safe arrival `(3,22)` in `C00R00`; actual outside walking core `(0,44)` and `(0,45)` |
| Boundary Waters | east gate meta `(62,18)`, safe arrival `(60,18)`; eastern-map local gate `(30,18)`, arrival `(28,18)`; actual outside walking core `(127,36)` and `(127,37)` |
| Chunk connections | reciprocal east/west at offset0 in both rows, reciprocal north/south at offset0 in both columns; bottom maps are only eight metatiles high |
| Native allocation | integration owner reserved group60 `MINNESOTA_VOYAGEURS_BORDER_LOOP`; maps1–4 exterior and5 cave; preserve HOME and the bounded native spawn table |
| Interior | retained identity `MnR20Cave1F`, now9×8 metatiles, template `UnionCave1F`, cave art/border01 |

This replaces the old128×96 sixteen-chunk blockout. Shared world, interior reservations, region atlas and Rust route registration must be reconciled before publication. Source JSON alone does not install neighbor records or earn story flags.

Original required flags remain all eight `BADGE_1`–`BADGE_8` plus `EVENT_MN_LIGHTHOUSE_SIGNAL`; quest remains `SQ03`. Check-in belongs to the International Falls ranger desk and sets `EVENT_MN_BORDER_REPORT`. Completed circuit reporting at that desk awards one `PP_UP`, guarded by `EVENT_MN_BORDER_CIRCUIT_REWARDED`. Route residents only describe these conditions. They grant no false report or circuit completion. Voyageurs never bypasses `EVENT_MN_SURVEY_RESTORED`/Q07 on the League approach. The existing original required-flag list was preserved; implementing the report/circuit regional authority remains the integration owner's explicit work.

## Traversal, encounters and discoveries

- The western outlet crossing uses meta x19–24, north rail row20/block82 and south rail row21/block81. The pale two-gameplay-tile deck crosses the wide outlet at core y41–42. The eastern crossing is shorter and higher: meta x45–49, north rail row17 and south rail row18, deck core y35–36. It approaches through a crooked shore headland and grass shoulder. Removing either entire crossing disconnects the route gates in the complete walking/grass graph; there is no dry shore bypass.
- The clear main road connects both gates when only the explicitly required eastern grass neck is added. Optional habitat is therefore an actual choice, rather than a silent grass detour caused by a broken painted road. Native half-block fence and bridge collision are included in this check. The cliff-foot southern walk has a deliberate clear gate at meta `(35,19)` through the low guide rail.
- The Boundary shore neck occupies core `[112,30,116,42]`, surrounded north and south by continuous native woodland. Every ordinary crossing costs at least four actual tall-grass steps. Surf is optional and not claimed to obey this walking-only encounter contract.
- The western lake pier extends over WATER through meta x4–8, rows12/13. Native post rails protect both sides;42/4a at x9 close the tip and corners. The Ultra Ball at core `(13,26)` has a real deck approach. Player camera `(15,26)` sees the deck, water and closed end together.
- The sheltered western camp uses native block56 at meta x9–10,row25. Upper approach is core `(19,49)`, hop start `(19,50)`, landing `(19,52)`, reverse wall `(19,51)`. Side barriers `(17,51)` and `(22,51)` seal the bank. The sole ordinary uphill gate is core `[12,50,14,52]`. The whole-route return is15 steps versus the three-tile directed displacement, saving12 steps. The camp contains Picnicker Lena and flowers; the return is not an empty lap around a reserve.
- The northern granite habitat houses Hiker Gneiss/T08 at core `(73,23)` beside the actual lake and cliff. The eastern north-shore habitat contains the retained Revive/ITEM04 at `(119,25)`. The southern peninsula meadow contains Fisher Rainy/T06, Cooltrainer Iris/T07 and Max Repel/ITEM03 at `(83,55)`. Western night grass and outlet sedge offer optional encounter movement beside the free road.

All ten original trainer IDs, parties, levels and `EVENT_MN_R20_TRAINER_01`–`10` remain. Exact positions, native trainer classes, sight, legal parties and dialogue are in the dossier. All four original item IDs, item identities and persistent flags remain: ITEM01 Hyper Potion moved into the actual cave; ITEM02 Ultra Ball is on the lake pier; ITEM03 Max Repel is in the southern meadow; ITEM04 Revive is in the eastern north-shore pocket. New three residents and five signs are individually authored. Ordinary canopy variants contain zero incidental Cut quadrants.

## Cave mouths and return ownership

| Mouth | Exterior tile | Safe passive return | Exterior map | Entry → room exit |
| --- | --- | --- | --- | --- |
| West granite face | CAVE core `(60,33)` | FLOOR `(60,34)` | `MnR20C00R00` | exterior warp1 → room warp1 `(5,13)` |
| East granite face | CAVE core `(76,31)` | FLOOR `(76,32)` | `MnR20C01R00`, local `(12,31)` / `(12,32)` | exterior warp1 → room warp2 `(13,13)` |

Both mouths use exterior passive return warp2, because they occupy different exterior maps. Room exit1 must target west exterior warp2; exit2 must target east exterior warp2. `exit_doors` explicitly retains this per-mouth pairing. No room exit may select the first matching exterior door accidentally.

The9×8 room contains two native stone chambers, one connecting seam and a persistent Hyper Potion at `(5,5)`. Both exits have LADDER collision and reachable floor approaches. The internal exit-to-exit walking path is16 steps; the exterior safe-return path is18. Warps and visible transition costs make this an indoor discovery loop, not a promised faster crossing. Initial clear arrival is `(5,12)`; actual second-mouth arrival/return and same-mouth exits require native verification.

## Cut and complete Surf boundary

No Cut gate is authored. The old undefined placeholder did not name a sealed useful target; adding a tree beside this free road would repeat the user's rejected use of Cut. `shortcut:null` and `cut_gates:[]` are intentional. Every placed native collision quadrant is scanned: zero `CUT_TREE` quadrants.

Native blue-water rock closure uses30/31/32 along exposed northern lake sections at meta row1; northern pine headlands replace intervening rock segments with solid shore. Southern outlet caps use38/39/3a at row30, west x20–24 and east x46–48. These IDs remain in `additional_water_blocks` so shoreline fitting keeps water beside the actual native rock/buoy glyphs. No land boulders were painted onto blue water. The whole128×64 core perimeter contains zero WATER, WATERFALL or WHIRLPOOL cells and only the four planned walking port cells. Check the clipped eight-row bottom maps, not a fictitious24-row reserve.

There are17 WATER↔WATER pairs across internal x63↔64 and14 across internal y47↔48. These seams are permitted only with actual reciprocal native neighbors. The source checker records their geometry; publication must verify corresponding neighbor metadata and production Surf traversal. Native collision blocking at the rock caps must also be exercised through `VisibleShellController`, not inferred from the image.

## Review loop and actual revisions

Read the reusable map skill/user guidance, repository ART/ACCEPTANCE/WORLD and requested Johto/Kanto reference. Inspected actual native Route44/Route27 source images, Johto/cave art and the old lattice. All geography is explicit authored polygons, paths and native metatile placements; the old generator is not used.

The first complete pass had a long rectangular lake, a broad granite slab and a whole western camera of empty pale lawn/path. Subsequent visual passes replaced the straight lake reserve with four crooked pine headlands and two granite shoals, contoured the cliff, put birch fingers and intentional night/sedge habitat against the western lane, and closed its otherwise empty T-junction with a native fence connected to woodland. The newly exposed eastern crossing camera then prompted a shore headland and near-road meadow shoulder. A northern cliff pocket became an actual trainer habitat and the eastern north shore an actual resource trip. An isolated four-cell grass patch exposed by collision checking was removed, rather than hidden in the render. The integration owner then identified the still-broad granite slab and paired eastern strips. A true pine-root notch now cuts the actual granite contour at meta33–34,y10–13, with source cliff walls around its recess; no prop was scattered onto the slab. An eastern bank birch finger forces the south spur to bend west and back east instead of running as parallel straight strips. Inspection of the next worst cameras exposed two unused southern lawn tails; outlet-bank woodland and the meadow forest heel now close those tails. The current worst reachable camera is core `(110,26)` and includes lake water, the bird trainer, a short turning road and close native pine boundaries. Its largest blank-ground square is two metatiles; this metric is only a review aid.

```sh
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_voyageurs_border_loop.py
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_voyageurs_border_loop.py --serve --port 8787
```

The warm watcher runs at `http://127.0.0.1:8787/`, process session68046, log `.local/voyageurs-border-loop-review.log`. A measured warm complete check/render took **301.1ms**. The checker records total render/check time and immutable whole, worst-camera, player-sheet and interior images. The candidate has41 native metatile varieties,16 authored player cameras, ten trainers, three exterior pickups plus the interior Hyper Potion, three residents, five signs and one interior. Source/static checks pass, including clear road continuity, exact exterior ports, no unintended Cut, actor/item/door approaches, cave mouth ownership, both necessary rail-protected crossings, required grass and directed return. The author inspected the whole map, all player cameras, automatic worst camera and actual native cave image by sight. Root accepted the revised source candidate; final user/native visual acceptance remains pending.

## Native work still required

Root owns shared schema/world reconciliation, native registration, source export and pack publication. No competing publisher or Cargo build was run here. After publication, compare every native block/object/warp and the four-map clipped dimensions; verify actual reciprocal water/land seams. Drive real input across both bridges, all rails and pier tip, forced walking habitat, optional meadow, native bank denial/hop/return, both cave entries and both same-mouth returns, trainer battles/PP and pickup Save/Continue absence. Compare headless native2D captures at matching position/time/facing/state. Finally exercise earned badge/lighthouse gating, Falls check-in and once-only SQ03 reward without weakening the independent Q07 League guard. A source render or fixture position is not evidence that those regional conditions are implemented.

## Native builder handoff

Root accepted source `602813e1` as an integration candidate after inspecting the actual granite-notch and weakest cameras; final user/native approval remains separate. The focused new [voyageurs_border_loop_modpack.rs](../../../crates/crystal-assets/src/voyageurs_border_loop_modpack.rs) exposes:

```rust
pub fn build_voyageurs_border_loop_modpack(
    base: &CompiledGamePack,
    layout_json: &str,
    dossier_json: &str,
) -> anyhow::Result<CompiledGamePack>
```

The leaf requires the existing `SQ03` dossier binding, delegates actual native compilation/publication to the existing authored-route publisher, then validates the resulting collision for both independently necessary protected outlet bridges, the closed landing, exact clipped extents and walking ports, no incidental Cut and the complete outer Surf perimeter. Its compiled-script audit keeps admission/economy commands and the Falls report/PP Up reward out of all five route modules. It does not create an alternative dispatcher, Safari zone, paid route or regional quest implementation.

[voyageurs_border_loop.rs](../../../crates/crystal-assets/tests/voyageurs_border_loop.rs) contains three meaningful native compilation tests. The positive fixture first builds the actual R16 overlay and then composes Voyageurs, checking every prior map/runtime file/metadata/tileset/trainer/item, unchanged audio and original HOME/recovery spawn table, four clipped payloads, both reciprocal neighbor directions, exact actor positions/parties/flags/pickups, and the cave's real source blocks/warp ownership. Negative cases change actual mouth bindings, duplicate exits, passive return index/position, one required badge at a time or the lighthouse flag, SQ03 identity/reward authority, an ordinary ledge return, an actual bridge, an unpaid dry outlet dam, required habitat, native hop, outer Surf water and incidental Cut canopy. These tests are prepared source; native passes are claimed only after Root executes them. They do not drive controller input.

Only the new leafs and this runbook belong to this native implementation handoff. Root owns these exact shared registrations:

1. `lib.rs`: `mod voyageurs_border_loop_modpack;` and `pub use voyageurs_border_loop_modpack::build_voyageurs_border_loop_modpack;`.
2. `authored_route_layout.rs`: `"MN_R20" => ([64, 32], "mn_headwaters", "johto_8_neighbors")`.
3. `authored_route_design.rs`: dimensions/count tuple `"MN_R20" => ([64, 32], 2, 2, 10, 3, 1, "MnR20")`. Exterior pickup count is **three**, with the fourth pickup in the room; add `"MN_R20" => 3` to that match. Sign count is **five**. Door count is **two**, so extend the special two-mouth count currently used for R05 to include R20, while interior count stays one. Keep common room/actor/item validation. R20's branch must retain exactly `BADGE_1` through `BADGE_8`, then `EVENT_MN_LIGHTHOUSE_SIGNAL`, one required grass crossing, one native ledge, no Cut gates, no shortcut and no battle-or-grass replacement contract.
4. `authored_route_modpack.rs`: add the RouteSpec below and include `"MN_R20"` in the native Headwaters-art installation branch when that compiled art is absent.

```rust
"MN_R20" => RouteSpec {
    group_id: 60,
    group: "MINNESOTA_VOYAGEURS_BORDER_LOOP",
    manifest: "minnesota-voyageurs-border-loop",
    landmark: "LANDMARK_MN_VOYAGEURS_BORDER_LOOP",
    label: "MN_VOYAGEURS_BORDER_LOOP",
    name: "VOYAGEURS BORDER LOOP",
    position: [104, 8],
    tileset: "mn_headwaters",
    music_source: "Route42",
},
```

Group60 was explicitly reserved by the integration owner. The proposed Pokegear marker `(104,8)` is a regional chart binding for Root to confirm. **Use `mn_headwaters`**, not unextended `johto`: frozen source blocks80–82 are the existing native rail recipes, while source Johto ends at7f. The integration owner confirmed the compiled art identity after identifying this mismatch; no geometry/art substitution is needed.

The existing shared two-mouth lowering already supports the required pairing and should remain authoritative: west CAVE `(60,33)` in `C00R00` uses exterior trigger index1 → room exit1; east CAVE `(12,31)` local to `C01R00` uses exterior trigger index1 → room exit2. Both exterior passive return records have index2 on actual FLOOR. Room exit1 must target west map/index2 and exit2 east map/index2, using `exit_doors` to select the corresponding door rather than the first door for the room. The passive record's own outbound target is not a new player-triggered entrance; FLOOR prevents an immediate warp loop. No additional shared warp dispatcher is requested.

Scoped `rustfmt --edition 2024` completed for both new files. No cold build, shared-cache compilation or pack export was run here. After registration, Root should warm-run `cargo test --profile wasm-dev --locked -p crystal-assets --test voyageurs_border_loop`. Default input is the actual ignored `content-packs/text-tui.crystalpack`; `CRYSTAL_VOYAGEURS_BORDER_BASE_PACK` may select another completed source without R16/R20, and is canonicalized before verified loading. Root's actual compile/test result, artifact publication, audio verification, controller transitions/quest persistence and native visual judgment remain to be recorded after execution.

## Prepared production controller fixtures

The focused [controller leaf](../../../crates/crystal-bevy/src/bevy_shell/tests/voyageurs_border_loop.rs) is prepared for Root's warm compilation after native publication. Root owns its shared registration:

```rust
#[path = "tests/voyageurs_border_loop.rs"]
mod voyageurs_border_loop;
```

The ignored tests read `CRYSTAL_VOYAGEURS_BORDER_LOOP_TEST_PACK`, defaulting to the canonical path of `content-packs/minnesota-voyageurs-border-loop.crystalpack`. That default is the intended published route pack, not a fabricated fixture export. No controller test has passed at author handoff; missing or incorrect native behavior must fail the actual assertions before any passing claim.

`voyageurs_controller_walks_seams_bridges_habitat_cave_returns_and_continue` explicitly starts an empty-party geometry fixture at core `(122,36)`. `RouteGrid` plans safe walking, while every gameplay action uses `VisibleShellController::press`. It identifies all three reachable walking chunk-neighbor pairs and traverses each both ways; the lower east/west seam is water and is not mislabeled a walking connection. It visits both route ends, walks both protected deck rows in both directions, tests actual rails and closed-pier denial, walks all four tall-grass cells across the required eastern neck both ways, traverses native `(19,49)` → `(19,50)` → `(19,52)` HOP_DOWN, denies reverse and drives all15 ordinary return steps. These movement checks supplement the asset tests' independent crossing-removal and grass-bypass checks.

The same fixture talks to all three residents and five signs, collects the three exterior rewards and cave Hyper Potion, and checks exact bag increments and persistent flags. West entry must reach room warp1 `(5,13)` and east entry warp2 `(13,13)`. Both through-room directions and both same-mouth returns must land at their own exterior FLOOR `(60,34)` or `(76,32)`. The test walks away from and back onto each passive return to expose warp loops or wrong-mouth ownership. Production Start, Save and Continue are exercised. After Continue all four pickup flags and the bag must persist, hidden item actors must stay absent, and actual input must enter each former pickup tile, including the cave cache. All ten trainer defeat flags must remain unset in this empty-party fixture.

`voyageurs_controller_wins_real_officer_battle_mutates_pp_and_continues_party_flags` explicitly creates native level60 Typhlosion and Feraligatr and eight obedience badge bits at initial core `(10,43)`. Those are labeled setup, not earned partners/badges or a balance claim. No party, badge, position, collision or event flag is rewritten afterward. Actual input talks to Officer Parker/T01, selects native legal damaging moves, observes selected PP consumption and all three authored enemy partners, wins and verifies the native money reward and `EVENT_MN_R20_TRAINER_01`. Actual movement, Save/Continue, exact full-party equality, persistent defeat flag and a defeated-trainer conversation without a repeat battle follow. The other nine trainer flags must remain unset. Neither fixture sets the lighthouse/report/League story flags or claims earned SQ03 completion or a full populated route journey.

Scoped formatting passes for the new controller leaf. Root owns warm build/execution and actual native publication/verification. Production Surf against rock caps and across water seams, the remaining populated route, earned badge/lighthouse entry, Falls check-in and once-only PP Up reward, the independent Q07 League guard and whole-route native visual judgment remain separate proof obligations.
