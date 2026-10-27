# Native review

Selected manifest: `manifest-selected.json`.

Selected native outputs: `target/minnesota-pokemon/proposals/MN_F080-v4-source-white4-ink12/`.

Both fronts are 56×56 and 784 bytes of native 2bpp; both backs are 48×48 and 576 bytes. Every PNG contains exactly its species' four declared RGB555 colors. Native size and the 8× review were inspected. Upper false faces, dark lower bodies and real white eyes remain distinct. The evolved false mask has substantially greater width and extra refractive bands.

The source is 1312×1199. A column split at x=590 excludes adjacent artwork in both rows; a row split at y=633 falls in the clear gutter. The back portraits enlarge the rear upper mask and upper true body, cropping the lower mass at the bottom.

Compared outputs retained under `target/minnesota-pokemon/proposals/`: SourceBacked white4/black12 (selected), SourceBacked white2/black12, Solid white4/black12 and Solid white4/black8. No pixels were manually repainted after generation.

Subsequently integrated at root's request on 2026-10-26. The canonical family manifest and `target/minnesota-pokemon/native/MN_F080/` now use all four selected views. The full-family review hash matches the manifest; the current production Rust graphics loader accepted all 14 runtime files. Only the roster's visual-design text changed to describe the displaced false face and true lower body. Profiles, types, traits, stats, moves and palette values remain unchanged; the prior family design and canonical manifest are preserved.
