# Cave and shoreline review composition

The written510-map review pack is
`content-packs/minnesota-northstar-map-refinements.crystalpack`, SHA256
`c16af00c82167c809620b956981ee80641ddc9fd1eeaf1e2c91f59ab16a9cc7d`.
Internal content identity:
`58563e27697450fea7325726e01fc88a18a44d0d69a5a9e4861ac2072c4d10b5`.
It builds on the completed Ryan-house/current-story510 composition, SHA
`c0c15e1193c816bcf6e5272af5c3dc716deed588d5b0d2070c01a59e723e2dc5`.
Generated packs and all render stages remain ignored and retained.

## Authored changes and visual evidence

R10's original12×8 gallery is replaced by the reviewed18×14 room. Its tapered
ore spine, irregular spring, four authentic stone stair lanes and short return
drop create a complete circulation loop. Original reader, clue, mouth and
Revive program/quantity/flag stay exact; the Revive explicitly moves to29,23.
The [whole native-palette gallery](../renders/caves/r10-native/e0cf8cf00b18c1b2/review-1793065743751041000/whole-native-nite-actor-review.png)
and [six actual player views](../renders/caves/r10-native/e0cf8cf00b18c1b2/review-1793065743751041000/production-decision-cameras.png)
were inspected. Those captures name their earlier written gallery stage;
the current composition's exact R10 contract is certified separately below.

R12 stays16×12. An offset zigzag climbs to separate shelter and cache spurs;
four native stairs and a two-lane short drop replace the large rectangular
hollow and lower ring. Otta, original two dialogue pages, Max Ether and all
warps remain exact. Its [whole current-palette room](../renders/caves/r12-native/21dfc23c61578ee3/review-01/whole-actual-pack-source-night.png)
and [eight actual player views](../renders/caves/r12-native/21dfc23c61578ee3/review-01/production-decision-cameras.png)
were inspected, including the weakest upper turn. The strict native comparison
passes108 checks against the immediate R10 stage. These captures use the actual
written cave stage SHA21dfc23c…; the final shore-only step retains that room
exactly. Source and native terrain match at180,480 measured pixels outside
OAM, plus1,408 opaque player pixels, with no mismatches.

Both caves keep their original forced NITE palettes. The R12 exit remains an
active exterior CAVE warp; native forced downward movement settles the player
on safe global66,24. No passive arrival record was invented. See the
[controller contract](../reviews/r12-shelter-controller-contract.md).

R11 retains its56×96 geography, falls, stairs, required grass, actors and all
story programs. Seven explicit hand strokes change only sixteen metatiles:
attached pine shoulders enforce the Stoney shore bend, while ordinary native
canopy edges tighten the Lester return to a two-tile southern road. No
walkable cell is added. The [three actual current-pack views](../renders/north-shore/current510-native/c16af00c82167c80/native-cameras-review01/production-decision-cameras.png)
show both changes and the widest remaining upper segment. All captured anchors
are actual unoccupied native Floor. Headless originals, raw GPU descriptors and
buffers, crops, enlargements, rejected versions and source snapshots remain.

The broad upper reader chamber in R10 and some older northern route stretches
still need visual judgment. Local publication does not establish final Johto
quality across all twenty routes. Other caves remain under review.

## Executed checks

All fourteen focused asset cases pass: five R10, five R12 and four shore-bend
cases against the actual current510 lineage. They cover exact preservation,
actual written readback/uncached assembly and negative source/geometry cases.
Logs: `.local/current510-gallery-shore-assets-diagnostics-fixed.log` and
`.local/r12-shelter-native-assets-mouth-fixed.log`.

The [actual final-pack preservation audit](../renders/northwoods-native/proof/current-refinements/README.md)
passes163/163 checks. All507 other modules, art/audio/catalogs/parties,
HOME/recovery/Fly, ICE/story programs, Ryan's house and city links remain exact.
Only the enumerated cave/shore changes, one R10 manifest and bounded identity
changes are allowed. R12 and R11 have no transport-cache exception; R10's
two exact canonical transport labels are explicitly checked.

All three R10 production-controller cases pass on the final510 pack:
mouth/return, four stairs, hop/reverse/ordinary return, original printed reader
pages, actual Revive, full-bag refusal, native Potion use/retry, once-only reward
and semantic Continue. Log: `.local/final510-r10-gallery-controller.log`.

All three R12 controller cases now pass on the final510 pack: four stairs,
both hop lanes/reverse rejection/ordinary returns, original mouth and safe
return, Otta's two fully printed pages, original Max Ether pickup/use/once,
Save/Continue, and real Ether/Max Ether/Elixer/Max Elixer effects and full-PP
refusal without consumption. All later mutations use production input.
Log: `.local/final510-r12-native-pp-dispatch-fixed.log` (12.35s).

The tests exposed a real PP frontend gap. Nullable amount means full restore;
native MOVE/POKEMON scopes now select the existing target menus. All-move PP
items use the existing PP mutation with no move slot, rather than the healing
mutation. Unchanged core effects perform restoration/refusal/consumption.
Fully printed success/refusal text and movement ownership are checked.

All three R11 expansion controller cases also pass on the final shore revision:
stairs, three ledges/ordinary returns, grass crossings, keeper room, eight-chunk
Surf/boundaries/Continue and all authored waterfall ascents. The shared Mom/exit
gate and battle-command/actual selected PP regressions pass. These component
runs total nine local map cases plus two shared controller cases. Fixtures
explicitly set initial local party/position/badges/RNG; they do not earn the
regional story. Logs are `.local/final510-shore-bends-controller.log`,
`.local/final510-shared-mom-gate-controller.log` and
`.local/final510-shared-battle-controller.log`.

Fresh Rust synthesis verifies all377 final-pack audio PCM hashes, frame counts
and loop ranges: `.local/current510-map-refinements-audio.log`.

## Reproduce the written composition

Use the pinned Rust toolchain and completed external baseline. Capture source
inputs before publication; do not substitute later live drafts.

```sh
CARGO_TARGET_DIR=target/runtime-fps CARGO_BUILD_JOBS=1 cargo build --locked \
  -p crystal-assets --bin revise_r10_gallery --bin revise_authored_cave \
  --bin revise_authored_route --profile wasm-dev \
  --config profile.wasm-dev.opt-level=0 \
  --config profile.wasm-dev.package.offset-allocator.opt-level=2

target/runtime-fps/wasm-dev/revise_r10_gallery \
  content-packs/minnesota-northstar-ryans-house.crystalpack \
  planning/minnesota/caves/MnR10Cave1F.previous.json \
  planning/minnesota/caves/MnR10Cave1F.candidate.json \
  content-packs/minnesota-northstar-map-refinements-r10.crystalpack

target/runtime-fps/wasm-dev/revise_authored_cave \
  content-packs/minnesota-northstar-map-refinements-r10.crystalpack \
  planning/minnesota/caves/MnR12Cave1F.previous.json \
  planning/minnesota/caves/MnR12Cave1F.candidate.json \
  content-packs/minnesota-northstar-map-refinements-caves.crystalpack

target/runtime-fps/wasm-dev/revise_authored_route \
  content-packs/minnesota-northstar-map-refinements-caves.crystalpack \
  planning/minnesota/revisions/native-map-refinements-2026-10-26/MN_R11.previous.layout.json \
  planning/minnesota/revisions/native-map-refinements-2026-10-26/MN_R11.revised.layout.json \
  planning/minnesota/revisions/native-map-refinements-2026-10-26/MN_R11.native-certification.json \
  content-packs/minnesota-northstar-map-refinements.crystalpack
```

The shore certificate names installed trainer MARA. The live planning name
Sivren is unpublished; this terrain revision does not rename the native party
or relax identity validation. The validator now resolves installed destination
constants and certifies the declared destination warp for both exterior door
records, preserving Ryan's existing passive room3 exactly.

The fast cave source preview reads forced native terrain/NPC palettes and
retains every render stage. A warm R10 review measured0.11s with the pack already
loaded; pack loading is additional. The atlas retains native room pixels and
sizes each gallery row to its actual largest room, avoiding clipped cave art.
Neither tool exports game content or generates terrain.

## Source promotion and checks

The reviewed R11 layout and R12 room/interior contract are promoted with guarded
single-record edits and immutable immediately-before source copies. The full
plan check passes20 routes,19 hubs,57 chunks,86 adjacency/gate contracts and160
planned maps. Its symbolic badge/champion reachability is not gameplay proof.
Log: `.local/current510-map-refinements-promoted-plan.log`.

Scoped Rust formatting passes. Workspace-wide `cargo fmt --all -- --check`
still reports unrelated existing formatting, beginning in
`crates/offset-allocator/src/lib.rs:408`; no broad formatting was applied.
Clippy's ordinary final run hits pinned-toolchain E0275 recursion while its
`useless_conversion` lint probes nested Bevy `Res` iterator candidates. The
isolated library/tests and final all-target assets/Bevy runs pass with that
single additional lint disabled,
alongside the previously recorded baseline lint allowances. No recursion limit
or production code was changed to hide it. The exact invocation and bounded
[diagnosis](../reviews/clippy-useless-conversion-diagnosis.md) distinguish the
confirmed lint isolation from the unlocated source trigger.
Final all-target log:
`.local/current510-map-refinements-clippy-toolchain-workaround.log`.
