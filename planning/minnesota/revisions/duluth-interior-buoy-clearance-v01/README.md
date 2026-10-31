# Final Duluth interior buoy clearance

The selected v03 child keeps the [rocky shoreline and route fence revision](../shore-rocks-open-routes-v01/README.md) and clears one last interior buoy corner at
Duluth metatile `(16,33)`. This was native buoy38; it is now ordinary water35.
Every remaining buoy in Duluth and Routes10,12,15,16,17 lies on an exposed whole
map boundary. The19 tree-backed Route16 buoy cells remain cleared, and actual
northward Surf stops at the existing trees. No buoy is needed behind them.

The38 shore boulders, open harbor, two floating fishing platforms, route fence
removals, fitted R10 cave and both North Shore cafes are retained. This child
changes exactly one metatile; all731 other complete maps, every actor/script,
all catalog data, native art and audio remain exact to the verified parent.

The guarded Rust publisher now accepts `clear_inner_buoys: true` in its plan,
using `clear_duluth_inner_buoys` for this bounded operation. It checks the complete
native source preimage, unchanged art, exact buoy/water collision and exact
one-cell output, then verifies the written pack. The transformation leaves the
borrowed parent unchanged. The existing rock/route publisher mode remains unchanged.

The executed build uses the same pinned Cargo command as the parent runbook.
Publication was:

```sh
.local/route-grass-native-target/wasm-dev/revise_shoreline_and_fences \
  planning/minnesota/revisions/duluth-interior-buoy-clearance-v01/plan.json \
  content-packs/minnesota-current-shore-rocks-open-routes-v03.crystalpack
```

The verified pack SHA256 is
`22ddc24c06b8e15dd4211a902e118a50daa2fefecc2a6d592b0db14420bd5c22`, identity
`703a440963aa08b9126fb136b0524c1a881ba08cee750a1578927d66baf0a296`. Use a fresh ignored output name for a replay.
`world.json` owns the current local selection and source index.

Fresh production inputs launch Surf from legal shore tile `(31,65)`, enter the
two former blocking buoy quadrants, and open Start. All16 calls have empty stderr.
Two matched native before/after images were inspected, with zero changed pixels
outside the one edited metatile. A complete six-map buoy audit confirms all145
remaining buoy metatiles are at exposed outer bounds. Parent verification supplies
154 production calls,12 native captures, route geometry, fisherman boarding and
Continue, and377 verified audio programs. Audio manifests and compiled payloads
are byte-for-byte equal in this child, so the parent's PCM hashes/frame counts/
loop checks remain applicable. These are local QA fixtures, not earned campaign
progress or a browser deployment.

The new source passes all-targets crystal-assets Clippy with unrelated warnings,
and changed Rust formatting passes. Whole-workspace formatting has unrelated
pre-existing differences. Captures, saves, generated receipts and packs stay
ignored under `planning/minnesota/renders/duluth-interior-buoy-clearance-v01`.
