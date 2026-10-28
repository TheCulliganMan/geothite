# Grand Marais south harbor closure

This is a focused source candidate against the published 681-map state pack,
not a native pack publication. It changes only the twenty metatiles in the
southernmost row of the existing 20×19 Grand Marais town. All are currently
ordinary Water (`35`). Native rock-water edge `39` forms the middle; native
`38` and `3a` close the west/east corners. No source pixels, tilesets, roads,
facades, actors, activities, programs, attributes or connections are changed.

The reachable harbor touches its unconnected south edge at core x0…39,y37.
The proposed rocks give every one of those forty lanes visible `BUOY`
collision. Their corner cells meet actual blocking terrain in the west
`MnR12C01R01` and east `MnMaraisCedarApproach` neighbors. The inner water row
x1…38,y36 remains open. The harbor retains 502 Water core cells. The only
other unmatched native Water edges belong to two fenced R02 drainage channels
with no ordinary bank launch or incoming Surf seam; they are not part of this
change.

`closure.contract.json` records the exact native module/block preimages,
every changed metatile and collision cell, corner joins, native recipes and
source checks. `MnGrandMarais.candidate.layout.json` is the complete town
layout, not just the edited patch. It compiles to the exact candidate block
array using the existing layout rasterizer. `review.py` reads the current
pack, checks all original anchors, compares closed/open Minneapolis gate
geometry and re-renders without publishing or modifying a pack:

```sh
.local/minnesota-review-env/bin/python \
  planning/minnesota/connections/grand-marais-harbor-closure-v01/review.py
```

Full-town originals, differences and exact 160×144 player cameras are under
`planning/minnesota/renders/outdoor-connections/grand-marais-harbor-closure-v01/source-v01/`.
Every PNG revision is retained by the render archive. The water camera uses
the existing native Surf sprite as a source review marker; it is not a GPU
capture or controller proof. The full town and both camera views have been
personally inspected.

Before native acceptance, apply these bounded cells through a guarded Rust
delta, recheck unchanged content, capture the actual town and water edge, and
use the production controller to launch Surf from shore, approach the complete
rock line and reject southward exit. The 681-map pack remains unchanged.
