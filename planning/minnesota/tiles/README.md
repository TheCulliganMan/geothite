# Original Minnesota tile items

Current State Fairgrounds direction: compact native Crystal tile construction,
Goldenrod Underground / Pokémon Center counters, and separate NPC vendors.
The user rejected the large illustrated booth direction. The earlier
[ImageGen prop proposal](state-fair-v1/README.md) is retained review history;
it is not accepted art for the fair map.
The [animated transparent campfire](campfire-animation-v1/README.md) remains available,
with no grass or ground patch. The user rejected the earlier six-prop batch below.
Retain those sources and renders as rejected history, not current art acceptance.

The [northwoods-v1 catalog](northwoods-v1/catalog.json) contains original
ImageGen canoe, campsite, portage and shore props. Each catalog item is a
placeable 32×32 metatile made from sixteen 8×8 source tiles. These are map
authoring items; inventory actions and sign dialogue belong to the map scripts.

The original opaque [sheet](northwoods-v1/source.png) and exact
[prompt](northwoods-v1/prompt.txt) are retained. Built-in ImageGen generated the
source. The source SHA-256 is pinned in the catalog. Generated native previews,
indices, packed 2bpp files and compiled packs stay ignored.

## Conversion and publication

The Rust `pack_minnesota_tiles` publisher reuses the Pokémon workflow's
`MinnesotaSheet` PNG decoder and native bitplane/PNG encoders. Its map-specific
variant samples a fixed 32×32 grid, then applies only declared integer offsets.
An opt-in fixed-grid area vote preserves thin source contours; reciprocal source
neighbors support larger multi-block objects while retaining exterior clipping checks.
It does not fit a silhouette, generate contours, interpolate, or invent pixels.
Source colors classify the input; native palette indices select the original
time-dependent background colors. Shared palettes are never replaced globally.
Fully empty 8×8 margins use original native ground/grass glyphs and their
unchanged palette entries, so blank source backgrounds do not become large
flat rectangles on textured ground.

Every crop must be an in-bounds integer enlargement of 32 pixels. Source hash
drift, clipping, duplicate IDs, unsupported collision, or any ink in a FLOOR
approach rejects publication. Preserve the source and create a new version when
changing art; the publisher also refuses existing pack/preview output paths.

`mn_northwoods_props` derives from original `johto`. Blocks 00–7f retain
their exact logical art, palettes, collision and Cut/Whirlpool replacement
behavior. The importer expands two 96-glyph banks to two 128-glyph banks, moves
the old bank-one bytes to their corresponding new physical bank, and allocates
only unused logical glyphs. Flower 03 and water 14 remain reserved and use the
renderer’s Johto animation alias. All base runtime files, maps, species and audio
are retained. This separate namespace does not claim any city-specific slots.

| Item | Block | Native palette | Collision NW / NE / SW / SE | Placement |
| --- | --- | --- | --- | --- |
| Canoe rack | 80 | Brown 5 | WALL / WALL / FLOOR / FLOOR | Upper pair occupied; approach from south |
| Split firewood | 81 | Brown 5 | WALL / WALL / FLOOR / FLOOR | Upper pair occupied; approach from south |
| Picnic table | 82 | Brown 5 | WALL / WALL / WALL / WALL | Entire block occupied; leave adjacent floor |
| Portage marker | 83 | Brown 5 | FLOOR / WALL / FLOOR / WALL | Eastern pair occupied; stand west and face east |
| Shore cobbles | 84 | Gray 0 | WALL / WALL / WALL / WALL | Solid land/Surf boundary; close gaps deliberately |
| Cattails | 85 | Green 2 | WALL / WALL / WALL / WALL | Solid marsh scenery, no encounter or Surf action |

The marker has a canoe pictogram, with no text painted into its pixels. Map
authors supply any BG read script at the occupied board quadrant. Neither the
canoe nor table implies an unimplemented interaction. The green palette has a
different ground color, so place cattails in grass/marsh rather than a pale road.
The rack/table retain their normal source-scale size; do not enlarge them in maps.

Build and publish a separately named ignored pack:

```sh
CARGO_TARGET_DIR=target/agent-minnesota-tiles CARGO_BUILD_JOBS=2 \
  cargo build --locked -p crystal-assets --bin pack_minnesota_tiles \
  --profile wasm-dev --config profile.wasm-dev.opt-level=0
target/agent-minnesota-tiles/wasm-dev/pack_minnesota_tiles \
  content-packs/minnesota-northstar-map-quality.crystalpack \
  planning/minnesota/tiles/northwoods-v1/catalog.json \
  planning/minnesota/tiles/northwoods-v1/source.png \
  content-packs/minnesota-northwoods-props-v2.crystalpack \
  planning/minnesota/renders/tiles/northwoods-v2
```

The authoring handoff came from the active mapping owner in
[minnesota-tile-request.md](../reviews/minnesota-tile-request.md). This batch
covers canoe/sign/campsite silhouettes and solid shore scenery. The requested
white-pine joins, fitted basalt/ore courses and tied buoy boom remain separate
future art families. Active maps and frozen city inputs are not modified by
this art extension.

Review the actual published source art at all three native outdoor times:

```sh
.local/minnesota-review-env/bin/python \
  planning/minnesota/tools/review_minnesota_tiles.py \
  --pack content-packs/minnesota-northwoods-props-v2.crystalpack \
  --output planning/minnesota/renders/tiles/northwoods-v2/gallery
```

The review tool reuses `SourceArt`, checks every original block against Johto
for morning/day/night, and saves native-scale, nearest-neighbor and collision
views. It also archives unchanged full-resolution source crops. This is an
art-item gallery, not a map traversal or campaign-progression proof.

The supplied-pack test independently decodes every new packed bitplane,
checks original art and collision, compares all original runtime files and
normalizes only the declared tileset/field-rule additions before comparing the
entire game dataset:

```sh
CRYSTAL_TILE_TEST_PACK="$PWD/content-packs/minnesota-northstar-map-quality.crystalpack" \
  CARGO_TARGET_DIR=target/agent-minnesota-tiles CARGO_BUILD_JOBS=2 \
  cargo test --locked -p crystal-assets --test minnesota_tile_art \
  --profile wasm-dev --config profile.wasm-dev.opt-level=0 -- --include-ignored
target/runtime-fps/wasm-dev/verify_crystalpack \
  content-packs/minnesota-northwoods-props-v2.crystalpack --audio --desktop
```
