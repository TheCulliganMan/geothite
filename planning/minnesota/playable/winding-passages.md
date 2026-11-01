# Winding passages and the split-roof shelter

The current map-quality composition is
`content-packs/minnesota-northstar-seep-shelter-review.crystalpack`:
**541 maps,251 species,251 moves,377 audio programs**.
File SHA256: `7e5f65b935b1f74766c810ee72f444ad60a7e71e096b5dec10c46f516595344a`.
Internal identity: `f26fb7584c6a66d0964b78f570fb6273bb2fef82c1d5895a246ae08dec10ca37`.

This folds the earlier nine-cave, R06/R16 road and ledge, and R10 outdoor
stone-stair work into the actual [Leech Shore story successor](leech-shore.md),
then adds the three exterior refinements and deeper R12 room described below.
It retains the current towns, gyms, checkpoints, dialogue, recovery, four new
shore links and visible road caps. The older510/538 map packs remain historical
review epochs. This is ongoing twenty-route work; no final user approval or
full earned regional playthrough is claimed.

## Hand-authored changes

| Place | Concrete traversal | Source contract |
| --- | --- | --- |
| R08 Cedar Lake | Winding dry shore walk reveals the lake around an attached promontory. The existing meadow saves8steps:36dry versus28with15grass. | [Explicit controller inputs](../routes/MN_R08.cedar.walk.json); [candidate review](../renders/exterior-quality-2026-10-27/r08-cedar-walk/README.md) |
| R15 Central Lakes | A fitted S approach joins cabin, dock branch and stone road. Western meadow landing now meets Ron's road at a framed junction, then turns past flowers toward the outlet bridge. | [Shore approach](../reviews/r15-shore-walk-refinement.md); [western junction and exact paths](../reviews/r15-western-junction-refinement.md) |
| R19 prairie/oak link | Attached oak hook bends the approach to the observation board and compulsory habitat. Both original oak-ramp loop and battle-versus-grass alternative remain. | [Source, collision and remaining weaknesses](../routes/MN_R19.prairie-approach.review.md) |
| R12 shelter | Low seep basin leads through a stone stair throat to separate Otta roof and Ether branches. A short two-lane drop returns to the basin; ordinary uphill returns take21/19steps. Three useful stairs replace the old parallel four-stair switchback. | [Whole room, nine cameras, exact stairs and branches](../caves/MnR12Cave1F.depth.review.md) |

Whole-route source renders and worst cameras were inspected before publication.
Every terrain stroke is authored explicitly; path finding plans verification
inputs and does not generate the maps. No new actor, item, HM, encounter table,
art asset or story reward was added by these four refinements.

R08's broader east/south roads and R19's upper arrival/bend remain quality work.
R18's shorewalk and winding dune ribbon are separate source proposals and are
not in this pack. Other caves require individual judgment; R12's improvement
does not establish acceptance of all nine rooms.

## Native appearance

[R08 actual player views](../renders/map-quality-538/native-passages-v01/MN_R08/production-decisions.png),
[R15 actual player views](../renders/map-quality-538/native-passages-v01/MN_R15/production-decisions.png),
[R19 actual player views](../renders/map-quality-538/native-passages-v01/MN_R19/production-decisions.png),
and [R12 actual player views](../renders/map-quality-541/r12-depth-native/production-decisions.png)
were personally inspected, along with each whole source map.

All27 actual headless640×576 frames match their complete expected native source
pixels, including actors and CHRIS. The18 exterior captures use the exact538
geometry stage `585d752c…`; independent final541 proof certifies unchanged
local terrain/actors/art at those cameras and the six disjoint retained R08
perimeter caps. The nine cave captures use final541 directly. Whole-map source
composites are labeled separately from production screenshots.

The current fullscreen renderer uses3physical pixels per source texel and shows
extra terrain around the LCD. R12's mouth camera clamps near the finite map
edge: unique full opaque CHRIS matching measures `(320,312)`, rather than
assuming `(320,288)`. Every other cave camera measures `(320,288)`. One
consistent native water animation phase is required across each whole frame;
grass priority uses original nonzero source indices. The actual grass frame
can obscure CHRIS; it is preserved rather than painted out. No production
image is edited to obtain a match. Initial comparison failures, every genuine
phase variant, GPU readbacks, conversions and crops remain retained.

Read-only reproduction uses the current Rust location tester and existing
source-art renderer:

```sh
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_shelter_depth_native.py \
  --pack content-packs/minnesota-northstar-seep-shelter-review.crystalpack \
  --pack-sha 7e5f65b935b1f74766c810ee72f444ad60a7e71e096b5dec10c46f516595344a \
  --source planning/minnesota/caves/MnR12Cave1F.candidate.json \
  --binary target/runtime-fps/wasm-dev/examples/render_at_location \
  --output planning/minnesota/renders/map-quality-541/r12-depth-native-reproduction
```

Use a fresh output directory. `--resume` checks retained original hashes before
reusing captures; it never substitutes a different capture or modifies one.

## Actual controller evidence

R08's populated journey, both explicit shore/meadow walks and native Cut gate
pass. They retain trainer victories with selected PP mutation, collected
resources and Continue. R15's three cases pass: both bridge rows, dock barriers,
both cabin carpets, actual meadow hop/reverse,3wild escapes,2trainer wins,
both resource branches and persistent party/bag/flags. The full journey now
walks the actual S bend and western junction, not old straight segments through
new woodland. R19's three cases pass, including its ramp/observation course,
real Headbutt battle/PP and refusal without Headbutt.

R12's three cases pass on final541: all three stairs in both directions and
their side walls, both actual hops/reverse and21/19-step ordinary returns,
original mouth/re-entry, all original Otta pages/input ownership, pickup once,
native Ether/Elixir effects and full-PP refusals, consumption and Continue.
The first geometry case caught stale editorial shoulder probes at the clear
upper lip. Metadata-only v04 corrects these to real Wall course cells
`(13,15)/(16,15)`; all rendered/native rows remain v03. The corrected case is
rerun separately; both original successful PP/resource cases remain recorded.
The earlier failed run is retained.

Fixtures declare party, badges, position and divider only at setup. They do
not prove earned access from a fresh game. All later movement, battles,
dialogue, resources, warps and save operations use `VisibleShellController`.

All six Leech Shore chapter controller cases also pass on final541, including
both-lane badge refusal/free return, both cabins/carpets, native care, Surf
coast/dismount/Continue, real blackout recovery and the continuous Pine loss,
care, Fire retry, badge/TM and home course. The first final-pack loss attempt
unexpectedly won using Psychic; the identical binary still lost on untouched
03c9. Changed road/encounter inputs consume the divider differently. The test
now selects its already-known, weaker neutral Cut for the actual loss attempt;
it preserves all party/HP/opponent/RNG state after setup. That complete replay
passes in29.11seconds. Both first failure and baseline comparison remain under
`renders/map-quality-541/shore-successor-controller/`.

## Publication and preservation

The exact story parent is immutable03c9:
`.local/minnesota-city-revisions/leech-shore-03c9d073629c60debc66368f07a4170d0b305026c84fd3d5fdcfd96f95c60a97.crystalpack`.
The12 successful strict Rust publication commands, exact inputs, stage outputs
and file hashes are retained in
`renders/map-quality-541/publication-receipts.json`; the13th R12 revision is
logged in `.local/map-quality-541-r12-depth-publication.log`. Publication uses
the existing strict terrain APIs and readback; no native module copying,
alternate exporter or story rollback is used.

[Independent final comparison](../reviews/winding-shelter-541-03c9-7e5f.md)
passes194 checks and rejects8 corruptions. Exactly13modules/300metatiles
change; all528other whole modules,70city/room modules,8gyms, story/programs,
recovery, links and actor records remain exact. Beyond owned blocks/payloads,
only the exact prior R10 stair alias dependency, its two exterior attributes,
five art files/two manifests, ordinary Repel200→100 and derived identity change.
The final identity is independently recomputed from the actual stored CBOR
tuple, not trusted from its declared hash.

All541 attribute/NPC/block and raw MapScripts/MapEvents section mirrors agree.
Nine pre-existing raw text-body disagreements remain byte-for-byte unchanged;
the report names them rather than claiming complete unrelated text coherence.
All377audio PCM hashes, frame counts and loop ranges pass on final541
(`.local/map-quality-541-final-audio.log`). Affected all-target Bevy Clippy
completes with existing warnings and the established four allowances; scoped
formatting passes. Global formatting retains unrelated failures beginning at
`crates/offset-allocator/src/lib.rs:408`.

Guarded source promotions preserve all exterior dossiers and every other R12
route value. Exact preimages/after files and receipts remain under
`reviews/winding-passages-promotion/` and `reviews/r12-depth-promotion/`.
Generated packs, images, saves, binaries and build products stay ignored.
