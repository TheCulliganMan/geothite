# Native city facades and rocky shoreline

The flat-roof Kanto Pokémon Centers and Marts now use the existing native
masonry grammar, including their slate roof, window courses, real door bay and
POKÉ/MART sign. An unrecognized service drawing stays on its source presentation;
it cannot select the generic red barrel-roof clinic. The complete native
service sign and door glyphs are required before modeling.

Kanto shoreline caps now have low fractured bedrock within their original
cells. All outer contacts remain at the original ground height. Their live
source pigment is muted into mineral colors, with the existing cliff grain.
The adjacent-ground continuation preserves raised mineral facets, including
steep facets, rather than repainting them as dirt. Water, walking/Surf support,
map geometry, native scripts and pack contents are unchanged.

The production capture pack is
`content-packs/minnesota-international-falls-v02.crystalpack`, SHA-256
`9615a405a730e9f2bd6a56af3dc9b9ea618d9336f4525ce046cc42a81b5642cf`. The final capture executable SHA-256 is
`a1788a87cf5b58baa5327b1f85fb2171bc25a7d43e4daac958f7048604697f82`. Fixture positions, south facing, empty party and hour 12
are explicit review setup, not earned campaign progression.

All native frames, raw GPU descriptors, original PNGs, source references,
conversions, differences and script snapshots remain ignored under
`planning/minnesota/renders/interior-life/`. Earlier passes are retained:
`city-coast-v01` still inherited dirt; `city-coast-v02` was rejected for yellow
regular facets; `city-coast-v03` improved composition but a new regression
caught steep facets still being repainted. `city-coast-mart-v01` retains the
first off-center Mart inspection. The current sequence is `city-coast-v04`.

Builds use the pinned toolchain, `CARGO_TARGET_DIR=target/interior-life`,
`CARGO_BUILD_JOBS=4`, `--profile wasm-dev`, and
`--config 'profile.wasm-dev.opt-level=1'`. The final production rebuild was:

```sh
CARGO_TARGET_DIR=target/interior-life CARGO_BUILD_JOBS=4 cargo build --locked \
  --profile wasm-dev -p crystal-bevy --features location-tester \
  --example render_at_location --config 'profile.wasm-dev.opt-level=1'
```

Captures use `CRYSTAL_RENDER_TEST_HEADLESS=1`,
`CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image`,
`CRYSTAL_RENDER_TEST_WINDOW_SIZE=640x576`, and `CRYSTAL_PREVIEW_MUTE=1`.
No visible window was opened. This pass does not establish full regional 3D
or interior acceptance; those obligations remain in [COVERAGE.md](COVERAGE.md).

The final eleven paired views were personally inspected: both clinics,
river bend, bridge, depot quay, south quay, basin cottage, fair cottages,
garden house, transit house and centered Mart. Their complete 640×576 native
2D frames match the exact source compositor with zero differing pixels across
all 4,055,040 pixels. Consistent native water/flower animation phases are selected;
no comparison pixel is excluded. `city-coast-v04/reference-comparison.json`
and `visual-review.json` retain the per-view evidence. The pack hash stayed
unchanged and the private capture executable was removed.

The 25 focused renderer regressions pass: native cap contacts/winding/scrolling,
five shoreline cases including steep-facet pigment preservation, 13 shared
brick-building cases, five house cases and the source-sign rejection case.
Logs are `target/interior-life/city-coast-final2-*.log`.

The production live-compositor regression also passes on the same Falls pack
(`city-coast-production-source-tests-v03.log`, 119.85s). It prepares the actual
three-layer city bridge scenes at three camera positions, checks real native
Center/Mart and shoreline model ownership, verifies fixed bank support, and
requires the authoritative checksum to remain unchanged across mesh building.
This brings the final targeted total to 26 passing tests.

`cargo fmt --all -- --check` still reports existing workspace differences
beginning in `offset-allocator`. Both new Rust leaves pass rustfmt and the
scoped diff passes whitespace checks. Required unfiltered all-target Clippy
still encounters existing `bad_bit_mask` and `approx_constant` denials in core
and scenery code. The affected-crate all-target check then passed with the
four previously documented baseline categories allowed (`bad_bit_mask`,
`never_loop`, `approx_constant`, `erasing_op`); no diagnostic names either new
leaf. Logs are `city-coast-clippy-v01.log` and `-v02.log`. The unfiltered lint
check is not reported as passing.
