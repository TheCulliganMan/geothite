# North Shore smokehouse and pie shop

Authoritative source: [`../north-shore-shops.json`](../north-shore-shops.json).
The shops use the selected compact, warm, native 2D North Shore parent named in
that file. The publisher rejects another parent identity and changed site cells.

- **North Shore Pies:** `MnR11C00R01`, door `(31,35)`, dry return `(31,36)`,
  sign `(36,36)`. Blue native roof and complete brick facade; a four-metatile
  square cafe with a baker, visitor, counter and tables.
- **North Shore Smokehouse:** `MnR11C00R03`, door `(35,7)`, dry return `(35,8)`,
  sign `(36,10)`. Native wooden cabin; a five-by-four cafe with a smoker,
  visitor, counter and tables. The original road sign behind it is preserved.

Coordinates above are local 16px walking tiles. Both interiors have active
carpet exits `(2,7)` and `(3,7)`, and passive arrival record 3 at `(3,6)`.
Exterior records 1/2 pair each door with its safe porch. Native map group 41
reserves room IDs 11 and 12 for the pie shop and smokehouse, respectively.
Neither business grants items or a healing service.

Reachable grass surrounds the sides and backs: 56 walking tiles at the pie
shop and 36 at the smokehouse. The precise rectangular habitat bounds are
recorded in the source. These are native `TALL_GRASS` tiles using the unchanged
regional encounter tables. Specific future species/spawn additions remain
unassigned. Roads, existing trainers and their approaches remain in place.

## Build and review

Use the parent listed in the source; packs and renders remain ignored. The
Rust publisher writes a fresh output and verifies its readback:

```sh
CARGO_TARGET_DIR=target/north-shore-shops CARGO_INCREMENTAL=0 \
  cargo build --locked -p crystal-assets --bin pack_north_shore_shops
target/north-shore-shops/debug/pack_north_shore_shops \
  content-packs/minnesota-current-outdoor-composed-towns-v01.crystalpack \
  planning/minnesota/landmarks/north-shore-shops.json \
  content-packs/minnesota-north-shore-shops-v01.crystalpack
.local/minnesota-review-env/bin/python \
  planning/minnesota/tools/review_north_shore_shops.py \
  --pack content-packs/minnesota-north-shore-shops-v01.crystalpack \
  --published --output planning/minnesota/renders/north-shore-shops-v01/published-v01
```

The review command also accepts the parent without `--published` for a fast
source-only render. It renders both complete changed chunks, both interiors,
player-scale door cameras, site views and the complete eight-chunk coast. It
never writes pack bytes. Every saved image is retained as review history.

The first visual pass exposed a malformed pie facade. The revised complete
native building fixes that join. Physical review also corrected the actual
native door quadrant, kept an existing trainer's cell unchanged, and retained
the original smokehouse-side road sign and road lane.

```sh
CRYSTAL_NORTH_SHORE_SHOPS_PARENT="$PWD/content-packs/minnesota-current-outdoor-composed-towns-v01.crystalpack" \
CARGO_TARGET_DIR=target/north-shore-shops CARGO_INCREMENTAL=0 \
  cargo test --locked -p crystal-assets --test north_shore_shops -- --ignored
.local/minnesota-review-env/bin/python \
  planning/minnesota/tools/check_north_shore_shops_mcp.py \
  --pack content-packs/minnesota-north-shore-shops-v01.crystalpack \
  --source planning/minnesota/landmarks/north-shore-shops.json \
  --client target/tamarack-ice/bin/geothite \
  --pie-fixture .local/north-shore-shops-v01/pie-fixture.crystalsave \
  --smoke-fixture .local/north-shore-shops-v01/smoke-fixture.crystalsave \
  --output .local/north-shore-shops-v01/stdio-fresh
```

The external-pack asset test checks unrelated complete maps, existing coast
programs/events/actors, art, encounters, trainers, audio and HOME, plus rejection
of stale parents, preimages, blocked habitat, unsafe doors/arrivals and occupied
native IDs. The production stdio case walks from the existing roads, enters both
rooms, talks to staff, verifies dialogue input ownership, uses both exits,
reads signs and checks indoor/outdoor Save/Continue. The capture tool renders
the reached saves through the production native renderer offscreen with `--2d`.
Initial local positions are QA fixtures, not earned progression.
The two fixture saves must match the newly built pack identity and start on the
road approaches listed in the source. This review initialized them with Rust
`RuntimeGameShell::new_game_at_runtime_tile` and `save`, with no party, rewards,
or badges; every action afterward used the production stdio controller.

## Verified local publication

Selected local review: `world.json` -> `north_shore_shops_revision`.
Pack: `content-packs/minnesota-north-shore-shops-v01.crystalpack` (732 maps).
SHA256 and content identity are recorded in that world revision.

- The Rust publisher's written pack readback passes. The publisher and external
  asset test were linked with the pinned Rust compiler against the newly
  completed native `crystal-assets` library in `target/runtime-fps/debug/deps`.
  This reused completed dependency output while the shared Cargo queue was busy.
- The external asset regression passes, including all six hostile-source cases
  and complete preservation of 728 other maps, existing coast programs/actors,
  trainer teams, encounters, art, audio, HOME and story constants.
- Both production stdio cases pass, with actual staff text/page changes,
  dialogue-owned input, both carpet returns, signs, Start and indoor/outdoor
  Continue. Each spawned client exits cleanly with byte-empty stderr. Logs and
  reached saves are retained under `.local/north-shore-shops-v01/stdio-v01`.
- All 377 audio programs pass PCM hash, frame-count and loop-range verification.
- Four headless production 2D screenshots from the reached porch/arrival saves
  were inspected in `renders/north-shore-shops-v01/native-v01` (relative to
  `planning/minnesota`). The exterior checkpoints use the native night palette;
  interiors use the original indoor palette. No 3D capture was made.
- Matching source views are in `matched-native-source-v01`. Outside visible
  actor footprints, 14,476 pie-shop and 18,568 smokehouse interior terrain
  pixels match the production captures exactly. The earlier day-bank interior
  preview is retained as history and superseded by the matched indoor render.
  The changed exterior site pixels also match exactly: 190,512 at the pie shop
  and 178,920 at the smokehouse, excluding the visible actor footprints.

Native captures used the completed existing `render_saved_session` tool with
`CRYSTAL_RENDER_TEST_HEADLESS=1`, `CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image`
and `--2d`; each source save is a genuine production-stdio reached checkpoint.
The canonical R11 terrain/doors/signs/rooms and shared interior contracts now
include both shops. The selected parent's other content remains preserved.
Specific new spawns and item purchases remain unassigned; this local selection
does not deploy a browser release or prove earned statewide progression.

The canonical R11 layout retains the same 15 pre-existing source/native stair
and terrain differences as the selected parent. All 41 new shop-site cells
match the canonical source; no pre-existing residual is introduced or changed.
The guarded publisher builds from the completed native parent, so these older
source differences do not affect reproduction of this shop overlay.

Affected-crate `cargo clippy --locked -p crystal-assets --all-targets` passes
with existing warnings and no diagnostics in the new files. Changed Rust files
pass `rustfmt --check`; the required workspace format check still reports the
existing offset-allocator and other unrelated formatting differences.
