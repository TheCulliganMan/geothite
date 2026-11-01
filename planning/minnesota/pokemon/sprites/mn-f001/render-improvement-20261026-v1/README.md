# Grass starter render improvement

The accepted [v2 source sheet](front-back.imagegen-v2.png) was edited with built-in ImageGen and converted by the repository's Rust sprite importer. Pouchid has clearer dark eyes and a readable pouch face. Slipperina keeps its thin dancing stem while its hood, visible eye and petal roots read more clearly. Cyprielle has a distinct green face with separated white eyes below its crown, continuous leaf arms, separate hip petals and slipper feet.

All three fronts remain 56×56 and all three close rear portraits remain 48×48. The existing individual four-color RGB555 palettes, species IDs, types, evolution links and stats are unchanged. The full source and all twelve native starter views were visually inspected; independent checks compare PNG pixels, palette-index JSON, 2bpp tiles, immutable archives and the family boards. This is static artwork work; content packs and battle animations were not rebuilt.

The [final prompt](prompt-v2.txt) and [reference mapping](reference-mapping-v2.json) record the accepted edit. The [first draft](front-back.imagegen-v1.png), its [prompt](prompt-v1.txt), the original manifests and every previous source remain saved. Draft v1 improved surfaces but still lost eye separation at native size. The [canonical manifest](../manifest.json) selects the reviewed v2 imports, using source-backed contours and white16/black16 conversion thresholds. Native candidate alternatives and their immutable versions remain locally under ignored `target/starter-render-improvement/MN_F001/`; production frames and their versions are under ignored `target/minnesota-pokemon/native/MN_F001/`.

Rebuild from the repository root with the pinned Rust toolchain:

```sh
CARGO_TARGET_DIR=target/starter-renders cargo run --locked -p crystal-assets \
  --example minnesota_sprite_sheet -- --family MN_F001
```
