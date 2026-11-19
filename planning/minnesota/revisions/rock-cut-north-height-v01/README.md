# Rock Cut north entrance

The [statewide cave revision](../crystal-cave-faces-v01/README.md) supersedes this archived design.

The north doorway retains its full 16-pixel opening. The front of the limestone
beside it is one 8-pixel course high. Both sides of the doorway use the same
stone at the same height. Pines close the back of the bluff. Its side walls
meet the trail at the base, and a grassy shoulder separates the eastern
corner from the existing stairs. The stone above the doorway is inaccessible
from the road.

## Build and check

Start with the matching Field or Mothlight Rock Cut trail pack. The receipts
identify the exact input and output for each. Run the Field version from the
repository root using a fresh output filename.

```sh
src=planning/minnesota/revisions/rock-cut-north-height-v01
parent=content-packs/minnesota-rock-cut-trail-v01.crystalpack
output=content-packs/minnesota-rock-cut-north-face-v01.crystalpack
CARGO_INCREMENTAL=0 nice -n 14 cargo run --locked -j1 -p crystal-assets \
  --example pack_outdoor_contours -- --rock-cut-north-face "$parent" "$src" "$output"
uv run --with cbor2 --with pillow python "$src/review.py" "$parent" "$output"
```

The compiled-pack audit checks the visible stone course, the entrance and its
return, the stair lanes and the road past the trainer. It compares the rest
of the pack with the parent, including the cave interior and southern trail.
