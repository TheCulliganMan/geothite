# Iron Range descent: snow to Superior

The complete 48×20-metatile MN_R10 route now has an authored winter-to-green
transition. The western bedrock terraces match snowy Grand Rapids. An uneven,
broad frost-on-green band follows the middle benches; the eastern Superior
shore and Duluth approach remain native green. This changes presentation only:
all 3,840 collision quadrants, both native stairs, both gallery mouths, actors,
rewards, grass crossings and story conditions are preserved. No new ICE is added.

## Source and current candidate

[`source-v01.json`](source-v01.json) is the frozen complete-row source, SHA256
`83009e35f33336350e22b0d0d5ea2ba56dfcd19cbda2e5a4c73d37bb75b4ef29`.
Its original proposal status is retained; this runbook records later verification.

Parent: `content-packs/minnesota-tamarack-snowline-v01.crystalpack`, SHA256
`13554605a69f58a1ad3855f47bf64ad577ee2623c86742bc75ffb789399ba248`.
Child: `content-packs/minnesota-iron-descent-snowline-v01.crystalpack`, SHA256
`83aae389479014f62a12bfd2f97d494972a27ca3c2b7fd796b9c4bea9ac3874f`, content
identity `9efcd1bf314b20b386d24ed260ec9f63e0ed31da0fc3b586bd29a3b84bebf301`.
The child retains the Bemidji roof/plateau repair, ten inland winter maps, R09
rock clearance and four-chunk Tamarack snowline. Its later
[`descent cliff repair`](../../revisions/descent-rock-clearance-v01/README.md)
is a separately verified successor. Continue from the newer
[Headwaters snowline](../headwaters-snowline-v01/README.md), which retains both
this thaw and the cliff/grass-Cut repairs. All candidates are **unselected**; `world.json` still
selects `north_shore_passing_width_revision`. No app or browser release changed.

| Map | Metatiles | Presentation |
| --- | --- | --- |
| MnR10C00R00 | 32×20 | Snowy western benches, broad contoured thaw |
| MnR10C01R00 | 16×20 | Original green Superior shore and Duluth mouth |

The source's explicit cold/melt regions use whole-route metatile coordinates.
Weather uses whole-route **16px core X**: full snow through X30, then
`snow = clamp((64 - x) / 34, 0, 1)`. Scheduled rain supplies `1-snow` of the
ordinary warm rain strength. The eastern chunk adds 64 to local X and is fully
warm before the actual Duluth seam. This does not freeze water or block paths.

## Native art and masked grass

The source is the exact installed `mn_r10_rockstairs`. New private aliases
`mn_winter_iron_descent` and `mn_iron_descent_thaw` preserve native glyphs,
palette addresses, stair pieces and animations. Both map chunks use the partial
alias. It retains all 134 original metatiles and adds 36 cold plus 36 melt
variants. Original warm glyphs and all animated frames keep their native clock
colors; per-glyph native background addresses prevent the green end being
recolored by the alias's snow backing.

Full grass is native green `03`, snow `88`, melt `ac`. The three existing
half-grass/cliff metatiles require their own quadrant recipe:

| Original | Snow | Melt |
| --- | --- | --- |
| 81 | a5 | c9 |
| 83 | a7 | cb |
| 84 | a8 | cc |

Only TALL_GRASS quadrants receive icy foreground inks. Their original bitplanes
and every priority hole remain exact; the wall quadrants retain their native
glyphs. The melt compositor must never apply its white-ink remap to these grass
glyphs. All eight cold/melt grass blocks select icy rustles, while original
green blocks keep the original Johto rustle cache. The forest border follows
the adjacent original `05`, snowy `8a` or melt `ae` forest; collision is unchanged.

## Reproduce review and publication

Fast source review reads the exact parent, checks every source row/module/art
hash, composes art in memory and preserves every original/crop/conversion:

```sh
PYTHONPATH=.local/ely-backroute-python python3 \
  planning/minnesota/tools/review_iron_descent_snowline.py \
  --source planning/minnesota/regions/iron-descent-snowline-v01/source-v01.json \
  --output planning/minnesota/renders/iron-descent-source-FRESH
```

The measured loop is **0.117 seconds after pack load**. The full source render
is 1536×640 pixels; it is not a cropped route segment. Initial reviewed source
images and the repeated fast loop are retained under
`planning/minnesota/renders/iron-descent-snowline-v1125/` and
`iron-descent-fast-loop-v1132/`.

Native publication is Rust, using the pinned toolchain and serialized Cargo
slot. The executed builder and all-target crystal-assets Clippy passed:

```sh
CARGO_TARGET_DIR=target/verdigrim-scene RUST_MIN_STACK=67108864 \
  cargo build --locked -p crystal-assets --bin build_iron_descent_snowline
target/verdigrim-scene/debug/build_iron_descent_snowline \
  content-packs/minnesota-tamarack-snowline-v01.crystalpack \
  13554605a69f58a1ad3855f47bf64ad577ee2623c86742bc75ffb789399ba248 \
  planning/minnesota/regions/iron-descent-snowline-v01/source-v01.json \
  content-packs/minnesota-iron-descent-snowline-FRESH.crystalpack
```

Choose a fresh ignored output. The builder validates all complete preimages,
collision, private alias files and the full residual pack, then strictly reads
back the result. Wrong-parent and unlisted-source probes refuse without writing
a pack (`.local/iron-descent-guards-v1135/`). Shared art-guard reuse rebuilt the
prior Tamarack pack byte-for-byte identically (`.local/snowline-publication-v1128/`).

## Gameplay and preservation evidence

`renders/iron-descent-installed-v1130/receipt.json` confirms 983,040 source and
installed pixels with zero differences; all 741 other complete maps and all
original art, NPCs, scripts, encounters, rewards and 602 audio records exact.
Every original collision quadrant is retained.

`renders/iron-descent-gameplay-v1131/receipt.json` records **3,083 actual
production stdio-MCP calls**, five wild encounters escaped through native input,
both route chunks and these real endpoints:

| Route edge, core coordinates | Actual neighbor |
| --- | --- |
| R10C00R00 west `(0,10/11)` | Grand Rapids east `(width*2-1,24/25)` |
| R10C01R00 east `(31,24/25)` | Duluth west `(0,44/45)` |

Both terrace stairs pass down and up: `(11,19)↔(11,22)` and
`(33,11)↔(33,14)`. The tour crosses actual habitat between `(45,20)` and
`(57,20)`, reaches Duluth, returns through Grand Rapids, opens Start and repeats
after Continue. Stderr remains empty. Initial healthy legal Minnesota partners,
eight badges, signal, supplies and `(1,10)` placement are declared QA fixtures;
all later travel uses real input, with no state rewrites and no Surf. This does
not prove an earned campaign, cave-interior play or Surf exploration.

The installed-art regression
`iron_descent_keeps_warm_prefix_and_corner_grass_masks_at_every_clock` passes
for all four clock banks, comparing actual warm RGBA/priority buffers, native
animation frames and schedules, corner-grass indices, alpha masks, icy inks and
melt background colors. Weather and exact grass/border selection tests pass.
Tamarack's installed-art regression also passes on this child.

## Remaining review

All eight original native GPU day/night captures were inspected in
`renders/iron-descent-native-v1134/`: western arrival, upper stairs, central
melt saddle and green Superior shore. Snow remains continuous at Grand Rapids;
the mixed grass retains native player masking; the shore keeps green foliage
and ordinary rain. Night uses the native clock banks. The directory retains
all originals, a derived review sheet and the scoped visual assessment.
`.local/iron-descent-runtime-v1133/` records the rebuilt native renderer,
focused/self-contained and installed-art tests and all-target Bevy Clippy.

The existing boulders at C00R00 `(25,7)`, `(26,7)` and `(25,8)` are now moved
inward, with the native cliff/floor/corner restored, in the linked successor.
Both stair paths, gallery mouths and public-road reachable floor remain intact;
native input and Continue verify the restored sides block height changes.
Do not restore the retired CARRIER_BANK_DROP claim: the current exterior has no
active HOP lane. Appended grass now has verified native Cut in the
[grass-Cut successor](../../revisions/inland-grass-cut-v01/README.md), including
cliff preservation and corrected player priority on cleared south crowns.

The wider goal still includes R14/Itasca thaw (the R08/Leech transition is now verified), current
story/Ely-town integration, remaining cave and terrain quality work, combined
controller/field-move/native verification and final selection.
