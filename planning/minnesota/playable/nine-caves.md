# Nine cave layouts: written-pack review

The later [541-map composition](winding-passages.md) now integrates these cave
revisions with the current Leech Shore story. It further replaces R12's parallel
switchback with a seep basin and separate shelter/cache branches. The510-map
epoch, four-stair R12 evidence and comparison below remain historical.

The [later road-bends/stairs composition](road-bends-and-stairs.md) retains all
nine rooms unchanged. The exact cave publication epoch remains documented below.

The 510-map cave composition is `content-packs/minnesota-northstar-repel-review.crystalpack`.
File SHA-256: `31258a1788a986b158d0679e4f3a26c4dd476964311f707a6e09c9640cad48c6`.
Content identity: `be6c444a94b990020e56433609628723e4a1a0164a386ff544ef88c4080b5a13`.
This was the cave-review branch before the current541 merge; its independent
source/story lineage is retained here rather than presented as the latest game.

[Review all nine caves at the same native scale](../renders/caves/batch-review/geometry-96a-review03/all-nine-before-after-native32.png).
These are source-art composites from the actual packs, with native cave time,
NPC palettes and facing. Actual GPU player views and controller evidence are
separate. R10 alone grows from12×8 to18×14; the other eight keep their extents.
All original renders, rejected candidates, conversions and source snapshots
remain retained under the ignored renders directory.

## Authored changes and actual gameplay

| Cave | Layout and retained gameplay | Evidence |
| --- | --- | --- |
| R01 hollows | Two useful climbs, fitted Antidote alcove, original guide and reward | Two controller cases; six native player views; [runbook](mine-and-hollows.md) |
| R03 bank | Attached dry bank nose and short blind bend to original Great Ball; original walker | One controller case, including all printed pages, pickup once, reentry and Continue; six native player views |
| R05 Rock Cut | Bent lower cut, healer turnout, one actual stair, upper fork and both original mouths | Two controller cases on this final pack; eight native player views inherited unchanged from96a |
| R09 mine | Three offset stair benches, short9/11-step returns and scout shelf | Four controller cases retain actual foreman win/loss/prerequisite and separate cache recovery; nine native player views; [runbook](mine-and-hollows.md) |
| R10 gallery | Fitted spine, spring, upper reader and lower Revive shelf; tighter return geometry | Three controller cases; both actual hop lanes and13/15-step ordinary returns; eight native player views, strict whole-frame water phase comparison |
| R12 shelter | Winding ascent, Otta pocket, Max Ether spur and short two-lane drop | Three controller cases, including actual PP restoration/refusal/Continue; eight native player views; [runbook](northwoods-map-refinements.md) |
| R13 refuge | Raised healing terrace and separate supply notch | Three controller cases; seven native player views; [runbook](r13-refuge.md) |
| R17 tor | Attached eastern rock tongue and short west bend revealing original Revive | One controller case with directional ceiling, printed pickup, once/reentry/passive return/Continue; six native player views |
| R20 cleft | Two mouths, two stairs, supply shelf and short lateral drop | Three controller cases; nine native player views; [runbook](r20-cleft.md) |

The layouts preserve their original actors, rewards, mouths and story anchors.
The mine repair also restores19 exact original Q06 roots and synchronizes its
historically split NPC/event mirrors; it does not add a new story.
The [geometry proof](../renders/caves/batch-review/geometry-96a-review03/README.md)
passes210 checks and rejects nine corruptions. Relative to c16, only eight cave
rooms change (R12 was already revised), their owned payloads, the exact mine
mirror/root repair and derived identity. All502 other native map modules remain
exact. The [final extension](../renders/caves/batch-review/final-31258a1788a986b1-review01/README.md)
passes222 checks and rejects four further corruptions: only ordinary Repel's
200→100 duration and derived identity change after geometry96a; all510 maps,
art, audio, reports and other data are unchanged.

Rock Cut's final guarded source promotion is retained at
`reviews/r05-rockcut-promotion/31258a17-485652d7/application-1793074713261827000/`.
Independent post-apply readback proves every outside-owned byte and unrelated
JSON value exact. The other room promotions and their failed/preflight versions
remain retained alongside their respective native reports.

## Repel defects found by actual reward use

The exporter matched `RepelEffect:` inside `SuperRepelEffect:`. The shared parser
now matches the complete source label. The actual pinned external source gives
100/200/250 steps; five strict repair tests and three exporter tests pass.
The bounded repair CLI reads that external source and changes only the original
Repel duration, then strictly reads the written pack back. Existing saves are
not migrated. [Source certification and exact commands](../reviews/repel-source-labels.md).

The production Pack notice also requested nonexistent `RepelUseText`. It now
uses canonical `_ItemUsedText`, the native item name and the original `text_low`
line break. Existing active-Repel refusal requests use the canonical underscored
label. No text alias or catalog was fabricated. Rock Cut's two controller cases
pass on final31258 in3.34s, including the actual printed source notice, movement
ownership, consumption,100→99→98 walking count and Continue→97.
Execution receipt: `.local/r05-rockcut-controller-canonical-notice-receipt.json`.
The failed first execution remains retained. Active-use refusal has source-label
verification; this local Rock Cut case does not exercise a second carried Repel.

## Reproduction and scope

Use the repository's pinned toolchain. Root owns pack publication; consume the
completed pack and use a distinct target directory for concurrent work.

```sh
export CRYSTAL_R05_ROCKCUT_CONTROLLER_PACK="$PWD/content-packs/minnesota-northstar-repel-review.crystalpack"
cargo test --locked -p crystal-bevy --lib r05_rockcut_controller -- --ignored --test-threads=1
.local/minnesota-review-env/bin/python planning/minnesota/tools/check_plan.py \
  --pack content-packs/minnesota-northstar-repel-review.crystalpack
```

Native full-pack audio verification passes all377 PCM hashes, frame counts and
loop ranges (`.local/repel-final-native-audio.log`). The shared Mom/exit, actual battle-command/PP and thirteen-room mart controller
regressions also pass on this final pack. Scoped formatting passes.
All-target assets/Bevy Clippy completes with existing warnings and the documented
`never_loop`, `bad_bit_mask`, `approx_constant`, `useless_conversion` allowances
(`.local/nine-caves-final-canonical-notice-clippy.log`). No new cave/Repel leaf
warnings were reported. Whole-workspace formatting retains unrelated failures.

Source validation reaches all eight badges symbolically; it does not prove an
earned playthrough. Boundary Waters admission remains closed and its Safari
capture session is unfinished. Planned travel links and100 Minnesota species
are not integrated by this cave publication. Final visual approval remains with
the user; no claim that the whole region has reached Johto quality is made.

The [current connected atlas](../renders/atlas/minnesota-current-connected.png)
was regenerated from final31258 after all source promotions:20 routes,6 towns,
47 paired rooms,7 installed guarded walking links and33 planned endpoint leaders.
All20 route/native comparisons pass. Its immutable epoch is
`renders/atlas/nine-caves-31258a17-native-facing/`; PNG SHA-256
`54fba05909bbbb257cefe81fe13add93385b17c6b1de1bea957bd62ff8805800`.
