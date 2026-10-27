# Winding roads, short grass-bank drop and outdoor rock stairs

The later [541-map composition](winding-passages.md) retains this road/ledge/
stair work with the current Leech Shore story, further exterior refinements and
the deeper R12 cave. The510-map epoch below remains historical evidence.

The historical510-map review pack is
`content-packs/minnesota-northstar-rock-stairs-review.crystalpack`.
SHA-256: `6071912eeb2fde6982f9e857bed9157d8d86da0f7ae56dba38665c809ab100e1`.
Content identity: `22cdecd416fc2a9c92f4eef7ba205c82aeff80fae1220884e9f8e0a88e87e049`.
It retains [all nine revised caves](nine-caves.md), the source-certified100-step
Repel, previous Northwoods expansion, Ryan's house and the seven installed city
walking links. The later541-map publication integrates those changes without
replacing the newer story maps or gyms.

[Matched-camera before/after](../renders/exterior-bends/final-6071912e/movement-before-after.png)
uses source-art composites. Actual GPU views are linked separately below.
[Complete current connected atlas](../renders/atlas/minnesota-current-connected.png).
All originals, failed iterations, raw readbacks, conversions and source epochs
remain retained in the ignored renders directory.

## Implemented terrain and gameplay

| Route | Concrete change | Actual verification |
| --- | --- | --- |
| R06 Pine Country | Two bends and attached pine shoulders replace the straight northern road. A small side nook retains the watcher; the sign, northern arrivals and legitimate western loop remain.28 metatiles change inside [19,3,25,12). |78 source checks including13 rejected variants;152 native checks and five full-LCD captures. Both production cases pass: exact bends both ways, watcher/sign, five real trainer victories, required grass encounters, resources, Cut, cabin and Continue. |
| R16 Red River Prairie | A two-core hayfield lip replaces the24-core jump face, with working-field fence around it. The farm turn narrows with fitted fences. Both drop lanes still require at least six grass steps to the road.18 metatiles change. |33 source checks and nine rejected variants;188 native checks and seven full-LCD captures. Both actual hop lanes/reverse barriers, original59-step right-lane return, bridges, pickups, dialogue ownership, real Camper battle/PP and Continue pass. The ordinary left-lane return is57 steps. |
| R10 Iron Range Descent | One exact native cave12 stair replaces the western lawn slit; an attached rock shoulder closes its lower-left toe. The right lane at core x11/y19–22 remains two-way. Only two metatiles change. |Five strict Rust asset cases;180 native preservation checks and five full-LCD captures. Four actual controller cases pass: stair both ways/wrong-side barrier, full geometry/refined gallery, Calder battle/Continue and Surf seam/outer rocks. Standard current source reviewer rejects19 bad variants; legacy reviewer still rejects its15. |

Root inspected every whole route and all17 actual player views. All391,680 LCD
pixels match their native source/OAM/foreground expectations exactly, including
the R16 fence occlusion. No screenshot painting, player shifting or excluded
pixel masks were used for the full-LCD proofs.

Native evidence:

- [R06 five actual views](../renders/pine-country/north-road-native/95384a3a48908329/review-01/production-decision-cameras.png).
- [R16 seven actual views](../renders/red-river-prairie/refinement/native/8826137c74842b68-review01/production-decision-cameras.png).
- [R10 five actual views](../renders/iron-range-descent/rock-stairs-native/6071912eeb2fde69/review-01/production-decision-cameras.png).

The first two captures use their immutable publication stages; their complete
route modules/art remain exact in final6071. All eight controller cases run on
that single final pack. Exact execution receipt:
`.local/exterior-bends-final-controller-pocket-receipts.json`; binary SHA-256
`250c34008027211e8e7f1b44c8f5620a57cf956a7f8280ba47168082faf5d9bb`.
R06 two cases:9.81s; R16 two:8.47s; R10 four:11.81s.
Older journey fixtures required observing Found after the production controller
had already settled to PocketText. Only those expected visible-page substrings
were corrected; all pickup quantities/events, ownership, item use, battle,
movement and Continue assertions remain. Failed sources/logs are retained in
`renders/exterior-bends/controller-first-failure/`.

## Publication and preservation

Exact chain, with all stages retained outside tracked source:

1. Nine-cave/Repel31258 → R16 `minnesota-northstar-prairie-bend-review.crystalpack`,
   SHA `8826137c74842b68a184a957a14d15cdb3f9ed3bdaca349a046924a049a20a41`.
2. R16 → R06 `minnesota-northstar-road-bends-review.crystalpack`,
   SHA `95384a3a48908329b89ab2bbe43ce99a18d3a02d5dcca9f9034a6341e08e8ce7`.
3. R06/R16 → final6071 using `revise_r10_rock_stairs`.

The [whole-composition proof](../renders/exterior-bends/final-6071912e/whole-composition-proof.json)
passes526 checks; a separate15-check split-storage proof checks exact cached
attributes and encoded payloads. Only48 metatiles across five native modules
change. All505 other modules, all rooms/actors/programs/story, field-move tables,
items including Repel100, audio and original runtime files remain exact.

R10 uses a dedicated `mn_r10_rockstairs` alias. Its original128 Johto blocks retain
all collision, referenced physical glyph bytes and palette indices; appended80
is exact cave12. Shared Johto is untouched. Exactly five new alias runtime files
and two explicit art manifests are added. The derived content hash and runtime
modpack ID change accordingly. No existing-save migration is claimed. Both
original R10 city seams and all five installed mouth patches remain exact.

Source promotions use full preimage guards, independent unrelated-byte checks
and post-apply readback:

- R06: `reviews/r06-north-road-promotion/6071912e-97d1560c/application-1793076090190196000/`.
- R16: `renders/red-river-prairie/refinement/promotion-314d972a78bcf479/application-1793075977571610000/`.
- R10 layout only: `renders/iron-range-descent/rock-stairs-candidate/promotion-1793076078297044000/`.

[Exact R16 source contract](../reviews/r16-prairie-refinement.md) and
[R10 stair/art builder runbook](../routes/MN_R10.rockstairs.review.md) retain the
source/native epochs and negative cases. Standard R16 checks derive the current
ledge bounds and reject a wider undeclared jump face. Standard R10 checks support
this exact alias/recipe and retain the original Johto branch. Source checks and
renders run in milliseconds; native publication and gameplay remain separate.

## Reproduction and remaining scope

Use the repository's pinned toolchain and completed local pack. Root owns Cargo
and pack publication; concurrent agents consume the pack with distinct targets.

```sh
export CRYSTAL_PINE_COUNTRY_TEST_PACK="$PWD/content-packs/minnesota-northstar-rock-stairs-review.crystalpack"
export CRYSTAL_RED_RIVER_PRAIRIE_TEST_PACK="$CRYSTAL_PINE_COUNTRY_TEST_PACK"
export CRYSTAL_IRON_RANGE_DESCENT_TEST_PACK="$CRYSTAL_PINE_COUNTRY_TEST_PACK"
cargo test --locked -p crystal-bevy --lib pine_country_controller -- --ignored --test-threads=1
cargo test --locked -p crystal-bevy --lib red_river_controller -- --ignored --test-threads=1
cargo test --locked -p crystal-bevy --lib iron_range_descent_controller -- --ignored --test-threads=1
```

Root executed with `CARGO_TARGET_DIR=target/runtime-fps CARGO_BUILD_JOBS=1
RUST_MIN_STACK=33554432`, `--profile wasm-dev`, and Cargo config overrides
`profile.wasm-dev.opt-level=0` and
`profile.wasm-dev.package.offset-allocator.opt-level=2`.
All377 actual audio PCM hashes/frame counts/loop ranges pass on final6071
(`.local/exterior-bends-final-native-audio.log`). Scoped Rust formatting passes;
all-target assets/Bevy Clippy completes with existing warnings and the documented
four allowances (`never_loop`, `bad_bit_mask`, `approx_constant`,
`useless_conversion`). The R16 battle fixture's unchanged nested-if warning
predates these edits. Whole-workspace formatting retains unrelated failures.

Source-plan validation passes. The current atlas has20 native route comparisons,
six towns,47 paired rooms, seven installed guarded walking links and33 planned
endpoint leaders; its PNG SHA-256 is
`3a2a90ac67471b889de2c3d4ce745baee30c896c671f35f8913b12c682a31db3`.
Its immutable epoch is `renders/atlas/road-bends-6071912e-native-facing/`.
The cave gallery pixels are identical to the reviewed31258 epoch.

These changes address specific weak passages. User visual approval, full earned
regional progression, Safari admission/capture sessions, remaining travel links
and100 Minnesota-species integration remain open. This report does not certify
the entire region as matching Johto quality.
