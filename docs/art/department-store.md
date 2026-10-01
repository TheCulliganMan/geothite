# Department store architectural courses and closed display islands

This original kit refines the two stores' six shared floor layouts. It does not
change runtime maps, collision, events, warps, actor support, inventory or shop
logic. The eight cached prototypes use the existing bounded gzip-in-JSON model
storage and are appended into the existing terrain batch.

## Source ownership and honest coverage

The external `content-packs/core-modular.browser.crystalpack` was inspected in
memory. Source images remain temporary review material, outside shipped source.
The adapters embed only source identity hashes, sparse masks and placement
metadata, not a second content catalog.

| Floor, in both stores | New kit wall ownership per map |
| --- | ---: |
| 1F | 56 |
| 2F | 28 |
| 3F | 52 |
| 4F | 36 |
| 5F | 52 |
| 6F | 36 |

The wall kit owns 520 source cells across 12 maps. Older `WindowWall` signatures
recognized patterns for 416 of these cells, but required a missing floor sample;
the prior production audit recorded no store-map window ownership. The joined
kit uses the verified native floor and now consumes all 520 cells. The other 104 add
lift-side fluting/call-panel housings, live directory housings and complete
counter/wall junctions. Resolving this kit before filtering old interior
placements is essential to avoid duplicate windows or a reservation conflict.

The earlier 704-cell wall family estimate also contains 184 cells deliberately
left on their existing renderer: 96 elevator door/threshold cells and 88
escalator/stair cells. Do not count those as newly converted wall geometry.
Elevator cabins, the roof and B1F are outside this kit.

Each 5F assembly is one 8×8-source-cell object at source anchor `(12, 8)`.
Its 56 owned cells comprise the entire closed lower 8×4 display (32 cells),
plus 24 connected counter cells above. Across both maps, the kit owns 112 cells:
all 64 lower-display residuals plus 48 existing counter-drawing cells.
The upper 4×2-cell inset is genuine FLOOR occupied by staff; it remains unclaimed
and free of authored geometry. The entire lower half is WALL and remains a
closed volume, including the dark central display lid. It must never acquire
an invented walk-through U opening. No geometry extends onto the approach carpet.

## Art and placement

Joined cream paper frames, a continuous cornice and ochre toe course surround
recessed teal glazing. Separate lift-side ribs and copper directory rims retain
their source-sized silhouettes. The directory's four source tiles and the
lift call-button tile are sampled from the current atlas onto outward faces;
lettering is not baked into the original authored assets.

The island has a connected sales rail, two returns, a sealed lower display lid,
four shallow drawer/glazing fronts, copper pulls and a continuous dark toe.
Its maximum height is the existing eight source pixels. Wall courses remain
16 pixels high and three pixels deep, entirely within the native blocked north
row. Models use exact source widths rather than stretching one pane across a
long run. The staff floor inset stays at its original support height.

All marked geometry and live plaques participate in the existing camera-aware
cutaway. Underlays sample the same-atlas native `mart` block `$04`, tile `$01`.
Custom live profiles take priority. A changed tileset, block, subtile phase,
tile identity, map, source origin or incomplete guard rejects the entire
corresponding network. The unchanged old renderer is then responsible for it.

## Rebuild and checks

```
python3 tools/build-department-store.py target/department-store --runtime-only
blender -b --threads 2 --python tools/build-department-store.py -- target/department-store
python3 tools/check-department-store.py --pack content-packs/core-modular.browser.crystalpack
cargo test -p crystal-voxel-view department_store
```

Runtime assets are deterministic products of the named editable parts in
`tools/build-department-store.py`. The compressed editable Blender source belongs
in the existing source manifest as `department-store.blend`.
The Python check verifies closed oriented parts with positive volume, valid
normals, unchanged source identities and collisions across all twelve maps,
exact warp exclusion, the staff-floor inset, the sealed lower centre and the
editable-source hashes. Rust tests cover rejection at every source field,
native padded/scrolled frames, cropped guards, custom profiles, atomic append,
non-square scale/normal fitting, live inscription UVs and unchanged footing.

## Native review still required

Studio front/rear reviews verify the original model shape and materials only.
Before art signoff, run the Rust tests and production coverage audit, build the
native client, and inspect the integrated geometry in its actual lighting:

- Capture both stores' 1F and 5F; include 2F/4F shelves and 6F vending/stairs to
  check the shorter wall runs. Confirm no doubled panes, underlay patches or
  seams through fixture silhouettes
- On 1F, approach the receptionist from the native counter side and advance her
  text. On 5F, stand at gameplay `(8, 3)`, face south and interact across the
  native counter to the clerk at `(8, 5)`. Verify shop opening and closing,
  and that the staff remains grounded in the two-square floor inset
- Read each directory from `(14, 1)` facing north; read the lift call panel from
  `(3, 1)` facing north. Check the source lettering remains readable under both
  camera directions and cutaway
- Enter the lift through gameplay `(2, 0)`, travel and return. Use every existing
  stair warp in the inspected floors, including Goldenrod's 6F roof connection
- Orbit both sides, approach the island, and confirm that the apparent lower U
  centre remains physically closed while the existing staff-floor inset stays
  clear. No new collision/footing behavior is implied by a visual capture

Integrated voxel tests and the all-map audit pass. Ten native views cover both
stores on floors 1, 2, 4, 5 and 6; the ordinary 5F-to-4F stair route also passes.
Shop/lift interactions and further camera/movement routes above remain separate
checks.
