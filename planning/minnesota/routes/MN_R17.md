> Current grass revision: see [Border Pines verification](../revisions/route-grass-v01/current-r17-v03/README.md).
> Four native grass crossings add11metatiles (92→136walking quadrants). The ice
> route requires20grass steps each way; the existing Cut return reduces this to16.
> Original ice, Cut, cave, seasonal aliases and the warm North Shore remain exact.
> Six current-parent native cases,15stdio calls and fresh2D/audio checks pass.
> Current native Minnesota teams/encounters supersede historical source below.

# Border Pines — woodland source proposal

## Current executed native evidence — 2026-10-26

Root published `content-packs/minnesota-border-pines.crystalpack` with **420 maps**:
three clipped forest chunks and two rooms. Exact comparison passed **1,792
exterior metatiles**, all exterior events and both authored room/return
contracts. All three current asset suites pass, including exact native timber
art, measured **local 66→6** activity link and schema-guarded walking/Surf
negatives. The actual native Cut-return camera was inspected by the integration
owner; no final user or whole-route native visual approval is implied.

The populated Ellis battle fixture **PASS** proves selected Surf PP consumption,
actual Ursaring/Golbat/Delibird victory, the defeat flag and Continue of party/bag.
The complete rerun now passes **all three production-controller fixtures**.
Geometry proves all three clipped chunks, timber rails, ordinary resources and
both rooms' passive returns/re-entry. The Cut fixture proves move/badge denial,
native `0f→17`, the actual local **66→6** walking link, cache persistence/tree
reset through Continue, recutting from the south and actual hut HP/PP healing.
Native Down inputs through the carpet after clearing arrival suppression resolve
the original hut-return fixture failure; no source geometry was changed.
Explicit initial party/badge/position/divider setup is not earned SQ03 or
regional travel.
Current integration audio evidence is the shared **443-map** composition with all
**377** PCM hash/frame-count/loop checks passing. Older pending-publication notes
below are historical handoff snapshots; current runtime limits remain explicit.


This **32×56-metatile** route replaces the repeated 64×144 twelve-map reserve
with three clipped forest maps. Its identity is a daylight woodland passage:
closed sedge channel, protected timber crossing and birdwatch deck, two old-tree
roosts, a small granite den, and a real ranger rest hut. Native Ilex Forest art
supplies its foliage and building; the composition is individually authored.

[Whole route](../renders/border-pines/whole-route-6c5bbcb6.png) ·
[player views](../renders/border-pines/viewpoints-6c5bbcb6.png) ·
[quiet forest spur](../renders/border-pines/viewport-quiet_forest_spur.png) ·
[widest ranger yard](../renders/border-pines/viewport-widest_ranger_yard.png) ·
[rooms](../renders/border-pines/rooms.png) ·
[Cut before](../renders/border-pines/cut-before-6c5bbcb6.png) ·
[Cut after](../renders/border-pines/cut-after-6c5bbcb6.png).

These are source-pixel terrain and static event checks. Native publication,
controller behavior, regional progression and final visual acceptance are
separate requirements.

## Traversal and physical anchors

Coordinates below are global 16px gameplay tiles unless marked meta. The first
two chunks are 32×24 metatiles; the last is 32×8. Chunk IDs are
`MnR17C00R00`, `MnR17C00R01`, and `MnR17C00R02`.

| Feature | Exact contract |
| --- | --- |
| Bemidji approach | South gate meta `(16,54)`, inward landing `(16,52)`. Native lower-map local gate `(32,12)` and landing `(32,8)`. Only outer walking cells `(32,111)` and `(33,111)` open south. |
| International Falls approach | North gate meta `(18,1)`, landing `(18,3)`; local core `(36,2)` and `(36,6)`. Only outer cells `(36,0)` and `(37,0)` open north. No Canadian connection exists. |
| Story requirement | Preserve `BADGE_7` and side quest `SQ03`. The route hut gives rest/directions; the International Falls ranger desk owns check-in and its report flag. |
| Sedge grass neck | Core `[22,88,28,92)` forces at least four grass steps. All optional branches and the Cut lane preserve this crossing. The route connects on foot before Cut or Surf: 162 steps excluding battles. |
| Timber crossing | Meta x=11–14, north rail row32 block `3b`, south rail row33 block `3a`. Inner walking rows are core y=65/66; outer rail rows y=64/67 block. Actual WATER remains below at meta row34. Before Cut, removing this crossing disconnects the north/south route. |
| Wetland lookout | North rail meta `(14,29)`, south rail `(14,30)`. Native rock `28` at `(13,29)` and `(13,30)` closes the west end; WATER remains below `(14,31)`. Reed `(29,59)` leaves the second inner deck row free. The deck is a closed lookout, not an undeclared Surf launch. |
| Marsh-side supplies | Max Repel `(37,73)`, walking approach `(36,73)`. Watcher `(37,77)` gives a specific cache/return hint and leaves the other path column clear. Both are reachable before Cut by the longer ordinary loop. |
| Cut return | Native forest `0f` at meta `(18,34)`, CUT_TREE `(36,69)`, approach `(36,68)`. Exact source replacement is `17`; its other quadrants remain unchanged. Require party CUT and native Hive Badge permission (`ENGINE_HIVEBADGE`, badge index 1). North mouth `(36,67)` to the supplies' approach `(36,73)` costs 66 steps before Cut and six afterward; Surf before Cut still costs 66. This is a useful 60-step shortcut to side activities. |
| Cut scope | Only one native CUT_TREE quadrant exists. `cut_gates` is empty because this is a shortcut, not an exclusive area. The direct regional endpoint trip changes only 162 → 160; do not claim a60-step regional shortcut. One new FLOOR tile opens. Native denial/removal/reset/persistence still needs production input tests. |
| Granite den | Small shallow outcrop at meta `[3,19]` to `[7,21]`. Native derived cave block `34` at `(5,21)` gives CAVE `(10,43)`. Entrance1 targets `MnR17Cave1F` entrance1; passive exterior return2 is safe FLOOR `(10,44)`. Its rocky top is scenery, not an unused promised walking terrace. |
| Den interior | `MnR17Cave1F` **7×8**, template `UnionCave1F`, explicit hexadecimal rows. Native LADDER `(9,13)`, safe arrival `(9,12)`; exit pairs with exterior `(10,43)`. Existing Revive `MN_R17_ITEM_04` is at `(5,5)`. |
| Ranger hut facade | Complete authentic Ilex recipe `1d1e1f /212223 /242627` at meta `(22,22)`. Actual DOOR `(45,49)` lies in `MnR17C00R01` local `(45,1)`; passive safe return `(45,50)` local `(45,2)`. Entrance1/return2 targets `MnR17RangerHut` entrance1. The facade crosses a graphic chunk seam, but its door and approach share one native map. |
| Ranger hut interior | **5×4**, `VioletNicknameSpeechHouse` template, explicit traditional-house rows. Exit carpet `(5,7)`, safe arrival `(5,6)`. Keeper `(5,5)` declares existing native `heal_party`. This service sets no SQ03 flags or circuit rewards. |
| Forest junction | Native timber sign `(27,45)` points specifically west to the den and east to the hut. The quietest through segment is two gameplay tiles wide, not a broad empty lawn. |

Both ends and every ordinary actor/reward remain accessible without field moves.
The Cut lane is an optional return convenience. No artificial ledge was added
as a quota: this route's walking choices come from forest, habitat, marsh and
its measured return link. The native crossing is required before Cut; after
cutting, the dry return intentionally bypasses it.

All 280 WATER tiles lie inside forest/shore boundaries; zero reach an outer map
edge. No external water connection is implied. Native publication must preserve
reciprocal offset-zero walking seams at y=47/48 and 95/96. Original tree/shore
variants are fitted deliberately; ordinary canopy never uses the Cut variant.

## People, pickups and SQ03

Exterior trainers `MN_R17_T01`–`T06` are Brooke, Dana, Reed, Ellis, Norek and Alex.
Their original IDs, defeat flags and three-member parties are retained: 18
opponents total. All source parties/classes/sprites/text and approaches are
checked. Trainer07–10 reservations retain their original IDs, parties and flags
under `reserved_trainers`; none is placed.

The four original item identities persist: Hyper Potion `(55,27)`, Ultra Ball
`(11,59)`, Max Repel `(37,73)`, and the den Revive `(5,5)`. The first two cost an
optional habitat excursion. The marsh supplies offer a concrete reason to use
the Cut return. The Revive uses a real room/return rather than a decorative cave
mouth. Three outdoor residents and nine source signposts provide specific
navigation or wildlife context; one additional hut keeper offers rest.

SQ03 remains a regional circuit: check in at the International Falls ranger
desk; later, with all badges and the harbor signal, traverse R20 to Boundary
Waters; report the completed circuit for one PP Up. Preserve
`EVENT_MN_BORDER_REPORT` and `EVENT_MN_BORDER_CIRCUIT_REWARDED` at their
appropriate scenes. R17's hut does not grant either flag, replace the desk, or
bypass Q07 at the League approach. The source viewer does not execute story.

The old snow-fleck styling note remains unfulfilled. No invented snow pixels,
unsupported winter palette or ice movement mode is claimed by this revision.

## Source loop and native integration

```sh
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_border_pines.py --negative-checks
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_border_pines.py --serve --port 8792
```

The tool reads the supplied ignored source pack and reuses existing source-art,
collision, event and room helpers. It writes ignored review images only. Warm
full checks/renders measured approximately 0.3–0.7 seconds; optional negative
checks are one-shot and do not slow the watch loop.

Ten malformed copies fail: walking and Surf sidesteps around Cut, accidental
Cut canopy, outer water and walking holes, wrong timber rail orientation, a free
marsh crossing, a dry grass bypass, a fake hut door and a wrong cave exit ladder.
Both timber blocks match exact native Kanto77 source rows; no pixels are painted.

Native art work belongs to the integration owner: `mn_border_pines` inherits
existing `mn_gunflint` 28–39 and forest field moves, then appends `3a`=Kanto77
(FLOOR/FLOOR/WALL/WALL), `3b`=the same rows reordered `[2,3,0,1]` with native
Johto75 collision (WALL/WALL/FLOOR/FLOOR), through
`border_pines_art::build_border_pines_art_pack`. Keep the forest day palette and
water animation. No source exporter or pack publisher is run by this tool.

Shared integration must replace the old twelve-map world/port reservation,
register the real ranger hut, update the resized den's room contract, and add
the exact layout/dossier to typed Rust. Native controller checks still need all
walking seams, actual grass encounters, both room returns, denied/successful Cut,
its intended reset/persistence after rooms/region/Continue, trainer victory and
PP/flag changes, all four once-only pickups, actual `HealParty`, and the earned
SQ03/R20/Q07 story gates. Static floods and source renders do not establish those
results or final visual approval.

## Native art and builder handoff

New focused Rust leafs are
[border_pines_art.rs](../../../crates/crystal-assets/src/border_pines_art.rs),
[border_pines_modpack.rs](../../../crates/crystal-assets/src/border_pines_modpack.rs)
and [border_pines.rs](../../../crates/crystal-assets/tests/border_pines.rs).
They are authored and pass pinned scoped rustfmt. Root owns their shared
registration, warm compilation, publication and production input verification;
the author ran no competing build or export.

The art importer inherits all 58 `mn_gunflint` blocks, preserving their metatile
bytes, physical pixels, palettes and bank addresses. It adds only:

| Native block | Exact source recipe | Collision |
| --- | --- | --- |
| `3a` | Kanto `77`, all four original tile rows | FLOOR/FLOOR/WALL/WALL |
| `3b` | Kanto `77`, tile rows `[2,3,0,1]` | Native Johto `75`: WALL/WALL/FLOOR/FLOOR |

The target is `mn_border_pines`, manifest `minnesota-border-pines-art`, with
forest behavior inheritance for native animation and Cut `0f→17`. The sheet
retains its 192-glyph extent. Allocation protects both referenced logical
glyphs and physical bank aliases, reuses identical pixels/palettes where
possible, and writes only unreferenced physical slots. No new source catalog,
redrawn pixels or unrelated map edits are introduced.

Shared compiler registration contracts:

- Layout: `MN_R17` → `[32,56]`, `mn_border_pines`, `forest_8_neighbors`, highest
  block `3b`; schema2 and cliff-cut rejection remain active.
- Dossier: `[32,56]`, one column/three rows, six trainers, three residents,
  three outdoor items, nine signs, two doors/two rooms, prefix `MnR17`;
  `BADGE_7`, one required grass band, no ledges and no exclusive `cut_gates`.
- RouteSpec: group57, `MINNESOTA_BORDER_PINES`, manifest
  `minnesota-border-pines`, landmark `LANDMARK_MN_BORDER_PINES`, label
  `MN_BORDER_PINES`, name `BORDER PINES`, music source `IlexForest`.
- Three exterior native IDs have heights24/24/8 and reciprocal north/south
  connections. The den and hut make **five native maps** total.

Native preflight found one event-index defect in the frozen dossier. With root's
explicit authorization, the hut's `warp_index3/return_warp_index4` was corrected
to **1/2** because it occupies a different exterior chunk from the den. Every
chunk numbers its own warp list independently. No geometry, event identity,
room rows or global coordinates changed; whole image remains `6c5bbcb6`.

| Room pair | Exterior chunk / local entry | Passive return | Interior exit target |
| --- | --- | --- | --- |
| Granite den | `MnR17C00R00`, CAVE `(10,43)`, record1 | FLOOR `(10,44)`, record2 | LADDER `(9,13)` → exterior record2 |
| Ranger hut | `MnR17C00R01`, DOOR `(45,1)`, record1 | FLOOR `(45,2)`, record2 | carpet `(5,7)` → exterior record2 |

The hut coordinates above are local; global door `(45,49)` and safe return
`(45,50)` are unchanged. Source checks confirmed independent `[1,2]` index
sequences in both chunks and safe original room exits. The ten source negatives
still pass after the bounded indexing correction.

The legacy shortcut requires more than its old tile/approach parser hook.
Native validation must retain the authored `compare_from:[36,67]`,
`compare_to:[36,73]`, `minimum_saved_steps:12`, actual source `0f`, replacement
`17` and optional cache/watcher purpose. Asset tests calculate the actual
serialized walking/Surf graph before and after that source replacement:

- Local cache/watcher approach: **66→6 steps**, saving60; pre-Cut Surf is also66.
- Regional Bemidji→Falls passage: **162→160**, saving2. This is never presented
  as a60-step regional shortcut.

The tests reject a walking sidestep, a false local target, inflated saving,
incorrect badge, old global hut indices and unsafe room returns. A native WATER
sidestep beside the tree must produce an error specifically identifying the
shortcut and Surf, rather than passing because of an unrelated parser failure.
Those assertions require root's shared typed legacy-comparator hook. The asset
suite also checks source glyph equality, rail directions, all room return pairs,
original parties and the compiled native hut `HealParty` command. It does not
simulate that service, Cut input/reset/persistence, battles or earned SQ03.

Current source hashes:

- Layout unchanged: `e2aeb100c42d190c8a1b728d917f5321e8452a28bba448caf5741c7b7104d488`
- Dossier after authorized indexing fix: `0fff6bdb9750a62018ccbb93277fd1ac72414d20ac4b0643a97e690921dfe075`

Root must compile the registered art/route regressions, publish and compare the
actual native maps, then inspect headless whole/viewport captures and exercise
both room returns, native Cut denial/removal/traversal/reset, battle/PP and hut
healing through `VisibleShellController`. Native/runtime and final visual
acceptance remain pending until those checks are executed.


Warm follow-up: root reports the native art fidelity test passes. The remaining
native positive/negative tests are under root's shared legacy-shortcut validation
work. All newly authored R09/R17 terrain-negative layers now carry valid names,
and the negative suites reject missing-field/type/variant parsing failures as
invalid test evidence. A schema-only failure is never reported as successful
geometry enforcement. Source terrain and image remain frozen.

## Production controller fixtures — executed local evidence

`crates/crystal-bevy/src/bevy_shell/tests/border_pines.rs` is a focused shell-test
leaf, requiring root's module registration and the published external
`CRYSTAL_BORDER_PINES_TEST_PACK`. It does not build a pack. Its three ignored
tests now all pass Root's warm run after the bounded native carpet-departure
fix. Their setup and exact local coverage are:

- `border_pines_controller_tours_three_chunks_rails_rooms_and_open_resources`
  starts an explicitly empty-party geometry fixture at global `(32,108)`. It
  walks both directions over the clipped seams `(24,95)↔(24,96)`,
  `(10,47)↔(10,48)` and `(40,47)↔(40,48)`, both timber deck rows `y65/66`, and
  rejects movement into the north/south rails and closed lookout end. All nine
  signs, three exterior residents, three open exterior resources and the den's
  Revive are exercised. Both rooms enter, exit to passive FLOOR, remain outside
  when revisiting that return tile, and permit deliberate re-entry. Start opens
  the production menu. Empty-party geometry does not prove encounters or combat.
- `border_pines_cut_shortcut_denies_opens_walks_rests_and_resets_on_continue`
  initializes a source-compatible local MEW, Hive Badge condition, injury and
  one missing PP before gameplay. Separate fixtures test native denial without
  Cut and without the Hive Badge. The success case walks the actual dry
  **66-step** approach-to-cache route before removal, then uses native input to
  replace forest `0f→17` and walks the actual **six-step** link. The native
  post-input metatile is read only to plan inputs; no terrain override is
  injected. Max Repel and watcher remain reachable before Cut. Collected cache,
  party and PP must persist through Continue while native vegetation resets to
  `0f`. The fixture then recuts from the south, walks through, uses the hut's
  authored `HealParty`, checks actual HP/PP restoration and Continues that result.
- `border_pines_populated_fixture_wins_spends_selected_pp_and_continues`
  initializes legal Feraligatr48/Surf and Ampharos46/Thunderpunch beside Ellis at
  `(11,46)`, seven badges and a deterministic divider before input starts. It
  must defeat the actual Ursaring35/Golbat35/Delibird34 party using the production
  battle menus, spend selected Surf PP, set `EVENT_MN_R17_TRAINER_04` and retain
  party, bag, PP and victory after Continue. No state/position/divider rewrite
  occurs after setup. This is local combat evidence, not earned regional play.

Optional fixture exports are `CRYSTAL_BORDER_PINES_FIXTURE_SAVE` for geometry and
`CRYSTAL_BORDER_PINES_JOURNEY_SAVE` for the combat result. Disposable Continue
saves use the system temporary directory and are removed by a successful test.
Source-only preflight confirmed all fixture waypoint approaches and both room
interaction floors exist, the pre-Cut 66-step path contains no grass or trainer
sight, and the dry path from shortcut north arm to the hut exists. That read-only
preflight is not a substitute for executing these native tests. Root alone owns
shared test registration, warm compilation, pack publication and native review.

Root's subsequent warm run reports all three R17 native asset regressions PASS,
including source art fidelity, the actual local activity-distance comparison,
room/event lowering and corrected named/schema-guarded walking/Surf negatives.
The ten shared native surface unit tests also pass. This is asset/static native
evidence. Native publication and all three production-controller fixtures
subsequently passed as recorded above. Earned regional progression and complete
whole-route visual acceptance remain separate.
