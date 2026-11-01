# Bemidji to Leech Lake snowline

The complete forty-by-forty-eight-metatile lake route now descends from snowy
Bemidji terraces through frosted pine portages into the original green Leech
Lake road. The two lakes stay open water. Snow is ordinary floor; no sliding
ice or new gameplay barrier is added. Existing grass, trainers, fitted stairs,
Cut pier gate, rewards, story, docks and actual town connections remain intact.

## Authoritative source and continuation candidate

[`source-v01.json`](source-v01.json), SHA256
`efac4a83335a645163ceb89c364a31c4efdd8461b5011731369bb30bec3739c5`, contains
all four complete before/after map rows, exact source-art bindings, native
recipes, hand-authored cold/melt bands and the existing pier-gate contract.
Whole source originals and every crop are retained in
`renders/headwaters-snowline-source-v1182/`; the after-load review loop takes
0.965 seconds. The whole route and all six detail crops were visually inspected.

Parent: `content-packs/minnesota-inland-grass-cut-v01.crystalpack`, SHA256
`5d9989ea250033955704748c2f915350d27e89160574d293a3c500ec6f32e68b`.
Candidate: `content-packs/minnesota-headwaters-snowline-v01.crystalpack`, SHA256
`2b8ea8076b18ee740e832beb11d1fd4ae3c5c1ef883945b8db78f579a6c2df48`, content
identity `1c264e662ee893d9665a9292cc53d45380ec4bdf6cfe2f63d161f65e04d2cd7b`.
This art stage is followed by the verified
[stair-boulder successor](../../revisions/headwaters-rock-clearance-v01/README.md),
SHA256 `d2b9d98f8204d216b9cdf7eb386441efd374a47424ee615c891071764f630bf9`.
That child is followed by the verified [Itasca snowline](../itasca-snowline-v01/README.md). Use the Itasca child for further work. This stage retains all preceding
Bemidji roof/plateau, winter belt, R09/R10 cliff/boulder, Tamarack/Duluth thaw and
grass-Cut repairs. `world.json` still selects `north_shore_passing_width_revision`.
No running client, browser release, staged source or remote branch was changed.

| Chunk | Metatiles | Native art binding |
| --- | --- | --- |
| MnR08C00R00 | 32×24 | mn_winter_headwaters_stairs |
| MnR08C01R00 | 8×24 | mn_winter_headwaters |
| MnR08C00R01 | 32×24 | mn_headwaters_thaw |
| MnR08C01R01 | 8×24 | mn_headwaters_thaw |

The full stair alias retains all137 source blocks and inherits the original
Headwaters water, flower and four Whirlpool animation clips. The original stair
alias had no own animation JSON, so those six native schedules must continue
through its actual ancestor. Animated background palette addresses stay native.
The partial alias keeps all134 warm blocks and appends23 cold plus23 melt
variants, with native logical glyph addresses and grass priority holes intact.
Warm grass03 retains original rustles. Cold88 and melt9f retain icy blades and
receive native grass-Cut rules88→87 and9f→9e during publication.

Snowfall follows whole-route **16px core Y**, with the southern row adding48:
`snow = clamp((80 - y) / 26, 0, 1)`. Full snow reaches Y54 and ends at Y80,
before the Y95 Leech mouth. Both banks use the same latitude. Occasional rain
uses the original schedule at `1-snow` strength. The weather interface now takes
both local X and Y; previous eastward Tamarack/Duluth rules retain their exact X
behavior. The coast, Grand Marais and Boundary Waters/Safari/camp remain excluded
from winter at every tested clock hour.

## Reproduce source review and Rust publication

```sh
PYTHONPATH=.local/ely-backroute-python python3 \
  planning/minnesota/tools/review_headwaters_snowline.py \
  --output planning/minnesota/renders/headwaters-snowline-source-FRESH
CARGO_TARGET_DIR=target/verdigrim-scene RUST_MIN_STACK=67108864 \
  cargo build --locked -p crystal-assets --bin build_headwaters_snowline
target/verdigrim-scene/debug/build_headwaters_snowline \
  content-packs/minnesota-inland-grass-cut-v01.crystalpack \
  5d9989ea250033955704748c2f915350d27e89160574d293a3c500ec6f32e68b \
  planning/minnesota/regions/headwaters-snowline-v01/source-v01.json \
  content-packs/minnesota-headwaters-snowline-FRESH.crystalpack
```

Serialize Cargo with `.local/minnesota-cargo.slot.lock`. Python previews mutate
only a decoded in-memory view and never write a game pack. Packs, images,
logs, native saves and verification output remain ignored. Every native and
source original, conversion and rejected pass is retained.

The first publication refused its residual check because an earlier stair
patch had changed the compiled module's alias while retaining `mn_headwaters`
in the raw map-attribute table. The builder now restores each original table
and module preimage independently during comparison; it does not relax the
residual guard. The successful Rust publication passes strict readback.
`renders/headwaters-snowline-installed-v1189/receipt.json` verifies all739 other
complete maps, all original runtime files and602 audio records exact. All7,680
collision quadrants stay exact. All1,966,080 installed route pixels match the
independent native source composition with zero differences. Original tree
rules remain exact; only two appended grass rules are added.

## Native input, field moves and visual evidence

```sh
CARGO_TARGET_DIR=target/verdigrim-scene RUST_MIN_STACK=67108864 \
  cargo build --locked -p crystal-runtime --example inland_grass_cut_fixture
CARGO_TARGET_DIR=target/verdigrim-scene RUST_MIN_STACK=67108864 \
  cargo build --locked -p geothite
PYTHONPATH=.local/ely-backroute-python python3 \
  planning/minnesota/tools/check_headwaters_snowline_mcp.py \
  --pack content-packs/minnesota-headwaters-snowline-v01.crystalpack \
  --fixture-writer target/verdigrim-scene/debug/examples/inland_grass_cut_fixture \
  --client target/verdigrim-scene/debug/geothite \
  --output planning/minnesota/renders/headwaters-snowline-gameplay-FRESH
```

The first completed six-session course in `renders/headwaters-snowline-gameplay-v1194/`
passes3,961 production calls with empty stderr: all four chunks, the fitted stair
in both directions, actual Bemidji and Leech seams and return, Start and Continue,
no-move/no-Hive Cut refusals and the original pier gate. That gate stays blocked
before native Cut. On Continue the tree regrows; ordinary input reaches its
inside face, cuts again and returns to the public road. The expanded course adds
cold/melt grass Cut, both refusals and Continue against a native green control;
it passes **18 sessions and4,367 production calls**, with byte-empty stderr,
in `renders/headwaters-snowline-gameplay-v1196/`.
All parties, Cut/Hive, supplies and initial coordinates are explicit local
fixtures. No state is rewritten after input, and this is not earned campaign
progression or new Surf/reward-collection evidence.

The native GPU originals in `renders/headwaters-snowline-native-v1195/` were
visually inspected at eight checked floor/grass camera positions: Bemidji
arrival, fitted stair, pier gate, northern portage, both thaw banks, green Leech
mouth and matching night bank. Warm glyphs, stair ancestry, all six animation
clips, icy bitplanes/priority holes, rustles and clock colors pass the installed
four-clock regression. The older v1192 camera batch includes invalid wall
placements and remains diagnostic history; it is not visual acceptance.

`.local/headwaters-runtime-v1188/` contains the successful renderer/test builds
and all-target Bevy Clippy. `.local/headwaters-regressions-v1191/` records passing
weather, exact aliases, cliff-floor priority and complete installed priority
catalog checks, plus Headwaters/Tamarack/descent installed-art regressions.
Owned Rust files and Python syntax pass; whole-workspace rustfmt still reports
pre-existing differences. No new warnings are attributed to the changed leaves.

## Remaining regional work

This accepts the scoped native winter transition, not all existing route design.
The northern stair's neighboring boulder/face spacing is repaired and verified
in the [six-cell successor](../../revisions/headwaters-rock-clearance-v01/README.md). Continue reviewing the
wider lake clearings during the terrain pass. The [R14 Bemidji-to-Itasca thaw](../itasca-snowline-v01/README.md) now passes its scoped checks. Integrate
the latest story/Ely-town parent, continue the wider cave/terrain audit and run
combined controller, field-move, native and connected-atlas checks before final
selection. Existing save overrides are not migrated by this art stage.
