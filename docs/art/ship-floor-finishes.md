# Fast Ship floor finishes

Four source-aware materials replace high-frequency floor patterns in B1F and
its three cabin maps: warm cabin carpet, ivory mess linoleum, blue-gray corridor
panels and a navy border with restrained brass trim. The existing 1F timber
deck, door/ladder drawings, map geometry, scripts and collision are unchanged.

The exact native inventory is 2,424 source cells: 796 in B1F, 544 in the NNW group,
588 in the SE/captain group and 496 in the SW group. All are verified native
FLOOR cells. The pass can also finish an existing donor quad under a complete,
source-proven model; it never creates backing beneath unknown/custom drawings.
Own source identity, donor UV corners, whole-model ownership and live-profile
exclusion must agree. Four-sided crops use world-stable pattern phase.

Each existing surface is partitioned at its original height, with no overlapping
quad layer, extra footing or per-frame geometry/material work. The maximum is
ten triangles per finished source cell. A surface finish adds no object count.

The standalone source check covers the full pack inventory, all 16 room warps,
2,116 material/crop cases, finite colors/normals, exact area, unchanged height
and the triangle budget. Integrated regressions exercise custom/missing/borrowed
samples, donor guards, original solid/cutaway buffers and whole-table backing.
Thirty-one native table/floor views include all 16 ship front/back variants;
room/corridor distinctions and repaired full beds remain intact. B1F's 120
remaining front-wall/cap cells and the PC drawing's own backing are separate
architectural/detail work, not missed floor-mask cells.

Run `python3 tools/check-ship-floor-source.py --pack <external.crystalpack>`
and `cargo test -p crystal-voxel-view ship_floor`. Source assets remain in the
external pack. Measured frame costs and limits are in
[the performance notes](../johto-3d-performance.md).
