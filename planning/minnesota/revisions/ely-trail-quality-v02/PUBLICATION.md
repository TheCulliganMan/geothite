# Executed native publication

Run from the repository root with the completed ignored compact-coast parent,
reviewed native Minnesota sprite banks and current source files present. Each
stage below was executed successfully. Preserve these immutable outputs; use
fresh names for another publication. The first stage is the existing bounded
Ely/clinic Rust publisher; the remaining stages use the standard native species,
move and regional appearance publishers.

```sh
export RUST_MIN_STACK=33554432

target/runtime-fps/wasm-dev/build_ely_shore_pass \
  content-packs/minnesota-superior-compact-v03.crystalpack \
  planning/minnesota/revisions/ely-trail-quality-v02/source-v01/native.contract.json \
  3dde583414e13f81ab84d0b46baff667ac6581e0322bd3db0c63625f6ed57130 \
  content-packs/minnesota-ely-pine-knolls-geometry-v02.crystalpack

target/interior-life/wasm-dev/pack_fairy_type types \
  content-packs/minnesota-ely-pine-knolls-geometry-v02.crystalpack \
  content-packs/minnesota-ely-pine-knolls-types-v02.crystalpack

target/interior-life/wasm-dev/pack_minnesota_pokemon \
  content-packs/minnesota-ely-pine-knolls-types-v02.crystalpack \
  planning/minnesota/pokemon target/minnesota-pokemon/native \
  planning/minnesota/pokemon/species-profiles.json \
  content-packs/minnesota-ely-pine-knolls-species-v02.crystalpack

target/interior-life/wasm-dev/pack_northwoods_moves \
  content-packs/minnesota-ely-pine-knolls-species-v02.crystalpack \
  content-packs/minnesota-ely-pine-knolls-moves-v02.crystalpack

target/interior-life/wasm-dev/pack_fairy_type moves \
  content-packs/minnesota-ely-pine-knolls-moves-v02.crystalpack \
  content-packs/minnesota-ely-pine-knolls-fairy-v02.crystalpack \
  planning/minnesota/pokemon/fairy-moves.json

target/interior-life/wasm-dev/pack_northstar_regional_cast \
  content-packs/minnesota-ely-pine-knolls-fairy-v02.crystalpack \
  planning/minnesota/story-pokemon.json planning/minnesota/pokemon/roster.json \
  planning/minnesota/pokemon/roster-curation.json \
  planning/minnesota/pokemon/species-profiles.json \
  content-packs/minnesota-ely-pine-knolls-v02.crystalpack
```

Build the named Rust binaries with the pinned toolchain when needed; executable
paths reflect the already completed local builds consumed here. No Python
script publishes, serializes or patches a content pack.

The geometry audit explicitly takes the new contract, rather than accidentally
checking the old straight-road v01 contract:

```sh
PYTHONPATH="$PWD/.local/ely-backroute-python" python3 \
  planning/minnesota/revisions/ely-backroute-v01/verify.py \
  --pack content-packs/minnesota-ely-pine-knolls-geometry-v02.crystalpack \
  --epoch knolls-geometry-v02 \
  --contract planning/minnesota/revisions/ely-trail-quality-v02/source-v01/native.contract.json

PYTHONPATH="$PWD/.local/ely-backroute-python" python3 \
  planning/minnesota/revisions/ely-trail-quality-v02/audit.py \
  --pack content-packs/minnesota-ely-pine-knolls-v02.crystalpack \
  --geometry content-packs/minnesota-ely-pine-knolls-geometry-v02.crystalpack \
  --source planning/minnesota/revisions/ely-trail-quality-v02/source-v03 \
  --epoch native-v03

target/runtime-fps/wasm-dev/verify_crystalpack \
  content-packs/minnesota-ely-pine-knolls-v02.crystalpack --audio --desktop
```

All three commands passed. Native-v01 retains the earlier, longer tie-broken
zero-grass course for comparison; native-v02 binds the shortest zero-grass
return to the exact final pack SHA. The first geometry-audit attempt omitted
`--contract` and correctly rejected the different old-source preimage; its
failure log remains retained.

Logs: `.local/ely-trail-quality-{publication-v01,types,species,moves,fairy,regional}.log`,
`.local/ely-trail-quality-geometry-audit-v02.log`,
`.local/ely-trail-quality-native-audit-v02.log` and
`.local/ely-trail-quality-audio-v01.log`.

## Final production checks

The final fresh controller build used the pinned toolchain, the existing own
Cargo cache and a low-optimization override for the large test crate:

```sh
CARGO_TARGET_DIR=target/runtime-fps CARGO_BUILD_JOBS=3 RUST_MIN_STACK=33554432 \
cargo test --profile wasm-dev --locked -p crystal-bevy --lib --no-run \
  --config profile.wasm-dev.package.crystal-bevy.opt-level=0 \
  --config profile.wasm-dev.package.crystal-bevy.codegen-units=256

CRYSTAL_INTERIOR_PACK="$PWD/content-packs/minnesota-ely-pine-knolls-v02.crystalpack" \
CRYSTAL_ELY_KNOLL_PLAN="$PWD/planning/minnesota/renders/ely-trail-quality-v02/native-v03/input-plan.json" \
CRYSTAL_ELY_KNOLL_SAVES="$PWD/planning/minnesota/renders/ely-trail-quality-v02/reached-v06" \
RUST_MIN_STACK=33554432 \
target/runtime-fps/wasm-dev/deps/crystal_bevy-2219d470e331065d \
  bevy_shell::tests::ely_pine_knolls:: --ignored --nocapture --test-threads=1
```

Both tests passed in 4.78 seconds (`.local/ely-trail-quality-controller-v06.log`).
They use a deliberate Kindlit60/Birchit30/Pouchid30 fixture for the traversal
case, with Kindlit initially injured, burned and missing PP. A solo high-level
lead does not guarantee suppression under the preserved wild-challenge rules;
the two explicit reserves lower that fixture's wild-level anchor while leaving
all actual game rules intact. Repel is acquired in initial fixture setup and
consumed through the native Pack UI. The separate wild case has a healthy solo
Kindlit and no Repel, observes a native Minnesota opponent and uses native RUN.
No party or encounter state is rewritten after gameplay starts.

The other existing Ely-pass test also passed on the final pack, covering both
seam lanes and the original Ely north exit (`.local/ely-trail-quality-seams-v01.log`).

The production stdio replays were executed successfully:

```sh
python3 planning/minnesota/revisions/ely-trail-quality-v02/check_mcp.py \
  --pack content-packs/minnesota-ely-pine-knolls-v02.crystalpack \
  --save planning/minnesota/renders/ely-trail-quality-v02/reached-v04/center-arrival.crystalsave \
  --proof planning/minnesota/renders/ely-trail-quality-v02/native-v03/input-plan.json \
  --output planning/minnesota/renders/ely-trail-quality-v02/mcp-reached-v02

PYTHONPATH="$PWD/.local/ely-backroute-python" python3 \
  planning/minnesota/revisions/ely-trail-quality-v02/check_encounter_mcp.py \
  --pack content-packs/minnesota-ely-pine-knolls-v02.crystalpack \
  --save planning/minnesota/renders/ely-trail-quality-v02/mcp-reached-v02/live.crystalsave \
  --proof planning/minnesota/renders/ely-trail-quality-v02/native-v03/input-plan.json \
  --output planning/minnesota/renders/ely-trail-quality-v02/mcp-encounter-v03
```

Both exit with byte-empty stderr. The first proves clinic/Start, the entire dry
return, native westward jump, reverse refusal and both seams back into Ely.
The second observes Bandrake in the required grass at `(4,29)`; the earlier
native capture also observed Boomruff. These are actual scaled wild encounters.
Remaining Repel is not claimed exhausted: an equal/higher scaled wild may pass
it. The source tool correctly uses the production `wildbattle` phase name.

Final headless images load actual native-controller checkpoints without
position, facing or clock overrides. The ledge lip/landing and Ely pine-gate
images in `renders/ely-trail-quality-v02/native-reached-v02/` were all visually
inspected. The Center arrival is in `native-reached-v01/`. The warm daytime
fixture in `native-fixtures-v01/` is retained as an explicitly positioned visual
reference, separate from the reached night images. No final 3D visual acceptance
or earned regional progression is claimed.

The current source epoch is source-v03, with the same contract hash/terrain as
v01. Earlier input-course errors are retained rather than hidden: v01 labeled
northward steps Down; v02 fixed path tie-breaking; v03 fixes labels and the
native audit independently checks every input vector. The first encounter
fixture only turned in place; grouped inputs now execute walking. A solo-lead
Repel fixture could encounter equal-level scaled wilds; the final three-member
fixture accounts for this preserved rule. Hash/missing-proof attempts were
rejected before any route acceptance. All rejected logs/renders remain saved.

`cargo fmt --all -- --check` was executed and fails on the existing shared-tree
formatting differences, beginning in `crates/offset-allocator/src/lib.rs:408`.
The new Rust leaf passes rustfmt. Required `cargo clippy --locked -p crystal-bevy
--all-targets` (with the wasm-dev profile) was executed and fails on the existing
denied `clippy::bad_bit_mask` in `crates/crystal-core/src/multiplayer/mod.rs:1332`.
Logs: `.local/ely-trail-quality-fmt.log` and `.local/ely-trail-quality-clippy-v01.log`.
The shared unrelated tree was not reformatted or refactored.
