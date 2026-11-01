# State Fairgrounds native art kit

**Superseded proposal, not accepted fair-map art.** The user subsequently
requested compact tile-based construction in the original Pokémon style,
specifically Pokémon Center / Goldenrod Underground counters with separate
native NPCs. Do not place the large illustrated booth designs into the final
fairgrounds map. Retain this source and every conversion as review history.

Four original placeable objects, made with built-in ImageGen and converted by
the Rust Pokémon-image pipeline's map-specific variant. The art pack is
`content-packs/minnesota-state-fair-props-v5.crystalpack`; it adds the separate
`mn_state_fair_props` tileset. Campaign maps remain unchanged. The mapping owner
can use this kit for the [fairgrounds design](../../zones/state-fairgrounds.json).

![Native cutouts](../../renders/tiles/state-fair-v5/contact.png)

| Object | Native size | Blocks, row-major | Intended district |
| --- | --- | --- | --- |
| Ferris wheel | 64×64 | `80 81` / `82 83` | Midway landmark |
| Corn-roast / food-on-a-stick kiosk | 32×32 | `84` | Food row |
| Cookie-pail stand | 32×32 | `85` | Food row, Doughlin / Crumband |
| Refrigerated butter bust | 32×32 | `86` | Dairy exhibit, Chislet / Buttermuse |

All seven blocks have **WALL / WALL / WALL / WALL** collision in NW, NE, SW, SE
order. Keep the wheel's four blocks together. Its transparent spoke openings
are scenery inside the fenced ride footprint, not walkable floor. Give it a
south loading approach at relative core coordinates `(1,4)` and `(2,4)`.
Each kiosk/display needs adjacent south FLOOR at `(0,2)` and `(1,2)`.

[handoff.json](handoff.json) records the exact assembly rows, relative core
approaches, district uses and integration scope. These are map-authoring items.
The map supplies vendors, menus, demonstrators, stamp flags, queues and prizes;
this art extension does not register inventory items, working rides or doors.
Do not graft these block IDs onto St Paul's existing Kanto tileset. Use the
separate namespace or have the integration owner merge its glyphs explicitly.

## Sources and conversion

The four `*.imagegen-v1.png` files are byte-preserved generated originals.
[prompts.json](prompts.json) contains the exact prompt for each image. Generation
used the built-in tool, with transparent backgrounds. Subjects were requested
separately; no grass, painted ground, labels or vendor characters were requested.

Each original is 1254×1254 RGBA. [atlas.json](atlas.json) places its unchanged
pixels in a transparent 2560×2560 atlas with 13px padding inside each 1280px
cell. The atlas performs no resizing or alpha flattening. Its SHA-256 is
`46e20b2e571a7d8f72820cec21f1c6effa21b0a17b178f5635fc1a832f20e2f1`.
The [catalog](catalog.json) pins this exact source before conversion.

The importer reuses `MinnesotaSheet` PNG decoding and the Pokémon bitplane/PNG
encoders. It samples fixed 32px metatile grids; no creature silhouette fitting
or generated contour is applied. Reciprocal N/E/S/W neighbor declarations
permit the wheel's internal seams only when source crops adjoin exactly.
Unpaired, shifted, missing or exterior seams reject publication.

The butter case uses an aligned 8×8 area vote within each native pixel. This
retains source-backed narrow dark cabinet edges that center sampling missed.
At least 16 of 64 samples must be opaque, and a dark contour requires 12 source
samples. This option requires alpha and an unsplit crop. It does not rescale
the silhouette or paint new contours. Other objects retain center sampling.

Each 8×8 glyph has three foreground indices plus ground zero. The cookie roof
uses a separate red/gold palette from its cream/gold counter, preserving its
red canopy and pale cookie pails. Foreground overrides apply only to newly
allocated glyphs; zero follows the native floor at the current time. Exported
cutout PNGs keep genuine alpha, without ground pixels. Original Johto glyphs,
collision, shared palettes, animations and field-move replacements are retained.

## Review history

The source originals and all five converted passes are retained. Immutable
`catalog-initial-v1.json` and `catalog-conversion-v2/v3/v4.json` recipes identify
earlier passes. Their corresponding packs and renders remain ignored.

- First pass: the wheel read clearly at 64px, but cookie highlights merged
  into gold. Second pass restored highlights but lost too much red canopy.
- Third pass split roof and counter palettes to recover both features.
- Fourth pass tested a 10px crop-phase change for the thin cabinet outline;
  it left an uneven frame and was rejected.
- Fifth pass restored the original crop and used fixed-grid area voting for
  the cabinet. The dark enclosure is now readable on pale day ground.

The review uses original Goldenrod source art for scale comparison. Saved
morning/day/night assembled views and 160×144 wheel-approach crops inspect the
actual packed blocks. The gallery is an art-review layout, not evidence of a
playable fairgrounds destination.

The Ferris-wheel and exhibit theme draws on the State Fair's own
[landmark descriptions](https://www.mnstatefair.org/history/buildings).
The artwork is original, rather than copied photographs or game graphics.

## Reproduce publication and review

Use a new output name if these immutable outputs already exist:

```sh
CARGO_TARGET_DIR=target/agent-minnesota-tiles CARGO_BUILD_JOBS=2 \
  cargo build --locked -p crystal-assets --bin pack_minnesota_tiles \
  --profile wasm-dev --config profile.wasm-dev.opt-level=0
target/agent-minnesota-tiles/wasm-dev/pack_minnesota_tiles \
  content-packs/minnesota-northstar-north-shore-story.crystalpack \
  planning/minnesota/tiles/state-fair-v1/catalog.json \
  planning/minnesota/tiles/state-fair-v1/source.png \
  content-packs/minnesota-state-fair-props-v5.crystalpack \
  planning/minnesota/renders/tiles/state-fair-v5
.local/minnesota-review-env/bin/python \
  planning/minnesota/tools/review_minnesota_tiles.py \
  --pack content-packs/minnesota-state-fair-props-v5.crystalpack \
  --base content-packs/minnesota-northstar-north-shore-story.crystalpack \
  --catalog planning/minnesota/tiles/state-fair-v1/catalog.json \
  --output planning/minnesota/renders/tiles/state-fair-v5/gallery
.local/minnesota-review-env/bin/python \
  planning/minnesota/tools/review_state_fair_tiles.py \
  --pack content-packs/minnesota-state-fair-props-v5.crystalpack \
  --output planning/minnesota/renders/tiles/state-fair-v5
```

Native PNG cutouts and nearest-neighbor previews are in that render directory.
`fair_ferris_wheel.cutout.1x.png` is the assembled 64×64 transparent wheel;
`corn_roast.1x.png`, `cookie_stand.1x.png` and `butter_display.1x.png` are the
32×32 props. Packed 2bpp graphics are embedded at
`gfx/tilesets/mn_state_fair_props.2bpp`, with block rows in
`data/tilesets/mn_state_fair_props_metatiles.bin`.

```sh
CRYSTAL_TILE_TEST_PACK="$PWD/content-packs/minnesota-northstar-north-shore-story.crystalpack" \
CARGO_TARGET_DIR=target/agent-minnesota-tiles CARGO_BUILD_JOBS=2 \
  cargo test --locked -p crystal-assets --test minnesota_tile_art \
  --profile wasm-dev --config profile.wasm-dev.opt-level=0 -- --include-ignored
target/runtime-fps/wasm-dev/verify_crystalpack \
  content-packs/minnesota-state-fair-props-v5.crystalpack --audio --desktop
```

Verification results and native renderer captures are recorded in
[VERIFICATION.md](VERIFICATION.md). Packs, converted art and review captures
remain ignored and are never commit inputs.
