# Bemidji to Itasca snowline

The complete48×32-metatile route now thaws westward from snowy Bemidji pines,
through frosted encounter banks beside the young Mississippi, into the green
Itasca survey trail and actual gatehouse. Snow remains ordinary floor and
encounter grass; all6,144 collision quadrants are unchanged. No sliding ice,
water obstruction or new route gate is added.

## Source, publication and continuation

Authoritative source: [`source-v02.json`](source-v02.json), SHA256
`ced7b7a8196f401200c19982a63bb6487feafe1e97c2407b4eb920cdd2e5f52d`.
The frozen source retains its proposal status; this runbook records completed
native verification. Version1 remains history: its thaw missed the western
column's encounter grass. Version2 bends the cold bank around the young-river
crossing, including four cold and four melt grass metatiles.

Parent: `content-packs/minnesota-headwaters-rock-clearance-v01.crystalpack`,
SHA256 `d2b9d98f8204d216b9cdf7eb386441efd374a47424ee615c891071764f630bf9`.
Child: `content-packs/minnesota-itasca-snowline-v01.crystalpack`, SHA256
`d7139c4a1e4ada8aa9b389675b07aaf119bb670c2003fa21974ddf9ac6856c88`,
content identity
`e5019950f1e8bb4bf6b5e176ee478d519461ebf638220754492b5a1c554004e4`.
This stage is followed by the verified [Ely pine-town integration](../../towns/ely-pine-camp-v01/WINTER-INTEGRATION.md). Use that latest unselected child for continuation. It retains the preceding
Bemidji roof/plateau, winter belt, R09/R10/R08 cliff/boulder, Tamarack/Leech/Duluth
thaw and grass-Cut repairs. `world.json` still selects
`north_shore_passing_width_revision`; no app, browser release or selector changed.

| Chunk | Metatiles | Native art |
| --- | --- | --- |
| MnR14C00R00 | 32×24 | mn_itasca_thaw |
| MnR14C00R01 | 32×8 | mn_itasca_thaw |
| MnR14C01R00 | 16×24 | mn_winter_itasca_trail |
| MnR14C01R01 | 16×8 | mn_winter_itasca_trail |

The partial alias retains all134 warm source blocks, then appends41 cold and41
melt variants. The eastern alias recolors the complete original catalog while
retaining block IDs. Both retain the six native animation schedules: water,
flowers and four Whirlpool clips. Snow strength on both western chunks is
`clamp((localCoreX - 34) / 24, 0, 1)`; both eastern chunks have full snow.
Occasional native rain retains its schedule at `1-snow` strength. The Itasca
gatehouse is indoors; Boundary Waters/Safari/camp, North Shore and Grand Marais
remain outside the winter policy.

```sh
PYTHONPATH=.local/ely-backroute-python python3 \
  planning/minnesota/tools/review_itasca_snowline.py \
  --output planning/minnesota/renders/itasca-snowline-source-FRESH
CARGO_TARGET_DIR=target/verdigrim-scene RUST_MIN_STACK=67108864 \
  cargo build --locked -p crystal-assets --bin build_itasca_snowline
target/verdigrim-scene/debug/build_itasca_snowline \
  content-packs/minnesota-headwaters-rock-clearance-v01.crystalpack \
  d2b9d98f8204d216b9cdf7eb386441efd374a47424ee615c891071764f630bf9 \
  planning/minnesota/regions/itasca-snowline-v01/source-v02.json \
  content-packs/minnesota-itasca-snowline-FRESH.crystalpack
```

Serialize Cargo with `.local/minnesota-cargo.slot.lock`. Use fresh ignored
outputs. Python changes only an in-memory source view and never serializes a
pack. The complete source review loop takes0.290 seconds after loading.
`renders/itasca-snowline-source-v1217/` retains the full route, all five crops,
pine gate/replacement and ten native field-move variants; these were inspected.
Every original, conversion and rejected pass is retained.

Strict Rust readback passes. The installed/source comparison in
`renders/itasca-snowline-installed-v1221/` proves all739 other complete maps,
all original runtime files and602 audio entries exact, with all1,572,864 route
pixels matching independent native source composition. Actors, scripts,
encounters, rewards, room warps, ledges and reciprocal seams remain exact.
Wrong-parent and unlisted-row probes refuse without writing a pack
(`.local/itasca-snowline-guards-v1226/`). The shared builder also rebuilds the
previous Tamarack and R10 snowline packs byte-identically.

## Native field moves and actual traversal

The existing pine shortcut remains at MnR14C01R00 metatile `(3,10)`:
native5b→3c, retaining the neighboring Headbutt quadrant. Whole-route core
approach `(71,21)` reaches `(71,19)` by a34-step ordinary walk or a2-step Cut
link. Missing move/badge denies Cut; map return and Continue regrow the tree.
Ordinary movement reaches its inside face and cuts again to return.

The partial alias preserves all five original native Cut rules and adds ten
exact cold/melt replacements: grass88→87 andb1→b0, plus eight matching tree
variants. Other quadrants retain their native collision and glyph masks.
The full alias retains the original five replacements. Native grass-Cut
regrowth/Continue, move/Hive denial and successful clearing pass for cold,
melt and full-cold grass against a native green control: **16 sessions,
408 production calls**, empty stderr, in
`renders/itasca-grass-gameplay-v1227/`.

```sh
PYTHONPATH=.local/ely-backroute-python python3 \
  planning/minnesota/tools/check_itasca_grass_mcp.py \
  --pack content-packs/minnesota-itasca-snowline-v01.crystalpack \
  --fixture-writer target/verdigrim-scene/debug/examples/inland_grass_cut_fixture \
  --client target/verdigrim-scene/debug/geothite \
  --output planning/minnesota/renders/itasca-grass-gameplay-FRESH
```

All five existing R14 controller regressions pass in
`renders/itasca-controller-v1220/`: four clipped maps and habitats/Continue,
real Cut shortcut and refusals, Mara's battle with PP expenditure and persistent
outcome, dry one-way survey drops with grassy uphill returns, and an actual
wild encounter/Run/Continue. Their local partners now use native-created
Minnesota Borealynx/Voyagear, legal natural moves or native-compatible Cut,
and consistent prior-badge flags. Explicit setup is not earned progression.

The retained, actually played Cut checkpoint then passes a separate production
stdio round trip: **4,733 calls across play and Continue**, empty stderr, in
`renders/itasca-snowline-route-v1223/`. It walks all four chunks, four dry hops
and their ordinary uphill returns, both Bemidji seam lanes and the actual
Itasca gatehouse/Itasca return. No state is rewritten after input. Reproduce
with `check_itasca_snowline_mcp.py --pack PACK --save PLAYED_CHECKPOINT
--client CLIENT --output FRESH` using the retained controller save listed in
that receipt. This is walking/field-move evidence, not a new Surf claim.

## Native presentation and remaining work

All eight GPU originals in `renders/itasca-snowline-native-v1222/` were inspected:
snowy Bemidji arrival, pine Cut gate, melt and cold encounter banks beside the
river, original green survey ledge and gatehouse, southern snowy fen and the
same frost bank at night. Native pine, grass, shore, fence and clock palettes
remain coherent. The installed four-clock regression verifies all134 warm
glyphs, six clips, grass opacity holes and native rustle colors. Weather tests
cover every clock hour and both chunk rows, with coast/preserve exclusions.

Assets and Bevy all-target Clippy pass in `.local/itasca-final-assets-v1225/`
and `.local/itasca-runtime-v1219/`. Broader cave/terrain/clearing quality, complete statewide progression and remaining terrain review,
connected native atlas and final selection remain open. Existing saved terrain
overrides are not migrated by this art stage.
