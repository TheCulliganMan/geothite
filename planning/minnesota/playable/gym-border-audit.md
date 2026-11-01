# Gym integration and connected-border collision — 2026-10-27

The gym integration audit found a real input defect on R05. Walking south from
`MnR05C00R00` at `(14..27,47)` entered `MnR05C00R01` inside its visible solid
forest. The connection code verified destination bounds but never sampled the
neighbor's collision. This was reproduced through `VisibleShellController::press`,
not inferred from a render or a floor strip.

`game_data_connections.rs` now checks the target map's native collision and
saved block edits before committing the off-map step. Walk/Surf and directional
walls retain their normal permissions. A normal initial turn keeps its usual
behavior; continuous input and forced steps check their actual landing. An ice
slide blocked at a connection ends its stored direction so the player can turn
again. Malformed saved edits return an error before indexing.

The source art, terrain and pack contents are unchanged. The visible forest
itself stops the player; no extra invisible fence or terrain patch was added.
The focused production regression covers every one of the fourteen affected
tiles. Four self-contained Rust tests cover all four map borders, saved open
and closed blocks, malformed edits, Surf, directional walls, normal turns,
continuous input, ice and directional walk tiles. The latter cases execute
native overworld input and require a blocked step, unchanged source position
and no connection transition.

The eight-gym integration suite now earns Saven's Wind Badge before trying
St Cloud's north departure. It approaches and returns through the actual
checkpoint scripts before planning across the guards. Both north lanes and
both south lanes retain reciprocal crossing checks. No extra badge, guard
visibility or player position is written to pass those checks.

## Executed results

- Four collision unit tests pass, including native grid/continuous/ice/walk input.
- The production R05 regression passes all fourteen blocked forest entries.
- All eight actual-facade gym courses pass together on the 541-map Leech shore
  pack (109.03 seconds). Required battles, puzzle progression, badge/TM rewards,
  Continue and exterior returns remain covered.
- All five lake-country cases pass on their exact chapter pack, including the
  continuous earned Wind/Lake/ranger round trip. All six Leech shore cases pass
  on its successor, including actual Pine loss, clinic recovery, retry and home
  travel. Review party, prior badges and initial position remain fixtures.
- The corrected Gym4 v2 candidate passes its three controller cases and actual
  stdio route/facade/Start/return on the rebuilt client, with byte-empty stderr.
- Both candidates pass all377 audio PCM, frame-count and loop-range checks.
  The refreshed Gym4 native views each match all1,024,000 source pixels.
- Changed Rust leaves pass rustfmt. Scoped all-target assets/Bevy Clippy passes
  with the existing `never_loop` allowance and no warning in the changed leaves.
  Workspace format still fails first in `offset-allocator/src/lib.rs:408`;
  the previously observed full-Clippy blocker is the unrelated always-false
  bit mask in `crystal-core/src/multiplayer/mod.rs:1332`.

## Reproduce

```sh
CARGO_TARGET_DIR=target/gym-interiors cargo \
  --config 'profile.wasm-dev.package.crystal-assets.opt-level=0' \
  test --locked --profile wasm-dev -p crystal-assets --lib connection_collision

export CRYSTAL_GYM_TEST_PACK="$PWD/content-packs/minnesota-northstar-leech-shore.crystalpack"
export CRYSTAL_WINONA_TEST_PACK="$CRYSTAL_GYM_TEST_PACK"
RUST_MIN_STACK=67108864 CARGO_TARGET_DIR=target/gym-controller \
cargo --config 'profile.wasm-dev.package.crystal-assets.opt-level=0' \
  --config 'profile.wasm-dev.package.crystal-runtime.opt-level=0' \
  --config 'profile.wasm-dev.package.crystal-bevy.opt-level=0' \
  test --locked --profile wasm-dev -p crystal-bevy --lib gym_transition_audit \
  -- --ignored --nocapture --test-threads=1

RUST_MIN_STACK=67108864 CARGO_TARGET_DIR=target/gym-controller \
cargo --config 'profile.wasm-dev.package.crystal-assets.opt-level=0' \
  --config 'profile.wasm-dev.package.crystal-runtime.opt-level=0' \
  --config 'profile.wasm-dev.package.crystal-bevy.opt-level=0' \
  test --locked --profile wasm-dev -p crystal-bevy --lib \
  minnesota_regional_gyms_controller_all_eight_actual_facades_and_local_courses \
  -- --ignored --nocapture --test-threads=1
```

The audited Leech shore candidate has internal content identity
`43b3d65bda72a8904cfad17781cc0da408045c64d91ee22fb5c6b3b1634006db`.
Its [chapter runbook](leech-shore.md) describes its exact publication lineage.
The [lake-country tests](lake-country-story.md) that assert closed future
Brainerd branches must use the earlier lake-country pack: Leech shore explicitly
opens R07. Running that old cap assertion on the successor correctly finds an
open route; it is not evidence of a broken cap in the earlier chapter.

These checks use explicit initial party, prior-badge and placement fixtures.
They verify the gym courses and legal road mechanics. They do not establish
ordinary-party balance or a complete earned starter-to-League campaign.
