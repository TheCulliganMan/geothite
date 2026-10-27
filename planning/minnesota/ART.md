# Minnesota source art and composition

## Style authority

Study `reference/johto-kanto-original.png` and the native-scale references under `renders/source/`. These are the original game's packed tiles, rendered by the same addressing and palette rules used for the requested atlas. The rejected images invented large conifers, volumetric rocks, shaded barns, waterfalls, and overhead-perspective props. Those assets do not define this pack.

| Property | Required treatment |
| --- | --- |
| Art unit | Original 8 by 8 pixel source tile |
| Movement square | 16 by 16 pixels |
| Map block | 32 by 32 pixels, four by four source tiles |
| Camera | Fixed original top-down map convention |
| Trees | Coherent source metatile clusters with source-scale crowns |
| Houses | Complete original facade recipes with real door quadrants |
| Paths | Source light ground and grass transitions; preserve room to see entrances |
| Water | Original palette, source shoreline corners, existing animation schedule |
| Rocks and ledges | Source wall quadrants and directional collision; no painted height |
| Text | Put planning labels outside playable pixels; in-game text uses the game's font and dialogue |
| Enlargement | Integer nearest-neighbor for review; inspect 1:1 pixels too |

Do not color-correct or filter the source PNGs to match the rejected Minnesota illustration. Do not increase sprite sizes to make the overview more decorative. Source fidelity and route complexity are separate requirements: a complex route gains meaningful playable geography, not larger props.

The user's prairie direction is **yellow grass**. Use the original grass glyphs
with native background palette7 in a regional tileset variant; keep dark stems
and native TALL_GRASS collision. Explicit dry-field strokes select the yellow
lawn. Preserve green trees and wet banks rather than changing a shared green
palette globally. R02 and R16 implement this treatment; see the
[native prairie art and review loop](playable/prairie-grass.md).

## References and what to study

| Packed source map | Tileset | Metatiles | Relevant visual lesson |
| --- | --- | --- | --- |
| Route29 | johto | 30 by 9 | Compact tree borders, grass gates, sign placement and legible parallel lanes |
| Route30 | johto | 10 by 27 | Longer sequence of clearings, modest buildings and restrained water margins |
| Route34 | johto_modern | 10 by 27 | Urban edge meeting natural ground; useful for city approaches |
| Route42 | johto | 30 by 9 | Low earth terraces, lake partitions and cave approaches |
| Route45 | johto | 10 by 45 | Interlocking cliff channels and distinct upper/lower paths |
| NewBarkTown | johto | 10 by 9 | Building-to-street proportions and a small starter-town composition |
| GoldenrodCity | johto_modern | 20 by 18 | Dense but readable city blocks |
| OlivineCity | johto_modern | Read actual pack | Port-side town composition without inventing 3D harbor props |
| NationalPark | park | Read actual pack | Composed open space, boundaries and a destination in the center |
| IlexForest | forest | Read actual pack | Forest corridors; its forced night palette is not a global Minnesota palette |

Use source maps as composition references. Do not paste entire Johto maps together or merely rename their NPCs.

## Existing source blocks

These IDs refer specifically to the local `johto` tileset and must be checked against the chosen base pack. They are references, not an additional asset catalog.

| ID | Use | Collision implication |
| --- | --- | --- |
| 01 | Light open ground | Four FLOOR quadrants |
| 02 | Textured green ground | Four FLOOR quadrants |
| 03 | Tall grass | Four TALL_GRASS quadrants |
| 04 | Source flowers | Four FLOOR quadrants |
| 05 | Dense source tree border | Four WALL quadrants |
| 0a | Rock block | Inspect source quadrants before placement |
| 18, 19 / 16, 1e | Small source house facade recipe | Block 16 carries the door; reserve its exact core-tile quadrant |
| 35 | Water interior | Four WATER quadrants |
| 45 | Ground sign | Read-event coordinates must match the source sign quadrant |
| 57 | South-facing ledge | HOP_DOWN and WALL quadrants; never a substitute for bidirectional stairs |
| 68 through 72 | Earth terrace borders and floor | Wall and floor positions differ by block |
| 73 | Cave face | CAVE in the southwest quadrant |

The current brush renderer uses approximate shoreline variants and cuts paths through some terrace borders. These are visible fitting tasks, not accepted final blocks. Replace each cut with a source-authenticated ramp/stair recipe and test movement from both directions. Resolve shore corners from neighboring quadrants rather than repeating a corner block along every edge.

## Asset work that must be explicit

The illustration's long timber boardwalks, wind turbines, grain silos, canoe racks and lighthouses are not promised existing Johto assets. First try an existing facade, sign, rock, fence, or port-floor recipe that communicates the place. When a required bridge cannot be composed faithfully, author a small bounded derived tileset through `CompiledTilesetExtension`: preserve the original 8-pixel tile grammar, define collision for every quadrant, retain the original assets unchanged, and record its block-replacement behavior. Do not paint a road across WATER to make a screenshot look connected.

Custom art is a separate work item with a native-scale before/after comparison. The first Winona-to-Rochester slice should rely on authenticated source art. Decorative tents without an entry action must visibly read as scenery; every facade that looks enterable gets a real interior or a clear closed-door state.

## Route composition rules

Finish and review one complete map at a time. Maps can be small or large; worthwhile features, variety and pacing determine their extent. Size and counts of districts, branches or loops are not quality targets. Use paths normally 2–4 gameplay tiles wide, purposeful approaches and decisions with distinct costs or rewards. A wide clearing must support an activity, entrance or readable composition. Do not pad a route to fill reserved chunks.

Use native directional ledges to shape one-way returns and shortcuts. A downhill ledge must have a safe landing, blocked reverse approach and closed sides; retain an intentional uphill path elsewhere. Prove the hop and rejected reverse through production input. Do not substitute an ordinary wall or painted stripe.

The latest Johto direction calls for winding approaches and short ledges that
route the player through grass. Bend paths around purposeful rock/pine features;
compose short drops with safe grassy landings, closed side bypasses and a useful
uphill return. Use authenticated cave stair blocks for outdoor rock climbs and
verify their real bidirectional collision. Review cave interiors as distinct
places with coherent chambers, elevation, loops and discoveries; a rectangular
hollow with scattered boulders remains unfinished.

Roads must physically join. Include short intentional grass crossings rather than making every habitat freely avoidable. Use fences and bollards to frame the road, mark entrances and control water access; gaps must be deliberate and wide enough to use. Docks have real walking decks, closed ends, rail collision and water below. Review the whole map and the actual viewport before adding more area.

The user clarified that compactness is not required for every map. The rejected quality was sparse, overdesigned space without worthwhile features. MN_R15's one-chunk replacement is one iteration, pending review. R11–R14 retain useful implementation evidence and need fresh feature-density, pacing and visual review; they do not automatically need to shrink. Their size and circuit counts are not templates.

Route 1 now uses one compact 32×20 exterior. Its longer northern limestone bench climbs by a native eastern ramp and returns through a south-facing one-way ledge, providing a westbound alternative to central grass. The reverse trip cannot climb the ledge. A farmhouse, two-hollow cave and gated creek dock provide short rewards. There is no Cut gate. See [the exact directional contract](routes/MN_R01.md).

For each route, refine its district reservation into a distinctive arrangement. Reject duplicated six-room geometry, repetitive rectangular lakes, unmotivated grid corridors, copied source-map layouts, and unexplained empty terrain. Terrain previews expose these issues; passing a graph check does not approve the art.

## Kanto water fitting for Central Lakes

Read the actual source pixels before reusing a Johto shore rule. Native Kanto
`43` is open water; `64` and `65` are the east and west vertical banks.
`1d`, `1f` and `1e` are the northwest, northeast and north banks. `2e` is only
the small northwest concave corner, and `2d` the northeast concave corner.
Using either small corner as a full bank or open-water fill creates detached
brown fragments. The renderer and Rust compiler now use these verified native
recipes, preserving all four WATER collision quadrants.


## Prairie Farm bank and native facades

MN_R02 uses a west-facing Kanto `58` ledge: its right quadrants are HOP_LEFT,
its left quadrants WALL. Approach from the east, land two gameplay tiles west
of the hop cell, and close the ends with real stone walls. The southern farm
lane supplies the deliberate reverse journey; do not open an uphill gap beside
the bank. Both the directed design check and production controller input must
pass.

Kanto `00` looks blank but has CUT_TREE collision. Use `01` natural floor and
`31` gravel for playable farm ground. Work the nursery rows with unchanged `2c`
soil and `0b` habitat. The native blue house is `38 39` over `3c 3d`; mixing
`02 03` beneath that roof creates a visibly doubled roof line. The tool shed
uses `20 54 21` over `37 3a 7e` and a separate traditional-house workshop. These
are source facade recipes, not copied map geography.
