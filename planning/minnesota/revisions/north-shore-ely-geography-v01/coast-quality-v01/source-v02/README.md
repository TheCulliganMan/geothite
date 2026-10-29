# Compact direct coast: detached v02

The former24-metatile-wide northern eastern chunk becomes4×24; Grand Marais
moves20metatiles west to[226,-400]. Its west entry and shared Superior remain.
The result is an11-step ordinary bend across this chunk, rather than a separate
long pine corridor. Existing C00 and every lower coast chunk keep their native
footprints, actors, doors, falls, grass, ledges, Ryan's House and all programs.

Source-v01 and both complete render epochs remain preserved. v01 was rejected
in boundary audit: native30 has a WATER bottom-right quadrant, so its proposed
far-east reef leaked. v02 replaces those six terminal reef metatiles with native
0a rock (WALL in all four quadrants). No generated art or new tile recipe.

`boundary-and-anchor-proof.json` independently audits actual source collision:
all24 walking quadrants connect, both western core12/13 and eastern8/9 lanes
are reached, east Superior16..35 stays water, all other eastern rows are visibly
closed, north is closed pine, west water lanes match unchanged C00, and every
open southern water quadrant matches unchanged C01R01. Native wall/reef visual
review and production controller tests remain publication requirements.

`MN_R11.layout.before.json` is the exact current canonical source preimage;
`MN_R11.layout.candidate.json` adds only96 explicit cells for the cropped chunk.
The56×96 canvas is an authoring surface, not a rectangular published footprint:
`ragged-footprint.contract.json` declares actual eight chunk dimensions and the
480 retired source cells atx36..55,y0..23. Native builders, parity tests and
connected renders must use actual per-chunk dimensions, never publish those
retired cells or draw them behind town. Dossier actors/items/doors/grass/ledges/
Cut data retain exact hash guards; two source-only water cameras outside the
new footprint are explicitly retired and replaced. The former north endpoint
is replaced by the east town endpoint. These are detached proposals only.

The north inland branch is root-owned and intentionally absent from this coast
preview. Current integration proposal changes Cedar/R12 and retires Neck; do
not reuse the optional older Cedar/Neck placement hint as an inland contract.
