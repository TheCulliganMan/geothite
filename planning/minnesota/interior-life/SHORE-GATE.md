# Grand Marais shore-watch gate

Publication lineage correction: the pack receipts below are retained history.
Current source refuses their obsolete coastal geometry; preserve the rooms and
[rebase onto the compact coast](COMPACT-REBASE.md) before further publication.

Current scope correction: Red Wing's train station is excluded by user direction.
The [current five-stop publication](RAIL-STOP-CORRECTION.md) supersedes earlier
station reservations below; historical pack/test receipts remain unchanged.

The native **2D** publication is
`content-packs/minnesota-shore-gate-v04.crystalpack`: 713 maps, 475 species,
270 moves, 377 audio programs and 212 installed Minnesota non-gym interiors.
It adds one room over the protected [Winona child](WINONA-STATION.md).
The runtime content hash is
`da8243273e178dbf5838d5c7891af58742dac49a30aae68e325138fa15c7dc7d`;
the file SHA-256 is
`d97bb1782b58fbf4a77f71a0f446c5b86911889de0c466549bda2324a526cec0`.
Use a fresh save for this identity. Packs and every render remain ignored.

The public shelter occupies the existing winding Cedar Approach. Its counter
attendant describes the family shore-watch bell and the lamp kept on for the
last walker. A returning paddler has dry socks, agates and Grandma's spare hat.
Both repeatable conversations use ordinary native scripts. They add no fee,
healing, reward, chapter flag or regional shortcut.

The room retains the exact 5×4 Ilex Forest–Azalea gate shell. Both exterior
mouths have two active side carpets and two separate Floor returns. Indoors,
exits 1–4 pair with passive arrivals 5–8. The attendant stands at (5,2) and is
spoken to from (5,4) through the actual counter at (5,3); the paddler stands at
(7,5), with a clear approach at (7,4). Eight bounded exterior blocks place the
native facade, side carpets and two lawn shoulders. The original forest below
the building remains intact in the 2D composition.

## Source lineage and remaining rebase

This source overlay deliberately binds the **actual 32×19 Johto Cedar Approach
in the preserved Winona lineage**. The separate canonical Cedar source is
20×12 with winter art and different connections. This publication does not
claim that the old overlay fits that source. `canonical_rebase_required=true`
is retained in the source and both registries. Rebase the gate's physical
entrances when composing the canonical geography branch.

The obsolete shortcut between R11 and the town is not restored. Both gate
mouths lie on Cedar's existing road, and all original town/route connections,
actors, programs, checkpoints and grass crossings remain exact. The gate does
not shorten the existing winding walk or grant regional access.

## Executed publication and checks

Using the pinned Rust toolchain, the executed publisher build was:

```sh
CARGO_TARGET_DIR=target/interior-life CARGO_BUILD_JOBS=4 cargo build --locked \
  --profile wasm-dev -p crystal-assets --bin pack_shore_gate \
  --config 'profile.wasm-dev.opt-level=1' \
  --config 'profile.wasm-dev.package.crystal-core.opt-level=0' \
  --config 'profile.wasm-dev.package.crystal-assets.opt-level=0'
target/interior-life/wasm-dev/pack_shore_gate \
  content-packs/minnesota-winona-station-v03.crystalpack \
  planning/minnesota/towns/MnGrandMaraisShoreGate.json \
  content-packs/minnesota-shore-gate-v04.crystalpack
```

The last command is the publication receipt; choose a fresh output filename
for a replay. The publisher refuses an occupied output and strictly rereads
its completed pack. The native content is deterministic and also matches the
earlier eight-patch v02 candidate. The ten-patch v03 draft and every rejected
render remain retained; its front rock blocks are absent from v04.

The fresh asset integration case passes (18.47 seconds). It rebuilds the written
identity from the source, reconstructs raw/typed native map data, checks every
old map and program, exact catalogs/art/audio/spawns/recovery, the eight bounded
patches and all warp pairs. Eight unsafe sources and duplicate installation
are rejected. Log: `target/interior-life/shore-gate-preservation-2d-v04.log`.

The fresh production-controller test executable uses the normal **2D client
features**, without `voxel-view`. Both gate cases pass (52.51 seconds):

- Enter and exit both sides in both lanes; passive returns do not retrigger.
- Speak through the actual counter and hear the paddler; movement is rejected
  while dialogue owns input, pages advance, and input returns afterward.
- Open/close Start and Save/Continue at the passive arrivals.
- Retain money, bag and all Minnesota story flags.
- Cross both actual Grand Marais road lanes and both Cedar/R11 lanes in both
  directions through the production controller.

Their source is `crates/crystal-bevy/src/bevy_shell/tests/shore_gate.rs`.
The executed test filter was `shore_gate --ignored --nocapture`, with
`CRYSTAL_INTERIOR_PACK` set to the absolute v04 pack path and
`RUST_MIN_STACK=67108864`. The fresh executable was
`target/interior-life/wasm-dev/deps/crystal_bevy-8e2ff9039d737c59`.
Log: `target/interior-life/shore-gate-controller-2d-v04.log`.

The source review passes full-room/whole-approach geometry, native counter
footing, all arrival-to-exit/NPC paths, no walking/Surf bypass and four unsafe
counterexamples. Six unsafe registry variants are also rejected by
`planning/minnesota/tools/check_shore_gate_plan.py`.

Eight complete 640×576 **2D-only** production frames match the source compositor
with zero differing pixels across 2,949,120 pixels; no pixels are excluded.
Both frontage views, all four indoor arrival lanes, the counter and the
paddler were personally inspected. The complete source room/approach and every
native original, reference, difference and program snapshot remain under
`planning/minnesota/renders/interior-life/shore-gate/`, with the accepted native
sequence in `native-2d-v01`. Noon, position, south facing and an empty party are
explicit visual fixtures, not an earned campaign journey. No visible render
window was opened. Capture mode was `--view 2d` with headless image targets.
The private capture executable was removed and the pack hash stayed unchanged.

All 377 audio programs pass PCM hash, frame-count and loop-range verification.
Log: `target/interior-life/shore-gate-audio-2d-v04.log`.

The focused Rust leaves pass rustfmt. Whole-workspace formatting still reports
existing differences beginning in `offset-allocator`. Unfiltered all-target
Clippy stops at the existing core `bad_bit_mask` denial. The assets all-target
check passes after the known `bad_bit_mask`/`never_loop` baseline categories are
allowed; neither new gate leaf is named by a diagnostic. The combined assets/normal-2D-client all-target check also passes with only
those same two baseline categories allowed; no new gate leaf is named. Logs
are `shore-gate-clippy-2d-v01.log` (unfiltered failure), `-v02.log` (assets) and
`-v03.log` (combined), under `target/interior-life/`. The global plan checker
fails both before and after this registration at the existing assertion about
44 original recomposition rooms (`check_city_recomposition_plan.py:212`). The
focused gate registration check passes; the global check is not reported as
passing.

The [full scope](COVERAGE.md) remains open: at least 217 non-gym interiors,
including the missing Staples/Red Wing stations and three registered fishing
houses, canonical geography composition, train service and regional native 2D
room-quality review. This single gate does not establish whole-game acceptance.
