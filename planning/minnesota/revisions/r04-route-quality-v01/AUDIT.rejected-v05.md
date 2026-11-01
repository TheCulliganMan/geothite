# R04 route-quality audit — read-only

This audit compares the actual R04 in the published house pack
`63117daf68439663d7632293f8bded7352bf8e39b96d30edcc14f035871d4d6b`
with the frozen pond/bridge `source-v04` candidate. The actual R04 module still
matches that candidate's original preimage exactly:
`ca7e08ba7ec44571661298e28ab8d21560f5287eccfee5d5797983f68d47d6ca`.
The candidate's 51 terrain changes and four appended metatiles are **not yet
installed** in this pack. No canonical/frozen source, Rust or pack was changed.

I inspected the complete route, six legal-footing native-scale source cameras,
and authentic National Park/Route34 terrain at whole-map and player scale.
These are source-art reviews, not GPU or live-input claims. Latest evidence is
`planning/minnesota/renders/r04-route-quality-v01/audit-v05/`.

## Three remaining design limitations

1. **The east approach is wider than its current activity supports.** At core
   x40–41, the apron spans six walking tiles vertically, y16–21. The gray
   promenade and yellow lawn are both ordinary `FLOOR`; a painted two-tile
   line does not make the travel itself tighter. The single elm tongue creates
   a bend, but both sides still present broad flat shoulders with no additional
   decision. Inspect `east-open-forecourt` `(40,18)` and `east-elm-bend`
   `(46,20)`. National Park justifies wider paving through a stepped perimeter
   and habitat loop; Route34 puts its ordinary lane against meaningful habitat
   and bank edges. Do not narrow R04 everywhere as a universal rule.

2. **The flower displays repeat the same closed-box beat.** The west bed is
   meta `[2,3,6,6)`, the larger east bed `[19,5,22,8)`, and the smaller east bed
   `[24,5,26,7)`. All use the same raised rectangular border and floral fill,
   with nearby bench/bin/sign arrangements. Their different sizes do not give
   the two banks much compositional contrast. This is a visual variety limit,
   not a broken collision or a reason to remove all civic rectangles. Keep
   their native border/corner grammar if a later author changes their shape.

3. **The southeast destination reads as a grass/cache chute, not a picnic
   nook.** It has a useful optional Schoolboy encounter and four-grass approach
   to the Great Ball, but the habitat is another straight two-by-three-metatile
   rectangle and the tiny final clearing has no seating. The nearest existing
   eastern bench is up at meta `(23,7)`, separated from this pocket by the main
   promenade/rail. The player camera at `(43,27)` mostly shows a rectangular
   grass wall. Preserve the paid grass entry, but make its destination more
   recognizable when the player reaches it.

The southwest pocket is **not** a priority defect after close inspection:
two benches, the gardener and Ether give its compact space a purpose. The
earlier audit-v02 annotation calling it blank is superseded. The west battle
versus four-grass alternative is meaningful and should remain. The frozen
v04 pond closures and tight two-row bridge should also remain; do not introduce
a new Surf path or fill atlas gaps. A civic park does not need a Cut gate,
ledge or extra trainer merely to satisfy a quota.

## One bounded hand-edit proposal

Start with the southeast nook; this adds a readable destination without a new
quest, room, reward or activity quota. This proposal is **not applied or
accepted**, and does not resolve the separate east-apron/rose-display concerns.

| Metatile | Before → proposed | Purpose |
| --- | --- | --- |
| `(22,12)` | `03` grass → `00` elm | Step the upper-left habitat edge |
| `(23,14)` | `03` grass → `00` elm | Step the opposite lower edge |
| `(24,13)` | `02` lawn → `0e` native bench | Seat the existing cache clearing |

The bench's upper quadrants block and lower quadrants remain `FLOOR`, retaining
the actual Great Ball at core `(49,27)` and its approach `(48,27)`. Static
actor-present minimum grass cost from `(42,27)` to that approach is four steps
before and after. Every actor footing is unchanged. The proposal is disjoint
from all 51 frozen pond/bridge cells and leaves west/east seams, the original
Cole choice, Mica's position, signs, item flags and bank passing lanes alone.
It uses existing native art only. Movement topology does change in the three
declared cells; any accepted implementation still needs native traversal and
resource verification.

See `proposal-only/whole-source-scene.png`,
`proposal-only/cache-approach-before-proposal.png` and
`proposal-only/proposal.json` in the latest render epoch. The first entrance
camera intentionally shows the habitat shape; the cache-approach camera shows
the seated destination. All previous originals and failed camera-setup epochs
are retained. Frozen source-v04 and the installed pack remain unchanged.
