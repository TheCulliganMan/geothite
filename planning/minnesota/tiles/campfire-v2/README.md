# Transparent stone-ring campfire

The [animated version](../campfire-animation-v1/README.md) adds a looping flame
through the same ambient tile system as flowers. This directory remains its
authoritative static source and native conversion recipe.

The user rejected the earlier prop sheet and requested: “Need better no grass
that is bad focus on campfire those suck.”

This source focuses on one campfire: two flame tongues, two crossed charred
logs and a distinct cool-gray stone ring. The PNG has true transparency, with
no grass, dirt patch, ground tile, glow or smoke.

The rejected first native pass had only 13 light/dark-gray pixels, losing the
ring. The revised cutout retains 118 gray pixels and clear separate stones.
The exported 32×32 PNG has 705 fully transparent pixels, with zero RGB in those
pixels, so no green background survives in the asset.

The [source](source.png), [edit prompt](prompt.txt) and [catalog](catalog.json)
are retained. Built-in ImageGen edited the first campfire source; the first
source and its original generation prompt remain in the sibling campfire-v1
directory, alongside its rejected native conversion.

Import uses a reviewed square crop and an explicit vertical source split:
the flame half maps to the upper sixteen native rows and the ring/log half
to the lower sixteen rows. This keeps yellow/orange native palette 4 separate
from gray palette 0. Source alpha determines the silhouette, so opaque pale
stone highlights stay solid. No grass palette is used. Empty native map glyphs
reuse unchanged ordinary floor; exported cutouts remain transparent.

The item is `campfire`, block 80 in `mn_campfire`, with WALL collision in
all four 16px quadrants. This is static scenery. A map author supplies a clear
adjacent approach and any separately authored gameplay interaction.

Publication command:

```sh
target/agent-minnesota-tiles/wasm-dev/pack_minnesota_tiles \
  content-packs/minnesota-northstar-map-quality.crystalpack \
  planning/minnesota/tiles/campfire-v2/catalog.json \
  planning/minnesota/tiles/campfire-v2/source.png \
  content-packs/minnesota-campfire-v3.crystalpack \
  planning/minnesota/renders/tiles/campfire-v3
```

The original Johto blocks and unrelated content remain unchanged in this
separate ignored art pack. Native alpha PNGs, importer indices, full-resolution
source crop and morning/day/night views are retained under the corresponding
ignored render directory.

Night review caught a yellow rectangle caused by the native yellow palette's
different nighttime color zero. The final pack supplies foreground-only colors
for the newly allocated flame glyphs. The production renderer keeps those three
warm fire colors while taking background zero from the current native floor.
The gray stones still follow the native time palette. Original blocks and
shared palettes are unchanged. Two focused production-palette regressions
verify the night-floor choice and unchanged original yellow artwork.

Verified final artifact:
`content-packs/minnesota-campfire-v3.crystalpack`, SHA-256
`b114cf6b176f1b13cda6b6e1d67de7c167ca043ee26fe7d6e6b837a065f29a60`.
The 32×32 transparent PNG and 8× nearest-neighbor preview are
`planning/minnesota/renders/tiles/campfire-v3/campfire.1x.png` and
`campfire.8x.png`.

Inspected the final native cutout plus actual packed day/night source-art
renders. The night rectangle is gone and the flame stays yellow/orange.
Independent readback checks its packed bitplanes and all 128 original block
images at morning/day/night. Every original runtime file, unrelated game field
and audio manifest matches the supplied base. Rust desktop pack validation
passes (538 maps, 251 species, 251 moves, 377 audio programs).

Four tile/import/preservation tests, fourteen sprite-sheet regressions and
two focused production-palette tests pass. The final tile tests were compiled
against the current Rust library with the standard Rust test harness, avoiding
an unrelated rebuild of every pack CLI. `cargo check --locked -p crystal-bevy
--lib --profile wasm-dev --config profile.wasm-dev.opt-level=0` passes.
Focused Clippy passes with only the previously recorded blocking
`bad_bit_mask` and `never_loop` lints allowed; normal all-target Clippy still
stops at the existing core `bad_bit_mask` error. Global formatting checks still
report unrelated existing differences; changed standalone Rust files pass
`rustfmt --check`. Logs are retained under the ignored final render directory's
`verification/`.
