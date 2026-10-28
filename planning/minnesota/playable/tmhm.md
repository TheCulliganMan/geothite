# Minnesota TMs and HMs

The Minneapolis Department Store's fifth-floor clerk sells all 50 TMs followed
by all seven HMs, in machine-number order. TMs retain their pack prices
(¥1,000–¥3,000); each HM costs ¥3,000. HMs remain reusable. Species compatibility,
move replacement restrictions, existing story gifts and field-move badge gates
use the existing production systems.

Twenty additional hidden TMs reward exploration of existing route side paths
and caves. Each cache grants one TM once, with its own saved event flag. They
do not replace existing items, trainers, encounters or story rewards. The exact
map, interaction tile and adjacent approach are authored in
[tmhm-distribution.json](../tmhm-distribution.json). Terrain, scenery and audio
are inherited from the completed Minnesota cries parent.

| Route | Hidden TM |
| --- | --- |
| R01 cave | Mud-Slap |
| R02 | Sunny Day |
| R03 cave | Dig |
| R04 | Return |
| R05 | Steel Wing |
| R06 | Rollout |
| R07 | Headbutt |
| R08 | Ice Punch |
| R09 | Earthquake |
| R10 cave | Shadow Ball |
| R11 | Rain Dance |
| R12 | Blizzard |
| R13 | Hyper Beam |
| R14 | Solarbeam |
| R15 | ThunderPunch |
| R16 | Protect |
| R17 cave | Toxic |
| R18 | Sandstorm |
| R19 | Thief |
| R20 cave | Nightmare |

The Rust publisher requires the supplied ignored parent
`content-packs/minnesota-cries.crystalpack` and the authored
distribution JSON. It checks the parent's content hash, the existing clerk,
all 57 machine definitions, cache collision and event conflicts. It writes a
separate ignored pack and verifies that the written identity matches.

```sh
CARGO_BUILD_JOBS=2 CARGO_TARGET_DIR=target/tmhm cargo run --locked --profile wasm-dev \
  --config profile.wasm-dev.package.crystal-assets.opt-level=0 \
  --config profile.wasm-dev.package.crystal-core.opt-level=0 \
  -p crystal-assets --bin pack_minnesota_tmhm -- \
  content-packs/minnesota-cries.crystalpack \
  planning/minnesota/tmhm-distribution.json \
  content-packs/minnesota-all-tmhm.crystalpack
CARGO_BUILD_JOBS=2 CARGO_TARGET_DIR=target/tmhm cargo test --locked --profile wasm-dev \
  --config profile.wasm-dev.package.crystal-assets.opt-level=0 \
  --config profile.wasm-dev.package.crystal-core.opt-level=0 \
  -p crystal-assets --test minnesota_tmhm -- --ignored
RUST_MIN_STACK=33554432 CARGO_BUILD_JOBS=2 CARGO_TARGET_DIR=target/tmhm-controller cargo test --locked --profile wasm-dev \
  --config profile.wasm-dev.package.crystal-assets.opt-level=0 \
  --config profile.wasm-dev.package.crystal-core.opt-level=0 \
  --config profile.wasm-dev.package.crystal-runtime.opt-level=0 \
  --config profile.wasm-dev.package.crystal-bevy.opt-level=0 \
  -p crystal-bevy --lib minnesota_tmhm -- --ignored --test-threads=1
```

The preservation test compares every unrelated dataset and all runtime/audio
files, purchases and teaches every machine, checks initial PP and verifies TM
consumption versus HM reuse. Controller tests use explicit local position,
party and money fixtures; every purchase and discovery after setup uses
`VisibleShellController::press`. They test cancellation, confirmation, all 57
inventory entries, hidden-item flags and Continue persistence. These local tests
do not establish earned travel through the entire region.

Verified on 2026-10-27:

- Published and read back the 664-map, 474-species, 270-move pack. Its content
  hash is `d0c1cb4c6da36acf80eb86a257fae759fc491fc1ab09570d6b78b88a45a47a47`.
- The asset preservation/purchase/teaching test passed, including a second use
  of each purchased HM and compatibility with original Minnesota species.
- Both controller tests passed: all 57 purchases and cancellations, and all
  20 one-time discoveries across save/Continue. The unoptimized fixture needs
  the documented 32 MiB test-thread stack.
- `target/tmhm-controller/wasm-dev/verify_crystalpack
  content-packs/minnesota-all-tmhm.crystalpack --audio --desktop` passed all
  600 PCM hashes, frame counts and loop ranges.
- The existing production text client loaded the written pack. Stdio MCP
  initialize, observe, press START and the production PACK menu passed with
  empty stderr. The dump was a separate rendering smoke check.
- All new Rust files pass rustfmt. Workspace-wide formatting still fails on
  existing files, starting with `offset-allocator/src/lib.rs`. Required broad
  Clippy fails on the existing `bad_bit_mask` in
  `crystal-core/src/multiplayer/mod.rs:1332`; dependency-free Clippy also finds
  11 existing `never_loop` errors in asset validation. The scoped all-target
  check with `--no-deps -- -A clippy::never_loop` completed with existing
  warnings and no diagnostics in the new TM/HM files.

An existing Northstar recruitment test helper now borrows its controller
mutably, as required by `snapshot`; this one-line correction unblocks test
compilation. Source placement cameras are retained under
`.local/tmhm-review-v2/`. These check placement context; no scenery was changed.

The output has its own pack identity. Use a save created with this pack;
older saves retain their original pack identity.
