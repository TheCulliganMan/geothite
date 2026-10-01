# Lighthouse stone floor finish

The lighthouse walkway uses broad muted slate slabs with restrained, staggered
joints. It is emitted into the existing solid terrain mesh when that terrain is
built. There are no new textures, materials, entities, or per-frame mesh edits.
The slate is deliberately quieter than the native grain/checker pattern, so the
joined masonry, actors, stair openings, and pit edges remain the visual focus.

## Source boundary

The finish applies only in `OlivineLighthouse1F` through `OlivineLighthouse6F`,
with the exact `lighthouse` tileset, a valid 4x4 subtile coordinate, the original
metatile identity, and the exact alternating `$2e`/`$2f` source sample:

- `$27`: the full floor drawing
- `$28`: its upper half; the lower pit stays native
- `$2e`: its upper half; the lower warp carpet stays native
- `$31` and `$3a`: the three floor quadrants; their southeast ladder drawings stay native

These guards were checked against the ignored external
`content-packs/core-modular.browser.crystalpack`. In the six native maps they
select respectively 420, 416, 460, 480, 572, and 484 source cells, before any
model-underlay quads are counted. Every selected source quadrant has native
`FLOOR` collision, but collision is evidence for verification, never the matcher.
No extracted source images or game catalogs belong in this change.

The distinct `$0d`/`$1d` checker chamber on 6F is intentionally outside this
finish. Its furniture, machine, Amphy, and alternate flooring need their own
bounded treatment. Fast Ship shares the tileset but receives no change.
Unknown maps, tilesets, block identities, tile samples, and source phases stay
on the original rendering path.

## Mesh and underlay behavior

The existing interior finish pass first proves a complete, upward-facing,
flat, single-cell quad and its complete source-cell UV rectangle. It chooses the
material from that actual atlas sample, including the `$27` floor sample used
beneath lighthouse masonry. A sampled floor cannot recolor a non-floor
destination unless the existing successful model append marked it specifically
as lighthouse masonry or lighthouse window masonry. Merely sharing a map,
sampling a nearby floor, or having an unrelated prop label is insufficient.

Each slab fragment partitions the original quad at its original height. The
finish preserves support/footing, native collision and warp data, existing
solid geometry, animated surfaces, and all existing wall cutaway ranges. It
does not mark floor cells as authored objects. Rejected surfaces keep their
native UVs and artwork; accepted surfaces use the established untextured-solid
domain. Slab color and joint phase use absolute source-grid coordinates so
cropping or shifting a viewport cannot reset the pattern.

## Verification

Four focused Rust tests cover source/mixed-block exclusions, actual sampled
source and successful-model underlay guards, coplanar area/footing/cutaway
preservation, and matching slab phase across cropped and negative origins.
The integrated checkpoint passes the full 702-test voxel suite (two existing
benchmarks ignored). Current native 4F review verifies slate with the player
cutaway; 6F review verifies that the distinct checker chamber and source
furnishings are retained. These representative captures do not cover the full
movement and camera matrix.

Remaining native review:

1. Compare 1F and 4F against the native cutaway captures. The floor should read
   as coherent low-contrast stone; keep ladders, pits, and the entrance carpet
   clearly distinguishable.
2. Inspect every floor, including the untouched 6F checker chamber and its
   furniture. Walk across a ladder/warp and verify unchanged footing.
3. Orbit and move through the verified lighthouse cutaway. Existing wall
   masks must still reveal the actor; floor fragments must never become
   cutaway-eligible.
4. Capture a Fast Ship room and an ordinary interior as negative controls.
   Verify stable joint placement when changing viewport origin or cropping.

The floor finish does not complete the remaining 6F furnishing work.
