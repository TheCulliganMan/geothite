# Gunflint Trail — current source review candidate

## Current executed native evidence — 2026-10-26

Root published the current **439-map** pack with six clipped exterior maps and
two original rooms. Exact comparison passed **3,584 exterior metatiles**, current
events and room returns. All **three native asset suites** and **three production
controller fixtures PASS**. Input proves the five actual exterior seam pairs,
both portages, ordinary resources and both original room returns; native Cut
denial/removal, sealed Hyper Potion and tree reset through Continue; and Milo's
actual Noctowl/Xatu battle with selected Surf PP, victory and persisted party/flag.
June preserves HP/PP deficits because she has no authored healing service.

Initial placement, legal partner/badge/divider setup is explicit fixture setup,
not earned Q07 or regional progression. The old cave-roof scenery seam is
excluded. Current audio evidence is the shared 443-map composition's **377**
PCM hash/frame-count/loop checks. Final whole-route/user visual acceptance and
earned regional travel remain separate. Earlier pending notes below are
historical handoff snapshots, superseded by these executed results.

The current proposal is **64×56 metatiles**, with six clipped exterior maps and
both original rooms. Review the [whole map](../renders/gunflint/whole-route-b85c90e4.png),
[player views](../renders/gunflint/viewpoints-b85c90e4.png),
[after-Cut whole map](../renders/gunflint/after-cut-3cb00ca6.png), and
[room source preview](../renders/gunflint/interiors.png). These are exact native
source pixels over authored terrain; current native publication and focused
controller evidence are recorded above.
The earlier 96×96 native pack, screenshots and controller fixtures describe
historical geometry and do not validate these changes.

The old full map and saved production montage were inspected before redesign.
Its broad empty lawns, similar lake circuits, avoidable habitat and pink masonry
bridge did not meet the current guidance. The revised main trip uses narrow
forest paths and a short required habitat band. West Portage has a long shaded
shore discovery; Loons has two lake lobes separated by a pine/sedge isthmus, with
a real short-portage/long-bank choice. Stone Bog has an actual grassy land neck
between separate pools. Raven Rock is a small shelter excursion, while the
southern ranger trail connects a real hut, spring and Bear Hollow habitat.

The first draft had an isolated geology turnout, an accidentally open Cut cache,
a southern bypass around the required grass and an excessive ranger yard.
Those defects were fixed and the complete map and widest-yard camera inspected
again against native daytime Ilex Forest and Route42. The small Raven Rock top
is scenery; it is not a promised walkable terrace. There is no invented canoe
controller, train, fake dock, automatic healing service or ledge quota.

## Authoritative files and fast review

- [MN_R12.layout.json](MN_R12.layout.json) contains explicit orthogonal strokes,
  exact source metatiles and the native art recipe. No terrain seed or generator.
- [MN_R12.json](MN_R12.json) contains event coordinates, parties, story conditions,
  room rows, portage costs, Cut targets and shared-integration requirements.
- [review_gunflint.py](../tools/review_gunflint.py) checks native source collisions,
  before/after walking and Surf, actor/door approaches, rooms and malformed copies.
  Its shared `flood`, `neighbors`, `check_rooms`, `VECTORS` APIs retain their behavior.

```sh
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_gunflint.py \
  --pack content-packs/minnesota-gunflint-art.crystalpack --negative-checks
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_gunflint.py --serve
```

Live source review uses <http://127.0.0.1:8767/>. Whole-map/camera rendering takes
about 0.5 seconds warm; source changes refresh without Rust compilation or pack
publication. The checker uses read-only supplied content and an in-memory
collision view. When the prepared art-only pack is supplied, all 40 original
forest blocks and 18 native imports compare pixel-for-pixel with source art.
The route uses 39 distinct source metatiles. No new art imports are required.

## Connections and progression

One metatile contains four 16px gameplay tiles. Event coordinates below are
**global gameplay tiles**. Chunk origins are `(column×64,row×48)`; subtract them
for native local coordinates. Rows 0/1 are 32×24 metatiles; row 2 is 32×8.
The six IDs are `MnR12C00R00` through `MnR12C01R02`. The old third column and
fourth row are no longer part of this proposal; root owns their shared removal.

| Port | Meta gate / clear landing | Global gate / clear landing | Native chunk / local gate |
| --- | --- | --- | --- |
| Grand Marais east | `(62,28)` / `(60,28)` | `(124,56)` / `(120,56)` | `MnR12C01R01` / `(60,8)` |
| Boundary Waters west | `(1,28)` / `(3,28)` | `(2,56)` / `(6,56)` | `MnR12C00R01` / `(2,8)` |

The only outer walking openings are `(0,56)`, `(0,57)`, `(127,56)`, `(127,57)`.
All actual water is enclosed; outer WATER count is zero, including clipped edge
chunks and shoreline corners. Existing lakes deliberately permit Surf from
ordinary accessible shores; none leads off-map. Main travel does not require
Surf or Cut. Physical hub arrivals, reciprocal ports, gatehouse behavior and
regional recovery still require root's integration and controller evidence.

**All eight badges plus `EVENT_MN_LIGHTHOUSE_SIGNAL` remain required.** Q07
starts at Duluth's receiver and continues to the Team Lode chief/restoration at
the free Boundary Waters story shelter. June offers directions only. Neither
room grants a badge, lighthouse signal, chief-defeat or restoration flag. Paid
Safari entry is not a prerequisite for the free story path.

## Traversal and habitat contracts

| Feature | Exact walking anchors / grass bounds | Actual source comparison |
| --- | --- | --- |
| Main Windthrow habitat | grass `[56,64,58,68)` | Every ordinary east↔west path crosses at least four grass steps; no free dry shoulder |
| Loons pine isthmus | west `(82,36)`, east `(120,36)`; grass `[98,36,108,38)` | 38-step short path includes 10 grass tiles; avoiding that band takes 86 walking steps |
| Stone Bog portage | north `(32,78)`, south `(32,90)`; grass `[32,80,34,90)` | 12-step land crossing includes 10 grass tiles; clear bank alternative takes 86 steps |

Main endpoint travel is 164 ordinary walking steps before and after Cut. The
Loons water bodies are separated by actual dry forest/grass, and the bog pools
are separate enclosed bodies. Surf is an intentional native option at open
shores; the dry-path cost comparisons do not claim Surf costs. The native grass
encounter tables/rates remain the original R12 proposal. Total reachable habitat
is 100 gameplay tiles. Outdoor clear and grass geometry yields 2,040 pre-Cut
walking tiles and 2,066 after Cut; walking+Surf yields 3,584 and 3,610 respectively.
Actor bodies are excluded from each graph. All six chunks contain reachable
terrain. Ordinary main travel can avoid initial trainer sight.

All ten original trainer IDs, names, parties, levels and defeat flags remain;
all six residents remain. Their exact revised tiles/facings are in the dossier.
There are 24 original trainer opponents, four outdoor pickups and the original
cave Max Ether. No party or event flag is generated by the renderer. The existing
six sign IDs remain, with Stone Bog's text corrected to describe the grassy
portage. No reserved trainer identities were silently discarded.

## Actual Cut gate

This replaces the old unmeasured return shortcut with a **sealed optional herb
cache**. `shortcut` is null; `cut_gates` names `MN_R12_SHORTCUT` to preserve its
identity. It makes no claim to shorten regional travel.

- Original native forest `0f` at meta `(27,11)` contains `CUT_TREE` at `(54,23)`.
  Approach from `(54,24)`; native replacement is exactly forest `17`, preserving
  its remaining headbutt-tree quadrant.
- The only gated actor is `MN_R12_ITEM_01`, the original Hyper Potion at `(53,19)`.
  Its explicit interaction approach is `(54,19)`. This target is unreachable
  before Cut even with walking+Surf, and reachable afterward. The gate opens
  26 additional walking tiles. Forest and rock close every other entrance.
- The source field-move catalog requires a party member knowing **CUT** and
  Johto **HiveBadge**, badge index 1. All other outdoor actors, pickups, room
  approaches and required traversal remain reachable before cutting.
- Every source `CUT_TREE` quadrant is inventoried; there is exactly this one.

The historical native implementation regrew temporary vegetation through its
map/Continue callback. The revised source does not prove denial, removal,
traversal, leaving/returning, saving or reset behavior. Root must test those
against this revision and preserve actual engine semantics. Exact source crops:
[before](../renders/gunflint/cut-before-b85c90e4.png) and
[after](../renders/gunflint/cut-after-b85c90e4.png).

## Rooms and native event lowering

| Entrance | Exterior native warp | Clear step out | Native exterior chunk / local warp | Preserved interior |
| --- | --- | --- | --- | --- |
| Raven Rock | CAVE `(66,23)` | `(66,24)` | `MnR12C01R00` / `(2,23)` | `MnR12Cave1F`, 16×12 |
| Ranger hut | DOOR `(85,83)` | `(85,84)` | `MnR12C01R01` / `(21,35)` | `MnR12RangerHut`, 6×5 |

Both exterior warps are index 1 in their separate chunks, target interior warp 1,
and return onto the exact exterior warp before walking to the safe step.
The cave uses existing native imported `34` (`Johto73`) and the hut uses the
complete canonical Ilex facade `1d/1e/1f`, `21/22/23`, `24/26/27`. The source
checker inventories every CAVE/DOOR quadrant, rejecting unclaimed fake entrances.

Both original room block arrays, borders, NPCs and pickup remain unchanged.
Raven Rock has two passages around its central rib, Otta `(5,5)`, Max Ether
`(27,5)` with `EVENT_MN_R12_CAVE_ETHER`, native LADDER exit `(17,23)`, and safe
footing `(17,22)`. The hut retains June `(7,3)`, wall map `(4,0)`, carpet exits
`(4,9)`/`(5,9)` and safe arrival `(4,8)`. June's navigation text now describes the
saddle rather than the removed masonry crossing. No healing service is claimed.

## Evidence and remaining work

The source review passes all room/event footing checks and twelve deliberately
malformed copies: Cut walking/Surf bypasses, an accidental Cut variant, outer
water/walking holes, paved main/Loons/bog grass, fake cave/hut doors, a diagonal
closing edge and the wrong cave ladder quadrant. The old published pack is
explicitly rejected by `--verify-published` because its metatiles differ.
R09 and R17 source checks still pass after the shared helper-preserving changes.

Root owns shared world/interior registration, typed Rust lowering, pack
publication, audio-preservation checks and current native/controller review.
In particular, remove old fixed 96×96/12-chunk and `35` masonry assumptions from
the R12 publisher/fixtures; lower `cut_gates` and the current required grass
contract. The existing `mn_gunflint` art set is sufficient. Do not present the
historical 14-map overlay or its explicit level-60/Mew/badge fixture as validation
of this 8-map proposal or as earned regional progression.

Native captures must be offscreen and include the current wide yard, isthmus,
Cut gate and both room returns. Final visual acceptance remains open until root
and the user have reviewed the current whole map and native views.

## Current native migration fixtures

The native asset suite in `crates/crystal-assets/tests/gunflint.rs` now targets
this frozen **64×56 / 2×3 / eight-map** proposal. It composes over current R03,
not the independently revised R11. Preservation checks cover all base maps,
runtime files, art, metadata, trainers, encounter tables, Pokémon/items/marts,
HOME/recovery and audio. It checks six clipped exteriors, source room arrays,
all actor positions, original parties, the single declared Cut quadrant,
26-tile opening, sealed pre-Cut walking+Surf target, required habitat and closed
outer water. Terrain-negative strokes are named and schema-only errors are
rejected as invalid evidence. The specific Surf bypass must name
`MN_R12_SHORTCUT` and Surf in the error. These migrated Rust tests are awaiting
root's warm execution; writing or formatting them is not a passing result.

Both original room **trigger-return** contracts remain exact. Their exterior
warp index is1, with no passive FLOOR warp record. Raven Rock returns onto
`(66,23)` in `MnR12C01R00`, then permits `(66,24)`; the hut returns onto `(85,83)`
in `MnR12C01R01`, then permits `(85,84)`. Native arrival suppression must prevent
an immediate re-entry loop while standing on the return trigger. No new return
indices or revised source rows were invented to fit a test.

The replacement shell leaf `crates/crystal-bevy/src/bevy_shell/tests/gunflint.rs`
uses `CRYSTAL_GUNFLINT_TEST_PACK` and production input only after explicit local
setup. Its three ignored tests have now **passed Root's native runtime run**:

- `gunflint_controller_walks_six_clipped_maps_portages_and_original_rooms` uses
  an empty-party geometry fixture at `(124,56)`. Five actual internal walking
  seam pairs are crossed in both directions. The apparent upper `(63,17)` seam
  is isolated cave-roof FLOOR behind a terminal CAVE entrance and is explicitly
  excluded; the lower east/west adjacency is also terrain-closed. Both portages, all six residents/six signs, three
  open outdoor rewards, Otta, original cave Max Ether, June and the wall map are
  exercised. Both room return triggers must stay stable while settling, permit
  the safe step out and support deliberate re-entry. Both hut carpet exits are
  used. Production Start is checked. The sealed Hyper Potion is excluded.
- `gunflint_cut_cache_denies_opens_collects_and_continues_native_tree_reset`
  starts separate source-compatible MEW/move/badge fixtures at `(54,24)` before
  input. Native denial is checked independently without Cut and without the
  Hive Badge. Successful input applies `0f→17`, walks into the actual cache and
  collects Hyper Potion. Continue must retain party, bag and reward flag while
  regrowing the native tree; recutting from inside must allow escape. A real
  June visit preserves the partner's HP/PP deficits because she has no authored
  healing service. The completed room return is saved and Continued again.
- `gunflint_populated_fixture_wins_spends_selected_pp_and_continues` starts a
  legal local Feraligatr52/Surf and Ampharos46/Thunderpunch fixture beside Milo
  `(107,52)`, eight badges and a deterministic divider before gameplay. It must
  beat the actual Noctowl40/Xatu40 party using production menus, spend selected
  Surf PP, set `EVENT_MN_R12_TRAINER_02`, and retain party/bag/PP/flag through
  Continue. No post-start state, position, divider or story-flag injection is
  used. These local fixtures do not prove earned badges, lighthouse signal,
  regional gates or Q07 completion.

`CRYSTAL_GUNFLINT_FIXTURE_SAVE` optionally exports geometry;
`CRYSTAL_GUNFLINT_JOURNEY_SAVE` optionally exports the combat result. Disposable
Continue saves are removed after successful tests. Pinned Rust1.99 formatting
passes for both migrated leaves. Cargo, native publication and module
registration remain root-owned. The source layout/dossier hashes above remain
unchanged; a read-only source recheck passed with effective walking/Surf cache
bypass, paved main habitat and reachable outer-water mutations rejected.

Root's warm migration run now reports all three current native R12 asset suites
PASS. It also exposed a test metadata lookup using a display name rather than
the native map constant; that lookup was corrected before the passing rerun.
All three migrated production-controller fixtures subsequently passed Root's warm run.

Root's native controller review corrected an overclaimed source preflight seam:
my seam search had allowed CAVE as a through cell, which incorrectly connected
the isolated Raven Rock roof. The production fixture correctly treats exterior
doors as terminals; root removed `(63,17)↔(64,17)` from its walking tour and
asserted that scenery is excluded. Five real seam pairs still connect all six
chunks. No source geometry changed, and no cave-roof traversal is claimed.
