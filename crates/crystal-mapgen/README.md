# Geographic Crystal worlds

`crystal-mapgen` builds semantic scenes from OpenStreetMap and lowers them to
Crystal metatiles, events, encounters, and map extensions. Implementation is
Rust; artwork is read from an external compatible `.crystalpack`.

## Current pipeline

Coordinates → frozen source → districts and routes → complete structures and
services → reserved destinations → contextual vegetation → tiles and events.

Generator version 2 is the only generator. The old decoration, biome, roadside,
and quota/repair pipelines have been removed. Normalized input must explicitly
use source schema 2; older source files must be fetched again. Existing playable
packs remain independent artifacts: the generator does not rewrite old saves.

Source normalization retains OSM type/ID, building geometry, multipolygon holes,
selected descriptive tags, and bridge/tunnel/layer semantics. Tunnel and negative
layer transport are excluded from surface routes. Retaining layer tags is not yet
a complete simulation of grade-separated crossings in a single 2D map.

Scenes distinguish urban, residential, waterfront, woodland, meadow, and rocky
families. Mapped land cover wins over procedural decoration. Buildings use
complete, recorded recipes with source IDs and entrance coordinates; four house
variants reuse existing pack art. Local repetition is bounded. Courtyards, rest
areas, clearings, and outcrops are placed only where a complete footprint and
readable approach fit. Open space does not fail a scenery quota. Rocks and
ledges are decorative composition, not measured elevation.

The region API allocates map and spawn IDs through a persisted registry, gives
local script/object identifiers a cell namespace, creates residential interiors,
and connects selected reciprocal H3 crossings with explicit warp panels. Portal
landings are separate, reachable, non-warp floor positions. Assembly checks that
original maps, runtime files, compiled audio, and audio manifests remain equal.
Published output cannot be overwritten with a different pack.

## Run and verify

Run from the repository root. A compatible pack is required for pack assembly
and runtime checks. See [content setup](../../docs/game-content.md).
A fresh pret checkout alone does **not** provide the intermediate runtime export
required by `pack_core`.

Static scenery can also be rendered directly from an external pret checkout:

```sh
cargo run --locked -p crystal-mapgen --example render_proof -- \
  /absolute/path/pokecrystal /absolute/path/source-v2.json \
  output/scenery-proof-v2
```

This reuses the production atlas builder and palette renderer, reading original
art, collision and palette definitions in memory. It writes full maps, day/night
160×144 walking-scale crops, a labeled HTML gallery, and six synthetic habitat
fixtures. It checks repeat generation for determinism. These are static scenery
images, without characters or gameplay simulation; they do not validate a pack.
Keep the checkout outside this repository and all render output ignored.

```sh
cargo test --locked -p crystal-mapgen --lib --tests

CRYSTAL_MAPGEN_TEST_PACK=/absolute/path/core-modular.crystalpack \
  cargo test --locked -p crystal-mapgen --lib --tests -- --ignored
```

The ordinary suite uses synthetic geography and includes a seven-cell H3
resolution-6 region with 96×96 metatiles per cell. It checks deterministic
composition, source reordering, holes/islands, empty terrain, habitat distinction,
service access, reciprocal regional seams, ID allocation, and safe portal sites.
Existing antimeridian, pentagon, road-overlap, and boundary regressions also run.
Pack-dependent tests are explicitly ignored unless requested; they never silently
pass when assets are missing.

Build the connected Minneapolis region:

```sh
cargo run --locked -p crystal-mapgen -- \
  --lat 44.9475196 --lon -93.3253477 \
  --h3-res 6 --h3-generate-cells 7 --grid 96 \
  --build-region --h3-render-proof \
  --base-pack /absolute/path/core-modular.crystalpack \
  --output-dir output/minneapolis-v2
```

`--source /absolute/path/source.json` uses a frozen schema-2 source instead of
fetching OSM. Omit `--h3-render-proof` and `--build-region` to generate and audit
regional scenes without game assets. Single-map generation remains available by
omitting the H3 batch flags. `--h3-plan-cells 5000` plans topology without fetching
or rendering cells.

A built region writes `region.crystalpack` and `world.json`. The latter records
generator/base/source identity, the start map, and map registry. Per-cell grids,
source snapshots, and audits accompany it. Optional proof rendering writes the
exact-art full-cell PNGs and region mosaic. Use a new output directory when the
source, generator, settings, or base pack changes. All these outputs stay ignored.

## Remaining acceptance work

This implementation establishes the scene/region foundations; the full visual
and gameplay showcase is not yet accepted. Remaining work includes:

- A reproducible compatible pack/export workspace, then fresh full-region,
  walking-scale, interior, and day/night captures and visual review.
- Bounded theme-specific derived atlases and source-art identity in 2.5D
  profiles. The current implementation still uses the existing generated Johto
  atlas. More road-surface variety, footprint/frontage-aware compression, and
  grade-separated crossing behavior also need work.
- Separate cave interiors, generated-region starter onboarding, and trainers
  with persistent battle/reward flags. Contextual residents currently converse.
- Runtime proof for all residential doors, healing HP/status/PP, shopping,
  blackout recovery, save/reload, and two-client presence/chat/battle/trade.
- Measured generation time, pack size, browser memory and walking performance,
  including the requested comparison against a same-device baseline.

Synthetic tile counts and successful audits do not establish visual quality or
substitute for these runtime checks. See [world architecture](world-architecture.md)
for the deliberately deferred on-demand generation design.
