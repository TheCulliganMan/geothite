# City renderer follow-up

The subsequent native facade and rocky shoreline correction is recorded in
[CITY-COAST-FOLLOW-UP.md](CITY-COAST-FOLLOW-UP.md). The earlier captures below
retain their own executable and review history.

The repaired St. Paul houses and Pokémon Centers now have a headless 3D review,
including the river bends, both bridge approaches, depot quay and working basin.
The International Falls clinic and Mart also render as complete buildings, with
smaller snow-covered pines on their actual snow floor.

The pack remains `content-packs/minnesota-international-falls-v02.crystalpack`:
711 maps, 210 installed Minnesota non-gym interiors, file SHA-256
`9615a405a730e9f2bd6a56af3dc9b9ea618d9336f4525ce046cc42a81b5642cf`.
This renderer pass does not republish content or change gameplay state.

## Corrections

- Neighboring cities no longer exhaust a 16-object presentation namespace.
  Neighbor IDs reserve 16 bits for their object slot and remain disjoint from
  authoritative active-map IDs.
- The Falls art bank preserves the complete original Modern building bank.
  Both live rendering and streamed preparation now retain that native geometry
  identity while using the real colored texture. Snow remains its own surface.
- Snow canopies require a real Falls snow-floor sample. Their model uses warm
  pine bark and snow on upward-facing needles, at a smaller scale than Ilex's
  ancient broadleaf trees. Unknown floor art keeps its existing fallback.
- Bridge blocks `80` and `82` reuse native Kanto post drawings `1b` and `5f`.
  Projection checks all sixteen source glyphs against the supplied runtime pack
  before selecting their solid post models. Changed or incomplete drawings are
  refused. The live three-layer bridge source path remains required.
- The complete eight-block depot exceeded the five-block brick-building limit.
  The existing roof, story and entrance grammar now supports widths through
  eight blocks. Every source cell, door bay and window still must match.

## Retained visual evidence

`renders/interior-life/voxel-follow-up-v05/` contains 25 paired production views,
all personally inspected. Twenty passed local composition review. Five cameras
were rejected: two exposed the wrong snow-tree family; the bridge showed flat
post fragments; and two cameras showed the same flat depot. The original failed
St. Paul timeout remains in `voxel-follow-up-v01/`; no rejected frame was removed.

`renders/interior-life/voxel-follow-up-v06/` resolves those five cameras and adds
two city controls. All seven paired views were personally inspected. All seven
full 640×576 native 2D comparisons pass with zero differing pixels, including the
clinic's coherent native flower phase. Every pixel participates. GPU readbacks,
original PNGs, source references, differences, commands, executable/source hashes,
comparison scripts and review findings are retained there. Private executable
copies were removed and the pack hash remained unchanged.

The final capture executable SHA-256 is
`2a9423efb68959eb808e1e1615d21bb0f06dd7adf7a8d40008129729dd6e588f`.
Fixtures explicitly place an empty-party trainer facing south at hour 12; these
captures do not claim earned campaign progression.

## Verification

Twelve targeted regressions pass: real-pack source projection and unchanged
authoritative checksum, complete-drawing rejection, neighbor actor IDs, genuine
snow-backed canopy geometry, snow pigment with unchanged bark/mesh, native forest
behavior, whole native posts, and ordinary/stacked brick buildings through the
depot's 32×12 source drawing. The final log is
`target/interior-life/voxel-follow-up-final-regressions-v03.log`.

Build and test compilation used the pinned toolchain, `CARGO_BUILD_JOBS=4`,
`CARGO_TARGET_DIR=target/interior-life`, profile `wasm-dev`, and
`--config 'profile.wasm-dev.opt-level=1'`. The production example was rebuilt with
`cargo build --locked --profile wasm-dev -p crystal-bevy --features location-tester
--example render_at_location --config 'profile.wasm-dev.opt-level=1'`.
All captures used headless image targets at 640×576 with preview audio muted.

New Rust leaves pass formatting and the scoped diff passes whitespace checks.
Whole-workspace formatting still reports pre-existing differences beginning in
`offset-allocator`. Normal feature-enabled Clippy also encounters existing
approximate-constant denials in `scenery_finish.rs`, `foliage/pine.rs` and
`mesh/shore_ecology.rs`, in addition to the existing core `bad_bit_mask` and
`never_loop` denials. After those baseline categories were allowed, the all-target
check reached another existing `erasing_op` denial in
`mesh/enclosed_ponds.rs:463`. Both logs are retained as
`target/interior-life/voxel-follow-up-clippy-v01.log` and `-v02.log`;
no warnings identify the new source-projection, connected-actor, snow-ground or
snow-pine leaves. The unfiltered all-target lint check is not reported as passing.

These checks close the defects identified in these city/town cameras. The full
regional 3D audit, three missing stations, baseline shore-gate reconciliation and
six-stop Glacier Line remain open in [COVERAGE.md](COVERAGE.md).
