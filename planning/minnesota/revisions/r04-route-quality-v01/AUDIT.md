# R04 route-quality audit and detached shoulder proposal

The audited native parent is `content-packs/minnesota-house-joins-v04.crystalpack`
(SHA `63117daf68439663d7632293f8bded7352bf8e39b96d30edcc14f035871d4d6b`).
Its R04 module exactly matches the frozen pond/bridge preimage, module SHA
`ca7e08ba7ec44571661298e28ab8d21560f5287eccfee5d5797983f68d47d6ca`.
The frozen source-v04 candidate adds four metatiles and changes 51 cells;
those pond/bridge changes are absent from this specific pack. This audit does
not claim to have inspected later packs published by the parent agent.

I inspected the complete native parent, the frozen candidate, six player
cameras on legal unoccupied walking tiles, and authentic National Park/Route34
terrain at whole-map and player scale. These are native-pixel source previews,
not production GPU captures or input traversal. Latest evidence: render epoch
`planning/minnesota/renders/r04-route-quality-v01/audit-v08/`.

## Three concrete remaining limitations

1. **The eastern ordinary walking apron is too deep for its activity.**
   At core x40, y16–21 is six consecutive walking tiles; lawn and paving are
   both ordinary `FLOOR`. The gray road line alone does not narrow the walking
   space. The east return lane has another six-tile-deep shoulder. The single
   elm tongue creates a bend, but leaves broad unoccupied shoulders around it.
   Inspect `(40,18)`, `(46,20)`, `(50,18)`. Wider bench turnouts are useful;
   the proposal narrows the ordinary through sections rather than every space.

2. **Three raised flower rectangles repeat one composition.**
   The west display is meta `[2,3,6,6)`, east main display `[19,5,22,8)`, and
   east small display `[24,5,26,7)`. Their sizes change, but their closed rose
   fill, raised border and neighboring civic props repeat. This is a visual
   variety limitation, not broken collision. A later flower-bed hand edit
   should preserve corner/edge grammar and provide a distinct silhouette.

3. **Both habitat approaches present straight rectangular grass walls.**
   The west habitat is meta `[8,4,10,7)` and southeast habitat `[22,12,24,15)`:
   both two-by-three blocks. At the southeast entry `(43,27)`, that rectangle
   dominates the camera; the useful seated cache only becomes visible farther
   right, at `(48,27)`. This is lower-priority silhouette/anticipation polish,
   not a missing feature. The existing picnic bench is already `(24,12)`.
   The optional trainer, paid four-grass cache approach and useful destination
   should remain. Do not add props simply to fill the picture.

The southwest rest area already has two benches, the gardener and Ether;
leave it. The west trainer-versus-grass choice is meaningful. The frozen pond
closures, two-core-row bridge, bank passing space and both park ports remain
constraints. No new Cut tree, ledge, trainer or quest is required as a quota.

## Bounded next hand edit: eastern elm shoulders

This is **a detached proposal, not applied or accepted**. It uses six existing
ordinary native elm blocks `00`, replacing lawn `02`:

| Metatile | Purpose |
| --- | --- |
| `(19,8)`, `(20,8)`, `(21,8)` | Join a north shoulder to the existing elm tongue |
| `(24,8)`, `(25,8)` | Frame the north side of the east return lane |
| `(24,10)` | Close the unused lower shoulder after the bend |

The ordinary apron at x42 narrows from seven to five core walking tiles
(including the gold fence’s walkable upper half); the
return lane at x50 narrows from six to two. At x40 the functional picnic spur
continues south, so its full connected vertical run is twelve to ten, not
six to four. That useful junction and the bench turnout remain wider.
This changes circulation by removing 24 ordinary walking quadrants, rather
than adding decorative props. Native block `00` is four `WALL` quadrants;
there are no incidental `CUT_TREE` quadrants. Neither trainer nor an entrance
occupies a narrowed section.

Static checks preserve both ports' actor-present walking connection, every
actor footing and interaction access, sign access, the exact western route,
and the entire southern habitat/cache/bench area. Cache minimum grass cost
remains four. All six changed cells are disjoint from the 51 frozen pond/bridge
cells; no metatile, glyph, palette, event, reward, story, seam or art recipe is
added. Static paths are evidence of geometry, not production input execution.

Review the latest epoch's `proposal-only/whole-before-proposal.png`,
`proposal-only/player-camera-before-proposal.png`, and `proposal-only/proposal.json`.
The first three camera pairs show the actual walking width; the last two show
unchanged western choice and picnic destination. Source160 originals and
individual enlarged camera pairs are retained alongside the contact sheets.

## Rejected history and next verification

The earlier three-cell picnic proposal is rejected. Its premise that the
clearing lacked a bench was incorrect; adding one at `(24,13)` duplicates the
existing bench at `(24,12)`. Its images/proofs remain in audit-v03/v05, with a
rejection note. `AUDIT.rejected-v05.md` and
`rejected-picnic-proposal-v05.json` preserve that superseded diagnosis.
Audit-v02's earlier blank-southwest annotation is also superseded. Failed
camera-setup epochs and every render remain available.

If the parent accepts the shoulder composition, implement only the six-cell
patch after the frozen pond/bridge candidate, preserve the source contract,
then verify native captures and traversal with the production controller.
The read-only audit changed no canonical/frozen source or pack. The approved
combined handoff is now frozen separately in source-v05 with three unregistered
Rust leaf files; publication and native checks remain pending. The
broader goal of improving all routes remains open.
