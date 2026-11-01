# Duluth rock stairs and street heights

This revision repairs roads that walked through rock faces without a stair.
The town remains 64×46 metatiles. All 38 doors, 76 warps, interior rooms, services,
trainers, items, story programs and both regional connections are retained at
their current coordinates. This is a terrain repair, with no new story gates.

The selected local revision is `world.json.duluth_cliff_stairs_revision`:
`content-packs/minnesota-current-duluth-cliff-stairs-v03.crystalpack`, SHA256
`6878059cebf9826b319b840bbcb2a8f243a73556c68d00cc0d135ad4a597265f`.
It composes onto the latest State Fair building parent and preserves all 729
other maps exactly. See [verification](VERIFICATION.md).

[Current whole town](../../renders/duluth-cliff-physics-v01/whole-town-v04.png)
· [Native before/after](../../renders/duluth-cliff-physics-v01/native-before-after-sheet-v01.png)

The initial Rainy River review parent was
`content-packs/minnesota-current-grass-rainy-icefields-v01.crystalpack`, SHA256
`2bd46f3452af8ea3699a75938862d07e62771fc64dab7096648fa6509f0f03b8`.
`before.layout.json` reproduces its entire Duluth exterior exactly.
`after.layout.json` changes 29 metatiles. `publication.contract.json` binds the
complete original native module and every indexed anchor without relocation.

The Observation and civic upper streets now share one continuous bluff.
Previously an internal rock wall implied two elevations while a flat paved gap
let the player cross them. The joined street removes that false height change.
The hillside supply street and weather-house apron now descend by actual stairs.
Their rear and side cliff edges stay closed around those approaches. The garden
side corner is a continuous wall; its northern road crosses rock on a stair,
and its southern stair provides the ordinary uphill return from the ledge.

Every flight uses the exact original cave36 pixels and native collision
`FLOOR,WALL,FLOOR,WALL`, appended as private Duluth block `8c`. The left walking
column carries the climb and the right column is real rock collision. Existing
140 metatiles and all their pixels remain unchanged. Six previous half-height
stairs become full flights, and five formerly flat crossings gain stairs.

| Flight | Metatile | Upper walking tile | Lower walking tile |
| --- | --- | --- | --- |
| Observation west |7,7|14,13|14,16|
| Observation east |18,8|36,15|36,18|
| Civic upper |23,7|46,13|46,16|
| Hillside west bend |35,6|70,11|69,13|
| Hillside front |40,7|80,13|80,16|
| Outdoor supplies |44,5|87,10|88,12|
| Weather west bend |49,6|98,11|97,13|
| Weather front |53,7|106,13|106,16|
| Weather east |57,5|114,9|114,12|
| Garden north |23,12|46,23|46,26|
| Garden waterfront |27,20|54,39|54,42|

The west bend flights turn west at their foot. Outdoor supplies turns west at
its crest onto the existing door apron. These corners are intentional; a
straight-line fixture through a wall or facade is not a valid stair test.

Ten ledge lanes have clear upper approaches and dry landings. The eastern
drop moves from metatile (41,7) to (42,7) because its original adjacent stair made
the ordinary return only seven steps. The old lip becomes continuous rock. They
retain native downward hops, blocked reverse travel and ordinary stair returns.
The western garden lane is reached around the actual sign body. Its lower
quadrant is clear; the first audit incorrectly treated the whole sign block as
a wall. Actual controller/MCP input confirms the legal approach and hop.

## Build and verification

Publication uses Rust only:

```sh
CARGO_TARGET_DIR=target/runtime-fps CARGO_BUILD_JOBS=1 \
  cargo build --locked -p crystal-assets --bin revise_duluth_cliff_stairs
target/runtime-fps/debug/revise_duluth_cliff_stairs \
  content-packs/minnesota-current-grass-rainy-icefields-v01.crystalpack \
  2bd46f3452af8ea3699a75938862d07e62771fc64dab7096648fa6509f0f03b8 \
  planning/minnesota/revisions/duluth-cliff-physics-v01/before.layout.json \
  planning/minnesota/revisions/duluth-cliff-physics-v01/after.layout.json \
  planning/minnesota/revisions/duluth-cliff-physics-v01/publication.contract.json \
  content-packs/minnesota-current-duluth-cliff-stairs-v02.crystalpack
```

The executed Rainy review composition has SHA256
`884b7ec5cc6b1daa1269f2e59f74f274e210698e29491396918d2e0b842484e2`,
content identity
`d31241613fdb6a47520fb3f277e8a11122e03c5daf41f7744e1cd23f1b01ce54`.
Rust publication/readback, strict whole-pack preservation, all377audio checks,
11bidirectional stairs with Continue and all38doors/both room returns pass.
The three production stdio journeys pass215RPC with empty stderr, including
the sign-adjacent tenth ledge, its reverse denial and its walked stair return.
Five inspected native2D frames each match the source compositor exactly. The
expanded ten-lane unit replay and both affected-crate all-target Clippy checks
also pass. The first source render is retained as a rejected pass because simply
closing the garden's old rear gap disconnected northern streets. The corrected
v02 introduces a real stair there and preserves all38ordinary door returns.

Immutable full-town previews, native before/after captures, logs, saves and the
independent preservation receipt belong under
`planning/minnesota/renders/duluth-cliff-physics-v01/`. The audit is
`tools/audit_duluth_cliff_stairs.py`. Controller coverage belongs to
`duluth_cliff_stairs.rs`: all11flights both ways plus Continue, all10clear ledge
lanes with reverse denial and walking return. The existing38-door/76-return
controller case passed in143.03seconds on this exact candidate. Explicit local fixtures
establish geometry, not earned statewide progression.

Five original native frames and their complete source comparisons are retained
in `pixel-comparison-v01/`; every compared640×576pixel matches. The full town
is `whole-town-v02.png`, with27native actors. Older v01pack/renders are retained
as the rejected seven-step eastern-return candidate. The missing outdoor
spawn513 was added to the relocation inventory unchanged at(41,32); the first
failed publication and original contract are retained.

## Current Fair-parent publication

This command passed with the same frozen sources and complete anchor inventory.
Outputs are immutable; choose a fresh versioned output filename for a replay.

```sh
target/runtime-fps/debug/revise_duluth_cliff_stairs \
  content-packs/minnesota-current-state-fair-buildings-v01.crystalpack \
  69f162aa4e8fc2b6ff35b9ad13e0dae90a99387bf5dd6fb1cb19f9ddeb498156 \
  planning/minnesota/revisions/duluth-cliff-physics-v01/before.layout.json \
  planning/minnesota/revisions/duluth-cliff-physics-v01/after.layout.json \
  planning/minnesota/revisions/duluth-cliff-physics-v01/publication.contract.json \
  content-packs/minnesota-current-duluth-cliff-stairs-v03.crystalpack
```

The current pack repeats all eleven bidirectional stairs and Continue in
61.53 seconds, passes 32 production stdio calls with empty stderr, and verifies
all 377 audio programs. Its complete Duluth module, tileset and art match the
review composition exactly. A new current-parent native capture also matches
the reviewed image pixel for pixel. The three Fair buildings remain installed.

## Fast source previews

The following one-shot command passed. Add `--watch` to load and verify the pack
once, cache its native art, and rerender when the layout changes. Warm complete
town renders, including the saved originals and preview, took 0.666 and 0.624
seconds. Each pass gets a unique source-hash/time directory; `latest.json`
points to that retained version. This tool writes images, never packs.

```sh
.local/minnesota-review-env/bin/python \
  planning/minnesota/tools/render_duluth_cliffs.py \
  --pack content-packs/minnesota-current-duluth-cliff-stairs-v03.crystalpack \
  --layout planning/minnesota/towns/MnDuluth.layout.json \
  --output planning/minnesota/renders/duluth-cliff-physics-v01/fast-preview-cli-v01
```

The complete preview terrain matches the installed native terrain exactly.
The latest whole-town image uses production actor origins and authored facing;
the earlier overview marker versions remain retained.
