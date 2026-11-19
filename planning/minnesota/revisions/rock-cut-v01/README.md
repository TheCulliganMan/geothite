# Route 5 · Rock Cut

The upper trail follows a low limestone spur into the north mouth. At the south
mouth the path divides as it leaves the pines. The left branch climbs the
outdoor stair and the right branch enters the hill. Each sign has a clear
reading spot. The approach through the grass takes four encounter steps.

Inside, daylight marks the two exits. Walk onto the light and continue down
to leave. The stair beside the upper shelf goes both ways. From the Repel,
either of the two worn spots on the shelf lets the player drop to the lower
passage. One loose rock below the stairs holds the Hard Stone. The healer
stands in the wider bend on the way to the south exit.

Crystal's Union Cave supplies the visual reference for a ground-level exit.
The native cave bank pairs a directional floor with a light on the wall just
beyond it. Rock Cut fits those pieces, the rock faces and the stairs to its own
two-mouth layout in a private copy of the installed cave bank. Its floor,
stone, daylight and steps all use the original 8×8 glyphs.

## Build and verify

The Field and Mothlight receipts identify their respective brown-shore
parents and fresh output filenames. Run the Field build and audit from the
repository root.

```sh
src=planning/minnesota/revisions/rock-cut-v01
parent=content-packs/minnesota-brown-shore-edges-v01.crystalpack
output=content-packs/minnesota-rock-cut-trail-v01.crystalpack
CARGO_INCREMENTAL=0 nice -n 14 cargo run --locked -j1 -p crystal-assets \
  --example pack_outdoor_contours -- --rock-cut "$parent" "$src" "$output"
uv run --with cbor2 --with pillow python "$src/review.py" "$parent" "$output"
```

The builder checks the step onto each interior threshold, the next outward
input, the exterior arrival and the return to the cave. It also exercises the
stair, both drops, and the inherited regional water, walking and cave checks.
The independent audit compares both complete pack payloads against the authored
changes, checks grass and original shore tiles, and traces access to both items,
the healer and both exits with the pickup present and after collection.
