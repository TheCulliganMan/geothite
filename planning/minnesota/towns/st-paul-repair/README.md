# St. Paul facade and river repair

Current city-render work is **native 2D only**. The current whole-city and
production player-view checks are recorded in [2D-REVIEW.md](2D-REVIEW.md).

The selected 730-map
[interior/terrain union](../../interior-life/selected-union-v01/README.md)
preserves this complete repaired city. Its three fresh city controller cases
pass, and both clinics and the river bend were freshly captured and inspected
in native 2D with zero differing pixels. Red Wing remains excluded.

The current compact-lineage replay uses
[compact-contract-v01.json](compact-contract-v01.json), retaining the revised
regional geography and Minnesota fishing tables. Its verified 695-map output is
`content-packs/minnesota-compact-st-paul-repair-v01.crystalpack`; the build and
review history below describes the older interior branch. Current native 2D,
controller, preservation and audio receipts are in [2D-REVIEW.md](2D-REVIEW.md).

```sh
target/interior-life/wasm-dev/recompose_authored_city \
  content-packs/minnesota-compact-shore-gate-v01.crystalpack \
  planning/minnesota/towns/st-paul-repair/layout.json \
  planning/minnesota/towns/st-paul-repair/compact-contract-v01.json \
  content-packs/minnesota-compact-st-paul-repair-v01.crystalpack
```

That completed output already exists. Preserve it and choose a fresh output
name for another replay.

This revision answers the rejected city render. It replaces 17 malformed or
misleading facades: both clinics, the two shops, the bakery/cafe/inn and ten
mixed house assemblies. Ordinary homes use complete native Kanto houses or
brick townhouses. Clinics use the original two-block Pokémon Center frontage;
the old widened frontage left a dangling wall beside its sign. The bakery,
cafe and inn no longer carry Mart frontage.

The river has three broad bends, a western pine headland, a continuous southern
stone quay and a small working basin beside the boat co-op. Straight northern
banks use Kanto `1f`, convex corners use `1d/1e`, and concave joins use `2e/2d`.
Repeating `1e` along a straight bank was the source of the serrated shore.
Floating fence strips have been removed. Both existing bridge decks and their
Surf underpasses remain exact. The four eastern and four southern water lanes
keep their original reciprocal connections and offsets.

The explicit source is [layout.json](layout.json), with every native indexed
anchor in [contract.json](contract.json). [facades.json](facades.json) records
complete replacement assemblies and their authentic source references. Twenty
indexed warp positions move by one walking tile to meet the complete house's
actual door. Door indices, room identities, all interiors, services, scripts,
gyms, NPCs, items and audio stay intact. This same-extent revision is applied
after the earlier compact city and interior overlays; their frozen bootstrap
sources remain available for that build chain.

## Build

Executed from this checkout with Rust 1.99.0:

```sh
export CARGO_TARGET_DIR=target/interior-life
export CARGO_BUILD_JOBS=4
cargo build --locked --profile wasm-dev -p crystal-assets \
  --bin pack_ely_league --bin recompose_authored_city \
  --config 'profile.wasm-dev.opt-level=1'
target/interior-life/wasm-dev/pack_ely_league \
  content-packs/minnesota-interior-layout-v06.crystalpack \
  planning/minnesota/towns/MnEly.json \
  planning/minnesota/towns/MnEly.layout.json planning/minnesota/story.json \
  content-packs/minnesota-interiors-ely-v03.crystalpack
target/interior-life/wasm-dev/recompose_authored_city \
  content-packs/minnesota-interiors-ely-v03.crystalpack \
  planning/minnesota/towns/st-paul-repair/layout.json \
  planning/minnesota/towns/st-paul-repair/contract.json \
  content-packs/minnesota-st-paul-repair-v03.crystalpack
```

The v03 republication carries the v06 interior layouts and protected canonical
story text through Ely. Its St. Paul exterior and all indexed building/river bindings are
identical to the original repaired v01.

The completed output is an ignored 693-map pack with 251 species, 251 moves and
377 unchanged audio programs. The Rust publisher writes and rereads it through
the strict verifier. Use a new save for its pack identity. Regeneration should
choose fresh output names; keep the completed packs and all render history.

An earlier composed child is
`content-packs/minnesota-international-falls-v02.crystalpack` (711 maps). Its complete
St. Paul module equals the reviewed fair/ice v01 module exactly. The
[harbor-care runbook](../../interior-life/HARBOR-CARE.md) records the parent
lineage, followed by the [Detroit publication](../../interior-life/DETROIT-LAKES.md)
[Moorhead](../../interior-life/MOORHEAD.md), and [International Falls](../../interior-life/INTERNATIONAL-FALLS.md). The child retains every repaired facade and the compact fair approach.

Fresh review of the unchanged 699-map care parent passes fourteen complete native
frames, with zero differing pixels in every frame. The Detroit child preserves
that entire city module exactly. All three city controller cases also
pass on the 705-map child (21.51 seconds): 39 ordinary doors/returns, both clinic
services and PCs, both bridge walks, both Surf underpasses, reciprocal river
seams and Save/Continue. The four applicable asset cases pass on the rebased
Ely parent (11.05 seconds). All 377 audio programs pass on the final child.
The current [complete city](../../renders/st-paul/repair-current-v03-native/MnStPaul-whole-native-source.png)
and [native review pages](../../renders/st-paul/repair-current-v03-native/native-review-page-1.png)
([waterfront](../../renders/st-paul/repair-current-v03-native/native-review-page-2.png),
[downstream](../../renders/st-paul/repair-current-v03-native/native-review-page-3.png))
have been personally inspected. The earlier 23 camera passes remain applicable
because the city module is unchanged. The historical off-map rejection below
is retained; a fresh native two-hop context comparison now passes independently.

## Review and verification

The exact module preimage and every actor, sign, coordinate event, warp and
outdoor spawn must match before publication. Same-size refinement reuses the
production movement graph, door/return, ledge, recovery and perimeter validators.
Retained water-only connections have their own declared native Water lanes;
both directions must project onto the exact partner tile. Missing river
contracts, stale preimages, false lanes, blocked entrances and unsafe arrivals
are rejected.

The preservation regression passes. It checks every interior and other map,
source art/runtime files, all original programs, numeric identities, catalog,
items, recovery data, audio and report fields. Ely's two independent
preservation/native-reconstruction regressions also pass on the preceding
interior parent. No Fair or ice overlay is required for this city repair.

```sh
export RUST_MIN_STACK=67108864
export CRYSTAL_CITY_RECOMPOSITION_BASE="$PWD/content-packs/minnesota-interiors-ely-v03.crystalpack"
export CRYSTAL_CITY_RECOMPOSITION_LAYOUT="$PWD/planning/minnesota/towns/st-paul-repair/layout.json"
export CRYSTAL_CITY_RECOMPOSITION_CONTRACT="$PWD/planning/minnesota/towns/st-paul-repair/contract.json"
cargo test --locked --profile wasm-dev -p crystal-assets --features test-fixtures \
  --test city_recomposition --config 'profile.wasm-dev.opt-level=1' -- --ignored \
  --skip city_recomposition_rejects_false_water_seams_and_neighbor_drift \
  --skip city_recomposition_rejects_incomplete_or_impassable_water_gate
export CRYSTAL_INTERIOR_PACK="$PWD/content-packs/minnesota-st-paul-repair-v03.crystalpack"
cargo test --locked --profile wasm-dev -p crystal-bevy --lib st_paul_refinement \
  --config 'profile.wasm-dev.opt-level=1' -- --ignored --test-threads=1
```

The two skipped asset cases require the separate Minneapolis Falls recipe and
neighbor-terrain revisions. The retained-water test covers this city's existing
river connections without altering its neighbors.

All source studies, rejected passes, untouched GPU frames and source comparisons
are retained under ignored `planning/minnesota/renders/st-paul/repair-*`.
The source/native camera fixtures are in [views.json](views.json). Native review
uses `review_city_native.py` with a completed `render_at_location` executable,
headless image capture and one consistent native water animation phase per
frame. Placement, a healthy/Surf partner, prior optional trainer wins and a Surf
badge are explicit test setup; these checks do not claim earned campaign
progression.

The four applicable asset regressions pass (15.70 seconds), including the new
retained-river rejection cases. All three production controller cases pass:
39 ordinary entrances/returns, both walking bridges and city Continue; both
clinics' free HP/status/PP care and native PC input ownership; and Surf under
both bridges, Continue beneath a bridge, and both river seams in both directions.
The first two cases took 41.36 seconds together; the Surf case took 8.07 seconds.
The initial Repel fixture was corrected to include its active item identity
before the successful saved-state run.

Twenty-three full native camera comparisons pass exactly and have been visually
inspected, including every replacement facade and the river landmarks. The
completed pack's whole source image also matches the authored proposal exactly.
[Whole repaired city](../../renders/st-paul/repair-final-review/st-paul-repaired-whole.png),
[six native landmark views](../../renders/st-paul/repair-final-review/native-camera-review.png),
and [all facade camera pages](../../renders/st-paul/repair-final-review/facade-views-1.png)
([second page](../../renders/st-paul/repair-final-review/facade-views-2.png))
are retained at their original resolutions.

The old camera at `(7,64)` exposed 15,081 differing pixels below the active
city. Its reference supplied a repeated border but omitted real scenery from
`MnMississippiConfluenceC02R00`, reached through the southern connection's second
hop and beginning at source `(128,1088)`. The production 2D compositor already
includes two connection hops. This was a comparison omission, not missing terrain.

The connected review helper now supports an explicit two-hop source contract,
with native breadth-first placement, reciprocal deduplication and farther-map
precedence. Water animation respects those same authored rectangles. Every
640×576 pixel still participates; no off-map strip is excluded. Two fresh
headless captures at `(7,64)` and the `(7,60)` control pass with zero mismatches
on the 708-map Moorhead child and were personally inspected. Their explicit
fixture is [context-views.json](context-views.json), with original GPU frames,
raw descriptors, source references and differences retained under
`renders/st-paul/context-v02-native/`. The first context attempt rejected an
overlap outside the visible frame; that raw capture is retained in `context-v01-native/`.

The original rejected GPU frame and every comparison iteration also remain
retained. The historical 23-pass count is unchanged; these two current context
passes are separate evidence for the corrected comparison.

Changed Rust leaf files pass formatting. Whole-workspace `cargo fmt --all --
--check` still reports existing formatting in `offset-allocator` and other
unrelated files. Normal all-target Clippy for both affected crates stops at the
existing `bad_bit_mask` denial in `crystal-core/src/multiplayer/mod.rs:1332`.
The library/test Clippy check with only the two existing `bad_bit_mask` and
`never_loop` denials allowed completes successfully; it reports existing
warnings elsewhere and none in the new/refined city modules or controller case.
The rejected border frame's difference bounds are `(368,480)..(640,576)`;
all pixels within the active city extent matched even in that old comparison.
The fresh context evidence above resolves the complete frame without excluding
that area.

## Historical 3D follow-up

The current user direction is native 2D only. This earlier renderer work is
retained as history and does not define the current review.

The [renderer follow-up](../../interior-life/RENDERER-FOLLOW-UP.md) records 25
personally inspected paired city/town views and seven final corrected/control
views. Both clinics and the ordinary houses are complete models. Native bridge
post provenance now removes the flat rail fragments, and the eight-block depot
uses the existing complete brick-building producer instead of falling back to
source art. All seven final native 2D comparisons and twelve targeted regressions
pass. The original timeout and every visual rejection remain retained; broader
regional acceptance remains open.
