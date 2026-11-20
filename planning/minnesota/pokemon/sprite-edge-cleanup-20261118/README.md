# Minnesota sprite edge cleanup

The audit covers all 225 active species (450 front/back views), both the
registered native artwork and the selected Crossings pack. Original source
sheets and previous manifests remain unchanged.

Four registered views include pieces of adjacent creatures from their source
sheet crops. New `edge-cleanup-20261118-v1` manifests use guarded native edits:

| View | Removed pixels | Native bounds, inclusive |
| --- | ---: | --- |
| Scutlet back | 6 | x45–46, y44–47 |
| Dynarill front | 24 | x47–49, y39–51 |
| Emberm back | 11 | x45–46, y41–47 |
| Mudnip front | 8 | x47, y36–39 and y51–54 |

The installed Crossings pack has older art than the registry. Its affected
views are Scutlet back, Dynarill front, Mudnip front, Pikeling front (7 pixels),
Pikeman front (6 pixels), and Crookit front (3 pixels). Emberm's installed older
back is clean. The pack repair removes exactly 54 pixels without switching
any installed creature to a newer design. It updates matching PNG and 2bpp
bits together, guards changed images by their original SHA256, and validates
the resulting pack. Repeating the repair is a no-op.

Fortigard's inset emblem, Shawlter's tassels, floating magical bodies, sparks,
and other intentional disconnected details are retained. No global component
filter is applied to artwork.

Generated review boards, native exports, packs and the local Pokédex candidate
remain ignored. This repair does not deploy a release or change world selection.

## Verified outputs

- Independent native verifier: 20 views, 10/10 species in the four affected
  families pass PNG/indexed-grid/2bpp/palette/source/archive checks.
- Native before/after comparison: exactly 49 pixels removed from four views;
  all other pixels in those families are unchanged.
- Crossings child pack: `target/sprite-wrap-audit/minnesota-sprite-edges-v01.crystalpack`,
  identity `131d2d4d2596df8c3056ed8bf2414346b177278cd59aa1e153d61e23882950e8`.
  Verified pack readback and repeat-repair identity pass. Independent comparison
  confirms exactly 12 changed runtime files, 54 removed pixels, synchronized
  PNG/2bpp indices, identical file keys, and unchanged gameplay/audio/report fields.
- Local Pokédex candidate:
  `target/minnesota-pokedex-web/pokedex/releases/20261118-sprite-edge-cleanup`.
  Only eight sprite images change: the four corrected views in normal/shiny
  palettes. Each removes the same guarded pixels; all other sprite bytes match.
- Both native and installed-pack before/after boards were visually inspected.

Reproduction (all generated output paths must be fresh and ignored):

```sh
CARGO_INCREMENTAL=0 cargo build --locked -p crystal-assets \
  --example minnesota_sprite_sheet --example publish_pokedex_sprite_revision \
  --bin pack_minnesota_sprite_edges
for family in MN_F084 MN_F088 MN_F102 MN_F132; do
  target/debug/examples/minnesota_sprite_sheet --family "$family"
done
RUST_MIN_STACK=67108864 target/debug/pack_minnesota_sprite_edges \
  content-packs/minnesota-crossings-v01.crystalpack \
  target/sprite-wrap-audit/minnesota-sprite-edges-v01.crystalpack
target/debug/examples/publish_pokedex_sprite_revision \
  target/minnesota-pokedex-web/pokedex/releases/20261104-rerender-181-186-45 \
  target/minnesota-pokedex-web/pokedex/releases/20261118-sprite-edge-cleanup \
  MN_182 MN_190 MN_221 MN_291
```

`cargo fmt --all -- --check` and `cargo clippy --locked -p crystal-assets
--all-targets` pass (existing warnings remain outside this change). The focused
regression passes with the real external Crossings pack and proves preservation
and idempotence:

```sh
MINNESOTA_SPRITE_EDGE_PACK="$PWD/content-packs/minnesota-crossings-v01.crystalpack" \
RUST_MIN_STACK=67108864 CARGO_INCREMENTAL=0 cargo test --locked -p crystal-assets \
  --lib sprite_edge_repair_preserves_pack_and_is_idempotent
```
