# Voyageurs Border Loop: Tamarack snowline

This implements the inland winter belt's eastward thaw across the existing
**Voyageurs Border Loop (MN_R20)**. Tamarack is this climate revision's name;
the public route, NPCs and story are not renamed. Rainy River remains snowy.
The route reaches a broad frost-on-green bank before returning to fully green
Boundary Waters. Boundary Waters, its Safari and camp, North Shore and Grand
Marais receive no winter terrain or snowfall from this revision.

## Exact source and lineage

Authoritative source: [`source-v04.json`](source-v04.json), SHA256
`d34e0e85dcbe8a69f92073c3f17b801161f1bb261b6c0a14cbf81e3c143d7831`.
The source retains its original proposal status: the verification below records
what was subsequently implemented and tested without changing the frozen bytes.
Earlier v01-v03 and their rejected sharp-edge/melt previews remain retained.

The parent is `content-packs/minnesota-iron-rock-clearance-v01.crystalpack`,
SHA256 `5b6435171e2be96d49cc8a5ebbfc45d8223f5701d202dd71aa79e658b52ab982`.
This preserves the reviewed Bemidji roof/plateau, ten inland winter maps and
Iron Range rock/edge repair. The strict-readback child is
`content-packs/minnesota-tamarack-snowline-v01.crystalpack`, SHA256
`13554605a69f58a1ad3855f47bf64ad577ee2623c86742bc75ffb789399ba248`, content
identity `865bd35765f6e659ee9e14ad1da8e1c34d6031d5f710c2ca9d845967df8073b5`.
The subsequent [`Iron Range descent thaw`](../iron-descent-snowline-v01/README.md)
extends this child. Use that runbook's latest child when continuing winter
geography; an older parent loses already reviewed changes.

The child is **unselected**. `world.json` still selects
`north_shore_passing_width_revision`. No running client, browser release or
world selection was changed. This stage still needs integration with the latest
composed story/Ely-town work and the remaining southern thaw routes.

## Authored geography and gameplay contract

The complete route is 64×32 metatiles, split into four existing maps:

| Map | Metatiles | Presentation |
| --- | --- | --- |
| MnR20C00R00 | 32×24 | Full snowy pines |
| MnR20C00R01 | 32×8 | Full snowy pines |
| MnR20C01R00 | 32×24 | Snow, broad melt bank, then native green |
| MnR20C01R01 | 32×8 | Same continuous eastward transition |

Source v04 explicitly records every original and replacement row. Its cold and
melt regions follow uneven bank/forest contours, not a single straight stripe.
Coordinates in those region lists are eastern-column local metatile X and
whole-route metatile Y. The ordinary player path, all 8,192 collision quadrants,
doors, NPCs, teams, pickups, scripts and progression conditions are unchanged.
No new sliding ICE is added; snow is FLOOR and icy habitat remains TALL_GRASS.
This is a free public endgame road with its existing eight-badge/lighthouse
requirements, not proof that paid Safari admission is open.

Actual reciprocal walking seams, in 16px gameplay coordinates:

| West side | East side |
| --- | --- |
| Rainy River Icefields east `(63,44/45)` | R20C00R00 west `(0,44/45)` |
| R20C01R00 east `(63,36/37)` | Boundary Waters west `(0,24/25)` |

Snowfall is full through eastern-chunk core X14 and fades to zero at X38:
`snow = clamp((38 - x) / 24, 0, 1)`. Scheduled rain supplies the remaining
weather strength. Both eastern row chunks use the same X rule. The western
column remains snowy; the eastern Boundary Waters seam has no snowfall and
uses the normal warm rain schedule.

## Native art and fast review

`mn_winter_tamarack` and `mn_tamarack_thaw` derive from the exact bound native
`mn_headwaters` art. The partial alias preserves all 134 original metatiles and
adds 41 cold plus 41 melt variants. Its grass blocks are native green `03`,
snowy `88` and frost-on-green `b1`. All three retain native grass bitplanes,
zero-index priority holes and encounter behavior. No generic texture replaces
the source art.

The private `*_native_backgrounds.json` files retain native palette addresses
for original warm glyphs and moving water/flowers. The renderer resolves those
addresses against the actual clock palette. Cold glyphs use snow backing;
melt glyphs use native clock-lit green. Original warm pixels and all six native
animation glyphs keep their source frame colors and schedules at all four
clock periods. The narrow off-map forest continuation follows the adjacent
authored cold/melt/green forest block; it does not change the gameplay border.

The source review mutates only a decoded in-memory view. Rust alone publishes
packs. With the existing local Pillow/cbor2 environment:

```sh
PYTHONPATH=.local/ely-backroute-python python3 \
  planning/minnesota/tools/review_tamarack_snowline.py \
  --source planning/minnesota/regions/tamarack-snowline-v01/source-v04.json \
  --output planning/minnesota/renders/tamarack-source-FRESH
```

Choose a fresh output path; every original, conversion and rejected render is
retained. The measured warm source loop is **0.151 seconds after pack load**.
It produces the full 2048×1024 route, all four chunks and readable approaches.
Source previews establish daytime composition, not runtime weather or traversal.

Native publication uses the pinned Rust toolchain, the serialized Cargo slot
and the existing `target/verdigrim-scene` cache:

```sh
CARGO_TARGET_DIR=target/verdigrim-scene RUST_MIN_STACK=67108864 \
  cargo build --locked -p crystal-assets --bin build_tamarack_snowline
target/verdigrim-scene/debug/build_tamarack_snowline \
  content-packs/minnesota-iron-rock-clearance-v01.crystalpack \
  5b6435171e2be96d49cc8a5ebbfc45d8223f5701d202dd71aa79e658b52ab982 \
  planning/minnesota/regions/tamarack-snowline-v01/source-v04.json \
  content-packs/minnesota-tamarack-snowline-FRESH.crystalpack
```

The executed publication receipt is
`.local/tamarack-publication-v1110/receipt.json`. The CLI rejects an existing
output, a nonignored output, a changed parent and unlisted source edits. It
reads the resulting pack back strictly. Generated game content remains ignored.

## Verification and limits

- `renders/tamarack-installed-v1114/receipt.json`: all 739 other complete maps,
  all original art/audio/programs/NPCs/encounters/rewards exact; all 602 audio
  source/PCM/frame/loop records retained; 2,097,152 reviewed pixels with zero
  source/installed differences; all collision quadrants unchanged.
- `renders/tamarack-gameplay-v1115/receipt.json`: 3,262 production stdio-MCP
  calls through all four chunks, both real endpoint connections in both
  directions, six wild encounters escaped through native input, Start and
  return journey after Continue; stderr stayed empty. Initial legal Minnesota
  partners, badges, signal, position and supplies are explicit test setup.
  No Surf was required and no state was rewritten after input began. This is
  local traversal evidence, not an earned complete campaign.
- `renders/tamarack-native-v1116/`: untouched native GPU day/night views of full
  snow, cold middle, melt bank and green approach, plus an actual played-return
  save and the green Boundary Waters seam. All ten images were visually read.
- `.local/tamarack-runtime-v1111/checks.json`: focused weather, clock palette,
  foreground/background, winter and installed-art tests; native build and
  all-target crystal-bevy Clippy passed. Installed mixed-art regression checks
  every original warm metatile/glyph, all animation frames/schedules, all clock
  banks and the three grass variants' original priority masks.
- `.local/tamarack-legacy-v1117/checks.json`: all four prior seasonal, winter,
  grass-rustle and outdoor-stair animation regressions passed.
- `.local/tamarack-guards-format-v1118/`: wrong-parent/unlisted-edit refusal and
  scoped leaf formatting passed. Whole-workspace `cargo fmt --all -- --check`
  still reports pre-existing formatting differences; unrelated files were not
  reformatted.

The north-edge camera diagnostic in `renders/tamarack-border-v1119/` starts
inside closed forest solely to inspect off-map scenery. It is not a playable
position or traversal claim. Preserve that original before/after visual history.
The corrected native cold/melt/green forest-edge views are retained in
`renders/tamarack-border-v1123/`, with the before/after comparison and visual
assessment. The green strip at the snowy border is removed. Runtime rebuild,
focused tests and all-target Bevy Clippy passed in
`.local/tamarack-runtime-v1120/`. Existing formatting differences in
`graphics_assets.rs` and the wider workspace remain; the new leaf is formatted.

Global work remains: southern thaw sections toward Duluth, Leech Lake and
Itasca; current story/Ely-town integration; broader cave/terrain quality review;
combined native/controller/field-move verification and eventual selection.
