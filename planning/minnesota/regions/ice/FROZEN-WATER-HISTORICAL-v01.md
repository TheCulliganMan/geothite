Historical rejected icy North Shore lineage. Never republish these sources.

# Frozen lakes, rivers and Crystal cave rock boundaries

The `frozen-water-v01.json` contract overlays the completed fruit-tree pack.
It freezes the two snowy R11 shore chunks and their northern offshore partners,
the R17 marsh lake, and dresses existing R17/Rainy River ice rims with original
Crystal Ice Path rocks. International Falls has no open water to replace.
The southern warm shore and other green routes retain open water.

There are 6,072 water quadrants changed to native slippery ICE and 610 cave-rock
quadrants. Existing buoys and the frozen gorge waterfalls become solid rock;
walls next to ice receive the same visible native crystal rocks. A deliberately
authored two-tile snow opening at R17 `(22,19)` and `(23,19)` makes its formerly
sealed lake playable. All original Floor, grass, entrances, ladders, Cut and
Headbutt tiles remain; no actor, fruit tree, event, trainer, encounter table or
connection is changed. Frozen waterfalls are solid for this winter geography.

Rock pixels come from the original `ice_path` block `19`. The native winter
index map replaces the cave background with snow white and merges the two
darkest shades, retaining its light-blue and dark indoor colors. The publisher copies them to unused glyph slots in the two
private winter tilesets, appends mixed metatiles, and leaves every existing
used glyph and collision definition unchanged. No art is generated externally.
Unmapped screen padding uses the same solid cave-rock recipe, keeping green
forest padding out of the frozen lake view. Each edge ice tile must have a real map connection whose projected coordinates
are within the neighboring map. Other exposed edges have solid, visible rock.

Executed Rust publication:

```sh
target/river-fun/wasm-dev/pack_frozen_water \
  content-packs/minnesota-fruit-groves-v02.crystalpack \
  planning/minnesota/regions/ice/frozen-water-v01.json \
  content-packs/minnesota-fruit-and-ice-v03.crystalpack
```

The output must be fresh. The publisher strictly validates the input, preimages,
native art, safe terrain changes, map edges and the written round trip.
Its content identity is
`71bb299b02d26ef4e763192f334202d9c066ae1af876756d8d24395a4b3225a8`.

Read-only review (choose a fresh output epoch):

```sh
.local/minnesota-review-env/bin/python \
  planning/minnesota/tools/review_frozen_water.py \
  --written content-packs/minnesota-fruit-and-ice-v03.crystalpack \
  --output planning/minnesota/renders/frozen-water-v01/written03
```

The review compares every written metatile to the independently rendered native
source recipe, checks all seven maps, preserves unrelated catalogs/art/audio,
and records native slide courses and exposed-edge cases. Whole R11 and R17
renders include their warm sections so the snow boundary is visible. Earlier
render epochs and pack revisions remain in ignored storage.

The final pack retains the [228 berry/apricorn trees](../../fruit-groves/README.md)
and both twenty-trainer Mississippi routes. Those river maps are byte-for-byte
unchanged by the ice overlay. Geometry fixtures explicitly provide local
starting positions and a Minnesota partner, rather than proving earned regional
progression.

Final pack SHA-256: `eea1880eac0ea1db7e6b8e2ab97ba9ebfbecf1d63fdac7096b9793e19bbb879c`.

Native acceptance passes on the final pack: four uninterrupted slide courses
(89, 61, 8 and 24 tiles), two real chunk crossings, exact crystal-rock stops,
Save/Continue at each stop, and four exposed-edge fixtures with repeated blocked
input. The written review also confirms every old used glyph and every new
mixed recipe matches the independent native-source render exactly. The first
published art candidate exposed a lawn-colored zero index around the rocks;
version02 uses the existing winter index map to give those pixels snow white.

Native fruit acceptance visited all 228 placements on the fruit parent; every
harvest and same-day repeat passed. The combined pack preserves those objects,
scripts and catalog bytes exactly, and repeats all sixteen item varieties with
Save/Continue and next-day regrowth. The full-pocket fixture uses all native
pocket slots, rather than mistaking one 99-item stack for a full pocket.

The production `geothite` client was rebuilt with the daily-reset change. Its
stdio-MCP check on the combined pack passes movement, the production Start menu,
the Winona home warp/Mom dialogue and byte-empty stderr. Same-pack native 2D
screenshots use explicit initial fixtures or the reached rock-stop saves.

Focused native tests and scoped formatting pass. Workspace formatting still
fails in unrelated files. The required all-target Clippy run remains blocked by
existing `bad_bit_mask` in `crystal-core` and approximate constants in
`crystal-voxel-view`; no new compiler warnings come from these leaf modules.
An unrelated rival-arc assertion was corrected to compare the `TilePosition`
coordinates so the shared native test harness could compile.

Final native captures are retained in `renders/frozen-water-v01/native-v02/`,
with a four-scene delivery board and machine-readable receipt in `delivery-v01/`.
All sixteen fruit varieties, same-day empty trees, Save/Continue, next-day RTC
regrowth and a genuinely full pocket pass again on version03. The final ice
acceptance also passes all four courses and four exposed edges after the rock
padding change. Original full-resolution screenshots were retained before
compositing the board; its berry and frozen-lake scenes use this same final pack.

Run the rebuilt local client:

```sh
RUST_MIN_STACK=67108864 target/river-fun/wasm-dev/geothite play \
  content-packs/minnesota-fruit-and-ice-v03.crystalpack
```
