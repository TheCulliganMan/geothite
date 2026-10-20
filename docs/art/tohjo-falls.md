# Tohjo Falls

The 3D view reads both complete cascades from the external pack's native cave
drawings. Their shared upper basin sits one source cell below the enclosing
rock tier: water at height 32 beneath the 40-pixel island crown. The receiving
water stays on the cave floor at zero. The closed island approach and its rim
join that crown; lower paths and exits retain their existing support. A clipped
cascade without a proven upper pool keeps its source fallback.

Each cascade uses its original UV footprint once over a curved sheet with
level ends. Moving Surf contact samples the same quarter-cell triangles. The
sheet owns its landing without a second overlapping water plane. Authenticated
source animation drives the ribbons, basin ripples and foam; the foam footprint
covers the whole landing lip and continues into the receiving pool. Native
water palette samples supply the color. Other caves retain their water style.
The island's low sculpted rock keeps its full surface at the upper walking
tier; taller walls retain the normal player reveal from the lower water.

Review `TohjoFalls (15, 8)` at noon with `render_at_location`, `--view both`,
orbit `0` and `1`, zoom `0`, against the paired 2D reference. Use
`CRYSTAL_RENDER_TEST_HEADLESS=1` and
`CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image`; `CRYSTAL_TILT_SHIFT=off` keeps the
whole scene sharp for inspection. Captures belong outside tracked source.

The external-pack regression checks both cascades, their basin levels,
continuous moving contact, the complete island approach, unchanged lower paths
and the cropped-source fallback:

```sh
CRYSTAL_RENDER_TEST_PACK="$PWD/content-packs/text-tui.crystalpack" \
  cargo test --locked -p crystal-voxel-view --test tohjo_falls
```
