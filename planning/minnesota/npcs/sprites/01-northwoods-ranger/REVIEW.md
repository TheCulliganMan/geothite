# Northwoods Ranger pilot

Status: **ACCEPTED**. See ACCEPTANCE.md and manifest.json for current evidence.
The notes below retain rejected and superseded diagnostic attempts; their old
live-job instructions are historical and must not be resumed.
Finish this NPC before starting another. The full goal remains at least 15
accepted NPCs with both overworld and battle sprites.

## Current source and native art

- Identity: RANGER ROWAN, broad hat, dark braid, green field jacket,
  tan trousers, binoculars.
- `source-v1.png` rejected for background glow and unsuitable native pixels.
  `source-v2.png` removes the glow; prompts are retained.
- Early battle conversions were rejected for excessive black and lost face detail.
  `battle-source-v3.png` and its prompt provide the simpler current battle source.
- `native-pixels-v1.json` is the authoritative native grid: six 16x16
  overworld frames and one 56x56 battle view, palette RGB555
  `[32767,18239,10794,0]`. Earlier `battle-refinement-v1.json` is superseded.
- Current generated native output: ignored `target/npc-sprites/ranger/native-v5/`.
  Independent tile decode passed for both banks. Source/PNG/tile/palette/alpha
  agreement, nonblank poses, and distinct walk pairs passed during publication.
  Enlarged drafts were inspected; actual encounter captures are still required.

Native overworld order is down idle, up idle, left idle, down step, up step,
left step. The engine mirrors the left views for right-facing movement.

## Published candidate

`content-packs/minnesota-original-ranger-v02.crystalpack` (~29 MiB) was built
successfully and passed readback identity, exactly one added trainer, preservation
of every existing trainer and runtime file, and preservation of the parent's
trainer-scaling setting. Publication session 28876 is terminal exit 0; log:
`target/npc-sprites/publish-ranger-v02.log`. Do not rebuild this candidate just
to resume verification. v01 is superseded because scaling preservation was fixed
after its compilation began.

Parent is `minnesota-northstar-regional-cast-v2.crystalpack`, an isolated older
pilot base. Final acceptance also requires integration with the latest compatible
gameplay pack.

`placement-v1.json`: MnR01C00R00 (4,22), west-facing; approach (3,22);
class MN_NORTHWOODS_RANGER; sprite SPRITE_MN_NORTHWOODS_RANGER;
BIRCHIT level 5 and POUCHID level 6, inheriting the parent's scaling behavior.
The builder validated placement, reachability and parsed trainer scripts.

## Implementation

- `npc_sprite_candidate --native SOURCE OUTPUT` emits the native art bundle.
- `build_original_npc_modpack` / `pack_original_npc` installs one original NPC,
  with unique art/class/sprite, explicit native behavior template, and native
  asset validation. Existing art and trainer identities cannot be replaced.
- The original `mn_` sprite loader uses its own gbcpal sidecar; other NPC
  palette paths retain existing behavior.
- Generic controller regression:
  `original_npc_controller_talk_battle_turn_and_save`.
  It defaults to this ranger and accepts `CRYSTAL_ORIGINAL_NPC_SOURCE` for
  later one-at-a-time cases. It checks production art loaders and palette,
  dialogue ownership, battle start, a PP-spending move, save/load, a complete
  legal-input win, defeat flag, and follow-up dialogue without restarting combat.
  None of these controller assertions has passed yet.

## Live verification work

**Current checkpoint (supersedes the older live-job notes below):**

- Pilot mounted-pack test passed: exactly 1 test, 0 failures, 29.61s; log
  `target/npc-sprites/ranger-controller-mounted-stack32-v02.log`. It verified
  production art loaders, talk, PP-spending turn, save/load, actual victory,
  defeat flag and follow-up dialogue. Assets were literal verified v02 bytes in
  an isolated mount, not replacement art. Use `RUST_MIN_STACK=33554432`, matching
  the repository's native capture configuration; the default test-thread stack
  overflowed before the successful rerun.
- Inspected pilot trainer-intro and first-turn PNGs in
  `target/npc-sprites/ranger/gameplay-mounted-v02/`: ranger identity, palette,
  silhouette and battle framing are readable. This is scoped battle-art review,
  not final NPC acceptance.
- Latest compatible parent is now `minnesota-trainers-v02.crystalpack`.
  The original tile was occupied by ADA; a publication attempt correctly rejected
  overlap. `placement-v2.json` moves Rowan to (8,24), south-facing, with approach
  (8,25) on an independently inspected two-tile-wide native floor path.
- `content-packs/minnesota-original-ranger-v04.crystalpack` published successfully
  on that latest parent, with all preservation checks. Log:
  `target/npc-sprites/publish-ranger-latest-v04.log`; session 70595 terminal exit 0.
- Corrected fixture compilation session 50539 completed successfully and produced
  `target/debug/deps/crystal_bevy-0e4f79a33ebb6d13`. Actual v04 runs reached the
  final defeated-flag assertion, which failed. Cause is NOT yet established:
  it may be a test-party loss or encounter-result handling. Logs:
  `target/npc-sprites/ranger-controller-v04.log` and `...v04-trace.log`.
  Do not weaken the flag assertion or count the ranger accepted.
- Diagnostic recompilation is live in session **43264**, log
  `target/npc-sprites/compile-controller-test-v05.log`. The test now captures
  the terminal overworld state and reports map, party HP and messages on failure.
  It records the flag BEFORE capture so rendering cannot make the check pass.
  Poll this job, then execute its exact test binary against v04 with
  `CRYSTAL_ORIGINAL_NPC_SOURCE` set to the absolute `placement-v2.json` path,
  `RUST_MIN_STACK=33554432`, `CRYSTAL_INTERIOR_PACK` pointing to v04, and a fresh
  `POKEGEAR_PC_RENDER_DIR`. Capture filename is now
  `original-npc-overworld-state.png`. Resolve the failure from actual evidence.
  Other NPCs remain parked.

Update: preparation session 91500 completed exit 0 and produced
`target/debug/deps/crystal_bevy-0e4f79a33ebb6d13` (24m37s first build).
The actual test ran exactly once against v02 and failed before gameplay:
`original_ranger.rs:59`, missing palette under the repository-style asset path.
This was a fixture setup error: production initialization already materializes
bundled runtime assets, but the test loaded art before initializing the controller.
The test now explicitly calls `runtime.materialize_runtime_files()` just like the
existing prairie-art fixture. No production palette-loader change was needed.
Failure log: `target/npc-sprites/ranger-controller-v02.log`; execution session
70860 is terminal exit 101. Recompilation of this fix is live in session **50539**,
log `target/npc-sprites/compile-controller-test-v03.log`, using the same Cargo
configs. Poll that session, then run its exact executable against the SAME v02
pack with captures directed to `target/npc-sprites/ranger/gameplay-v03` and log
`target/npc-sprites/ranger-controller-v03.log`. No NPC acceptance yet.

Preparation compilation is confirmed live: execution session **91500**
(Cargo PID 10369 at last inspection), log
`target/npc-sprites/compile-controller-test.log`. It is compiling renderer
dependencies. Preserve this live job; do not restart it on timeout or quiet logs.

The redundant queued Cargo test session 29647 was deliberately canceled while
waiting on the build lock (exit 130). No gameplay test failed. Instead, run the
exact test executable produced by session 91500 directly after compilation
succeeds, avoiding another Cargo traversal through unrelated workspace edits.
Find its exact path in that log's `Executable unittests` line.

Run that executable with filter `original_npc_controller_talk_battle_turn_and_save`
and flags `--ignored --nocapture`, setting:
- CRYSTAL_INTERIOR_PACK to the absolute corrected v02 pack path.
- POKEGEAR_PC_RENDER_DIR to the absolute ignored
  `target/npc-sprites/ranger/gameplay-v02` directory.

Save output to `target/npc-sprites/ranger-controller-v02.log`.
Confirm one actual test ran and passed. Captures should include
`original-npc-trainer-intro.png` and `original-npc-first-turn.png`;
inspect them and fix defects rather than relying on assertions alone.

Compilation command used:
`cargo test --locked -p crystal-bevy --config profile.dev.package.crystal-assets.opt-level=0 --config profile.dev.package.crystal-bevy.opt-level=0 --lib original_npc_controller_talk_battle_turn_and_save --no-run`.

Targeted changed-file rustfmt checks passed. Full workspace fmt reports unrelated
existing differences. Full Clippy failed on pre-existing
`crystal-core/src/multiplayer/mod.rs:1332`, `clippy::bad_bit_mask`; that failure
does not establish a clean ranger check.

## Remaining acceptance

1. Finish and run the controller regression; repair any failures.
2. Inspect actual game captures, all facings and walking registration at 1x/8x.
3. Confirm the overworld encounter appearance in context.
4. Apply to the latest compatible gameplay pack and rerun relevant verification.
5. Record acceptance only then, before progressing to NPC 2.
