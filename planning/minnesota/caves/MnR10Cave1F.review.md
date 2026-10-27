# R10 seam gallery — source review V04

Authoritative candidate: `MnR10Cave1F.candidate.json`, SHA256 `a3ea9a75a23c0b8e0a335aa47ceeffd5762efdccee16512168523260f115aef6`. Native 505-map review candidate published by Root; all three actual R10 production-controller cases now PASS in 8.57s. Guarded shared source promotion is applied. Parent owns publication and broader regional acceptance. The existing native room is12×8; Root's previous candidate already chose18×14. This refinement retains that candidate extent, not an additional area increase.

Review folder: `../renders/caves/r10-candidate/brainerd-gallery-v04/`. Whole,160px cameras, integer enlargements, original terrain/RGBA/RGB stages, exact candidate source, comparison with Root's previous whole, source check and numeric geometry proof are retained. Rejected V01/V02/V03 and Root's originals remain separate. V03's first stair had a blocked upper approach; V04 explicitly fixes it rather than counting the far-side detour as a working climb.

The previous candidate had a large rectangular central rock block, square spring and broad lower hollow. V04 tapers and bends the ore spine into the lower undercut, makes the spring an irregular diagonal-bank inlet, and folds the lower return round the spine. The entrance's160px camera now shows both the upward gallery stair and lower return stair, giving the player a visible choice. The upper gallery's short native cave39 drop is an actual fast return toward the mouth, with a13-step ordinary ascent back round the left stair. No new ladder destination, story flag or HM reward is implied.

Preserved exact native contracts:

- Exit core3,13 with exactly one LADDER quadrant and clear arrival3,12. Exterior trigger36,17 and safe return36,18 remain reserved unchanged.
- Original `MN_R10_GALLERY_READER` record is restored exactly from `routes/MN_R10.json`: tile9,5, east facing, native sprite, and original four lines beginning “Both passages”. The first Root candidate had changed the text and position; no new dialogue is requested by this refinement.
- `MN_R10_ITEM_04`: REVIVE×1, `EVENT_MN_R10_ITEM_04` retained. Candidate location29,23 is two core tiles south of Root's29,21 so the reshaped spring cannot place the item on WATER. The actual installed old item location17,5 will require an explicit reviewed relocation in a future native cave update; this is not silently installed now.

Exact stair lanes, all FLOOR both ways with blocking authentic side quadrants:

| Native block | Metatile | Continuous ordinary core lane |
|---|---|---|
| cave12 |3,4| x7,y7..10 |
| cave36 |10,3| x20,y5..8 |
| cave36 |10,4| x20,y7..10 |
| cave12 |3,7| x7,y13..16 |

Drop `GALLERY_MOUTH_DROP` uses native39 at metatile6,3. Upper approach12,5; hop starts on HOP_DOWN12,6, crosses WALL12,7, lands FLOOR12,8. Side barriers11,7 and14,7 are WALL. Ordinary return to12,5 is13steps with actors present; native reverse input must reject12,7. `geometry-proof.json` records every exact quadrant and ordinary return. These are source collision checks using supplied native data; the actual hop still needs `VisibleShellController::press` execution.

Fast loop used:

```sh
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_cave_candidate.py \
  --pack .local/minnesota-city-revisions/cities-37a09cd52d95b7425711237a4014ec387555a27cc3d4df61e246581b4d108cf6.crystalpack \
  --room planning/minnesota/caves/MnR10Cave1F.candidate.json \
  --output planning/minnesota/renders/caves/r10-candidate/brainerd-gallery-v04
```

Result: resident approach12steps; cache approach36steps; entire declared ordinary loop connected, no unpaired ladder, no outer water. Personally inspected the whole, entrance, upper-gallery/drop, east stairs, spring fork, cache and widest lower-bend cameras against native Union Cave/Dark Cave/Mt Mortar references. Remaining judgment: the east boulder shoulder is a modest floor fork, not a new mechanical obstacle; no claim of universal cave quality approval follows from this one candidate.

At the initial V04 source review, native acceptance was pending for: exact preimage/old room preservation checks; only this room's geometry/item relocation plus explicit metadata changes; paired entrance/exit through production controller; all four stair lanes both ways with side rejection; real39 hop and blocked reverse plus ordinary return; original resident text ownership; real Revive grant/flag/repeat/inventory-full behavior; Save/Continue; and matching headless native camera captures. No regional story completion is inferred from local fixtures.


Controller acceptance source is now `crates/crystal-bevy/src/bevy_shell/tests/r10_gallery.rs`. Three ignored cases target `CRYSTAL_R10_GALLERY_CONTROLLER_PACK`; all three now pass against Root's exact revised e0 pack; the initial authored/pending state and failed diagnostics remain retained. Initial local position/party/bag/RNG fixtures are explicit; these cases do not earn regional mine access.

- Original exterior mouth and passive return, all four stair lanes in both directions with native blocked sides, actual short downward hop/blocked reverse, and the thirteen-step ordinary return.
- Original reader text with movement ownership and actually completed visible pages, original Revive collection/event, no second reward after reentry, and semantic Save/Continue checks for party, bag, true event/engine flags, money and location.
- Full twenty-slot Item pocket refuses the original item ball without changing party, bag or collection event. The controller then opens Start/Pack, uses its single Potion on an injured partner (actual HP +20 and consumption), and retries the original Revive successfully. `crystal-core/src/systems/field_items.rs::pickup_field_item` returns `BagFull` before setting the event, so this retry tests the existing native program without changing it. No post-fixture state edits free the slot.


Production controller investigation: the corrected page observer found an actual successful-item presentation gap. The graphical schedule calls `begin_visible_field_item_sound` after a matching `FoundText` printer completes, then advances `advance_visible_field_item_fanfare_pause`; the renderer-neutral settling loop previously did neither. A then removed its field notice while leaving the item phase at `FoundText`. The actual revised R10 cases exposed this after both the first successful Revive and the inventory-full retry. Logs retain this failure; it was not accepted as an idle state.

The focused `field_item_controller_clock.rs` leaf now invokes the existing native transitions from the shared `VisibleShellController` settling path: matching fully printed found text, existing sixty-frame fanfare clock, then the native pocket text and A/B acknowledgement. The R10 cases additionally require both fully revealed pocket pages and rejected movement during the real notice, then no stale notice, unchanged party/PP, actual event/quantity, once-only collection and semantic Continue. Root compiled the production fix and all three strengthened actual R10 cases PASS in `.local/r10-gallery-controller-item-fixed.log` (8.57s).

Observation detail: the public controller snapshot intentionally projects only the current text page. Test page identity is resolved from the same underlying presentation snapshot used by the production printer, with full glyph completion and revealed-render equality; the public snapshot's current page is then checked against it. Bounded diagnostics preserve both views and identify every settling stage.


## Guarded source promotion — applied after native acceptance

Current shared preimages were frozen under `../renders/caves/r10-promotion/preflight-fb51461d98d7d988/`: exact full `MN_R10.before.json` and `interiors.before.json`, isolated `room.before.json` and `contract.before.json`, exact candidate copy, file hashes and native evidence pointers. That immutable preimage contains the original12×8 room; the shared route now contains the reviewed18×14 room. Its whole route SHA is `862a929551200665b9eaa4b43ad474636acc69b5961f4e438cf6729f21b92d77`; interiors SHA is `46759ac690e5c9790f9c5cbfbccbf1e329fdc1f61339a3d073552ec31e8f1f59` at this freeze. Unrelated later record changes are allowed, but the two owned room/contract records must still equal their frozen preimages.

`promote_r10_gallery.py` defaults to preflight and produces isolated proposed records in that ignored snapshot. Preflight was executed with `applied:false`, then Root confirmed all three native cases passed and explicitly authorized the successful `--apply` run. It verifies the exact candidate SHA, immutable native pack SHA `e0cf8cf00b18c1b25c7cab70e37ebbcc93f1ff25d67fb7aa7f4c4371d567085e`, all252 native source blocks,504 unchanged unrelated maps, original reader/program/pickup/warp preservation and matching NITE evidence. This is source JSON promotion only; it never writes a content pack.

The native palette is **NITE**, inherited from the original cave's native map attributes. The previous day-palette source previews remain retained and are not the palette authority. Exact native NITE comparisons and six headless production captures are in `../renders/caves/r10-native/e0cf8cf00b18c1b2/review-1793065743751041000/`. Parent owns their acceptance. No new cave palette or source pixels are authored by promotion.

```sh
python3 planning/minnesota/caves/promote_r10_gallery.py \
  --snapshot planning/minnesota/renders/caves/r10-promotion/preflight-fb51461d98d7d988
```

Root confirmed all three production-controller cases passed; the command was repeated with `--apply --controller-log .local/r10-gallery-controller-item-fixed.log`. The script rejects failed logs, missing cases, changed candidate/native pack, changed frozen owned records or mismatched native palette/evidence. No apply was run while waiting for that result; application occurred only after Root supplied the PASS result and authorization. The script splices only each owned JSON object, retaining every other byte in both files, stages both complete documents and rechecks the complete current files before replacement.

The proposed room is the18×14 candidate's exact geometry, rows, original whole Nerd/text record, sole Revive×1 program/flag with its explicit17,5→29,23 relocation, four native stairs and short native39 drop. Its candidate-only publication annotations are updated explicitly; the historical candidate file and all earlier renders remain unchanged. The interior contract records18×14 and inherited `nite`, keeps native ladder arrival3,13/clear3,12, exterior trigger36,17 and passive return36,18, and replaces the stale413/469-map12×8 status with bounded505-map native review wording. Regional Q06 progression and user final visual approval remain outside these local fixtures.


Promotion evidence is retained in `../renders/caves/r10-promotion/preflight-fb51461d98d7d988/application-1793067244569140000/`: exact complete files immediately before replacement, passing controller log, final SHA256 values, promotion receipt and post-apply source preservation checks. Only `MN_R10.interiors[MnR10Cave1F]` and its global interiors contract changed. All other parsed fields and bytes outside those objects were preserved. Native-content fields equal the frozen candidate; only publication annotations changed. The candidate JSON itself remains byte-identical at a3ea9a75….

`check_plan.py --pack .local/minnesota-city-revisions/gallery-e0cf8cf00b18c1b25c7cab70e37ebbcc93f1ff25d67fb7aa7f4c4371d567085e.crystalpack` PASS after promotion:20 routes,19 hubs,57 chunks,86 adjacency/gate contracts,158 planned maps; species/pickups resolve against the actual pack. Its symbolic story result is not earned controller progression. Actual e0 written-pack preservation is 125/125 PASS at `../renders/northwoods-native/proof/r10-gallery/comparison-ae43e7811f5c2cd5c2e88c1441ecf2d95053f144396119899111a3d1e505f3b8.json`; six headless production captures remain separate evidence. Root owns shared status, playable summary and atlas updates.
