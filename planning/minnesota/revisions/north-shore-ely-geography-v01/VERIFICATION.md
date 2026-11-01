# Superior coast and inland winter publication

The active correction is west-entry Grand Marais, a warm north–south North
Shore with an eastward terminal bend, a shared Superior shoreline, and a
separate northern inland winter journey. The older southern-quay proposal is
history and is not part of this publication.

## Actual Rust publications

| Stage | Ignored pack | File SHA256 | Content identity |
| --- | --- | --- | --- |
| Supplied parent | `content-packs/minnesota-superior-inland-art-v01.crystalpack` | `1c6ba5d59097546ff8b94e09c809d270727ce1272acb93759b89f3b3c7d9b90a` | `317e9c039914d27c83dfed01ef55b4720023867fd9d8d5aef9ea1b1a452aec3e` |
| Grand Marais | `content-packs/minnesota-grand-marais-superior-v01.crystalpack` | `71677fbbc2e4df293aff663195fedec37b7e8d184494b0f4c483d5da81a52b72` | `ed93d93577321048200d4ab549f549ab65c995e3a980a16895542223b61c9812` |
| Geography | `content-packs/minnesota-superior-inland-geography-v01.crystalpack` | `4126744fc23ff2d0d8c56f44528e1ccd6eee934d427cdef0a3fb81a7a50de928` | `bb8bc6f7ab863a2745e19aaff0e9e31dc242960fa4bbfb67fd4a7a019c28f578` |
| Inland winter | `content-packs/minnesota-superior-inland-winter-v01.crystalpack` | `65cf6f98e65ada839ec5afaa57728bf8289d3a50ba53ba6b9b50f08d3620bb17` | `f34280a5fc5d6ab8ccc3ec4ef4317a6ea9321070df342ea95aca5745514a0223` |

All three publishers completed strict written-pack readback. The final pack
contains 694 maps. The controller, audio, headless visual, canonical promotion
and Rust source reproduction gates below have all been completed for this
specific correction. This does not establish an earned statewide campaign.

The binaries were built with `--locked --profile wasm-dev`,
`CARGO_TARGET_DIR=target/runtime-fps`, `CARGO_BUILD_JOBS=1`,
`RUST_MIN_STACK=33554432`, and package opt-level=0 overrides for crystal-core,
crystal-assets, crystal-runtime and crystal-bevy. Only one agent owns Cargo and
publication. The replay commands, from the repository root, are:

```sh
target/runtime-fps/wasm-dev/build_grand_marais_superior \
  content-packs/minnesota-superior-inland-art-v01.crystalpack \
  1c6ba5d59097546ff8b94e09c809d270727ce1272acb93759b89f3b3c7d9b90a \
  planning/minnesota/revisions/north-shore-ely-geography-v01/grand-marais/source-v02/native-edit.contract.json \
  content-packs/minnesota-grand-marais-superior-v01.crystalpack
target/runtime-fps/wasm-dev/build_superior_inland_geography \
  content-packs/minnesota-grand-marais-superior-v01.crystalpack \
  planning/minnesota/revisions/north-shore-ely-geography-v01/geography/native-west-v02/contract.json \
  71677fbbc2e4df293aff663195fedec37b7e8d184494b0f4c483d5da81a52b72 \
  content-packs/minnesota-superior-inland-geography-v01.crystalpack
target/runtime-fps/wasm-dev/build_inland_winter \
  content-packs/minnesota-superior-inland-geography-v01.crystalpack \
  planning/minnesota/revisions/north-shore-ely-geography-v01/inland-winter-r12-r13/source-v02/native.review.contract.json \
  4126744fc23ff2d0d8c56f44528e1ccd6eee934d427cdef0a3fb81a7a50de928 \
  content-packs/minnesota-superior-inland-winter-v01.crystalpack
```

Outputs must be fresh; these commands intentionally reject overwriting packs.
The source parent and native art stay external/ignored. This is replay from
the identified parent, not a claim of a fresh statewide content export.

## Current evidence

- Actual final-pack source-art atlas:
  `renders/north-shore-ely-geography-v01/native-winter-connected-v01/connected-review.png`.
  The full 32px/metatile original and each map original are retained beside it.
- Its actual connection-derived graph reaches 101 exteriors with reciprocal
  offsets, no inconsistent cycles and no overlapping rectangles. It places
  Grand Marais at `[246,-400]`, Cedar at `[227,-419]` and the 65×19 connector at
  `[162,-419]`. No proposed terrain or invented gap-fill is composited.
- Final native collision audit:
  `renders/north-shore-ely-geography-v01/native-winter-connected-v01/surf-boundaries/audit.json`.
  Both audited gate states have zero unmatched water-boundary records in R11,
  Grand Marais, Cedar, the connector, R12, R13 and Ely. This is collision
  evidence, not an earned Surf/HM progression claim.
- Read-only canonical promotion checks passed against the exact final SHA for
  all three source groups. They checked native terrain, owned source hashes,
  town room preservation and reciprocal connections. All16 matching canonical
  files were then promoted in order: geography6, town/world/interiors6,
  winter4. Original proposal epochs remain unchanged. Source-promotion logs
  are retained under `native-publication-v01/` beside the executed test logs.
- All nine stage asset tests passed (210.28s): four Marais, three geography,
  two winter. They include exact terrain/mirror/registration checks, full
  residual preservation and deliberate unowned corruption rejection.
- The final composition audit retains all377 parent audio programs exactly,
  changes only the13 explicitly owned maps and adds only the inland connector.
  All680 other original map modules are exactly equal to the supplied parent.
  See `native-winter-connected-v01/composition-audit.json`. This comparison does
  not replace PCM synthesis verification.
- Actual `verify_crystalpack FINAL --audio --desktop` passed:694 maps,
  251 species,251 moves,377 audio programs; every PCM hash, frame count and
  loop range matches.
- All31 planned production headless views and seven reached-save views were
  captured and inspected. `VISUAL_REVIEW.md` records the exact directories,
  camera-framing differences and the retained green Cut pads.
- Sequential replays of all three Rust publishers to fresh ignored
  `*-replay-v01.crystalpack` outputs reproduced every stage file SHA exactly.
  The final replay is byte-for-byte identical to the published winter pack.
- The initial Marais asset execution found an inherited cached/uncached
  event-header mismatch. The bounded fix preserves only that transport-label
  omission while merging typed events; ordinary missing script keys still
  fail. All four Marais asset tests now pass.

## Completed controller and source gates

Ten new production controller cases passed on the exact final pack, across the
original run and focused repaired-fixture retries:

- Continuous R11 bend → west town → north Cedar → full connector → R12 and
  return through both walking lanes, including reached Save/Continue.
- Separate inland seam/connector return; both true one-way ledges with their
  ordinary returns; actual cold-grass encounter; Berry×3 once-only pickup
  across Continue and map reentry.
- Both west/north Foot gate lanes refuse missing badges or Duluth signal,
  permit qualified departure and permit unqualified inward nurse care.
- All20 physical Superior water lanes cross both ways; all21 checkpoint rows
  deny safely, Surf mode survives the inward wrapper and Continue, unmatched
  reefs block traversal and unqualified Surf arrival reaches actual healing.
- All six original room pairs return correctly, including real Field Station
  welcome/gift/nickname handling through controller input.
- All40 ICE quadrants produce their native forced slides and dry stops in
  both directions.

Six additional production controller cases passed: R12 and R13 full route/room
tours, both native Cut/cache denial/removal/reward/reset/Continue courses,
R13 keeper HP/status/PP care, and the R12 bog bay's actual Surf approach,
sign and trainer challenge. The optional bay now needs its existing water
approach; it is not described as a dry side passage through the ice.
A separate dry/ICE path-planner regression also passes. These are explicit
local party/badge/position fixtures followed by production input, not earned
regional progression. Reached saves were retained using the named save
directories and `CRYSTAL_MAP_RETAIN_SAVES_DIR`.

The actual Rust test
`superior_inland_canonical_rust_source_matches_every_published_terrain_cell`
passed1/0 in3.21s after promotion. It reads the six current canonical layouts,
parses the production Rust schema and rasterizes all13,231 terrain cells,
matching every native chunk and tileset in the final pack. Together with exact
source connection/native contracts and byte-identical publication replay,
this proves reproduction of this bounded correction from its identified
native-art parent. It is not a new full source export from pret.

The independent parent/final footprint comparison moved only Grand Marais and
Cedar, added only the connector, and retained all98 other existing exterior
origins. See `native-winter-connected-v01/footprint-preservation.json` under
the render directory.

All39 executed verification logs, including initial failures and successful
focused retries, are preserved in
`renders/north-shore-ely-geography-v01/native-publication-v01/logs/`.
The last room success is in `geothite-last-two-retests-20261027.log`, last R12
tour success in `geothite-r12-final-retest-20261027.log`, and final Rust parity
in `geothite-final-source-parity-20261027.log`. Earlier failed fixture runs
remain history; they are not presented as passing suites.

## Repository-wide check limits

Scoped rustfmt passes on all changed Rust leaves. Required
`cargo fmt --all -- --check` was run and reports existing unrelated shared-tree
differences, including `source_title_intro_scenes.rs`. Those files were not
reformatted as part of this correction.

The required normal assets and Bevy Clippy commands were run. Both are blocked
by the existing `crystal-core/src/multiplayer/mod.rs:1332` bad-bit-mask deny
lint. A diagnostic assets check then exposed existing `never_loop` errors in
`mutation_protocol.rs`/`script_parsing.rs`. Supplemental affected-crate
all-target checks with only those existing deny-lints allowed passed; introduced
helper warnings were fixed. These supplemental checks do not turn the normal
repository Clippy command green. No unrelated multiplayer/parser implementation
was changed to bypass the failures.
