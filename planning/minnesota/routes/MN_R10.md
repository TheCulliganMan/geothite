# Current installed grass revision, 2026-10-28

[Selected R10 revision](../revisions/route-grass-v01/current-r10-v01/README.md) adds7grass metatiles and5attached boulders on the705-map park/current-story child. The carrier circuit requires4grass steps up and0on the dry ledge return; the existing stair and all5city-mouth cells are preserved. Six native cases,13clean stdio calls and377audio programs pass. Selected as route_grass_r10_revision; accepted park and all earlier revisions are retained. Older isolated pack results below are historical.

# Iron Range Descent — current native geography revision

## Grass climb and dry carrier return — 2026-10-28

Accepted in isolated `minnesota-route-grass-r10-v01.crystalpack`, identity prefix
`3d7362f74a63`. Seven new grass metatiles join useful trails. The carrier-bank
circuit requires four grass steps climbing; the original ledge and lower road
keep a dry return. Five attached boulders close native cliff-foot bypasses.
Native grass encounters, stair, ledge, mine, Surf, Calder battle/PP, Continue,
stdio and audio checks pass. See [R10 review](../revisions/route-grass-v01/R10.md).
The shared game remains unchanged; earlier publication reports below are historical.

The current **471-map** `content-packs/minnesota-geography-review.crystalpack`
(`f9b04e67f2bf…`) publishes this terrain revision. Root inspected the complete
map, weak source cameras and fresh native 2D captures. Fresh geometry/cave, Calder battle/PP/Continue and Surf seam/barrier
cases pass. All unrelated pack fields, original art/events/rooms/story and
audio remain exact. See the [current native proof and replay commands](../playable/geography-revisions.md).
The source-candidate notes and earlier standalone-pack results below are retained
as authoring history; final user visual approval and earned regional travel remain open.

---

# Iron Range Descent — rocky source candidate and historical native proof

R10 descends along a pale gravel haul road between shaped ore ribs, exposed rock aprons and joined native erratics, then passes the dry mine cleft and short habitat saddle to the protected Superior shore. The **48×20 metatile** composition uses two exterior chunks, original Johto source pixels and one optional **12×8** mine gallery. The playable area follows that sequence; the northern woodland is a closed silhouette rather than another empty district. It replaces the old 96×96 repeated reserve. Variable size is deliberate: this road does not need twelve chunks to support its choices.

Authoritative files: [dossier](MN_R10.json), [explicit terrain strokes](MN_R10.layout.json), and [fast source review tool](../tools/review_iron_range_descent.py). The JSON stores exact actors, original dialogue, parties, loot, encounter slots and room blocks; this runbook records current native evidence and the remaining regional work.

The earlier ignored `content-packs/minnesota-iron-range-descent.crystalpack` contains **413 maps**. Its pre-rock-revision geometry passed exact source comparison, both native asset tests and **all three production-controller cases**: geometry/gallery/resources, Calder battle with selected Surf PP and Continue, and Surf seams/boundaries. Those results establish the earlier local mechanics; they do not prove this new rocky terrain. The current quality-review 471-map pack still matches the immutable pre-edit R10 source. This revision is a **SOURCE CANDIDATE**, pending Root’s visual review, native publication and fresh production-controller checks. Actual hub gates, earned Q06 progression, a complete six-trainer populated journey and final user visual acceptance remain open.

## Visual review and fast loop

```sh
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_iron_range_descent.py --pack content-packs/minnesota-quality-review.crystalpack --self-test
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_iron_range_descent.py --watch --serve --port 8782
```

The viewer is `http://127.0.0.1:8782/`. Editing either source JSON rerenders only this route; no pack generation is involved. The final rocky full review took **394 ms**, excluding initial pack decoding. The explicit command reads the current ignored quality-review pack for original art/collision and previously published seam metadata. It does not assert that this pack already contains the rocky candidate. All image output remains ignored under `renders/iron-range-descent/`.

Current reviewed complete image: [whole-route-2c234acd.png](../renders/iron-range-descent/whole-route-2c234acd.png). The [immutable before/after walking sheet](../renders/iron-range-descent/rocky-899990976317/before-after-walking-sheet.png) compares actual native-matching old source against the new source at core (17,17), (17,15), (32,23), (53,19), (81,25) and the current weakest ground frame (60,20). Full source pixels, RGBA originals, RGB versions, exact 160×144 crops and enlarged views are retained separately, including rejected passes. The [proof report](../renders/iron-range-descent/rocky-899990976317/proof-25cec0bb4d13c178.json) lists every one of **161 changed metatiles** and both before/after source hashes. Whole dimensions remain 1536×640 pixels.

The explicit revision replaces five former pine/lawn footprints with contoured native bedrock ribs, adds a continuous rocky headwall above the eastern promenade, and fits three exposed-rock aprons around existing activity turnouts. The north carrier and required-neck ribs have attached ore seams; the lower rib’s detached pine tuft became an attached native erratic. Pale native `01` gravel separates the main carrier lane from brown `71` bedrock, with actual `68`–`75` cliff quadrants rather than flat brown painted obstacles. Native `0a` erratics are WALL in all four quadrants; the gravel/aprons are native FLOOR. No art recipe, palette or collision was invented. This is a stronger rocky silhouette and camera composition, not a new quarry mechanic or a claim of final user approval.

The frozen `899990976317…` layout is retained under the review archive’s [sources folder](../renders/iron-range-descent/rocky-899990976317/sources/MN_R10-frozen-899990976317.layout.json). Native publication initially rejected the unsupported `exposed_rock` material name. The representation was normalized to **seven existing `block_rect` strokes with native block `71`**, replacing exactly the three authored polygons and removing that material alias. The normalized current layout SHA is **`2db59b63a1184b5f0975c30f3ff8fec1bebbce2bf6cf18c81895410f3ff94e5a`**; dossier SHA remains `dd6dafe871954ad0a5b348843114fe78a11a24732c89487c6784b54716355dc3`. [Normalization proof](../renders/iron-range-descent/rocky-899990976317/normalization-dff2bc2d3a701040.json) establishes all **960 raster blocks byte-identical**, every non-layer field identical to the pre-rock baseline, and the rerender preserves the full 1536×640 pixels / `2c234acd` image. All fifteen source negatives pass again in `.local/r10-rocky-normalized-source.log`. This fixes source-schema compatibility without changing geography, art or gameplay.

All gameplay dossier fields except description/district wording/status and appended review metadata/viewpoints are byte-equivalent JSON values to the backed-up source. Actors, parties, text, flags, ports, cave rows/warps, ramps, grass contracts, ledges and rewards remain exact. Original viewpoints are preserved as a prefix; three additional viewpoints expose the new rock transitions. Backups are `.local/r10-rocky-before.layout.json` and `.local/r10-rocky-before.dossier.json`.

The retained earlier visual iterations below explain the original route topology; their native evidence belongs to the pre-rock composition.

The first pass caught Marin and the road keeper in native WALL cells, a grove across the declared carrier lane, and an ore boulder on the west approach. These were corrected. The next pass was rejected for its broad upper bench. That earlier source revision replaced that polygon with a narrow stepped carrier lane, actual cliff cut-ins and a ledge peninsula. The former empty eastern shelf was removed. The coast moved four metatiles north, allowing four unused rows to be clipped without changing the mine or western ledge coordinates. The source Route45 crop at 160×144 was compared with the routine carrier lane; the worst original probe remains in the contact sheet for candid review. Source checks now assert four actual walking tiles between native cliff bounds for both routine lane sections, rather than confusing metatiles with walking tiles. The public shore uses continuous native rail `49` with corner caps `48`/`4a`; repeating a corner block would obstruct the promenade. The upper forest shapes, cave mouths, rail ends, lake visibility and cave floor were inspected as pixels, beyond merely accepting a green collision report.

## World, story and registration contract

Retain `MN_R10`, prefix `MnR10`, quest `Q06`, and both release requirements **BADGE_7 + EVENT_MN_DEPOT_CLEARED**. R10 grants no depot victory, report, instrument or story-completion flag. The mandatory Team Lode encounter and recovered instrument remain in R09; the Grand Rapids survey depot still clears Q06. This optional gallery cannot replace that scene or allow story flags before victory.

| Native map | Metatiles | Role |
| --- | --- | --- |
| `MnR10C00R00` | 32×20 | Carrier bench, dry mine cleft, required pine neck, western shore |
| `MnR10C01R00` | 16×20 | Coastal habitat, public landing and Duluth road |
| `MnR10Cave1F` | 12×8 | Optional two-chamber gallery, resident and Revive |

The registered builder uses group **55** (`MINNESOTA_IRON_RANGE_DESCENT`), manifest `minnesota-iron-range-descent`, music `MUSIC_ROUTE_45`, exterior tileset `johto`, and room template `UnionCave1F`. The focused native leaf is `crates/crystal-assets/src/iron_range_descent_modpack.rs`, publicly exporting `build_iron_range_descent_modpack`; shared registration and publication are complete. No derived blocks or cloned behavior tileset are needed. Original maps, files, audio, HOME and recovery are preserved.

| Route end | Gate metatile | Clear landing | Core gate / landing |
| --- | --- | --- | --- |
| GRAND RAPIDS, west | (1,5) | (3,5) | (2,10) / (6,10) |
| DULUTH, east | (46,12) | (44,12) | (92,24) / (88,24) |

The authoritative world plan records these route gates against Grand Rapids'
east port and Duluth's west port. Grand Rapids now has the native
`MnGrandRapidsEastCheck` checkpoint, requiring badge 7 and acknowledged MS09
clearance. It cannot substitute foreman defeat or meter pickup for handover.
The connector deliberately opens the reserved western mouth at meta `(0,5)`
through the native bank; it preserves the authored ramp, ledge and other terrain.
Actual release, reciprocal walking, inbound free care and Continue pass in the
502-map story/city candidate. R10's eastern connection to Duluth remains open. Inside R10,
the two exterior chunks retain reciprocal east/west connections at offset zero.
The seam at core x63/64 crosses road and lake; both ordinary walking and Surf
cross it through production inputs.

The old unused exterior reservations (`C02R00`, all columns of R01–R03) are retired from the current world chunk list. Trainer/event reservations 07–10 and `MN_R10_SHORTCUT` remain unused. The explicit `cut_gates: []` and `shortcut: null` are intentional: there is no useful sealed Cut target here. Native CUT_TREE inventory is zero, including incidental canopy quadrants. Adding a free-standing tree would violate the user's gating guidance.

## Traversal and choices

1. The western upper road has native ore outcrops and woodland fingers. Hiker Calder is visible near the arrival. An ore-inspection spur reaches Iva and a Max Repel pocket; the reward requires four actual grass steps. Native boulder ribs close the free side approach. The main west lane and cross-bench lane each measure four actual walking tiles; localized turns widen at the ramp and ledge.
2. The **actual native `74` ramp at metatile (5,10)** allows ordinary uphill and downhill travel. Its core quadrants are `[WALL,FLOOR,FLOOR,FLOOR]`, preserving a useful single-column upper entry and a passing bay below it. The main road never paints over a cliff face; layout schema 2 requires orthogonal strokes and `reject_cliff_cuts: true`.
3. The **native `56` south ledge** occupies metatiles (12,10)/(13,10). Upper approach (25,19), hop-from (25,20), landing (25,22); reverse (25,21) is WALL. Side barriers are (23,21)/(28,21). The ordinary return is **33 successful tile steps**, versus the three-tile hop displacement. Removing the ramp passage core rectangle [10,20,12,22] disconnects this return, proving no free uphill leak. It serves the upper ore-inspection/cache branch; it is not a decorative drop.
4. The lower carrier road offers a rest turnout, Rusk's optional battle and a Hyper Potion. The dry cleft reaches the optional mine mouth without another story lock.
5. The central pine barrier can be crossed on foot only through core rectangle **[50,18,56,22]**, all native TALL_GRASS. The shortest ordinary west/east route pays **six grass steps**. Trees prevent a dry walk around either end. Later Surf can traverse the actual connected lake, with its own move prerequisite and water encounters; the dry shortcut is not free.
6. East of the neck, the north turnout reaches Ansel, the optional bird habitat and Ultra Ball. Main travel has a narrower white lane with deliberate widening at this turnout, shore viewing bank, signs and landing. Marin stands at (85,25), outside the upper tree row.
7. A native gray fence protects the public lake bank. The lookout at **(67,26)** includes **26 WATER quadrants** in its source player-sized frame. The marked landing at **(73,29)** is FLOOR facing WATER (73,30), with native bollard side pieces at meta (35,14)/(37,14). Ordinary walking cannot cross the lake; Surf remains a real field-move transition.

## Exact event inventory

All actor tiles below are native **16-pixel core tiles**, not metatiles. All text lines are original and at most 18 columns. Dossier parties and dialogue are authoritative.

| ID | Tile | Facing / range | Native class and party |
| --- | --- | --- | --- |
| `MN_R10_T01` CALDER | (9,11) | east / 1 | HIKER: Graveler34, Machoke34 |
| `MN_R10_T02` RUSK | (15,25) | west / 1 | FIREBREATHER: Slugma34, Vulpix35 |
| `MN_R10_T03` ANSEL | (75,19) | east / 1 | BIRD_KEEPER: Pidgeotto35, Skarmory36 |
| `MN_R10_T04` MARIN | (85,25) | south / 1 | SAILOR: Magnemite35, Machoke35 |
| `MN_R10_T05` IVA | (37,13) | west / 1 | HIKER: Gligar36, Onix35 |
| `MN_R10_T06` WREN | (39,29) | east / 1 | PICNICKER: Stantler36, Teddiursa36 |

Each retains `EVENT_MN_R10_TRAINER_01` through `_06`; there are twelve opponents total. No initial trainer sight ray seals the route. Standing NPCs are SURVEY_WALKER (7,25), ROAD_KEEPER (41,23), LAKE_WATCHER (67,25). They provide local direction and never set Q06 flags. The cave reader is `MN_R10_GALLERY_READER` (9,5), facing east.

| Item ID / original persistent flag suffix | Location | Reward |
| --- | --- | --- |
| `MN_R10_ITEM_01` / `EVENT_MN_R10_ITEM_01` | exterior (17,27) | Hyper Potion ×1 |
| `MN_R10_ITEM_02` / `EVENT_MN_R10_ITEM_02` | exterior (85,19) | Ultra Ball ×1 |
| `MN_R10_ITEM_03` / `EVENT_MN_R10_ITEM_03` | exterior (27,9) | Max Repel ×1 |
| `MN_R10_ITEM_04` / `EVENT_MN_R10_ITEM_04` | gallery (17,5) | Revive ×1 |

Sign anchors: WEST (4,12), RAMP (8,24), MINE (38,24), PINE (48,22), DULUTH (90,22), SURF (76,28). Their actual native sign quadrants are WALL with clear adjacent approaches.

## Optional gallery and two-way warp safety

The published `MnR10Cave1F` uses the authored **12×8** rows, replacing the old 16×12 reservation. The two chambers circle a central wall through two openings; source boulder formations shape local paths. Every item/resident has a connected ordinary-floor approach. Cave graphics/collision come from the actual `UnionCave1F` template; dark outside blocks are impassable, not painted walkable void.

Exterior block `73` at meta **(18,8)** has native `[FLOOR,FLOOR,CAVE,WALL]`. Active mouth is **(36,17)**, exterior warp **1** to cave warp **1**. Both exterior arrival and safe return are **(36,18)**, a separate FLOOR; passive exterior record **2** supplies that target. The cave exit is **(3,13) LADDER**, native `07` bottom-right quadrant, returning to exterior record2. The inbound cave warp lands on **(3,13) LADDER**; the declared clear interior tile is **(3,12) FLOOR**, reached with real movement. The authoritative interior plan now records these exact coordinates. The passing controller fixture enters, moves clear of the arrival ladder, deliberately exits, steps off/on the passive exterior Floor without bounce, and repeats entry/exit. `clear_arrival` metadata alone is not treated as warp-input proof.

## Surf perimeter proof

Original source **water-rock** blocks `32/36/38/39/3a` keep blue water on the internal side and real `BUOY` collision on the outside. No brown land boulders are painted into the lake.

- South meta row19: x21=`38`, x22..46=`39`, x47=`3a`.
- East meta column47: y14=`32`, y15..18=`36`; the southeast corner belongs to `3a`.
- All **268** core perimeter cells are inspected; **65** are BUOY, and **zero** are WATER.
- The earlier native-matching source combined Walk+Surf reached **513 WATER cells** (the rocky candidate reaches 509), with **zero reachable WATER cells** on the outer perimeter. This check actually enters the water from the landing.
- The internal lake seam core x63/64 stays WATER at y28..38. Exact native reciprocal offset-zero neighbors are verified, and the passing Surf controller crosses this real seam in both directions.

## Current source checks and earlier native evidence

Current rocky source review passes with **38 native metatile varieties**, **1,202** ordinary/grass reachable core cells, **1,114** clear reachable cells and **88** accessible grass cells. Every actor, item, sign, cave mouth and declared camera has a source-collision approach. The required neck still costs six grass steps, the optional Max Repel costs four and the directed ledge retains its 33-step ordinary return. Both routine carrier sections retain exactly four walking tiles. All 268 exterior perimeter cells are audited: 65 BUOY, zero WATER; 509 reachable water cells remain inside, with the reciprocal internal lake seam intact. R10 still has no Cut gate or CUT_TREE quadrants.

**Fifteen source negatives reject** the original twelve failure cases plus removal of an attached ore toe, flattening the sculpted carrier face and erasing the pale gravel contrast. The added geology checks use actual native collision, not appearance alone. `.local/r10-rocky-source-final.log` records the passing source loop; the immutable proof additionally compares every unchanged gameplay dossier field and all 960 old source metatiles against the actual baseline native chunks. This establishes source preservation and collision contracts; the revised terrain has not yet passed native input.

The **pre-rock-revision** published native comparison in `.local/iron-range-descent-native-match.log` verifies all **960 exterior metatiles**, three authored maps, twelve exterior objects, actual events/parties and room rows against the source; native internal water neighbors pass. `.local/final-visual-iron_range_descent.log` records a later exact-match review of the same `95eb0c46` composition. The headless [native Superior lookout](../renders/iron-range-descent/native-superior-lookout.png) was inspected. Source and sampled native visual review do not constitute final user approval.

Both native asset tests pass in `.local/routes-integration-batch.log`:

- `iron_range_descent_composes_with_river_valley_and_preserves_source_content` composes this route over River Valley and preserves every prior map, runtime file, trainer, species, item, mart, tileset, HOME/recovery spawn, compiled audio and audio manifest.
- `iron_range_descent_rejects_changed_progression_and_broken_native_geometry` rejects thirteen invalid native progression/geometry inputs and duplicate installation. These are physical/event-contract checks, separate from the twelve Python source negatives.

The exact shared binding is `MN_R10 => ([48,20],2,1,6,3,1,"MnR10")`, with three outdoor items, six signs, one door/room, one required grass crossing, one ledge and no Cut. It retains requirements `["BADGE_7","EVENT_MN_DEPOT_CLEARED"]`, landmark `LANDMARK_MN_IRON_RANGE_DESCENT`, label `MN_IRON_RANGE_DESCENT`, position `[128,48]` and source music `Route45`. Shared publication now checks the complete native Cut inventory even for a no-Cut route and audits numerical walking/Surf perimeter reachability; the old early-return limitation is superseded.

`.local/iron-range-descent-audio.log` verifies the 413-map pack with **377 audio programs**, preserving every PCM hash, frame count and loop range. The source original content remains intact. No renderer, screenshot or pack dump is used as a substitute for controller inputs.

## Passing production-controller cases

`crates/crystal-bevy/src/bevy_shell/tests/iron_range_descent.rs` is registered and all three ignored R10 cases pass in Root's actual warm execution recorded in `.local/iron-range-controller-current.log`. That combined run reports six passes overall; **three are R10**, while the other three cover R09 and its Cut gate. The R10 results are:

| Case | Executed behavior |
| --- | --- |
| `iron_range_descent_controller_walks_geometry_events_and_cave` | Empty-party traversal of both clipped maps and endpoint approaches; repeated road seam crossing; six grass steps in both directions; genuine carrier ramp; downward-only ledge, blocked reverse and 33-step ordinary return; all six signs, three outdoor residents and gallery reader; four actual pickups/flags; lookout fence, landing bollards and walking-water rejection; cave entry, ladder exit to passive `(36,18)`, safe step off/on that Floor and repeat entry/exit; production Start menu. |
| `iron_range_descent_controller_wins_calder_and_continues` | Explicit initial Mew at level 60 with native Surf and seven badge bits; real challenge from `(9,10)` against Graveler 34 / Machoke 34; selected Surf through native Fight/move menus, actual PP reduction, surviving partner and `EVENT_MN_R10_TRAINER_01`; real Save/Continue preserving exact party, bag, location and defeat flag; continued defeated-trainer dialogue without another battle. |
| `iron_range_descent_controller_surfs_seam_and_blocks_outer_rocks` | Explicit initial Surf fixture at `(73,29)`; native A/Yes launch and mounting; actual water seam x63/64 in both directions; south BUOY rejection `(73,38)→(73,39)`and east BUOY rejection `(94,34)→(95,34)`; return to launch and native dismount. Incidental water encounters use the production RUN command. |

All gameplay uses `VisibleShellController::press`. Party, badges, location and replay-divider setup are explicit review fixtures; **no state, position, flag, inventory or divider is rewritten after setup**. The empty-party case proves geometry and pickup behavior, while the populated case proves Calder's battle and persistence. They do not combine into a claimed six-trainer journey. No R10 actor authors `HealParty`, so no healing service or medicine-use proof is invented here.

## Reproduce the verified local checks

Use the completed ignored native pack and Root's existing warm target:

```sh
CARGO_TARGET_DIR=target/runtime-fps cargo test --profile wasm-dev --locked \
  -p crystal-assets --test iron_range_descent
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_iron_range_descent.py \
  --pack content-packs/minnesota-iron-range-descent.crystalpack --verify-published
CRYSTAL_IRON_RANGE_DESCENT_TEST_PACK="$PWD/content-packs/minnesota-iron-range-descent.crystalpack" \
CRYSTAL_IRON_RANGE_DESCENT_GEOMETRY_SAVE="$PWD/.local/iron-range-descent-geometry.crystalsave" \
CARGO_TARGET_DIR=target/runtime-fps cargo test --profile wasm-dev --locked \
  -p crystal-bevy iron_range_descent_controller_ -- --ignored --test-threads=1
```

The optional geometry-save environment exports the actual empty-party state for subsequent MCP input review. `CRYSTAL_IRON_RANGE_DESCENT_JOURNEY_SAVE` exports the explicitly initialized Calder victory state. The victory regression uses a unique process/time temporary Continue file and removes it. Packs, captures and saves stay ignored; they are not source deliverables. One shared owner publishes packs; other agents consume the completed result without competing exports.

The leafs passed scoped formatting. Repository-wide `cargo fmt --all -- --check` encounters unrelated pre-existing formatting beginning at `crates/offset-allocator/src/lib.rs:408`; those files were not changed. See the central status record for current workspace Clippy conditions rather than treating this route's local test passes as a clean-workspace claim.

## Remaining acceptance work

- Bind and execute actual Grand Rapids/Duluth hub gates, prove denial/release under both required flags, retain safe hub arrival/recovery/Fly behavior and play the earned Q06 sequence. The current world plan and isolated route tests do not prove connected story travel.
- Play the other five authored trainer teams and a complete populated six-trainer journey, including actual loss/recovery and level/encounter balance. Only Calder's team has a passing populated R10 regression.
- Prove R10-specific TUI/MCP input using a fresh compatible actual save, and inspect any remaining native cameras at matched position/time/facing/state. General shared controller/MCP regressions are separate evidence.
- Obtain final visual review from the user. Passing geometry, asset preservation, audio and local controller cases do not establish Johto-level quality by themselves.
