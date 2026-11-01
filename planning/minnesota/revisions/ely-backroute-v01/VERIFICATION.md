# Ely pines, snowfall and Split Rock return: verified candidate

The ignored playable candidate is
`content-packs/minnesota-ely-snow-shore-pass-v01.crystalpack`.
Its SHA256 is `e6a17f18d5b19e5c2c6ef16aa6b2a85a75c894d89c8b49fc0a1eec08295c10d3`;
content identity is `c63a1c203ee44825494321a1a7e2c032b83c0e173cf30b150930f79b103c9c85`.
It contains 691 maps. The parent has 689; 687 original map modules are exact.
Original runtime files and audio bytes/manifest are exact. This is a child of
the compact Superior candidate, not the later superseded broad coastal branch.

## Executed source and Rust publication

Run from the repository root. The Python authoring/review tools require Pillow
and cbor2. They read the external parent and produce review contracts/images;
Rust performs content-pack publication. Source epochs and all render originals
are retained. Choose fresh epoch and output names for another review.

```sh
python3 -m pip install --target .local/ely-backroute-python cbor2
export PYTHONPATH="$PWD/.local/ely-backroute-python${PYTHONPATH:+:$PYTHONPATH}"
python3 planning/minnesota/revisions/ely-backroute-v01/prepare.py --epoch source-v12
```

The complete latest source loop took 5.70 seconds with the local read cache.
The accepted native contract remains `source-v08/native.contract.json` (v11/v12
add source verification only). The executed publisher command was:

```sh
RUST_MIN_STACK=33554432 target/runtime-fps/wasm-dev/build_ely_shore_pass \
  content-packs/minnesota-superior-compact-v03.crystalpack \
  planning/minnesota/revisions/ely-backroute-v01/source-v08/native.contract.json \
  c75cb89123ac57a4bbbbdb37fb6b80ec2d2acf5cde0443b117d73f0a98ffcd48 \
  content-packs/minnesota-ely-snow-shore-pass-v01.crystalpack
```

Build that executable with the pinned Rust toolchain using
`cargo build --locked -p crystal-assets --bin build_ely_shore_pass` when needed;
adjust its target/profile path to the chosen Cargo output directory. Publication
requires a fresh output path and checks the explicit contract SHA, original
parent identity, original cell values, paired seams and preservation. The R11
edits cannot overwrite water, cliffs, ice or gameplay gates. Repeating the
command with `minnesota-ely-snow-shore-pass-replay-v01.crystalpack` produced
the identical SHA256 above.

```sh
python3 planning/minnesota/revisions/ely-backroute-v01/verify.py \
  --pack content-packs/minnesota-ely-snow-shore-pass-v01.crystalpack \
  --epoch native-v01
```

This passed original-map/runtime/audio preservation, reciprocal connection
metadata, source patch preimages, complete native pine art parity and clinic
bindings. The receipt and all four whole-map renders are in
[`native-v01`](../../renders/ely-backroute-v01/native-v01/receipt.json).
The [connected review](../../renders/ely-backroute-v01/native-v01/connected-review.png)
uses real map origins and matching scale; it includes no invented connector.

## Production controller and renderer results

Two external-pack controller tests passed, in 16.37 seconds. They drive the
production `VisibleShellController`, covering both seam lanes, the existing
Ely north exit, the entire return walk, native Max Repel use, clinic entry and
both exits, nurse restoration of HP/PP/status, clinic recovery binding, PC,
Save/Continue and unchanged money/badges. The deliberate local traversal
fixture uses a level-60 Mew and an actual Max Repel, rather than claiming an
earned campaign journey.

The freshly built test executable was run with:

```sh
CRYSTAL_INTERIOR_PACK="$PWD/content-packs/minnesota-ely-snow-shore-pass-v01.crystalpack" \
CRYSTAL_ELY_PASS_SAVE_DIR="$PWD/planning/minnesota/renders/ely-backroute-v01/reached-v02" \
RUST_MIN_STACK=33554432 \
target/runtime-fps/wasm-dev/deps/crystal_bevy-26fe49fd0c52e9bc \
  bevy_shell::tests::ely_shore_pass:: --ignored --nocapture --test-threads=1
```

The same executable's `bevy_shell::ely_snow::tests::` filter passed: flakes
move, stay bounded and are confined to Ely's private winter exterior. The
headless renderer was freshly built with `location-tester`; final 2D captures
load the actual reached saves without overriding map, position or game clock.

```sh
CRYSTAL_RENDER_TEST_HEADLESS=1 CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image \
CRYSTAL_RENDER_TEST_WINDOW_SIZE=640x576 RUST_MIN_STACK=33554432 \
target/runtime-fps/wasm-dev/examples/render_at_location \
  --pack content-packs/minnesota-ely-snow-shore-pass-v01.crystalpack \
  --save planning/minnesota/renders/ely-backroute-v01/reached-v02/ely-east-snow.crystalsave \
  --view 2d \
  --screenshot planning/minnesota/renders/ely-backroute-v01/native-gpu-reached-v01/ely-pine-gate.png
```

The final images were personally inspected:

- [Snowy pine gate](../../renders/ely-backroute-v01/native-gpu-reached-v01/ely-pine-gate.png): dense native pines, continuous east lane and visible snowfall.
- [Coastal pass](../../renders/ely-backroute-v01/native-gpu-reached-v01/pass-coast.png): snowline to the west, grassy optional spur, warm coast to the east.
- [Split Rock Center](../../renders/ely-backroute-v01/native-gpu-reached-v01/split-rock-center.png): complete native facade, unobstructed door and forecourt below the stream.
- [Whole Ely](../../renders/ely-backroute-v01/native-v01/MnEly-whole.png): original lodge facade and town services retained within the snowy pine perimeter.

These are production native-art 2D captures. Final 3D visual acceptance remains
unverified; the 3D pine projection and Ely-only weather integration compiled.
No user visual acceptance is implied.

The native verifier's `--audio --desktop` run passed all 377 audio programs,
including PCM hashes, frame counts and loop ranges. Logs are retained under
`.local/ely-backroute-{publication-v04,native-verify-v01,audio-v01,controller-v02,snow-unit-v02}.log`.

## Repository check limits

The changed leaf Rust files pass rustfmt. The required whole-workspace
`cargo fmt --all -- --check` fails on existing unrelated formatting differences;
the enormous shared dirty tree was not reformatted. Required affected-crate
clippy was attempted and is blocked by the pre-existing denied
`clippy::bad_bit_mask` in `crates/crystal-core/src/multiplayer/mod.rs:1332`.
See `.local/ely-backroute-fmt.log` and `.local/ely-backroute-clippy.log`.
Focused gameplay, snowfall, preservation and audio checks passed independently.

The completed production `geothite mcp` client also loaded the reached pass
save, walked to the shore and back through the same seam, and opened the real
Start menu. It exited successfully with byte-empty stderr. The result is saved
in [mcp-receipt.json](../../renders/ely-backroute-v01/mcp-receipt.json).

After removing the temporary decoded-parent cache, source-v13 also completed
directly from the original external pack, with the same contract SHA256 and
all source paths passing. The initial system-Python retry lacked cbor2; the
local dependency command above resolved it. All prior render epochs remain saved.
