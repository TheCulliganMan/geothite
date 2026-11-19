# Brown shore rocks and Route 5

The small brown rocks once again follow the waterline on Minnesota's routes
and lakefronts. Restoring the 95 water-edge blocks and 64 adjoining land
blocks from before the rocky-shores revision also removes its freestanding
stones. Each restored block has the same movement permissions as the one it
replaces. Fishing, Surf and dismounting work at the rocky rim.

On Route 5, three stairs in the backs of the terraces use a rear rim and an
upper landing. Their sides join the cliff on the same level. The 27 changed
map blocks also correct the eastern ridge turn and the northern inlet. Its
southern rim follows the dry side of the bank, with open water across the
channel. The Rock Cut entrances lead to their original underground destinations.

## Build and check

Start from the selected rocky-shores pack. The Field and Mothlight receipts
identify the exact parent and result for each version. The builder takes a
fresh output filename. Run these commands from the repository root for Field.

```sh
src=planning/minnesota/revisions/brown-shore-edges-v01
original=content-packs/minnesota-cliff-faces-v01.crystalpack
parent=content-packs/minnesota-rocky-shores-v01.crystalpack
output=content-packs/minnesota-brown-shore-edges-v01.crystalpack
CARGO_INCREMENTAL=0 nice -n 14 cargo run --locked -j1 -p crystal-assets \
  --example pack_outdoor_contours -- --brown-shores "$parent" "$src" "$output"
uv run --with cbor2 --with pillow python "$src/review.py" "$original" "$parent" "$output"
```

The Rust builder uses the game's movement rules to check walking, Surf and
fishing, cave entrances and returns, and the authored Route 5 steps. The
independent audit compares the restored banks directly with the original pack's
native pixels, water animation and collision. It also checks every changed
Route 5 block and the map connections. Both lanes cross between all three route
sections and connect back to St. Cloud and Minneapolis. A 161-step dry path
links the two city ends across the southern footbridge.
