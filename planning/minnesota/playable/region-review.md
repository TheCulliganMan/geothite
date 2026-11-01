# Current Minnesota map review pack

`content-packs/minnesota-region-review.crystalpack` contains all twenty current
authored routes, Grand Marais and its six rooms, and the four Boundary Waters
preserve maps and actual Root Beer Cabin. It has 469 maps including the original
388-map game. The pack and generated review images remain ignored.

This is a composed local review, not an earned regional playthrough. The other
towns and regional hub gates remain implementation work. Safari admission is
closed without fees; session budgets and capture-only battles are not installed.
The [atlas](../renders/atlas/minnesota-current-connected.png) therefore keeps
regional links dashed and labels missing towns.

## Rebuild the composition

The current source-repaired 450-map Boundary pack is the input. Its build and
field-script restoration are documented in the
[preserve runbook](boundary-waters-safari.md) and
[content guide](../../../docs/game-content.md#additive-native-field-script-repair-on-2026-10-26).
Use one publisher and a distinct output for each stage:

```sh
mkdir -p .local/minnesota-region-stages
CARGO_TARGET_DIR=target/runtime-fps cargo build --profile wasm-dev --locked \
  -p crystal-assets --bin pack_driftless --bin pack_river_valley \
  --bin pack_twin_cities --bin pack_central_lakes \
  --bin pack_grand_marais --bin pack_prairie_farm

target/runtime-fps/wasm-dev/pack_driftless \
  content-packs/minnesota-boundary-waters.crystalpack \
  planning/minnesota/routes/MN_R01.layout.json \
  planning/minnesota/routes/MN_R01.json .local/minnesota-region-stages/r01.crystalpack
target/runtime-fps/wasm-dev/pack_river_valley \
  .local/minnesota-region-stages/r01.crystalpack \
  planning/minnesota/routes/MN_R03.layout.json \
  planning/minnesota/routes/MN_R03.json .local/minnesota-region-stages/r03.crystalpack
target/runtime-fps/wasm-dev/pack_twin_cities \
  .local/minnesota-region-stages/r03.crystalpack \
  planning/minnesota/routes/MN_R04.layout.json \
  planning/minnesota/routes/MN_R04.json .local/minnesota-region-stages/r04.crystalpack
target/runtime-fps/wasm-dev/pack_central_lakes \
  .local/minnesota-region-stages/r04.crystalpack \
  planning/minnesota/routes/MN_R15.layout.json \
  planning/minnesota/routes/MN_R15.json .local/minnesota-region-stages/r15.crystalpack
target/runtime-fps/wasm-dev/pack_grand_marais \
  .local/minnesota-region-stages/r15.crystalpack \
  planning/minnesota/playable/grand-marais.json \
  .local/minnesota-region-stages/grand-marais.crystalpack --preserve-home
target/runtime-fps/wasm-dev/pack_prairie_farm \
  .local/minnesota-region-stages/grand-marais.crystalpack \
  planning/minnesota/routes/MN_R02.layout.json \
  planning/minnesota/routes/MN_R02.json content-packs/minnesota-region-review.crystalpack
```

`--preserve-home` uses `build_grand_marais_region_extension`. It preserves the
entire existing HOME/recovery table. Omitting it retains the separate town-demo
behavior: start a new game in Tamarack's station. The extension regression checks
that both modes install identical town maps and encounters, that the extension
retains every original spawn, and that a second installation is rejected.

## Executed evidence

The composed pack passes native verification and all 377 audio PCM hashes,
frame counts and loop ranges. Every previous 450-map module and runtime file,
the original species/move/item/mart/audio catalogs, existing trainers and all
HOME/recovery destinations are unchanged. Exactly fourteen new trainer parties
match the five added route dossiers; new runtime art files belong only to the
native `mn_river_park` recipe.

The atlas compares all twenty routes' complete terrain, exterior actors and
authored room blocks against this one pack. It also checks all four preserve
chunks, native art, admission guards and cabin warps. All twenty match. Render:

```sh
.local/minnesota-review-env/bin/python planning/minnesota/tools/render_progress_atlas.py \
  --region-pack content-packs/minnesota-region-review.crystalpack
```

Pine Country's five-trainer journey, Leech Lake's full eight-trainer round trip,
and Northern Lakes' two-trainer journey pass against the composition, including
selected move PP, resources and Continue. North Shore's four cases, including
native Waterfall ascent/dismount/relaunch/descent, and Prairie–Forest's three
cases, including the actual Headbutt battle, also pass. The actual Mom/New Bark
gate regression passes, proving the
composition starts through the original HOME and production controller. These
route fixtures explicitly supply legal parties, badges and position; they do
not prove those resources were earned. Remaining route-specific evidence and
limits are recorded in [STATUS](../STATUS.md).

An actual Pine Country review save was exported after the successful composed
journey. It is an explicit fixture, not a fresh or earned regional playthrough:

```sh
target/runtime-fps/wasm-dev/geothite play \
  content-packs/minnesota-region-review.crystalpack \
  --load .local/minnesota-region-pine-country.crystalsave
```

Native art and whole-map visual review remain necessary. Passing these checks
does not establish final visual approval. Source edits should use the individual
route's fast renderer before native publication; an atlas mismatch remains amber
until the reviewed source is republished.
