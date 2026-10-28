# R03 exterior reward identity correction

The preserved native item at `MnR03C00R01` core `(41,7)` is **SUPER_POTION ×1**,
not Great Ball. Its object/script ID is `MN_R03_ITEM_01` and its original flag is
`EVENT_MN_R03_ITEM_01`. The adjacent `(40,7)` and item footing `(41,7)` are both
literal native `FLOOR`. The revised map is21×8 metatiles, has no warps, and its
trim removed only the separate blocking eastern margin. No item, flag, actor,
program, terrain cell or crystalpack was changed to make this correction.

The actual Great Ball is the separate cave pickup `MN_R03_ITEM_02` /
`EVENT_MN_R03_ITEM_02` in `MnR03Cave1F`. It is not the exterior boundary reward.

This corrects two historical human descriptions while retaining the exact
frozen executable inputs and their reviewed hashes:

| Frozen file | JSON pointer | Incorrect wording |
| --- | --- | --- |
| source-v01/integration.contract.json | /overlap_revisions/0/preserve | “Existing Great Ball x41” |
| source-v01/controller-input-plan.json | /additional_required_cases/8 | “collect original Great Ball at41,7” |

The integration contract remains SHA256
`2d758bfe1894ee42ddcd2e176d25fcd7147fb4f3632f066ac16468318c26756b`;
the original controller input plan remains SHA256
`4a561e7792d99ceba530326b068f5aa90cdcceea6b40b517844b7d8aa375866f`.
Historical copies under the retained `rebase-actual6b36/previous-source-v01/`
output tree carry the same wording and are intentionally not rewritten.
These editorial labels did not drive item lowering or replace native records.
The bounded publisher and written preservation audit retained the actual native
item object, complete pickup program and original flag unchanged.

Evidence was read directly from completed
`content-packs/minnesota-state-cohesion-v01.crystalpack`, file SHA256
`c4816242295a236fdfe2a6a96fb248cd3eab11de1b526be0decb7df36e5e95eb`.
A fifth focused production-controller case now targets the actual exterior
Super Potion, printed pocket pages, original flag, collected footing, eastern
boundary rejection and legal left return. Root compiled and executed this fifth case successfully in2.92s on c481;
`.local/state-cohesion-controller-v01/r03-edge-v01.log` records the result.
The exact production Save at41,7 facingRight after pickup and three rejected
Right presses produced the native640×576 edge capture; Root personally viewed
its closed tree edge and accepted it. The prior four passing controller cases
remain recorded independently (their105.50s log is not relabeled as five).
