# Executed park publication

Run from the repository root. Use fresh ignored output names for reruns and
retain every previous pack/render/save. Source and native publication are
separate; no shared source exporter or external build workspace is touched.

```sh
CARGO_TARGET_DIR=target/runtime-fps CARGO_BUILD_JOBS=3 RUST_MIN_STACK=67108864 \
cargo build --profile wasm-dev --locked -p crystal-assets \
  --bin pack_mille_lacs_park \
  --config profile.wasm-dev.package.crystal-assets.opt-level=0

RUST_MIN_STACK=67108864 target/runtime-fps/wasm-dev/pack_mille_lacs_park \
  content-packs/minnesota-current-grass-r09-v01.crystalpack \
  planning/minnesota/towns/mille-lacs-park-v01/town.json \
  planning/minnesota/towns/mille-lacs-park-v01/layout.json \
  planning/minnesota/towns/mille-lacs-park-v01/native.contract.json \
  content-packs/minnesota-mille-lacs-park-v04.crystalpack
```

The final native strict readback passes. Source/parent/module hash guards,
unused map group allocation, complete original-data preservation, actor/door/
edge collision, real connection projections, native ledges, dock rails and
Surf-only island validation run inside the Rust publisher. Fresh
`minnesota-mille-lacs-park-replay-v04.crystalpack` has identical bytes.

## Source, atlas, native pixels and audio

```sh
export PYTHONPATH="$PWD/.local/ely-backroute-python${PYTHONPATH:+:$PYTHONPATH}"
python3 planning/minnesota/towns/mille-lacs-park-v01/review.py \
  --pack content-packs/minnesota-mille-lacs-park-v04.crystalpack --published \
  --output planning/minnesota/renders/mille-lacs-park-v01/published-v06
python3 planning/minnesota/tools/review_connected_route_native.py \
  --pack content-packs/minnesota-mille-lacs-park-v04.crystalpack \
  --binary target/runtime-fps/wasm-dev/examples/render_at_location \
  --views planning/minnesota/towns/mille-lacs-park-v01/views.json \
  --output planning/minnesota/renders/mille-lacs-park-v01/native-v04
python3 planning/minnesota/tools/audit_outdoor_connections.py \
  --pack content-packs/minnesota-mille-lacs-park-v04.crystalpack \
  --output planning/minnesota/renders/mille-lacs-park-v01/atlas-v04 --scale 8
target/runtime-fps/wasm-dev/verify_crystalpack \
  content-packs/minnesota-mille-lacs-park-v04.crystalpack --audio --desktop
```

All pass. The independent complete decoded-pack comparison admits exactly 39
declared change paths, including additions for the one map and its registry.
No original data outside R18's connection fields changes. Native frames compare
2,580,480 pixels with zero differences. Audio verifies all 377 programs. The old
native roost camera on rock remains retained; the accepted camera uses actual
Floor(37,29). No production pixels are edited to hide a mismatch.

## Assets and production controller

```sh
CRYSTAL_MILLE_LACS_PARK_BASE="$PWD/content-packs/minnesota-current-grass-r09-v01.crystalpack" \
CARGO_TARGET_DIR=target/runtime-fps CARGO_BUILD_JOBS=3 RUST_MIN_STACK=67108864 \
cargo test --profile wasm-dev --locked -p crystal-assets --lib \
  mille_lacs_park::tests \
  --config profile.wasm-dev.package.crystal-assets.opt-level=0 \
  -- --ignored --nocapture --test-threads=1

CARGO_TARGET_DIR=target/runtime-fps CARGO_BUILD_JOBS=3 RUST_MIN_STACK=67108864 \
cargo test --profile wasm-dev --locked -p crystal-bevy --lib --no-run \
  --config profile.wasm-dev.package.crystal-assets.opt-level=0 \
  --config profile.wasm-dev.package.crystal-bevy.opt-level=0 \
  --config profile.wasm-dev.package.crystal-bevy.codegen-units=256

CRYSTAL_INTERIOR_PACK="$PWD/content-packs/minnesota-mille-lacs-park-v04.crystalpack" \
CRYSTAL_MILLE_LACS_PARK_SAVES="$PWD/planning/minnesota/renders/mille-lacs-park-v01/reached-v05" \
RUST_MIN_STACK=67108864 \
target/runtime-fps/wasm-dev/deps/crystal_bevy-01787b9da148a652 \
  bevy_shell::tests::mille_lacs_park:: --ignored --nocapture --test-threads=1
```

Two asset cases pass in 11.22 seconds; three controller cases pass in 10.84 seconds.
The final test build takes 1m45s. Read Cargo's reported executable path when
rebuilding. Local party/access setup is explicitly documented in the tests;
all later movement, dialogue, battles, modes, rewards and Continue use native
controller input. No saved progression is manufactured after setup.

## Actual stdio and reached views

```sh
PYTHONPATH="$PWD/.local/ely-backroute-python" python3 \
  planning/minnesota/towns/mille-lacs-park-v01/check_mcp.py \
  --pack content-packs/minnesota-mille-lacs-park-v04.crystalpack \
  --saves planning/minnesota/renders/mille-lacs-park-v01/reached-v05 \
  --output planning/minnesota/renders/mille-lacs-park-v01/mcp-v04

CRYSTAL_RENDER_TEST_HEADLESS=1 CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image \
CRYSTAL_RENDER_TEST_WINDOW_SIZE=640x576 RUST_MIN_STACK=67108864 \
target/runtime-fps/wasm-dev/examples/render_at_location \
  --pack content-packs/minnesota-mille-lacs-park-v04.crystalpack \
  --save planning/minnesota/renders/mille-lacs-park-v01/reached-v05/roost-collected.crystalsave \
  --view 2d \
  --screenshot planning/minnesota/renders/mille-lacs-park-v01/native-reached-v05/roost-collected.png
```

Four stdio processes exit 0 with byte-empty stderr. Reached captures load real
same-pack checkpoint state without overriding position, time or facing. Ledge
landing, collected roost and after-wild views are retained. Whole-source,
player-camera and actual reached visual review are separate evidence.

## Repository checks

Changed leaves pass rustfmt. Workspace `cargo fmt --all -- --check` still has
unrelated existing differences. Normal all-target Clippy stops at the existing
`crystal-core/src/multiplayer/mod.rs:1332` bad-bit-mask error. The supplemental
all-target Assets/Bevy run uses only the existing `bad_bit_mask` and `never_loop`
allowances and passes in 2m06s. No warning names the new park leaves. Its result
is bound in the final verification receipt. No
unrelated formatting or lint cleanup is included.

Logs are preserved as `.local/mille-lacs-park-*.log`. Old failed publication,
review and controller epochs remain historical evidence; only the exact final
pack/hash and successful result bindings establish acceptance.

## Acceptance and future composition

The completed proof is
`renders/mille-lacs-park-v01/verified-v04/receipt.json`.
The selection update verifies the exact current R09 parent and full world
preimage immediately before atomic replacement. Every prior sibling revision
and audit record is retained; only the actual park entry, real road contract
and explicit selector/revision record change. The canonical town descriptor
points to these single authoritative actor/layout inputs.

The original publication commands above remain bound to their immutable R09
parent. For a future revision, inspect the then-selected descendant and create
a fresh exact-parent contract instead of dropping subsequently accepted terrain.
Packs, renders, saves and build products stay ignored and are not staged.
