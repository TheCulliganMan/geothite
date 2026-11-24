# Northern cave detail

The new layouts follow the way Crystal uses the shape of a ledge or a shoreline
to tell the player where to go next. Tohjo Falls makes the water crossing the
purpose of its room. Union Cave's dry path leads past side trips into flooded
areas. Mt. Mortar gives longer trips distinct landing shelves. Dragon's Den
leads toward a distinct destination beyond its lake. The original pictures
and floor notes are in the [Crystal cave reference atlas](../crystal-reference.html).

| Room | What changed |
| --- | --- |
| Winding Cleft | The upper and lower routes fold around an angled fault with a stone tongue. A narrow stair in the scout's view leads to a perch overlooking the real cache island. The earlier crossing stair sits five walking columns farther west. The island is visible from the perch or the western water and still needs Surf for the Full Restore. |
| Gunflint Grotto | Otta's own dialogue now has a visible timber bench beside him. The first stair shows the spring between projecting stones. The low trail descends below a spring inlet and climbs back up on the far side. Both steps matter on foot, and Surf remains a shorter way to the far supply shelf. |
| Ore Line | A notched fault cuts off the old long open descent. The rock beyond the fracture makes the Star Piece a brief additional exploration after opening the seam. The south bank turns around a real pool inlet and gives the lower supply its own ledge. |
| Inner Basin | The dials sit around an uneven lower lake and stepped shore. The eastern turns use darker floor beside the stone face, so the lever approach reads in daylight. Opening the upper sill reveals a separate sanctuary framed around Kensigard's position. The nine terrain mutations use their original operands. |

The scout's conversation position shows the new lookout stair but is still too
far from the rock itself. The overlook is nine walking inputs from the scout's
western interaction tile. Its view includes the full marked rock. The
western approach on the water shows it too.

| Check | Wall finish | Detail |
| --- | ---: | ---: |
| Winding Cleft, west to east / east to west | 64 / 43 | 64 / 43 |
| Winding Cleft return on the ordinary path | 64 | 64 |
| Gunflint far supply, on foot / with Surf | 56 / 30 | 62 / 30 |
| Ore Line exit from inside a restored fracture | 39 | 39 |
| Ore Line Star Piece after Rock Smash | 40 | 44 |
| Basin reachable dry cells, closed / drained | 334 / 459 | 257 / 356 |

Steps count graph moves from arrival to a usable interaction tile, excluding
field move activation and warp input. Both Gunflint stair treads independently
cut the route on foot when blocked. The cast points, trainer ranges, resource
gates, repeat trips and usable wraparound routes are checked by the parent
review. The supplemental comparison checks every walkable quadrant for an
unintended pocket and looks at the largest empty square in each room. The
largest squares in walking tiles go from 5 to 4 in Winding Cleft, 6 to 5 in
Gunflint, 8 to 5 in Ore Line and 6 to 4 in Inner Basin. The Basin is reviewed
both closed and drained in the day and night palettes.

The comparison command reads both compiled archives and writes the native
renders and measurements to a new ignored output directory.

```sh
uv run --with pillow --with cbor2 python \
  planning/minnesota/caves/crystal-detail-v04/review_north_detail.py \
  content-packs/minnesota-cave-wall-finish-v03.crystalpack \
  content-packs/minnesota-cave-detail-v04.crystalpack \
  target/cave-detail-north/compiled-comparison
```
