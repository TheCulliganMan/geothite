# Minnesota map author requests for the separate tile-art owner

Current city integration proceeds with native source art. These optional future
assets must not modify frozen city sources or publish over the shared pack.
Deliver an additive Rust extension and immutable source/render/collision handoff.

Most useful distinct Minnesota additions, in order:

1. **White-pine / spruce canopy family.** A coherent32×64 two-metatile tree,
   plus32×32 cluster and forest-edge joins. Dark pine silhouette at Crystal
   player scale; normal blocking canopy, with no incidental CUT_TREE quadrants.
   Intended for Boundary Waters portages and northern towns/routes.
2. **Canoe rack.**32×32, recognizable from the same top-down angle as native
   houses/docks; WALL top pair, FLOOR bottom pair for its readable approach.
   Useful at Boundary Waters entry, camps and portage landings.
3. **Trail / portage marker.**32×32 sign-and-post composition with a distinct
   readable gameplay quadrant and safe adjacent standing floor. No new UI or
   automatic event behavior; authoring supplies the actual BG script.
4. **Basalt / ore bank family.** Fitted northern rock edges, corners and face
   courses, compatible with native outdoor cave stairs and HOP ledges. Supply
   exact collision per variant; decorative rock must not imply a climb. Useful
   for Duluth/North Shore and Iron Range, whose geology should look distinct.
5. **Shoreline buoy / tied log boom.**32×32 blockers that visibly close reachable
   Surf edges. Actual blocking WALL quadrants must line up with water edges;
   distinguish the playable opening from a sealed perimeter.
6. **Campsite picnic table and split-log stack.** Distinct32×32 props, preferably
   WALL on the occupied upper pair / FLOOR on the lower approach pair. Native
   benches already exist; prioritize these silhouettes over another bench.

Use four-color8×8 source tiles,16px gameplay quadrants and32px metatiles.
Preserve original source palettes/art; new art has explicit source provenance,
palette maps, native2bpp bytes and collision. No opaque painted concept image
should be labeled native-ready. Keep every generated/rendered/rejected source.

Current aliases: Minneapolis `mn_minneapolis_city` derives from `johto_modern`
with blocks80–98 occupied by25 exact Kanto/park imports; Duluth
`mn_duluth_expanded` derives from `johto_modern`, with80–8b occupied by exact
cave stairs/Kanto brick imports. St Paul uses original `kanto` unchanged.
Do not claim those slots. Original packed city graphics have192 physical glyphs
(two96-glyph banks); imports must preserve every referenced old glyph and its
palette. A separate candidate tileset namespace is preferable until Root checks
remaining VRAM addresses and native animation destinations03/14. No change to
shared collision, active NPC limits, field-move gates or runtime behavior is
authorized by an art-only handoff.
