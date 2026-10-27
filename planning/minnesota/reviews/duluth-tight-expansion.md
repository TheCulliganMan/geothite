# Duluth: tight 64×48 city expansion review

Frozen candidate: `planning/minnesota/renders/duluth/expansion-64x48/v08-correct-native-floor/`. The user superseded the earlier nine-times-area request with **“Smaller but tighter.”** This is a hand-recomposed city four times the original area, not a scaled-down96×72image. Every96×72and64×48original/rejected source/render remains retained.

The source-only component has **38 actual facade entrances** (seven original,31new), **41 new room maps**, a three-floor maritime co-op, nine distinct-stock vendor actors, three new care actors, eight optional native trainers, six field discoveries, and two finite local transactions. Original seven whole native room modules/Gym8/receiver programs, old fourteen warp indices, parties, rewards and story conditions remain authoritative and unchanged. Root owns publication, source promotion, native captures and controller acceptance.

The whole city,21legal160×144player planning views and41-room sheet were personally inspected. The final blue industrial room floor is native`facility1b`, with four ordinary Floor quadrants. Rejected`00`black-floor and`27`filing-cabinet previews are preserved as evidence; neither is the final floor.

## Frozen inputs and source proof

- Layout SHA`1f0cef53583e9af370eea44644a0ce833896803dd21b200844ef22c54f37f643`.
- Executable `expansion.contract.json` SHA`4e75bcf9907b1f88fbf04f93bf08006db4fc197e2de4dc8b94dec25813cfda99`.
- Rich dossier SHA`fb70da96ffe8c5ab3e4347d5477001628646cb75efe63c8f8f418986db0f28d1`.
- Exact native city preimage module SHA`ad4f81453ba40e40efe055adb55e323c060a74fcce4a46bd4afb4b9d55eb51cc`, from7e5f541-map pack. Root confirmed the current story/Minneapolis composition retains this entire Duluth preimage.
- Source review reports zero actor/door/stair/room errors. Directional collision proof uses actual`UP_WALL`enter/leave restrictions; four new hop lanes return in25/27and7/9ordinary steps, original hop retains47-step ordinary return. It also tests officer bodies sealing walking **and Surf**, then legitimate disappearance opening all four seam lanes.
- Eight corruption negatives reject a wrong pier end, open outer Surf edge, erased facade, missing officer override, invalid`takeitem`arity, stale whole-module epoch, opened ledge shoulder and unresolved reward branch.
- New pier endcaps use native`46`: ordinary Floor on the inside, Wall facing water. Expanded shores intentionally have274 bank-to-Water adjacency edges, fully enumerated in proof; the original six low-cove launch edges are retained. Do not claim that the whole expanded city has only six launches.
- Whole image and contact sheets are under `review/`; immutable hashes are in `freeze.json`. These are source planning renders, not production GPU captures.

Run the read-only source loop:

```sh
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_duluth_expansion.py \
  --pack content-packs/minnesota-northstar-seep-shelter-review.crystalpack \
  --candidate planning/minnesota/renders/duluth/expansion-64x48/v08-correct-native-floor \
  --output planning/minnesota/renders/duluth/expansion-64x48/v08-correct-native-floor/review
.local/minnesota-review-env/bin/python planning/minnesota/renders/duluth/expansion-64x48/v08-correct-native-floor/prove.py
```

For future iterations use a NEW revision/output directory, preserving this freeze and its original review timing/hash receipt.

## Actual old anchors and guarded regional seams

One whole`MnDuluth/MN_DULUTH`module,64×48metatiles/core128×96. Translate old ordinary exterior anchors by**[32,24]core**. Old rooms keep their targets and warp indices. The source/native old sign discrepancy was resolved from actual native background record`[4,22]`, not stale source`[4,24]`.

Twelve separate indexed overrides relocate four native officers (native object indices8..11, identified by`object_identifier`) and eight coordinate-event records. A historical96×72provisional matrix accidentally keyed officers by shared script/id; that uninstalled matrix is explicitly superseded by`expansion-64x48/preflight-v01/relocation-matrix.json`.

| Seam | City edge lanes | Actual neighbor lanes | City/reciprocal offsets | Officer bodies |
|---|---|---|---|---|
|West R10|0,48 /0,49|`MnR10C01R00`31,24 /31,25|+12 /−12|2,48 /2,49|
|North R11|64,0 /65,0|`MnR11C00R03`46,47 /47,47|+9 /−9|64,2 /65,2|

West outbound retains`ENGINE_GLACIERBADGE`+`EVENT_MN_DEPOT_CLEARED`. North retains all eight native engine badges+`EVENT_MN_LIGHTHOUSE_SIGNAL`. Inbound remains free, with native guard disappearance and two relative inward steps. Old guard/checkpoint commands/text are unchanged. A new actual recovery record is proposed for existing clinic`MN_DULUTH_POKECENTER_1F`ordinary Floor`[10,13]`; Root appends it without changing old HOME/Fly/recovery records. Historical tightv02published natively; finalv08still requires Root publication and actual controller acceptance.

## Direct production-controller acceptance plan

Use final published pack and exact frozen contract through`CRYSTAL_GYM_TEST_PACK`and`CRYSTAL_CITY_EXPANSION_CONTRACT`. All initial position/legal party/inventory/money/PP/RNG settings are explicit fixtures; subsequent actions must use production`press`, walking/warp helpers and native menus. Fixtures do not earn regional progression.

The focused controller leaf is `crates/crystal-bevy/src/bevy_shell/tests/city_expansion_duluth.rs`, registered beneath the shared city course. Its four ignored cases cover the requisition, two-win co-op award, six discoveries, and `duluth_expansion_controller_outdoor_stairs_hops_and_ordinary_returns`. The last case uses native collision planning restricted to ordinary Floor/UP_WALL, then real input for all five outdoor stair round trips/sidewalls and four hop/reverse/ordinary-return lanes. The author checked formatting and the existing helper interfaces; Root owns compilation and execution against the final native pack.

Root's first three-activity run passed the co-op and six-discovery cases. The requisition case exposed a test-helper mistake: native `yesorno` uses `yes_no_cursor`, while the helper checked `menu_cursor` and sent Down as an input-ownership probe, selecting No. The corrected helper checks the actual pending Yes/No state, proves ownership with Left, selects and asserts the real cursor, then presses A. The pre-fix leaf and failed log remain retained. That correction and the newly added outdoor case await Root's next executed results; they are not passing evidence yet.

### Cargo requisition: missing item, refusal, payment, full-bag retry and Continue

Start in`MnDuluthCargoOffice`at its passive arrival`[8,9]`. Actor`MN_DULUTH_X_CARGOOFFICE_F1_HOST`stands`[5,5]`; reachable southern approach`[5,6]`faces Up. The exact native script asks to trade one X DEFEND for one HP UP. Native `takeitem` has **one arg**, quantity1implicit.

Initial setup: legal partner with first learned move below full PP; money999999; nineteen distinct ITEM-pocket stacks including one ETHER, excluding X DEFEND and HP UP. No local flags. First interact/Yes prints missing-item refusal; bag, party and money unchanged; neither delivery nor reward flag set.

Walk through actual carpet/streets to`MnDuluthToolExchange`, native vendor`MN_DULUTH_X_TOOLEXCHANGE_F1_HOST`. Actor`[3,5]`, counter`[4,5]`, approach`[5,5]`facing Left. Actual stock order is X ATTACK, X DEFEND, DIRE HIT, GUARD SPEC. Buy **two X DEFEND** using list index1 and native quantity2. Assert exact money decrease`2×native X_DEFEND.price`, X DEFEND quantity2 and twenty occupied ITEM slots. Do not mutate inventory after this purchase.

Return to cargo. Test No first: the two items remain, no local flags. Test Yes: exactly one is removed, `EVENT_MN_DULUTH_X_REQUISITION_DELIVERED`becomes true; X DEFEND quantity1 still occupies its stack, so HP UP cannot fit and `EVENT_MN_DULUTH_X_REQUISITION_REWARDED`remains false. Printed full-pack refusal explains the delivery is on file. Repeat without space: no additional payment, HP UP0, same pending flag. Quantity2 is essential: taking the last X DEFEND would free a slot and would NOT exercise full-bag reward retry.

Use ETHER through actual Start/Pack/Use/party-move UI. Assert correct PP restoration and item consumption, leaving nineteen stacks. Talk again: HP UP1, reward flag true, no further X DEFEND removed, money unchanged. Repeat and Continue retain bag, party/PP, position and both finite flags; no duplicate reward. Main-story/badge flags remain unchanged. The delivery flag stays as historical payment evidence after reward.

### Co-op training award: two actual wins, full-pack refusal, retry, once and Continue

Start`MnDuluthMaritimeCoop3F`at safe arrival`[3,3]`. Explicit strong legal partner fixture; twenty ITEM stacks including one ETHER but no METAL COAT; one PP slot initially empty. Host`MN_DULUTH_X_MARITIMECOOP_F3_HOST`at`[5,5]`has approach`[5,6]`. First talk must print the two-opponent requirement and give nothing.

Fight actual native`MN_DULUTH_X_T04`NILS at`[6,5]`, approach`[6,6]`Up: BLACKBELT_T, POLIWRATH43/MACHOKE42. Its only defeat event is`EVENT_MN_DULUTH_X_TRAINER_04`. Assert PP spending and native battle completion. Host still refuses. Fight`MN_DULUTH_X_T05`IVAR at`[10,7]`, approach`[10,8]`Up: COOLTRAINERM, URSARING42/SKARMORY43. Assert actual party/class/name and second event, not synthetic flag setup.

With both wins and full pocket, award fails, `EVENT_MN_DULUTH_X_TRAINING_REWARDED`stays false and METAL COAT0. Use ETHER through actual PP UI to free a stack; claim METAL COAT1, only then set award flag. Repeat/Continue must not grant again or re-battle completed trainers. Actual battles award their native prize money; the award itself charges nothing. No badge, Gym8, depot or harbor signal flag is granted.

### Six actual discoveries

Start fresh in city`[69,36]`with empty ITEM pocket and explicit legal partner. Walk to the contract's six item actors, interact, fully print native item-ball phases, and assert matching once flag/quantity. Expected totals: NUGGET1, MAX REPEL1, ULTRA BALL2, PP UP1, FULL RESTORE1. Repeated A after each collection cannot grant again; after all six, Continue retains the same bag/events. The collected item no longer blocks its tile; do not reuse an interaction helper that asserts an uncollected actor still occupies that tile. Exact positions/flags come from the contract, not historical96×72coordinates.

### Shared course scope

Root's generic course covers actual doors, internal stairs, distinct vendor stocks/prices, care HP/status/PP and loss/Teleport recovery. It should exercise original seven entrances as well as new31; old room modules/return indices are preserved, but static collision checks alone are not real round trips. New outdoor five-stair both-direction/blocked-side probes and four added hop lanes should use the frozen rich dossier/source proof. The intentionally short eastern returns are7/9steps; earlier roof landing is rejected and retained historically.

## Remaining limits

No source promotion by this author. No finalv08native GPU/controller result is claimed. Original story progression, earned access, save migration, Fly additions, animated lift bridge, boat/rail travel or new HM systems are not added. Native city identity, parties and all pre-existing story must survive Root's independent bounded merge proof.

## Bounded v09 public-room amendment

`renders/duluth/expansion-64x48/v09-furnished-public-rooms/` preserves v08 and changes only 77 metatiles across the rows of ReadingHall1F/2F, WeatherStudy1F/2F and UnionHall1F/2F. The final executable contract is SHA`6fa4b23d7ee299cf51491fc71871984aaf8cb15c783f566e724d55284814a308`; the rich dossier is SHA`c1739aad3ad5dcb09b12ad3d582dc2647251ecb52fa5c3a908ff19d937cd7c2f`. Exterior layout bytes remain exact`1f0cef53583e9af370eea44644a0ce833896803dd21b200844ef22c54f37f643`.

Six explicit furniture/shelf compositions replace the repeated paired-table rooms. Every byte outside those six row values is independently exact in both input JSON files. All original Floor tiles beside actors/warps, actual arrivals, active warp rosters and ordinary Floor connectivity remain legal; eight corrupted variants are rejected. Exterior whole/camera pixels and the other 35 new room images remain exact. Final `six-room-before-after-final.png`, `review-pass4/six-room-player-decisions.png` and the complete 41-room sheet were personally inspected. Earlier failed/superseded passes and their exact source snapshots are retained. This is a source-only amendment requiring Root's native publication and controller review; all activity programs and test input plans above are unchanged.
