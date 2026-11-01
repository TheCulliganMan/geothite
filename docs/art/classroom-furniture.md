# Earl's classroom furniture

`mesh/native_classroom.rs` binds complete, authenticated lab drawings in
`EarlsPokemonAcademy`: eight desks, sixteen chairs, one lectern, and the rear
wall/blackboard course. Source edits, cropped drawings, other maps and custom
owners must not acquire an inferred replacement. Side returns borrow plain
wall paint, never repeat the blackboard.

The blackboard's original 48×16 source-pixel frame and lettering sit on a
shallow mounted body, 0.75 source pixels forward of the wall. Closed sides and
back use the source frame's charcoal swatch. Its lower ledge projects two source
pixels and is half a pixel thick. The board stays inside the existing
32-source-pixel wall height. The remaining wall pixels retain their original
plane and UVs; source claims, floor support, desks, chairs and seating poses are
unchanged. Lettering remains the original source artwork.

The external-pack classroom regression checks complete drawings and source
ownership, every front wall/board UV and its depth, the thin overall extent,
outward triangle winding, source mutation rejection and unchanged footing.
The cabinet-fit, seating and dining-table controls also pass: 18 distinct
selected checks in the 2026-10-19 private current-source test build. Standard
Cargo filters are `classroom`, `seating -- --include-ignored`, and `dining_table`
for `crystal-voxel-view`; supply the completed external pack through
`CRYSTAL_RENDER_TEST_PACK`.

Ignored native evidence is under `target/kanto-resume/school-room-review/`.
Front and both side comparisons use matched 2D/3D state, offscreen image targets
and mute. Front and east before images establish the former flush drawing.
These views verify the board finish and preserved furniture arrangement, not
every classroom interaction or the complete interior review.

The fresh audit covers 388 maps and 288 script-state variants with no mesher
errors. Against the preceding 311-map/225-state audit, only Earl's geometry
changes (+22 textured triangles, +44 vertices); all source/support fields and
the 225 original variant records are exact. The other 77 maps and 63 states
are additional audit coverage, not additional visual acceptance.
