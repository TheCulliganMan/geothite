# Cedar Shelter fishing bank and fitted care stair

This complete 10×8 native cave revision gives the original League Approach
shelter three useful branches: a quiet west fishing bank, Harper's raised care
terrace to the north, and the original Max Ether in the eastern supply notch.
The entrance remains at `(7,15)`. No random walking encounters or new mandatory
battle are added to this safe recovery stop.

The verified local pack is
`content-packs/minnesota-r13-cedar-basin-v01.crystalpack`. Active selection is
recorded in `planning/minnesota/world.json`. The parent is bound in
`contract.json`; do not publish against a different selected pack without rebasing.

`before.json` and `after.json` are complete native terrain rows, not generated
geometry. The native night palette, dimensions, original actors, script/runtime
commands, fishing tables, item event and room/exterior warp pair remain exact.
The west branch now reaches a contained, connected 24-tile basin. The north
climb uses native left stair `30`; the supply notch retains native `36`.

The whole cave and all eight source/native camera pairs were inspected. All
368,640 pixels of each production frame match native source art. The source
preview loop measured 0.052 seconds after loading the pack. Original, cropped,
converted and rejected renders remain in the immutable review folder.

- [Whole cave comparison](../../renders/r13-cedar-basin-v749/before-after-v01.png)
- [Eight production cameras](../../renders/r13-cedar-basin-v749/native-v752/MnR13Cave1F-production-cameras.png)
- [Native pixel comparison](../../renders/r13-cedar-basin-v749/native-v752/capture-progress.json)
- [Exact parent preservation](../../renders/r13-cedar-basin-v749/preservation-v752.json)
- [Complete acceptance](../../renders/r13-cedar-basin-v749/acceptance-v761.json)
- [Four native gameplay cases](../../renders/r13-cedar-basin-v749/controller-v757/receipt.json)
- [Production stdio replay](../../renders/r13-cedar-basin-v749/mcp-v760/receipt.json)
- [Actual reached-save views](../../renders/r13-cedar-basin-v749/reached-native-v759/actual-reached-views.png)

Executed Rust build and publication:

```sh
CARGO_TARGET_DIR=target/verdigrim-scene CARGO_BUILD_JOBS=2 RUST_MIN_STACK=67108864 \
  cargo build --locked -p crystal-assets --bin rebuild_r13_cedar_basin
target/verdigrim-scene/debug/rebuild_r13_cedar_basin \
  content-packs/minnesota-international-falls-windbreak-v01.crystalpack \
  7066f05448101c79b4dc761963b0f39a570bad07c4f80e3e777901ac2e207507 \
  planning/minnesota/caves/r13-cedar-basin-v01/before.json \
  planning/minnesota/caves/r13-cedar-basin-v01/after.json \
  planning/minnesota/caves/r13-cedar-basin-v01/contract.json \
  content-packs/minnesota-r13-cedar-basin-v01.crystalpack
```

The output must be fresh and ignored. Reusing that exact output path is rejected.
The publisher changes only this room's blocks and mirrored block payload, then
verifies a strict native readback. All 732 other maps and all original art,
audio and runtime files remain exact. All 377 audio programs pass PCM hashes,
frame counts and loop ranges.

Fast rerender loop; `--watch` rerenders each changed source into a new version:

```sh
PYTHONPATH=.local/ely-backroute-python python3 \
  planning/minnesota/tools/review_cave_source.py \
  --pack content-packs/minnesota-international-falls-windbreak-v01.crystalpack \
  --source planning/minnesota/caves/r13-cedar-basin-v01/after.json \
  --output planning/minnesota/renders/r13-cedar-next-preview --watch
```

The static audit checks the exact whole parent, legal regional fishing tables,
all walkable tiles, stair lanes, service approaches and enclosed connected water.
Four passing production tests separately exercise true passive exterior return, both stairs
in both directions, exact Harper pages and acknowledged HP/status/all-PP healing,
once-only native Max Ether collection/use, a real rod battle, and Continue.
These tests use explicit local party/position/rod fixtures. They do not prove
an earned regional campaign arrival or complete the broader map objective.
The two fresh stdio scenarios make 282 calls with empty stderr. Scoped all-target
Clippy and changed Rust formatting pass. Workspace formatting still reports
pre-existing differences beginning at `crates/offset-allocator/src/lib.rs:408`.

Executed native gameplay regression command, after the default-profile
`cargo test --locked -p crystal-bevy --lib --no-run --message-format=json` build:

```sh
CRYSTAL_R13_REFUGE_CONTROLLER_PACK="$PWD/content-packs/minnesota-r13-cedar-basin-v01.crystalpack" \
CRYSTAL_R13_REFUGE_SOURCE="$PWD/planning/minnesota/caves/r13-cedar-basin-v01/after.json" \
RUST_MIN_STACK=67108864 \
  .local/r13-cedar-controller-v751 r13_refuge:: --ignored --nocapture --test-threads=1
```

The frozen controller binary and all resulting saves remain ignored. The
run's review directory was `renders/r13-cedar-basin-v749/controller-v757`;
use a fresh directory when retaining a new run through
`CRYSTAL_R13_CEDAR_REVIEW_DIR` and `CRYSTAL_MAP_RETAIN_SAVES_DIR`.
