# Native grass Cut on the inland snowlines

The newer icy grass blocks had no native Cut replacements. The original R10
stair alias also lacked grass Cut. This repair restores native party-menu Cut
on green, snowy and thawing habitats without removing adjacent cliffs or
changing tree gates, badge requirements or map geometry.

## Frozen source and candidate

[`source-v02.json`](source-v02.json), SHA256
`7948e87690a561afb548089d19b5bd3e22c1f3521696a0b8cfa33901ef1988ef`, binds the
complete parent, original Cut catalog, five native aliases and twelve explicit
quadrant compositions. Rejected source-v01 remains history: its Python hash
sorted CBOR integer keys numerically, while Rust's JSON Value sorts their string
representations. Review hashing now converts keys to JSON strings first.

Parent: `content-packs/minnesota-descent-rock-clearance-v01.crystalpack`, SHA256
`d63518642625850a43223b82612a5b3530037205bdab59b55012b443bd8faac4`.
Child: `content-packs/minnesota-inland-grass-cut-v01.crystalpack`, SHA256
`5d9989ea250033955704748c2f915350d27e89160574d293a3c500ec6f32e68b`, content
identity `2bc2680254f492f276a52ece0b65b92bae9a1284072eb1fe1b665a40efe87836`.
The child is unselected. Continue later winter work from its
[Headwaters successor](../../regions/headwaters-snowline-v01/README.md), its verified
boulder clearance and the latest [Itasca snowline](../../regions/itasca-snowline-v01/README.md). The world selector still points
to `north_shore_passing_width_revision`.

Twenty-one new native grass rules use the existing Johto grass behavior. Twelve
appended metatiles replace only TALL_GRASS quadrants with their corresponding
native floor. Every wall quadrant, glyph and collision remains exact.

| Alias | Grass → native floor |
| --- | --- |
| mn_winter_border_pines | 37/3d → 01 |
| mn_winter_border_pines_south | 37 → 01 |
| mn_tamarack_thaw | 88 → 87; b1 → b0 |
| mn_winter_iron_descent | 03 → 02; 81/83/84 → 86/87/88 |
| mn_iron_descent_thaw | 03 → 02; 88 → 87; ac → ab |
| Same descent, green cliff grass | 81/83/84 → ce/cf/d0 |
| Same descent, snowy cliff grass | a5/a7/a8 → d1/d2/d3 |
| Same descent, thawing cliff grass | c9/cb/cc → d4/d5/d6 |

The three cleared south-cliff crowns have FLOOR/FLOOR/WALL/WALL collision.
The renderer's generic building-eave heuristic wrongly treated that floor as
an overhanging facade and hid part of the player. The exact four authored
cleared crown blocks now keep ordinary floor presentation. Building eaves retain
their existing foreground behavior; wall collision and source pixels are unchanged.

## Review and publication

```sh
PYTHONPATH=.local/ely-backroute-python python3 \
  planning/minnesota/tools/review_inland_grass_cut.py \
  --output planning/minnesota/renders/inland-grass-cut-source-FRESH
CARGO_TARGET_DIR=target/verdigrim-scene RUST_MIN_STACK=67108864 \
  cargo build --locked -p crystal-assets --bin repair_inland_grass_cut
target/verdigrim-scene/debug/repair_inland_grass_cut \
  content-packs/minnesota-descent-rock-clearance-v01.crystalpack \
  d63518642625850a43223b82612a5b3530037205bdab59b55012b443bd8faac4 \
  planning/minnesota/revisions/inland-grass-cut-v01/source-v02.json \
  content-packs/minnesota-inland-grass-cut-FRESH.crystalpack
```

Serialize Cargo with `.local/minnesota-cargo.slot.lock`; generated packs remain
ignored and each publication uses a fresh path. The source review takes
0.178 seconds after pack load. All original images, converted views and rejected
passes remain in ignored render history.

`renders/inland-grass-cut-installed-v1158/receipt.json` proves all743 complete
maps, all original glyph/clock data, tree rules and602 audio entries exact.
Only four owned runtime files change: the two R10 alias collision tables and
metatile layouts. The independent source/installed composition has zero pixel
differences; 705,536 original prefix pixels and every non-grass quarter remain
exact. Parent/source-wall negative probes refuse before writing a pack.

## Native input and visual evidence

Build the portable Rust fixture writer and the current production stdio client:

```sh
CARGO_TARGET_DIR=target/verdigrim-scene RUST_MIN_STACK=67108864 \
  cargo build --locked -p crystal-runtime --example inland_grass_cut_fixture
CARGO_TARGET_DIR=target/verdigrim-scene RUST_MIN_STACK=67108864 \
  cargo build --locked -p geothite
PYTHONPATH=.local/ely-backroute-python python3 \
  planning/minnesota/tools/check_inland_grass_cut_mcp.py \
  --pack content-packs/minnesota-inland-grass-cut-v01.crystalpack \
  --fixture-writer target/verdigrim-scene/debug/examples/inland_grass_cut_fixture \
  --client target/verdigrim-scene/debug/geothite \
  --output planning/minnesota/renders/inland-grass-cut-gameplay-FRESH
```

The completed course in `renders/inland-grass-cut-gameplay-v1171/` passes
**32 sessions and924 production RPC calls**, with byte-empty stderr. Twelve
actually placed styles include a native green control. Tests cover no-move and
no-Hive refusals, native party-menu Cut, exact saved floor overrides, walking
onto cleared floor, every adjacent wall of the three placed cliff pieces,
Continue, native regrowth and ordinary re-cutting. Party PP/HP, money and badges
stay unchanged. Local VOYAGEAR, move, Hive Badge, repel, held Smoke Ball and
positions are declared pre-input setup, not earned campaign progression.

Before using the client with these format26 fixtures, rebuild it. The older
coastal cached client cannot read them. A renderer loading a post-Cut save also
performs Continue and regrows grass, so such a screenshot cannot prove the live
cleared appearance. Correct captures run ordinary Start/A/A/A/B/B/B/B in the
same native process, without a post-Cut reload or gameplay-state rewriting.

The 24 final matched daylight originals and twelve reviewed pairs are retained
in `renders/inland-grass-cut-matched-native-v1181/`. Fixtures explicitly start
at hour11 and use the recorded `Etc/GMT-12` QA timezone to keep the native RTC
in the same clock palette; before and after both settle256 ordinary input frames.
Weather/animation phase can differ. The older v1175 pairs have mismatched night/day
palettes and remain rejected comparison history.

The actual Cut-and-step captures in `renders/inland-grass-cut-standing-native-v1185/`
exposed the south-crown player-occlusion bug. Corrected native captures in
`renders/inland-grass-cut-crown-fix-v1190/` were inspected at all three actual
cleared cliff floors; the player is fully visible and cliffs remain intact.
The self-contained regression preserves normal building eaves and the full
installed collision catalog still passes the production priority check.
All-clock Tamarack/descent assets, new Headwaters assets, weather and alias
regressions pass in `.local/headwaters-regressions-v1191/`.

The native capture loop is reproducible with
`planning/minnesota/tools/render_inland_grass_cut_native.py`; pass the pack,
built fixture writer, current `render_at_location`, gameplay `cases.json` and a
fresh output directory. Build the renderer with `--features location-tester`.
All captures are offscreen. This scoped repair does not establish complete
regional terrain/cave quality, full campaign progression or final pack selection.
