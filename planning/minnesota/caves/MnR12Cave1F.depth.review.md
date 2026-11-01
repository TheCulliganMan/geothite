# R12 split-roof shelter — depth candidate v04 metadata

## Native follow-through — 2026-10-27

Root published this geometry into the current541-map composition, SHA256
`7e5f65b935b1f74766c810ee72f444ad60a7e71e096b5dec10c46f516595344a`,
then inspected the whole native-source room and all nine actual player frames.
Every640×576 frame matches its full native source reference, including one
consistent cave-water phase and uniquely measured CHRIS position. The mouth
view includes the real finite-map camera clamp. All actual originals/raw GPU
bytes and initial failed camera assumptions remain retained.

All three controller cases pass: the three two-way stairs/side walls, both
native hops/reverse/21- and19-step returns, original mouth/re-entry, Otta pages,
once-only Max Ether, legal PP restoration/refusals/consumption and Continue.
Metadata-only v04 corrects shoulder probes to the actual Wall course at
`(13,15)/(16,15)`; it changes no rendered or native geometry. The corrected
geometry case passes separately from the two earlier successful resource cases.
Guarded promotion changes only the owned room's rows/purpose/status/review
contract in MN_R12, preserving every other route value. The canonical room
source is v04 SHA256 `bf39bc3d09073561fafef7053d66b09ce675b5227d84d5437a492916ec9357df`.

[Current runbook and native views](../playable/winding-passages.md) records the
full story merge, source/native distinctions and remaining quality work. The
source-candidate history below describes the pre-publication design stage.

The whole cave is redesigned from parallel zigzag shelves into a low seep basin, rising throat and two separate upper destinations. Otta remains under the western dry roof; the original Max Ether is discovered around the fitted northeast turn. A short native drop returns to the basin around an attached cliff blade, while the ordinary stair remains available in both directions. The closed pool defines the eastern bank instead of forming a central island ring.

The [concrete reference/design proposal](../reviews/r12-depth-redesign-proposal-20261027.md) records why this was the weakest of the nine caves and the authentic Union/Dark Cave references personally inspected. Root authorized a separate source candidate after agreeing with that critique. No active source or native pack is changed here.

## Frozen source and images

Final source folder: `planning/minnesota/renders/caves/r12-depth-candidate/v04-course-probes/`. V04 changes only the two controller Wall-course shoulder coordinates; its geometry and images are exactly frozen v03. The native pack was published from v03 rows.

| Artifact | SHA256 |
|---|---|
|`MnR12Cave1F.depth.candidate.json`|`bf39bc3d09073561fafef7053d66b09ce675b5227d84d5437a492916ec9357df`|
|`MnR12Cave1F.previous.json`|`75c7a97ce314158538d4ccbe36b201259ec7a7db14cfa14a4ef4e9b5dc771b3d`|
|`source-proof.json`|`61ae6efcd8fe28c1f55940b31f3e45799c459b7c5efc6f353c75926feb19cdbf`|
|`after/whole-source.png`|`846286c8a5f27bc40d830489e813057363b07a2c6f8d75f2029e15cd5f8e7ffb`|
|`decision-cameras.png`|`5cbcac110d548d9e5f3cdcc6be71d4f3032cdb9afb1470fb8073be4ca2dafed0`|
|`whole-before-after.png`|`3b7cc055ecca0aaa3fdf0029e7b63a127d5135fcfeeaba625c8be7655f1e6470`|

Personally inspected [whole before/after](../renders/caves/r12-depth-candidate/v03-cliff-toe/whole-before-after.png), [all nine player-art160×144 cameras](../renders/caves/r12-depth-candidate/v03-cliff-toe/decision-cameras.png), [upper fork](../renders/caves/r12-depth-candidate/v03-cliff-toe/after/upper_fork-player160.png), [Ether spur](../renders/caves/r12-depth-candidate/v03-cliff-toe/after/cache_approach-player160.png), [low return](../renders/caves/r12-depth-candidate/v03-cliff-toe/after/lower_return-player160.png), and [actual weakest Floor square](../renders/caves/r12-depth-candidate/v03-cliff-toe/after/ACTUAL_WEAKEST_FLOOR-player160.png). The upper-fork and Ether camera defects in v01 were visible: open floor dominated the fork and the cache bay remained rectangular. V02 adds an attached ceiling rib and northwest bay bevel. V03 replaces the lower rock-tail boulder course with a continuous native cliff toe, with identical collision. All v01/v02 sources, logs, renders and intermediate conversions remain retained. No rejected source is overwritten.

These are planning composites from actual native cave art, forced NITE palette, actual NPC facing and source-minus4px standing origin; the additional player frame uses native CHRIS art at its ordinary origin. They are not production GPU captures or proof of runtime foreground composition. The shared renderer's generic gallery/spring camera names are legacy labels; this candidate's authoritative named views are the nine-camera sheet and new review contract.

## Exact preserved authority

Frozen native preimage: `content-packs/minnesota-northstar-map-quality.crystalpack`, SHA256 `9aa950a35453f06cfee2e4a9b674c6d902af7c311f9ca64a083665ac6c60ddb2`. Complete route preimage `baseline-MN_R12.json` SHA256 `e893262149f4d455540b73c663beaecbda04fb255c14906565a0d70a3e70f8d5`. All192 native room blocks match that original source. Current publication may use a later complete parent with additional maps; do not hardcode538 or overwrite newer story content. Certify the chosen parent's exact R12 native preimage and preserve every other module/data field.

Only `rows`, `purpose`, `candidate_status` and `review_contract` differ. All81 changed metatiles lie inside `[1,1,15,11)`; exact outer perimeter and original mouth block `(8,11)` remain untouched. Extent remains16×12. Attributes, source room identity, border, template, original objects, directions, dialogue, item/quantity/flag, original warps and programs are unchanged. Otta `(5,5)` faces east and is an ordinary resident, not a healing service. Cache `(27,5)` remains MAX_ETHER×1 with `EVENT_MN_R12_CAVE_ETHER`. Mouth `(17,23)` and clear arrival `(17,22)` remain exact. Exterior active warp `(66,23)` and ordinary Down to `(66,24)` retain their original native semantics; no passive record is added.

`native-handoff.json` records exact retained-field, native object/event/art hashes and all controller probes. Native cave glyphs only; no extra art/tileset/import/alias/runtime system. `review_contract.native_review` is retained as historical preimage evidence, rather than misrepresented as acceptance for this new graph. New status is explicitly unpublished.

## Actual geometry checks and native handoff

```sh
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_r12_depth.py \
  planning/minnesota/renders/caves/r12-depth-candidate/v04-course-probes
```

Source checks pass in0.374seconds after pack loading. The narrow wrapper reuses `review_cave_candidate.py` for all whole-room native art/NPC rendering. It verifies196 actor-present Floor squares are reachable, including actual UP_WALL enter/leave masks in the213-square directional land graph. The maximum ordinary plain-Floor square is4×4core; its weakest camera `(8,5)` includes Otta and the stair, so the remaining space serves the inhabited roof. Counts support collision review, not visual acceptance.

Three deliberate native stairs replace the old four-bar quota:

| Role | Metatile / native glyph | Ordinary lane, including approaches | Wrong-side wall |
|---|---|---|---|
|Otta roof|`(5,3)` cave30|`(10,5)→(10,6)→(10,7)→(10,8)`|`(11,7)`|
|Ether spur|`(11,3)` cave12|`(23,5)→(23,6)→(23,7)→(23,8)`|`(22,7)`|
|Rising throat|`(9,7)` cave12|`(19,13)→(19,14)→(19,15)→(19,16)`|`(18,15)`|

Every lane is clear and bidirectional under native directional collision; wrong-side targets are solid Wall. Closing the Otta stair blocks only Otta, closing the cache stair blocks only the cache, and closing the throat blocks both upper destinations. This is actual directed-graph reachability with actor bodies present, not an assumption from painted stairs. Neither destination requires Surf, and adding Surf cannot bypass the main ascent.

The original two hop anchors `(14,14)/(15,14)` remain HOP_DOWN, with upper approaches `(14,13)/(15,13)`, landings `(14,16)/(15,16)`, Wall reverse course at y15 and both perpendicular course neighbors closed. Ordinary uphill returns are21/19steps respectively. Exact ordinary path coordinates are saved in the proof/handoff. The pool contains32 connected reachable Water quadrants, launch `(21,17)→(22,17)`, and no outer Water. Its boundary is the retained closed room perimeter; no unseen water seam is claimed.

Thirteen negative cases reject the original incorrect Floor-lip shoulder declaration, both opened Wall-course shoulders, moved Otta, changed quantity, outer Surf breach, missing mouth, each sealed stair, wrong stair side, opened reverse/drop-side course and a fake ladder. The active owned room and input pack stay unchanged during review. Production native Rust collision/controller cases are still required.

The new review-contract stair, waypoint, ledge-return and camera fields supersede the historical four-stair geometry probes. Root should update the local controller fixtures to use these exact probes and retain original printed Otta dialogue, actual Ether PP restoration/refusal/once, mouth reentry and Continue checks. Do not treat the old four-stair controller pass as proof of this redesign. Root owns strict native publication, all other-map preservation, headless captures and any later guarded source promotion.


## Metadata-only v04 correction after actual controller input

Root's production stair/side, both hop/reverse/21+19 return and original resource/PP/refusal/Continue assertions reached the final shoulder check. That check exposed a stale inherited source probe: `(13,14)/(16,14)` are legal **Floor lip** neighbors, not walls. The native drop correctly seals the lower **Wall course** at `(13,15)/(16,15)`. V04 changes only `review_contract.controller_probes.ledge_shoulders` to those lower coordinates. It does not change a single pixel, collision, other source field or native program. The reviewer now explicitly validates the declared course, rejects the old Floor-lip declaration and independently rejects opening either true course side. Frozen v03/source/proof/log/reviewer remain unchanged; its previous11-case check did not validate the stale declaration, so the additional check is necessary.

V04 machine handoff SHA256 `89a49df6cf9b210ae86a7dc5913067d530445d4278813d4095f1f31ba7992e11`;13 source-negative cases pass. The written native pack remains `content-packs/minnesota-northstar-seep-shelter-review.crystalpack`, SHA256 `7e5f65b935b1f74766c810ee72f444ad60a7e71e096b5dec10c46f516595344a`, with unchanged v03 geometry. Root owns the final corrected controller result and actual GPU captures. This runbook does not convert the earlier partial case into a full passing case or promote live source.
