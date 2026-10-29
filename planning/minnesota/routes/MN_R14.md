> Current grass revision: see [Headwaters verification](../revisions/route-grass-v01/current-r14-v01/README.md).
> The selected730-map R13 descendant adds native grass across the western survey
> climb: four grass steps uphill and a dry southbound ledge return. All13installed
> Lake Itasca port metatiles and the native gatehouse remain exact. Current native
> Minnesota parties, encounters and dialogue supersede the historical source below.

# Headwaters Trail — revised source candidate

The old96×72 native route had long camera-sized gaps of empty lawn, broad geometric circuits, incidental Cut canopy and river water reaching unconnected outer edges. Its historical native captures do not verify this replacement. The current route is a young-river woodland journey: a survey-pond landing and native bank on the west, a single protected river crossing, a mature-pine return circuit on the east and a smaller fen observation loop. All geography is explicitly authored; the old map generator is not used.

Current source review: [whole route before Cut](../renders/headwaters-trail/whole-route-64c05a5b.png), [whole route after actual native replacement](../renders/headwaters-trail/whole-route-after-cut-2850ab62.png), [all fourteen player cameras](../renders/headwaters-trail/viewpoints-389905a5.png), [emptiest reachable camera](../renders/headwaters-trail/viewport-emptiest_ground-e3875160.png), [Cut before](../renders/headwaters-trail/cut-gate-before-36d63bed.png), [Cut after](../renders/headwaters-trail/cut-gate-after-a433c8cc.png). These ignored native-source-pixel renders are review evidence, not proof of native input, earned progression or user acceptance.

## Exact exterior and story binding

| Property | Current contract |
| --- | --- |
| Dimensions | 48×32 metatiles;96×64 gameplay tiles;1536×1024 source pixels |
| Native chunks | `MnR14C00R00`32×24, `C01R00`16×24, `C00R01`32×8, `C01R01`16×8 metatiles |
| Bemidji | east gate meta `(46,13)`, safe arrival `(44,13)`; eastern-map local gate `(14,13)`, arrival `(12,13)` |
| Lake Itasca | west gate meta `(1,21)`, safe arrival `(3,21)` |
| Only outer walking cells | east core `(95,26/27)`; west core `(0,42/43)` |
| Internal connections | reciprocal east/west and north/south, offset0; respect16-column east and eight-row south clipped maps |
| Art | `mn_headwaters`: unchanged Johto art plus existing native source-row rail recipes80–82; no new art |
| Native group | existing group44 `MINNESOTA_HEADWATERS` RouteSpec; maps1–4 are the four current exterior chunks |
| Buildings/interiors | none; no facade door, room warp, cave, shop or invented rest service |
| Route access | preserve original `BADGE_6` |
| Quest | preserve `Q05`; its existing quest permission is independently `BADGE_7` |

The Lake Itasca marker grants `EVENT_MN_HEADWATERS_READING` on accessible dry ground. Bemidji's field officer grants `EVENT_MN_HEADWATERS_REPORT` only after the actual reading. Reading alone must never open the Iron Range gate. Route residents describe the trail and habitat; they grant no reading/report flag or healing. Distinguish the six-badge route access from the seven-badge quest permission rather than silently changing either.

All six original trainers retain their IDs, names, native classes, parties, levels and `EVENT_MN_R14_TRAINER_01`–`06`. Reserved `_07`–`10` flags remain unused. All six resident IDs, six sign IDs, four item IDs/identities/quantities/saved flags, encounters and original story identifiers were directly compared with the prior source and preserved. Positions and sight ranges now serve the actual geometry. Exact actors/parties/dialogue are in the dossier; actor positions select native16px quadrants.

## Main walk and useful branches

The main road is normally one metatile/two gameplay tiles wide. It joins both endpoints on clear floor plus only the explicitly required grass neck. Optional encounter shoulders therefore do not conceal a broken walking road. Wider bays hold the existing guide, botanist or trainer rather than becoming continuous empty lawn.

- **Young Mississippi boardwalk:** the sole walking crossing spans meta x21–26. North post rail is row18/block82, south post rail row19/block81; core y37–38 walks between them. Removing those crossing cells disconnects the endpoints in the complete floor/grass graph. There is no dry dam or alternate shore bypass. This replaces the old three-metatile-tall walking apron with two protected core walking rows.
- **Required river understory:** core `[56,34,60,42]` is tall grass, bounded north and south by continuous woodland. Every ordinary walking approach to the river crossing costs at least four grass steps. Surf is optional exploration; no claim is made that a water traveler must follow the walking encounter contract.
- **Survey Pond landing:** north rail meta x6–9,row8, south rail row9, with native40/48 at x5 closing the west end and both corners. The deck actually extends over the pond's southern inlet, joins the eastern bank and has a reachable player camera `(15,18)`. All tip and corner quadrants block walking. Bug Catcher Wade stands on the shore at `(19,17)` and the pond watcher at `(19,11)`.
- **Cedar spur and native bank:** Hyper Potion/ITEM01 is at `(31,15)`. The low bank uses native57 across meta x10–17,row12. Upper approach `(29,23)` reaches hop `(29,24)`, landing `(29,26)` and reverse wall `(29,25)`. Side barriers `(19,25)` and `(36,25)` are solid. The sole ordinary uphill opening is core `[14,24,16,26]`. Whole-route ordinary return is53 steps versus the three-tile directed displacement, saving50 steps. That return goes through the Itasca junction and pond approach; it does not cross the independent Cut gate.
- **Mature-pine circuit:** Rosie and the botanist occupy separate light gaps. Its north seed habitat holds Ultra Ball/ITEM02 at `(85,13)`. The first render revealed an unintended dry pickup approach; shortening the pale meadow entrance now requires **four actual grass steps** to reach any interaction approach. A coherent pine-root finger narrows the weakest upper camera and connects to the forest band rather than scattering props.
- **Fern hollow:** Lucy, the hollow walker, flowers and a clipped southern fern fan occupy a smaller returning circuit below the Itasca road. Max Repel/ITEM03 is `(11,57)`. This is an ordinary pause/interaction area and not an unsupported healing service.
- **Eastern fen:** Sel, the marsh watcher and Ben's actual water-facing fishing camera share a small bent pool with separate banks. The smaller shore loop leads to Revive/ITEM04 at `(91,55)`. Removing unused exterior reserve did not remove any of the original pickup identities or parties.

## Purposeful native Cut shortcut

The retained identity `MN_R14_SHORTCUT` now opens a useful north/south link through the pine circuit's real forest band. Source Johto5b at meta `(35,10)` has its only `CUT_TREE` quadrant at core **`(71,20)`**. Approach is `(71,21)` facing north; the opposite join is `(71,19)`. The native replacement is **5b→3c**, preserving the other three source quadrants. Core `(70,20)` is HEADBUTT_TREE and `(72,20)` is ordinary forest: there is no local walking sidestep.

Both arms remain available by the ordinary eastern loop. Before Cut their shortest whole-map walking connection is **34 steps**. After opening the native quadrant it is **two steps**, saving32. The named gated target is the previously non-walkable connection cell `(71,20)` itself; `gated_actor_ids` is empty because no item or actor is falsely described as sealed. `shortcut` is intentionally present because this is an actual measured shortcut; the required route and every ordinary actor remain reachable before Cut. This distinct route purpose does not copy an incidental decorative tree or another map's resource cache.

The source proof inventories every placed collision quadrant: exactly one `CUT_TREE`, matching the declared gate. Native permission requires a party move `CUT` plus Johto badge index1 (Hive). The original route `BADGE_6` story requirement is retained separately. Production proof must still deny missing move/badge, execute the actual native override, walk the short link and establish map-return/Continue reset behavior. Cutting is not assumed permanent. The old native R14 Cut fixture used different coordinates and does not prove this gate.

## Complete water boundary

The young river's unconnected north end is visibly closed at meta row1,x22–24 with native30/31/32, whose collision contains only BUOY and WATER. Its south end is closed at row30 with38/39/3a. These native blue-water rock blocks are listed in `additional_water_blocks`, preserving water shoreline fitting beside the rock glyphs. No land boulder is pasted onto water.

All96×64 core outer-edge cells are checked: zero WATER, WHIRLPOOL or WATERFALL, and no walking openings beyond the two endpoint pairs. There are zero WATER↔WATER pairs across x63↔64 and ten across y47↔48. Actual reciprocal north/south records are required for those internal water seams; the checker additionally validates all eight internal reciprocal neighbor records when native R14 chunks are present. The source pack correctly reports `published_reciprocal_neighbors_verified:false`. Native Surf movement against both rock caps and through actual water seams remains pending.

## Fast review and honest verification

Applied the reusable map-authoring skill and user guidance. Inspected the requested Johto/Kanto render, authentic native Route30/Route29 and Ilex Forest art, old complete R14 terrain, all old source cameras and the old native review sheet by sight. The source composition uses unchanged native Johto pixels and existing post-rail recipes, with30 metatile varieties.

The first replacement render exposed a disconnected fen entry, a sign on a map seam, a blocked tree-crown viewpoint and a free dry approach to a supposedly grass-priced reward. These were corrected in the authoritative layout/dossier. The current whole image, all fourteen camera views, actual widest/emptiest camera `(71,17)`, bank, landing and before/after Cut images were visually inspected. The weakest current camera contains the north-circuit turn, trainer, close pine anatomy and the actual Cut-band context. Its largest blank-ground square is three metatiles; this is a review aid, not a visual acceptance threshold. Root reviewed the frozen whole/worst camera and accepted the source integration candidate; final native/user judgment remains separate.

```sh
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_headwaters_trail.py
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_headwaters_trail.py --serve --port 8788
```

The current dedicated watcher is `http://127.0.0.1:8788/`, process session25722, log `.local/headwaters-trail-review.log`. A measured warm complete before/after check and render took **507.1ms**. It caches source art and writes immutable complete pre/post-Cut renders, viewpoint sheets, weakest cameras and before/after gate views. The shared `review_headwaters.py` `prepare` API remains unchanged for other route authors. That older renderer/runbook evidence is historical to the nine-map source; use this dedicated checker for the new proposal.

Source checks pass for clear-road continuity, both endpoint arrivals, all four useful chunks, legal native classes/species/items, text width, actor/resource/sign approaches, necessary bridge and rail quadrants, landing closure, whole bank return, required encounter cost, grass-priced seed resource, exact native Cut source/replacement/sole inventory and measured shortcut, orthogonal native strokes and the entire walking/Surf perimeter. Source checks do not execute native scripts or establish final visual quality.

## Pending native integration

Root owns shared world reservations, atlas, Rust registration and pack publication. No competing exporter, shared-cache compilation or Cargo build was run here. Reconcile the old96×72/nine-map geometry before rebuilding; old `minnesota-headwaters.crystalpack` evidence and native captures are stale for these coordinates. The old compiled pack may preserve historical native content, so publish this proposal deliberately from the completed shared source pack rather than treating extra obsolete chunks as current route connections.

After publication, run exact block/object/neighbor comparison with this dedicated checker and `--pack content-packs/minnesota-headwaters.crystalpack --verify-published` only after the integration owner rebuilds that ignored local pack. Drive `VisibleShellController::press` across the sole boardwalk in both directions, against its rails and the closed pond landing, through required grass and optional seed resource, through denied/successful Cut and its measured link, down the bank with reverse denial and ordinary return, and through both clipped native water seams and rock caps. Check trainer battles/PP, pickups and Continue persistence. Compare same-position/time/facing native2D captures to current source views. Finally bind the actual Bemidji/Lake Itasca ports and prove earned `BADGE_6` access, `BADGE_7` Q05 permission, marker reading and once-authoritative report without allowing raw reading to open Iron Range. No new native acceptance is claimed here.

## Four-map native test migration

Root updated the shared Layout/Design registration to48×32 metatiles and2×2 chunks. The Cut shortcut now explicitly compares core `(71,21)` to `(71,19)` with minimum12 saved steps. Root's shared native comparator measures actual pre/post-replacement paths with walking and Surf enabled separately; the current native surface regression expects34→2 in both modes. Those changes retain the frozen terrain and actual5b→3c replacement. No geometry was changed during this test migration.

The existing [asset test](../../../crates/crystal-assets/tests/headwaters.rs) now composes onto a stable, already revised **R16** overlay from the verified external source. It no longer depends on the obsolete R11/R12 source geometry. `CRYSTAL_HEADWATERS_BASE_PACK` may select another completed base without R14/R16; the selected path is canonicalized before verified loading. The default remains ignored `content-packs/text-tui.crystalpack`.

The positive test checks all four clipped native payloads, group44/maps1–4, reciprocal offset0 neighbors, no room warps, unchanged existing maps/runtime files/metadata/trainers/items/tilesets/audio/HOME/recovery, exact original actor IDs/coordinates/parties/defeat flags/item balls and inherited native art bytes. It reads the resulting native collision independently to check the single necessary bridge and its exact two-row rails, the closed survey landing, actual bank quadrants, sole Cut inventory,34→2 walking shortcut, exact four outer walking cells and complete Surf perimeter. Negative tests mutate the actual necessary bridge, required habitat, native hop, native Cut source, incidental Cut and unconnected river edge; separately they delete required dossier contracts or falsify shortcut comparison/savings/replacement and ledge return. These focused migration tests are prepared, not recorded as passing until Root's warm execution. The missing-contract negative cases deliberately expose any permissive shared R14 schema instead of weakening the assertions.

The existing [controller leaf](../../../crates/crystal-bevy/src/bevy_shell/tests/headwaters.rs) now has three separate ignored fixtures, retaining the existing test registration. `CRYSTAL_HEADWATERS_TEST_PACK` selects the canonical native route pack and defaults to ignored `content-packs/minnesota-headwaters.crystalpack`; the historical nine-map pack must be replaced before running these tests.

- The **empty-party geometry fixture** starts at core `(88,26)`, traverses all three reachable walking chunk-neighbor pairs both ways and both boardwalk rows in both directions, checks native rails and the survey landing's closed end, walks the four-cell required grass neck, performs the exact `(29,23)` → `(29,24)` → `(29,26)` native hop and reverse denial, then drives all53 ordinary uphill steps. It talks to six residents and six signs, collects all four persistent items, opens production Start, Saves/Continues and proves hidden pickups remain absent by walking onto their former tiles. None of the six trainer flags may be set.
- The **local Cut fixture** separately denies a partner lacking Cut and a Cut partner lacking Hive permission. Each test installs its native HM-compatible Mew/allowed moves/badges only at initial setup. The permitted fixture first walks the actual34-step dry connection in each direction before cutting, then executes native5b→3c and walks the two-step link. Leaving the native map over the real x63↔64 seam and returning must regrow temporary vegetation. After another actual Cut, Save/Continue must preserve the partner/position while native temporary vegetation regrows again; recutting from the northern arm must reopen traversal. No permanent Cut event or sealed reward is invented.
- The **battle fixture** initially installs native-created level45 Typhlosion and Feraligatr plus six obedience badge bits at core `(80,31)`. Those are fixture setup, not earned partners/badges or a balance claim. It talks to actual Sivren/T01 through normal input, selects legal native damaging moves, observes selected PP consumption, faces both authored opposing partners, wins and checks money plus the actual defeat flag. Actual movement, Save/Continue, exact full-party equality and a defeated-trainer conversation without another battle follow. Other trainer flags remain unset.

After initial setup every gameplay action uses `VisibleShellController::press` or its native presentation settling; `RouteGrid` only plans inputs. No party, badge, flag, coordinate or terrain is rewritten during those journeys. No story flags are injected, and these local fixtures do not claim earned Q05, marker/report access, the remaining populated route or final visual acceptance.

Both changed Rust test leaves pass scoped `rustfmt --edition 2024 --check`. This author ran no Cargo build or publisher. A fresh complete source rerender/check passed in485.8ms and retained the exact whole `64c05a5b`, post-Cut `2850ab62`, camera and gate hashes above. Root owns warm compilation/execution, publication, exact native comparison/audio verification and production captures; their actual results remain to be recorded.
