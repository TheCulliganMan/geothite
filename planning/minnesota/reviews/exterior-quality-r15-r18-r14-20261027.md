# R15 / R18 / R14 visual review — 27 October 2026

The eastern R15 road-to-dock/cabin approach was the weakest passage in this
three-route review. Its apron joined several destinations into one broad paved
space. The candidate fits connected woodland shoulders around a short S turn
while retaining the useful dock, cabin forecourt, encounters and return route.
It is a source candidate awaiting native publication and controller review.

## Exact current baseline

The reviewed pack was `content-packs/minnesota-northstar-lake-country.crystalpack`,
file SHA256 `da27b381f69f64d6530383216077de0802c5b9dc42b12c316319cbc5daeb6d50`,
538 native modules, internal content identity
`d7ad6d24e6716c7d594044c056046f08cbb57ca0943ea6443e0e422a9b00baae`.
All three current source grids matched the actual installed native blocks exactly.
No older 510-map review was substituted for this baseline.

[Revalidation and source hashes](../renders/exterior-quality-20261027/brainerd-audit/native-source-revalidation.json)
records the seven actual native module identities, all standing actor anchors
and the six exact source hashes. Original source snapshots and all render stages
are retained beside it. These are native-art source composites, not GPU captures
or evidence of runtime actor visibility. Conditional standing templates are shown.

| Route | Whole-map and weakest-view assessment | Action |
| --- | --- | --- |
| R15 Central Lakes West, 32×18 metatiles | The lake/grass fork and dock have useful identities. The eastern apron at core `(47,23)` and cabin approach `(53,15)` lacks a clear road shape; the older western road also retains an open apron. | Refine the eastern passage only. Preserve the western Ron/grass/ledge topology. |
| R18 Mille Lacs Shore, 40×24 | `WIDEST_ROUTINE_SHORE` at `(57,11)` is a flat four-core road with long green shoulders. The timber piers, habitat fork, Surf launch and beach have actual activities and visible water. | Retain this pass; a future shore-road rhythm pass may be useful, but is less urgent than the R15 apron. |
| R14 Headwaters Trail, 48×32 | The current source has coherent forest passages, habitat spurs, river views and short bridge approaches. `LOW_BANK` at `(29,23)` and `FERN_HOLLOW` at `(23,55)` are among its broader junction views. The current route is substantially more articulated than an older large R14 preview. | No source edits. Preserve its authored forest/grass/return structure. |

Fresh whole-map composites and every declared camera are in the
[R15](../renders/exterior-quality-20261027/brainerd-audit/R15/whole-current-native-source.png),
[R18](../renders/exterior-quality-20261027/brainerd-audit/R18/whole-current-native-source.png)
and [R14](../renders/exterior-quality-20261027/brainerd-audit/R14/whole-current-native-source.png)
folders. R18's routine shore view and R14's full camera sheet were personally
inspected, together with the three whole layouts.

## Authentic source comparison

Fresh [Johto Route34 / Route42 terrain cameras](../renders/exterior-quality-20261027/brainerd-audit/native-johto-reference-current538/authentic-johto-native160-source-terrain.png)
were rendered from the same current pack and inspected at the same native pixel
scale. Route34 gives broad forecourts a building/grass purpose; Route42 makes the
ordinary road bend around connected banks. R15 uses its existing Kanto art rather
than substituting Johto trees. The comparison informed the connected shoulders,
short turn and retained cabin court; it does not establish visual equivalence or
user approval. Native original reference pixels and all conversion/crop stages
are retained in that folder; no source-art catalog was exported.

## R15 candidate and visual result

[Whole before/after](../renders/central-lakes/refinement-20261027/v03/before-after-whole-native32.png)
and [ten player-scale views](../renders/central-lakes/refinement-20261027/v03/candidate-decision-cameras.png)
show the complete route and the requested `(47,23)` and `(53,15)` views.
The native-scale originals remain separate from the integer-scaled contact sheet.

Seventeen metatiles change inside meta `[19,9,30,15)`. The north tongue joins
existing woodland; the opposing shoulder narrows the cabin stem to two gameplay
tiles. The lower bridge approach is three gameplay tiles wide. The dock-side toe
joins an existing bank rather than becoming an isolated decoration. The useful
cabin forecourt retains its original six-by-six core activity area and two actors.
No new actor, object, Cut gate, encounter, reward, art recipe or story field is added.

The first pass's dock-side toe was detached. It was rejected, retained in `v01`,
and joined to the old woodland in `v02`. `v03` keeps that geometry and adds the
native player foreground mask plus automatically selected weak views. Whole map,
contact sheet, requested cameras and the new weakest eastern view were personally
inspected. The native `0f`, `6c` and `6f` canopy quadrants contain no `CUT_TREE`.

The largest remaining all-Floor square in the selected eastern area decreases
from six to five core tiles. Its new location `(45,14)` serves the dock bay and
watcher. The global eight-by-eight square at `(22,26)` is unchanged on the western
road outside this selected refinement. Both are shown, not hidden by camera choice.
An all-Floor square is a collision measure, not a claim that every pixel is empty.

## Preservation and limits

The [source report](../renders/central-lakes/refinement-20261027/v03/review.json)
passes 38 checks and rejects ten saved adversarial inputs. It checks real native
quadrants with actor bodies present, both endpoint directions, original dock
rails/two-lane gate/endcap, cabin door/passive return/sign approaches, unchanged
water/buoy boundary cells, no incidental Cut, attached added walls, the new S
course, the bridge approach and unchanged directed grass/resource prices.

West-to-east avoidance of every trainer sight still costs 12 grass steps;
east-to-west avoidance remains impossible because Ron controls that lane. The
western six-step grass band and original meadow hop/course/landing/return terrain
are exact. Hyper Potion entry remains west 12/east 6 grass; Revive entry remains
west 6/east 0. The original four-core hop is retained rather than shortened as an
unrelated change. Existing water closure prose in the historical dossier is stale:
the actual current538 blocks already include those closures and match source.

The warm source check measured 57.655ms; complete before/after positive rendering
measured 922.572ms, excluding pack loading and ten negative cases. All rejected
inputs, original full-resolution images, RGBA versions, crops and previews remain.
Static native-art compositing does not prove GPU fidelity, actual item/dialogue
input ownership, encounters, battle PP mutation, room return behavior, Continue,
earned progression or final user acceptance. Those remain the native handoff.

The exact candidate inputs and bounded native integration steps are in
[r15-shore-walk-refinement.md](r15-shore-walk-refinement.md).
