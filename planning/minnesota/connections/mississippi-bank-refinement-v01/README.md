# Mississippi banks, fruit and inland ice: compact delivery

The completed local delivery is
`content-packs/minnesota-fruit-ice-compact-v02.crystalpack` (691 maps).
Its parent is the current compact R10 gallery descendant; no retired northern
footprint returns. All eight North Shore modules remain warm and contain no
ICE. The shared world pointer is reserved for the integration owner and is
not changed by this bounded delivery.

Each winding wooded river has twenty trainers and ten berry trees. There are
29 grass patches (12 lower,17 upstream), with 952/792 grass gameplay tiles,
six once-only supplies and two medics. Regional harvests total 211 trees on 86
outdoor chunks: 138 berries and 73 apricorns. Both routes retain their full
64x68 extent, quiet wooded stretches, original water course, dry launches,
dismounts, crossings, connections and existing HM gates.

`revision.json` hand-places eight uneven landward grass recesses at the lower
north meadow, last portage, pine camp, eagle bank, upstream lookout, warbler
walk, gravel point and western reeds. Forty solid-canopy metatiles become
native grass:160 additional gameplay tiles. No original walkable, water,
object or seam tile changes. The public Rust publisher accepts only the two
wooded reaches, distinct interior cells, exact native preimages and original
canopy-to-grass/Floor replacements. It preserves every other data field,
report, native art file and audio payload. Current northern geometry and a
warm North Shore are mandatory in all three related publishers.

**Blocking integration finding:** a hash-only rebase on the newer regional
parent leaves all 160 added grass tiles unreachable. See
[INTEGRATION-BLOCKER.md](INTEGRATION-BLOCKER.md). The publisher now enforces
actual bank access as well as cell preimages.

**Integration dependency verified against R10:** its river modules have only
2 trainers per reach, and all seven bank-patch map preimages differ. Applying
`refine_mississippi_banks` directly to an R10 descendant that has not installed
the completed river/fruit stage must fail. It cannot independently provide the
20-trainer layouts or their existing grass and activities.

For a newer composed parent with revised Duluth/swamp/route shapes, use the
existing portable authoring path first:

```sh
.local/minnesota-review-env/bin/python planning/minnesota/tools/author_fruit_groves.py \
  --base content-packs/NEW_COMPOSED_PARENT.crystalpack \
  --output planning/minnesota/fruit-groves/NEW_SOURCE_EPOCH.json
target/river-fun/wasm-dev/pack_minnesota_fruit_groves \
  content-packs/NEW_COMPOSED_PARENT.crystalpack \
  planning/minnesota/fruit-groves/NEW_SOURCE_EPOCH.json \
  content-packs/NEW_FRUIT_OUTPUT.crystalpack
```

These names are integration placeholders, not a claim of an executed newer
composition. Authoring regenerates tree placements against the actual new
shapes and records exact source preimages. Review that fresh output with
`review_fruit_groves.py --source NEW_SOURCE --written NEW_FRUIT_OUTPUT --output
FRESH_REVIEW_DIRECTORY`, then apply this bounded bank patch. Do not disable
its preimage guard or copy an older whole candidate over the newer parent.

Compose in this order; all outputs must be fresh:

```sh
# First publish the current fruit contract per ../../fruit-groves/README.md.
CARGO_TARGET_DIR=target/river-fun cargo build --locked --profile wasm-dev \
  -p crystal-assets --bin refine_mississippi_banks --bin pack_frozen_water \
  --config profile.wasm-dev.package.crystal-core.opt-level=0 \
  --config profile.wasm-dev.package.crystal-assets.opt-level=0
target/river-fun/wasm-dev/refine_mississippi_banks \
  content-packs/minnesota-fruit-compact-v01.crystalpack \
  planning/minnesota/connections/mississippi-bank-refinement-v01/revision.json \
  content-packs/minnesota-fruit-bank-compact-v01.crystalpack
# Then use ../../regions/ice/FROZEN-WATER.md for the final inland ice overlay.
```

The bank publisher deliberately validates per-map preimages rather than the
whole parent identity, so the integration owner can compose it after unrelated
map revisions without changing source. Fruit installation has an exact parent
identity and recorded placements; rebasing onto a newer shared descendant
requires regenerating/reviewing that source on the selected canonical parent.
Do not copy the final candidate over another author's newer composition.

Final SHA256: `344775b2a985e21b2d10be8a5d4737dd704feb286846c132ceafe78127357331`.
Content identity: `b27de81ba40675ab2e7e336899920dcdbb451627dcb0740af0d3ad540ce77b8d`.
The Rust CLIs strictly verify inputs, in-memory output and written readback.
All old render epochs remain. Seven obsolete owned packs are quarantined as
`.retired` files with a SHA inventory under `.local/retired-map-candidates/`.

Verification receipts and native views:

- `renders/fruit-compact-v01/refined-banks-v01/` contains both whole routes,
  regional examples, clear tree rings, active Minnesota river teams/encounters,
  preserved original launches/water/boundaries, reachable activities, required
  supply grass and all 44 native input courses bound to the final pack.
- The final native controller run passes all 36 added trainer battles, all six
  pickups, both medics and wild encounters, including actual Surf launch and
  dismount, PP/XP/money, repeated interaction and Continue. Fixture partner is
  active Minnesota PEPIVERN, level 80, with legal Surf and declared initial
  badges/location. Log: `.local/river-compact-native-activities-v02.log`.
- All 16 berry/apricorn varieties pass native harvest, same-day retry,
  Save/Continue, production daily regrowth and full-pocket rejection on the
  final pack. All 211 placements have collision/access/ring checks. This pass
  tests one representative per item type, not 211 native harvests. Log:
  `.local/fruit-compact-native-v02.log`.
- The four native inland ice courses, seam, solid rock attempts and Continue
  pass. Written source-art recipes match native pixels. North Shore stays warm;
  every snowy exterior is free of liquid. See the frozen-water runbook.
- The final outdoor atlas has 97 maps, one connected component and zero
  footprint overlaps. Receipt: `renders/fruit-compact-v01/atlas-v02/native-connection-audit.json`.
  Its 94 exposed water lanes are inherited unrelated terrain, not a claim that
  every other map's Surf boundary is newly repaired by this stage.
- `renders/fruit-compact-v01/delivery-residual-v02.json` checks the exact bank
  inverse, unchanged warm coast, snowy liquids and exact audio. Source and
  actual native2D whole/player views were inspected, including quiet reaches,
  both revised grassy banks, and the unfrozen shore.
- The actual final pack passes stdio MCP movement, the production Start menu,
  the Minnesota home warp/Mom dialogue and byte-empty stderr. Receipt:
  `renders/frozen-compact-v02/mcp-v01/receipt.json`.
- Scoped Rust formatting and Python compilation pass. The asset regression
  checks current composition and rejects stale identity, boundary edits,
  duplicate river maps and actual oversized northern geometry. The focused
  test binary links the completed publisher library to avoid rebuilding all
  concurrent shared binaries; log `.local/river-compact-assets-direct-test-v04.log`.
- All-targets clippy was run. It stops in existing
  `crystal-core/src/multiplayer/mod.rs:1332` on `clippy::bad_bit_mask`.
  No source outside this task was changed to suppress that lint. Workspace
  formatting also reports existing differences beginning in
  `crates/offset-allocator/src/lib.rs:408`; all changed leaf files pass their
  scoped formatting checks.

These are explicit local controller/render fixtures, not earned campaign or
full 3D acceptance. The original full regional journey was verified on the
historical river-only pack; this rebase verifies owned launches/water and
activities and does not claim a new complete campaign/bridge itinerary run.
