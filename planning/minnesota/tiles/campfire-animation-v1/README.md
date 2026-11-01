# Animated campfire

The reviewed 32×32 transparent campfire now flickers through the same
`TilesetAnimatedTile` / `sequential_tileset_animation` path as the flowers.
Three original native poses play in order 0,1,0,2, with 8 game ticks per step
(about 0.53 seconds per loop). Only rectangle (8,0,16,14) changes. Stones, logs,
the lower flame anchor, transparency and collision stay fixed.

Built-in ImageGen produced the right/left flame variations from the reviewed
[campfire source](../campfire-v2/source.png). Original images, source hashes,
[recipe](animation.json), [right prompt](prompt-b.txt) and
[left prompt](prompt-c.txt) are retained here. Rust conversion preserves the
existing crop, native palette division and stable nighttime flame colors.

## Publish

```sh
CARGO_TARGET_DIR=target/agent-minnesota-tiles CARGO_BUILD_JOBS=2 \
  cargo build --locked -p crystal-assets \
  --bin pack_minnesota_campfire_animation --profile wasm-dev \
  --config profile.wasm-dev.opt-level=0
target/agent-minnesota-tiles/wasm-dev/pack_minnesota_campfire_animation \
  content-packs/minnesota-campfire-v3.crystalpack \
  planning/minnesota/tiles/campfire-animation-v1/animation.json \
  content-packs/minnesota-campfire-animated-v1.crystalpack \
  planning/minnesota/renders/tiles/campfire-animated-v1
```

The supplied base may also be a newer regional pack without the campfire; the
publisher first adds its original static tileset, then its animation. It refuses
existing output paths and existing animation metadata. Use the current renderer
from this checkout when consuming the animated pack.

Map authors use `mn_campfire`, block 80. All four gameplay quadrants remain WALL.
Only four uniquely owned glyphs animate; shared grass, ground, flowers and water
cannot be replaced. New runtime file
`data/tilesets/mn_campfire_animations.json` contains native 2bpp frame clips and
their tick period. They share the production ambient clock, visible tile
schedule and frame selection used by native flowers; no separate timer is added.

## Review

```sh
.local/minnesota-review-env/bin/python \
  planning/minnesota/tools/review_campfire_animation.py \
  --pack content-packs/minnesota-campfire-animated-v1.crystalpack \
  --base content-packs/minnesota-campfire-v3.crystalpack \
  --recipe planning/minnesota/tiles/campfire-animation-v1/animation.json \
  --output planning/minnesota/renders/tiles/campfire-animated-v1
```

The review reads actual published clips, independently decodes every phase and
compares it with the imported native frames. All runtime files from the static
pack, all gameplay data, the audio manifest and all 128 original block images at
morning/day/night are unchanged. The ring/log pixels remain stationary.
Transparent looping GIF/APNG previews and per-phase PNGs are retained in that
ignored render directory, with content-addressed histories.

Three animation tests pass, including rejecting shared-floor animation and bad
clip durations. The renderer library check and desktop pack validation pass.
Normal all-target Clippy and global formatting still encounter the previously
recorded repository failures; focused checks are recorded with the preview.

Final pack SHA-256:
`a8ea0e84d9a996537f82e763dd3c1a2379f7af6fd2c386448b49350dabded73b`.
GIF and APNG previews decode to the expected four native phases; day, morning
and night GIFs were checked frame by frame. GIF timing rounds to 130ms per step.
Game timing remains exactly 8 ambient ticks. Focused asset/compiler Clippy
passes with the two existing blocking lint classes allowed and no warning
locations in the new animation files.
