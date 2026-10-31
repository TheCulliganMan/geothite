# Gunflint spring grotto

The complete cave is recomposed from 16×12 to 20×14 metatiles. A small western
camper recess and a deeper eastern cache recess frame a rock fin, a fitted
hanging spur and a hooked spring. Separate north and south banks give the pool
a real walking circuit. A solid stone pillar holds an optional hidden Revive
that can only be searched from the water. The original entrance, rest service
and Max Ether remain accessible without Surf.

The verified local pack is `content-packs/minnesota-gunflint-grotto-v02.crystalpack`.
Its composed source index and the 20×14 shared cave reservation are selected in
`planning/minnesota/world.json`. Newer research services and every other map
are retained. This is local map acceptance; wider Minnesota review continues.

`before.json` is the complete original terrain. `after.json` contains deliberate
native rows and event anchors; all earlier source versions are retained.
`contract.json` binds the complete original map module and parent pack.
`plan.json` records every route, coordinate, service, flag and verification gate.
No native game art, palettes, music, encounters or audio programs are copied or
replaced.

- The original mouth is `(17,23)`, clear floor `(17,22)`. It retains its actual
  `MnR12C00R00 (40,15)` exterior pairing and settled return `(40,16)`.
- Otta stays at `(5,5)`, approached from `(6,5)`. His existing refusal,
  acknowledgement and free full-party HP/PP/status care programs remain exact.
- The original Max Ether moves from `(27,5)` to `(35,3)`, approached from
  `(34,3)`. Its original `EVENT_MN_R12_CAVE_ETHER` claim flag is retained.
- The straight stair uses native **30**, metatile `(3,4)`, walking column 6.
  The right-corner stair uses **36**, metatile `(17,4)`, walking column 34;
  its right side `(35,9)` remains a real wall. There are no ledge/cliff stacks.
- Two paths circle the central fin; two more circle the spring. Their exact
  production-input courses are in `plan.json`. The new pool stays contained,
  retaining the original shore `(21,19)` and water habitat `(22,19)`.
- `MN_R12_GROTTO_REVIVE` is a native hidden-item event at pillar `(28,16)`.
  Only water tiles `(27,16)` and `(28,15)` can interact with it. Native Surf
  move/badge requirements gate access. It awards one Revive with
  `EVENT_MN_R12_GROTTO_REVIVE`; Itemfinder can detect the unclaimed item.
- The quiet walking floor, rate-5 Minnesota water wildlife and original lake
  fishing group remain exact. No trainer is added to this shelter.

The latest executed Rust publication preserves the newer research-access parent:

```sh
.local/gunflint-grotto-publisher-v904 \
  content-packs/minnesota-early-research-v01.crystalpack \
  1fd1bfe59ad965809741259f88a7c251e59e2add5317bfc5eacd17da7b9b7548 \
  planning/minnesota/caves/gunflint-grotto-v01/before.json \
  planning/minnesota/caves/gunflint-grotto-v01/after.json \
  planning/minnesota/caves/gunflint-grotto-v01/contract.json \
  content-packs/minnesota-gunflint-grotto-v02.crystalpack
```

The output must be fresh and ignored. Rebuild the publisher with
`cargo build --locked -p crystal-assets --bin rebuild_gunflint_grotto`, using
the serialized shared Cargo slot and repository toolchain. Strict Rust readback
and the preservation audit retain all 734 other complete maps, every original
program, catalog, native asset, encounter table and audio program. The publisher
rejects stale complete-map inputs, dry pillar bypasses, an incorrect corner
stair, unpaired mouths and water on the outer boundary.

Fast source review:

```sh
PYTHONPATH=.local/ely-backroute-python python3 \
  planning/minnesota/tools/review_cave_source.py \
  --pack content-packs/minnesota-leech-mainland-v01.crystalpack \
  --source planning/minnesota/caves/gunflint-grotto-v01/after.json \
  --output planning/minnesota/renders/gunflint-next-source --watch
```

Every revision receives immutable full-resolution originals. The final installed
whole-room render took 0.101 seconds after loading the pack. All ten headless
native camera frames match the source at all 368,640 pixels each, using the
forced native night palette. The whole room and the complete native camera
sheet have been inspected. Otta's inherited rest controller regression passes;
all four new traversal, resource-use, capacity and water-discovery cases pass.
A fresh independent stdio replay passes 663 calls, including all four dry
branches, both stairs, the original rest, cache, exterior return and two
fresh-process Continue cycles. The new water case proves both Surf denials,
actual launch, Itemfinder before/after, once-only claim, native Revive use and
landing. Both caches retain their flags after a full-pocket denial, Continue
and native TOSS retry.
All 377 audio programs pass PCM hash, frame-count and loop-range verification.

- [Whole installed cave](../../renders/gunflint-grotto-v898/installed-v907/whole-original.png)
- [Ten native views](../../renders/gunflint-grotto-v898/native-v906/MnR12Cave1F-production-cameras.png)
- [Complete-parent preservation](../../renders/gunflint-grotto-v898/current-preservation-v921.json)
- [Publication binding](../../renders/gunflint-grotto-v898/publication-v918.json)

Controller fixtures declare local party, prior badge, position, injured resources
or full-pocket setup before input. Later actions use `VisibleShellController`;
these checks do not establish earned campaign progression. Previously saved
terrain/actor overrides are not migrated. Broader Minnesota map review continues.

The first Max Ether gameplay harness accidentally confirmed its combined
party/move target twice. It was corrected; all four final cases pass. The first
independent replay used an older client that predates water-only peaceful-floor
support and failed before traversal. The replay with the current client passes.
All failed records, reached saves and earlier captures remain retained.

Both affected crates pass all-target Clippy, with `location-tester` enabled for
crystal-bevy and existing unrelated warnings retained. The first Clippy pass
caught a redundant unstable `as_str()` call on an existing `&str` in concurrent
biome authoring; its one-line correction is included in the passing checks.
Changed Rust leaves pass formatting. Workspace-wide formatting reports existing
differences beginning at `crates/offset-allocator/src/lib.rs:408`.

[Final acceptance](../../renders/gunflint-grotto-v898/acceptance-v928.json) binds
all native checks, the complete-parent audit, regional roster policy, exact
publication replay, current source index and entrance reservation. The regional
policy audit covers 4,626 references against the 225 active Minnesota species.
All 377 audio checks pass. Old selected packs and every render remain retained.

To repeat the controller checks, set both `CRYSTAL_WINONA_TEST_PACK` and
`CRYSTAL_RENDER_TEST_PACK` to the completed grotto pack. Use a fresh
`CRYSTAL_GUNFLINT_GROTTO_REVIEW_DIR` for the four `gunflint_grotto_controller`
cases and a fresh `CRYSTAL_GUNFLINT_SPRING_REVIEW_DIR` for the inherited
`gunflint_spring_controller_rest_refusal_acknowledgement_all_party_resources_and_continue`
case. These are opt-in tests in crystal-bevy's default test profile; run serially
with `--ignored --nocapture --test-threads=1`. `check_mcp.py --help` documents the
independent production trip from its emitted initial injured camp save.
