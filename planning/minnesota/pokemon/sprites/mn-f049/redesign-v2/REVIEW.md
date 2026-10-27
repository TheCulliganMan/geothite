# Native review

Selected manifest: `manifest-selected.json`.

Selected native outputs: `target/minnesota-pokemon/proposals/MN_F049-v2-solid-white4-ink12/`.

The front is 56×56 with 784 native 2bpp bytes. The back is 48×48 with 576 bytes. Both PNGs contain exactly the four declared RGB555 colors. Native size and the 8× review were inspected: the sloped crown, sleepy eye slots, enclosed curled shadow, inward arms and distinct feet remain readable.

The source is 1161×1355 with the row split at y=693 in the clear gutter. The close back crop starts x=250 and covers 800 source pixels; it enlarges the crown and upper shoulder while cropping the far shoulder, feet and lower body. No adjacent front ink enters the back crop.

Compared outputs retained under `target/minnesota-pokemon/proposals/`: Solid white4/black12 (selected), Solid white4/black8, SourceBacked white4/black12 and SourceBacked white2/black12. No pixels were manually repainted after generation.

Subsequently integrated at root's request on 2026-10-26. The canonical family manifest and `target/minnesota-pokemon/native/MN_F049/` now use both selected views. The full-family review hash matches the manifest; the current production Rust graphics loader accepted all seven runtime files. Identity, mythical rarity, Ice/Psychic types, profile fields, stats, moves and palette values remain unchanged. The prior canonical manifest and native versions are preserved.
