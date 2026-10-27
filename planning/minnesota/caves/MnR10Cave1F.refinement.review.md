# R10 gallery — refinement of the installed18×14 room

Source-only candidate: `MnR10Cave1F.refinement.candidate.json`, SHA256 `be41b5fb81f3ef8536214e903a07d28685bd71b4383cb4c3fade9ebf8ab73161`. Exact current preimage: `MnR10Cave1F.refinement.previous.json`, SHA256 `2e320c5676fcbb330152cb088e75a688ef344942add7402f48b58e2265b4b5f7`. The baseline is the installed18×14 gallery from limestone pack e549c178… and active promoted R10 source, **not** the historical12×8 cave or the earlier candidate's unpublished annotations.

Evidence: `../renders/caves/r10-refinement/be41b5fb81f3ef85/`. Review `whole-before-after.png`, `whole-2x.png`, `decision-cameras.png`, `reader_approach-4x.png`, `lower_return-4x.png` and `actual_weakest_floor-4x.png`. Full original route/room/native field hashes and whole terrain/actor images are retained at `baseline-e549-1793071614573053000/`. Earlier rejected source snapshots, first complete4308… render and all transformations remain separate. Root personally viewed e574's whole and accepted the lower turn/reader silhouette for this bounded source review; native implementation is pending.

Sixteen explicit metatile edits fit the ceiling down round the reader gallery, extend a short attached central-spine toe and stagger the lower outer rim. The final one-cell refinement extends the existing east cliff face at10,9 using native08, bending the lower walkway without a freestanding prop. The first complete pass still exposed a4core square in the lower ring at19,20; the final broadest frame shifts to the unchanged original entrance fork9,10, where both stairs and the exit have an actual purpose. That entrance is deliberately not narrowed to meet a universal width quota. The lower turn passes beneath the attached toe in two ordinary core rows. Whole/player views were inspected after each complete pass.

All original fields except `rows` are exact, including reader9,5/east facing/all four lines, Revive×1 at29,23/event, original mouth3,13 and safe arrival3,12, purpose and the installed source's annotations. Refinement notes are nested under the existing `review_contract.refinement`; every original review-contract subfield is preserved exactly. Native source art/palette are unchanged. Every metatile containing spring WATER is byte-identical, so its bank/rock quadrants and whole water graph are preserved. No water, actor, item, HM requirement, story flag or room is added.

| Preserved traversal contract | Exact core lane |
|---|---|
| cave12 at3,4 |7,7→7,8→7,9→7,10|
| cave36 at10,3 |20,5→20,6→20,7→20,8|
| cave36 at10,4 |20,7→20,8→20,9→20,10|
| cave12 at3,7 |7,13→7,14→7,15→7,16|
| cave39 upper hop lane |12,5→12,6 HOP_DOWN→12,8 FLOOR across12,7 WALL|
| cave39 second hop lane |13,5→13,6 HOP_DOWN→13,8 FLOOR across13,7 WALL|

Both lip side caps11,7 and14,7 remain WALL. Exact ordinary uphill returns remain13 and15 steps for the two lanes, with both original actor bodies present. All ordinary Floor is connected; all four stair blocks/lanes and native wall sides remain exact. Original approach10,5 reaches the reader;28,23 reaches the Revive. The native full mouth metatile is unchanged. The low spring and its rocks remain closed within the map; the outer28×36-core perimeter remains WALL.

The source proof now includes directional land reachability matching native `collision.rs` enter/leave predicates. `UP_WALL` is land: entering it from above moving down is blocked, leaving it moving up is blocked, while its other directions remain legal. Five focused directional probes check that behavior; every Floor is reachable through the resulting directed graph. Both uphill return distances are independently recomputed in that graph and still13/15. These are source collision semantics checks, not controller execution.

The same graph records alternate routes with each stair lane closed. The west stairs' long ordinary alternatives change53→55 steps with the new lower bend. The east stairs retain their existing11-step side/loop alternative. No claim that these stairs uniquely gate an area is made. A Floor-only graph must not be used to treat their roof quadrants as solid or assert a false sole-feature cut.

```sh
.local/minnesota-review-env/bin/python \
  planning/minnesota/caves/MnR10Cave1F.refinement.review.py
```

The complete final review took2.032 seconds including pack load,15 negative checks, directed path checks and native-art renders. Negative source mutations reject moved reader/reward/exit, changed quantity, every removed stair, missing hop, opened side, changed spring, added water and open border. The pack file SHA is checked before review: `e549c178f37a3137e8ca7f119aeee79460164f1dbd3ea6d205be8928415fd04c`; all252 original blocks must match the captured18×14 preimage before rendering.

Source cameras use native NITE, the reader's correct east-facing sprite and native tile-top-left player placement, retaining all crops/RGBA/RGB stages. They are160×144 source composites, not production captures. Existing `tests/r10_gallery.rs` has no historical row assertion or moved-camera dependency: its three original controller cases can be reused with `CRYSTAL_R10_GALLERY_CONTROLLER_PACK` targeting a future native revision. Those cases still need an actual rerun on this geometry, and exact native module/data preservation plus matching captures remain pending. No Cargo, active source promotion, pack publication or shared test/API edit was performed by this refinement worker.


The initial native publication correctly rejected e574 because its new top-level `refinement_contract` field was outside the strict allowed source edits. The corrected be41 candidate moves those exact notes under `review_contract.refinement`; no native/API guard was changed. `metadata-migration-proof.json` certifies all remaining source data and whole/decision PNG bytes unchanged from e574. Both epochs remain complete. The source reviewer now also rejects an arbitrary added top-level editorial field and alteration of a retained original review-contract field. Corrected whole and actual weakest player frame were personally viewed again.

The corrected metadata then reached a separate pre-existing generic ledge predicate mismatch. Read-only `be41b5fb81f3ef85/native-ledge-predicate-audit.md` proves the installed gallery itself has FLOOR at upper lip sides11,6/14,6, while the actual crossed/reverse row7 and course-side caps11,7/14,7 remain WALL. Native movement reads HOP_DOWN at CURRENT y6 and jumps across y7 to y8. Root owns the predicate regression/fix; no accepted geometry was altered to appease it. The historical passing R10 controller case directly exercises only the left lane12. Lane13's native collision/algorithm/15-step source return are audited, but its independent production-controller execution remains unclaimed.

## Exact native capture review — passed

Root published be41 into the510-map tight-gallery pack fe57f83b… after fixing the generic course-side predicate with native regressions. Read-only150-check preservation proof and8 actual production captures now PASS; see `../renders/caves/r10-refinement-native/README.md`. All509 other modules and the complete normalized data remain exact; all252 candidate blocks, spring, actors/programs/warps/NITE match. The actual spring animation required a native4×8 water-family comparison; original static-phase failure is retained and a single genuine frame/scroll phase passes the complete existing strict pixel verifier. No production image was edited. Whole, native decision sheet, lower turn, widest entrance and spring were personally viewed. Root's expanded two-lane R10 controller rerun and active source promotion remain separately owned; this worker performed no Cargo/publication/promotion.

## Guarded source promotion preflight — not applied

Root subsequently confirmed all3 actual R10 cases PASS7.14s on fe57, now including BOTH native hop lanes/reverse/13+15-step real returns and the full-bag/native Potion use/original pickup retry. Seven generic assets cases and five R12 regression cases also pass. Root personally reviewed all8 native frames plus the phase audit and accepted the bounded refinement; no final user approval or earned regional progression is claimed.

Preview-only `promote_r10_refinement.py` prepared `../renders/caves/r10-refinement/promotion/preflight-1793073253959856000/`, freeze SHA256 `55db7b72a095634c398c568e2fc5b9f794a5cd63888a4b471062510b795c39cc`. Full exact before/proposed route and interiors files, isolated room/contract records, outside-object byte hashes, parsed-data normalization, native proof, actual passing logs/receipt and trusted Root-viewed receipt are saved. Proposed room changes only16 terrain metatiles and the new `review_contract.refinement` annotation; all existing metadata subfields are retained. Interior contract changes only current source SHA and native-status text. All doors, arrivals, reader, Revive, palette, dimensions and unrelated bytes remain exact. Shared source was rechecked unchanged. This helper has no apply path yet; Root reviews this concrete preflight before any source replacement.

## Guarded source application — completed

Root approved exact preflight55db7b72… and bounded-record proof87bdf745… . The first application check correctly rejected unrelated full interiors drift (the R10 owned room/contract were unchanged). Fresh preflight `preflight-1793073501729785000/`, SHA0dccfcd3… , preserved current unrelated bytes. `apply_r10_refinement.py` proved both owned before spans and proposed record bytes identical to the approved epoch, staged exclusive temporary files, rechecked full live hashes and atomically replaced only the two planned documents. Independent readback confirms every prefix/suffix byte outside each owned object and both complete resulting files exactly match the fresh proposal.

Application evidence is in `application-1793073559488993000/`: full exact before/after files, original/fresh epoch rebase proof, trusted Root authorization reference and receipt SHA256 `bfde3e162bee082fa4510c824ab588d0b83bb1be15be5e246d6f1f8ceb110741`. R10 route SHA at application is `0fd088086912e9a13ec64bae47750e856ac01d1faaedad34f80720b89a268ffa`; interiors SHA at that epoch is `cf44073ef21dd80315bcb50c3833a93c46073e1170c24e22417f617493d72972` (later unrelated promotions may change the whole-file hash). The original Root-reviewed preflight and every source/render epoch remain intact. Native packs were not written, and final user approval/earned regional progression are not claimed. Shared source was released to the R17 worker immediately after readback.
