# Descent cliff and boulder clearance

Three boulders were replacing the west face, terrace floor and southwest corner
of the tall eastern ridge on MN_R10. This restores the continuous original
cliff and moves those rocks onto the upper terrace. All seven existing route
boulders remain. Six metatiles change in `MnR10C00R00`; the public road, both
stairs, two cave mouths, grass, water, actors, rewards and story remain exact.

## Exact authoring and continuation parent

[`source-v01.json`](source-v01.json), SHA256
`4423204b844096d957cd20c8878a3ec0c8b00dd2d5afd53f0f8c78303e0fa51b`, binds
the complete original 32×20 module, before/after rows and six coordinate edits.
It uses the installed `mn_iron_descent_thaw` art without creating new graphics.
The original source proposal status is retained; verification is recorded here.

| Metatile | Before | After | Purpose |
| --- | --- | --- | --- |
| (25,7) | af | ba | Native west face, original block68 |
| (26,7) | af | c3 | Ordinary upper floor, original block71 |
| (25,8) | af | be | Closed southwest corner, original block6c |
| (26,3) | c3 | af | Boulder set inward from the west face |
| (27,4) | c3 | af | Boulder on the upper floor |
| (26,5) | c3 | af | Boulder clear of the lower corner |

Coordinates are 32px metatiles. The restored corner has
`[WALL,FLOOR,WALL,WALL]`; the side face has `[WALL,FLOOR,WALL,FLOOR]`.
They permit movement on the upper floor while closing west/downward exits.
No ledge is added. The upper ridge remains disconnected from the public road;
the local upper-rim QA placement does not imply a new public entrance.

Parent: `content-packs/minnesota-iron-descent-snowline-v01.crystalpack`, SHA256
`83aae389479014f62a12bfd2f97d494972a27ca3c2b7fd796b9c4bea9ac3874f`.
Child: `content-packs/minnesota-descent-rock-clearance-v01.crystalpack`, SHA256
`d63518642625850a43223b82612a5b3530037205bdab59b55012b443bd8faac4`, content
identity `25289671aed4bf70089f7dc95bc1b3cb110935c55b5f5d1cdb12207bc234e826`.
This scoped candidate retains the previous Bemidji,
ten-map winter belt, R09 clearance, Tamarack snowline and Duluth thaw. It remains
**unselected**. `world.json` still selects `north_shore_passing_width_revision`;
no running client or browser release was changed.

## Reproduce the fast whole-route review

```sh
PYTHONPATH=.local/ely-backroute-python python3 \
  planning/minnesota/tools/review_descent_rock_clearance.py \
  --output planning/minnesota/renders/descent-rock-source-FRESH
```

Choose a fresh output directory. The measured loop is **0.129 seconds after
pack load**. It renders the entire 48×20 route before/after, both chunks and
the cliff detail, preserving full originals and conversions. The read-only
source review never writes a game pack. The original review is retained in
`planning/minnesota/renders/descent-rock-source-v1138/`.

The conservative Floor/Grass audit proves the public road's entire reachable
set is unchanged, all remaining original upper floor stays connected, both
stairs/endpoints remain reachable and the upper ridge gains no accidental side
entry. Only the twelve walking cells deliberately occupied by the moved rocks
leave the old floor set. Grass, water, Cut, ICE, ladder and cave quadrants remain
exact. Native input provides the separate directional-collision proof below.

## Native publication and checks

Rust-only publication uses the pinned toolchain and serialized Cargo slot:

```sh
CARGO_TARGET_DIR=target/verdigrim-scene RUST_MIN_STACK=67108864 \
  cargo build --locked -p crystal-assets --bin revise_descent_rock_clearance
target/verdigrim-scene/debug/revise_descent_rock_clearance \
  content-packs/minnesota-iron-descent-snowline-v01.crystalpack \
  83aae389479014f62a12bfd2f97d494972a27ca3c2b7fd796b9c4bea9ac3874f \
  planning/minnesota/revisions/descent-rock-clearance-v01/source-v01.json \
  content-packs/minnesota-descent-rock-clearance-FRESH.crystalpack
```

The executed publication passes strict readback. Source/parent/module guards,
six explicit edit/collision checks and a full residual comparison reject other
changes. Wrong-parent and unlisted-source probes refuse without writing a pack
(`.local/descent-rock-guards-v1146/`). Builder and all-target crystal-assets
Clippy pass, with no warnings introduced by this leaf. Owned Rust formatting
passes; whole-workspace formatting still reports pre-existing differences.

`renders/descent-rock-installed-v1143/receipt.json` confirms all **742 other
complete maps**, every native runtime asset, NPC/program/encounter/reward and all
**602 audio records** exact. The changed chunk's 655,360 pixels match the
independent parent-art explicit-row composition with zero differences. Image
changes occupy exactly the six declared metatiles.

`renders/descent-rock-route-v1141/receipt.json` records **3,056 production
stdio-MCP calls** through both chunks, both stairs in both directions, the
central habitat and the real Grand Rapids and Duluth seams, returning after
Continue. Four wild encounters use native Run. Stderr stays empty. Initial
legal level80 Minnesota partners, eight badges, signal, position and supplies
are declared setup; no Surf or state rewriting is used after input begins.

`renders/descent-rock-physics-v1142/receipt.json` records **80 production calls**
from two independent zero-badge fixtures and their actual Continue saves. The
upper-rim course walks every restored floor quadrant, confirms the moved rock
blocks its new location and proves the west/down cliff sides remain closed.
The lower approach cannot climb through the corner. The original and played
saves, inputs and empty stderr are retained. Upper placement is a local
decorative-ridge fixture, not earned campaign access.

All six original headless GPU frames in `renders/descent-rock-native-v1144/`
were inspected: matched upper-rock and public-face daytime pairs, plus the
public-face night pair. The cliff now continues cleanly through the corner,
the boulders sit on upper floor and night keeps the native rock palette. The
directory retains every original, the derived comparison and visual assessment.

## Remaining regional work

Native grass-Cut behavior is now repaired and reviewed in the
[successor](../inland-grass-cut-v01/README.md). Continue from the newer
[Headwaters snowline](../../regions/headwaters-snowline-v01/README.md), which
also preserves this cliff repair and completes the Leech thaw. Before regional
promotion, finish the Itasca transition, integrate
the current story/Ely-town parent and finish the broader map/cave quality and
combined controller/field-move/native review. This scoped cliff repair does not
establish a complete campaign, old-save override migration or final selection.
