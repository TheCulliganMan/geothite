# Western component and detached compact placement checks

Exact inspected parent file SHA256 `65cf6f98e65ada839ec5afaa57728bf8289d3a50ba53ba6b9b50f08d3620bb17`. All calculations are in metatiles. No canonical geometry, pack or source is changed.

Excluding every `MnR12*` node and its incident edges, Boundary Waters reaches **17 physical exteriors** by native connections. Its rectangle union bounding box is `(-30, -499, 139, -321)` (half-open). Duluth is **not** in this component. Including native warp links in either direction yields **31 maps**, with the same physical exteriors and 14 attached interiors; there is still no path to Duluth. This is topology, not proof of story/HM access.

## Exact physical component

| Exterior | Origin | Extent | Half-open rectangle |
| --- | --- | --- | --- |
| MnBoundarySafariC00R00 | (75, -499) | (32, 24) | (75, -499, 107, -475) |
| MnBoundarySafariC00R01 | (75, -475) | (32, 24) | (75, -475, 107, -451) |
| MnBoundarySafariC00R02 | (75, -451) | (32, 8) | (75, -451, 107, -443) |
| MnBoundarySafariC01R00 | (107, -499) | (32, 24) | (107, -499, 139, -475) |
| MnBoundarySafariC01R01 | (107, -475) | (32, 24) | (107, -475, 139, -451) |
| MnBoundarySafariC01R02 | (107, -451) | (32, 8) | (107, -451, 139, -443) |
| MnBoundaryWaters | (66, -427) | (32, 24) | (66, -427, 98, -403) |
| MnEly | (71, -339) | (20, 18) | (71, -339, 91, -321) |
| MnPreservePineEntry | (74, -443) | (24, 16) | (74, -443, 98, -427) |
| MnR13C00R00 | (66, -403) | (32, 24) | (66, -403, 98, -379) |
| MnR13C00R01 | (66, -379) | (32, 24) | (66, -379, 98, -355) |
| MnR13C00R02 | (66, -355) | (32, 16) | (66, -355, 98, -339) |
| MnR20C00R00 | (2, -433) | (32, 24) | (2, -433, 34, -409) |
| MnR20C00R01 | (2, -409) | (32, 8) | (2, -409, 34, -401) |
| MnR20C01R00 | (34, -433) | (32, 24) | (34, -433, 66, -409) |
| MnR20C01R01 | (34, -409) | (32, 8) | (34, -409, 66, -401) |
| MnRainyRiverIcefields | (-30, -433) | (32, 24) | (-30, -433, 2, -409) |

Attached native interiors (undirected warp closure): `MnBoundaryRootBeerCabin`, `MnBoundaryWatersSurveyShelter`, `MnElyMart`, `MnElyPokecenter1F`, `MnLeagueChampion`, `MnLeagueHallOfFame`, `MnLeagueLobby`, `MnLeagueRoom1`, `MnLeagueRoom2`, `MnLeagueRoom3`, `MnLeagueRoom4`, `MnR13Cave1F`, `MnR20Cave1F`, `MnRainyRiverWarmingHut`.

## Every external connection edge in the original parent

| Source | Direction | Target | Offset |
| --- | --- | --- | --- |
| MnBoundaryWaters | east | MnR12C00R01 | 8 |
| MnR12C00R01 | west | MnBoundaryWaters | -8 |

Warp edges crossing the complete 31-map western travel component: **0**. All exterior-to-interior door/cave/League edges remain internal to that closure. The only severed physical link is the reciprocal Boundary Waters ↔ R12 connection above. No internal offsets, local doors or room coordinates need alteration for a rigid translation. Numeric map identities remain unchanged.

## Detached proposed graphs

Both checks below remove Neck and four retired right/bottom R12 exterior chunks, retain the two left-column R12 names (32×24 plus 32×8), shrink only the top R11 east headland from24 to4metatiles and derive the town origin from native links. All other footprints remain unchanged except the rigid western component placement. These are graph/rectangle checks; candidate terrain, collision seams, north/south story gates and player traversal are not established by them.

### Shortest requested placement

R12 origin `(190, -432)`, town `(226, -400)`, Cedar `(222, -412)` extent `20×12`, Boundary Waters `(158, -430)`. Western rigid translation `(92, -3)`, new component bounding box `(62, -502, 231, -324)`.

Checked all **96 reached exteriors** / **4560 rectangle pairs**: **0 overlaps**, **0 reciprocal/cycle/target errors**. Complete western rigid-translation equality: **True**. All retained original exteriors are reached: **True**.

| Map | Derived origin | Extent |
| --- | --- | --- |
| MnR11C01R00 | (222, -400) | (4, 24) |
| MnGrandMarais | (226, -400) | (20, 19) |
| MnMaraisCedarApproach | (222, -412) | (20, 12) |
| MnR12C00R00 | (190, -432) | (32, 24) |
| MnR12C00R01 | (190, -408) | (32, 8) |
| MnBoundaryWaters | (158, -430) | (32, 24) |
| MnR13C00R00 | (158, -406) | (32, 24) |
| MnR13C00R02 | (158, -358) | (32, 16) |
| MnEly | (163, -342) | (20, 18) |

### Earlier 31×19 approach

R12 origin `(176, -432)`, town `(226, -400)`, Cedar `(208, -419)` extent `31×19`, Boundary Waters `(144, -430)`. Western rigid translation `(78, -3)`, new component bounding box `(48, -502, 217, -324)`.

Checked all **96 reached exteriors** / **4560 rectangle pairs**: **0 overlaps**, **0 reciprocal/cycle/target errors**. Complete western rigid-translation equality: **True**. All retained original exteriors are reached: **True**.

| Map | Derived origin | Extent |
| --- | --- | --- |
| MnR11C01R00 | (222, -400) | (4, 24) |
| MnGrandMarais | (226, -400) | (20, 19) |
| MnMaraisCedarApproach | (208, -419) | (31, 19) |
| MnR12C00R00 | (176, -432) | (32, 24) |
| MnR12C00R01 | (176, -408) | (32, 8) |
| MnBoundaryWaters | (144, -430) | (32, 24) |
| MnR13C00R00 | (144, -406) | (32, 24) |
| MnR13C00R02 | (144, -358) | (32, 16) |
| MnEly | (149, -342) | (20, 18) |

### Earlier 45×19 alternative

R12 origin `(162, -432)`, town `(226, -400)`, Cedar `(194, -419)` extent `45×19`, Boundary Waters `(130, -430)`. Western rigid translation `(64, -3)`, new component bounding box `(34, -502, 203, -324)`.

Checked all **96 reached exteriors** / **4560 rectangle pairs**: **0 overlaps**, **0 reciprocal/cycle/target errors**. Complete western rigid-translation equality: **True**. All retained original exteriors are reached: **True**.

| Map | Derived origin | Extent |
| --- | --- | --- |
| MnR11C01R00 | (222, -400) | (4, 24) |
| MnGrandMarais | (226, -400) | (20, 19) |
| MnMaraisCedarApproach | (194, -419) | (45, 19) |
| MnR12C00R00 | (162, -432) | (32, 24) |
| MnR12C00R01 | (162, -408) | (32, 8) |
| MnBoundaryWaters | (130, -430) | (32, 24) |
| MnR13C00R00 | (130, -406) | (32, 24) |
| MnR13C00R02 | (130, -358) | (32, 16) |
| MnEly | (135, -342) | (20, 18) |

## Implementation caveat

The shortest proposal uses whole-R12 EASTcore63,54/55, which belongs to LOWER C00R01 local63,6/7. Native C00R01east→Cedarwest offsets are−4/+4, while+20/−20 describes conceptual whole-route origins and must not be written on the upper chunk. Cedarwestcore0,14/15 then matches exactly; Cedarsouthcore26/27,23 ↔ townnorth18/19,0 offsets+4/−4. The upper four Cedarwest metatiles touch R12upperchunk, so must be visibly gated rather than left as a nonexistent second west connection. Earlier31×19/45×19 proposals used upperchunkoffsets+13/−13 and need different sourceports.

For any subsequent translation `(dx,dy)`, retain every internal western offset exactly and check all translated component rectangles against every fixed exterior. Touching boundaries are legal; positive-area intersections are not. The inspection script recomputes the entire proposed graph from the guarded current parent.
