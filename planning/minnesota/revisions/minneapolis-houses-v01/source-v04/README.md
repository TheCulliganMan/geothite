# Minneapolis house tile joins — v04

The reported assembly defects are corrected in the published native pack:
house endcaps no longer appear inside wide façades, upper storeys no longer
repeat ground-floor foundations, and two-wide masonry fronts have right caps.
The rejected v03 shape/variety pilot was never published.

Native door block `1a` carried a left wall into interior/right door positions;
repeated roof block `18` repeated its left cap; ground-floor block `20` repeated
its side cap and `dd`/`df` foundation above ground. The rule is **outside caps
only at the actual sides, continuous wall/window rows upstairs, foundation only
at ground level, and the real door quadrant preserved**. Inspect joined façades
at 8px glyph scale rather than treating whole ground-floor blocks as upper walls.

The complete forty-front audit changes **83 cells across 32 fronts**, using
**12 appended recipes from existing logical glyph IDs**. Bitmap glyphs,
palettes, original recipes, all 5,120 city collision quadrants, doors/events,
rooms/scripts, other maps, audio and recovery remain unchanged.

## Published pack and evidence

- Pack: `content-packs/minnesota-house-joins-v04.crystalpack`.
- File SHA-256: `63117daf68439663d7632293f8bded7352bf8e39b96d30edcc14f035871d4d6b`.
- Content identity: `d2a3829ec3caa1f684a011b6004891b48aa65420a741c99795b0c89e1a4f984c`.
- Parent file SHA-256: `57eefb675b9cb32ac66a01819f56961392e51678a6f9dd959888fbb9427e4e62`.
- [Frozen contract](native-joins.contract.json) SHA-256: `6d58489a641a58173a68dc598cd774b9fcf3ddaaaeac974f13c8fdb7d7a49a85`.

Publication performs strict written readback. Three focused asset tests pass
in 11.40s (`.local/house-joins-assets-v04-exact.log`); the earlier zero-test log
does not count. Four renderer-guard unit cases pass, with two other external
fixtures ignored (`.local/house-joins-animation-tests-v04.log`). The actual
house-pack waterfall pixel/schedule regression passes in 5.61s
(`.local/house-joins-native-animation-v04.log`).

Independent written-pack verification passes **793 checks and 13 corruptions**:
643,072 old-art pixels and 49,152 new-recipe pixels match across four palette
banks; the whole city matches 1,310,720 source pixels; all forty door images and
all collision/event data remain identical. Proof is
`renders/minneapolis-houses-v01/independent-joins-v04/written-v02/proof.json`,
SHA-256 `7d4a47487d23b2a4b634edc634936d98453c8b49da632a2db79aa203c3ffd4f4`.

Root visually accepted the reported assembly corrections in three actual
headless 640×576 captures from this pack at noon: Boat House `(9,30)`, Supply
Coop `(7,10)` and Northeast Home `(74,10)`. They are retained under
`renders/minneapolis-houses-v01/native-joins-v04/gpu-v02/`. This is manual native
visual review; no zero-mismatch GPU comparison is claimed. The separate zero
pixel differences above concern independent source-pixel verification.

The renderer was rebuilt in 2m18s, binary SHA-256
`2485b3a36b4bd15b761ebcf8900b605a8135aab35119ff2cff78d9eb652f014d`.
Initial GPU attempts failed an older whole-recipe-only renderer guard and remain
saved as failures. Its bounded correction permits rearranged original glyphs
while rejecting new glyphs and private waterfall-glyph reuse.

Full pack, binary, capture and log hashes are recorded in
`renders/minneapolis-houses-v01/native-joins-v04/final-assembly-verification-v03.json`.
Source originals, annotations, all eighty frontage crops and 8× detail pairs
remain in `renders/minneapolis-houses-v01/join-correction-v04-review-v03/`.

## Executed command arguments

Root owns shared Cargo and publication. The publisher refuses an existing
output; preserve this pack and select a fresh path for any new publication.

```sh
target/runtime-fps/wasm-dev/build_minneapolis_house_joins \
  content-packs/minnesota-city-river-bend-v01.crystalpack \
  planning/minnesota/revisions/minneapolis-houses-v01/source-v04/native-joins.contract.json \
  content-packs/minnesota-house-joins-v04.crystalpack

CRYSTAL_MINNEAPOLIS_JOINS_TEST_PACK="$PWD/content-packs/minnesota-city-river-bend-v01.crystalpack" \
  RUST_MIN_STACK=33554432 \
  target/runtime-fps/wasm-dev/deps/crystal_assets-51037f981d106d3a \
  minneapolis_joins_ --include-ignored --test-threads=1
```

Scoped changed-Rust formatting passes. Workspace formatting has unrelated
pre-existing drift (`.local/house-joins-workspace-fmt-v04.log`). All-target
Clippy for both affected crates is blocked by the existing
`crystal-core/src/multiplayer/mod.rs:1332` `clippy::bad_bit_mask` error
(`.local/house-joins-both-crates-clippy-v04.log`); it is not a passing lint result.

Acceptance covers the reported tile-assembly defects. Broader city composition
and the all-routes Johto-quality objective remain unfinished. This cosmetic
change preserves native events/collision; it makes no new earned-progression
claim and does not require fresh doorway input to prove unchanged gameplay.
