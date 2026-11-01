# R15 western landing junction — second source epoch

This separate candidate composes ten western terrain changes after the frozen
eastern shore v03. It replaces the old eight-by-eight clear Floor apron with a
readable meeting of the meadow landing, Ron's road and the outlet bridge road.
The two branches serve existing play choices; no actor, item, encounter, gate,
story flag, ledge or art recipe is added. Root reviewed the whole map and all
decision views and approved this component for native testing, not final R15
or user acceptance. Active route sources remain unchanged.

The [whole before/after](../renders/central-lakes/west-apron-20261027/v03/before-after-whole-native32.png)
and [player-scale sheet](../renders/central-lakes/west-apron-20261027/v03/candidate-decision-cameras.png)
retain the approved eastern S bend. The western upper woodland joins Ron's
existing copse; a continuous timber bank joins the old lakeside copse. The lower
flower bend has an ordinary leafy edge instead of a broad paved skirt. The stem
is three core cells, widening to four beside the lower turn. This is a landing
junction with one corner, distinct from the eastern S course.

## Exact source boundary

| Input | SHA256 |
| --- | --- |
| Previous dossier, shore v03 | `57bdf7662fadc71473d4a9ca610f62141326abaf69fb6923093796d0ab3c3773` |
| Previous layout, shore v03 | `04c1a8ba6e19e0e267c7876596f53d9a0a50dfb65fbe17b20a4f5c05d4a5b335` |
| `routes/MN_R15.west-apron.candidate.json` | `c69dc26a8d4a93c82a00bd442b487a613066196f4d5a626225aadade45620c25` |
| `routes/MN_R15.west-apron.layout.json` | `aa954f898bba7714f00b29a3505e4a61f22591bc302c455a89043b72f56a0363` |

The previous sources are immutable under
`renders/central-lakes/west-apron-20261027/before-v03/`. Every original dossier
gameplay field is exact, including the original main-path and loop metadata.
Only another editorial review object is added. All previous layout metadata and
the entire shore-v03 layer prefix are exact. Ten final block deltas stay within
meta `[9,11,14,15)`, using native `0f`, `6d` and `5f`. The complete eastern region,
all water/buoy cells, original grass quadrants, exterior apertures, actors,
parties, items, signs, cabin/room/passive returns and meadow hop are exact.

The art and original western terrain were verified on actual538 baseline pack
`da27b381f69f64d6530383216077de0802c5b9dc42b12c316319cbc5daeb6d50`.
The entire shore-v03 source was not yet installed in that art-reference pack.
Native publication must use the new shore-v03 pack as its complete previous
terrain preimage, then the same existing Rust terrain-revision API. Only the
one existing exterior's ten blocks, its owned encoded block payload and derived
identity may change relative to that immediate native baseline.

## Source proof and render history

[The report](../renders/central-lakes/west-apron-20261027/v03/review.json)
passes 47 checks and rejects ten saved adversarial inputs. All four original
hop lanes retain ordinary return distances **31 / 33 / 35 / 37** tiles. Original
trainer-avoidance and resource grass prices remain exact: Hyper Potion west12 /
east6, Revive west6 / east0; eastbound full trainer avoidance costs12 grass,
westbound avoidance remains impossible because Ron controls that direction.
Native Cut collision is absent throughout the complete candidate.

`v01` retains the rejected all-solid corner, which made the original declared
main-path metatile `(10,13)` inaccessible. `v02` fixes that native corner with
the ordinary `6d` edge while retaining all original path metadata. `v03` keeps
those exact source bytes and adds the explicit input-path checker. The initial
centered weakest-edge crop is retained, with additional clamped160×144 variants
for the actual map edge. All original/RGBA/conversion/crop/rejected stages remain.

Whole map, every player view, Ron's approach, the old landing and lower flower
turn were personally inspected. The computed largest all-Floor square moves
from the old eight-core apron to the unchanged six-core west-entry area, core
`(3,25)`. It remains visible in the contact sheet; a clamped source camera is
saved separately. The largest eastern square remains the five-core dock bay.
These measures are collision diagnostics, not claims that all Floor art is blank.

## Explicit production-controller walking targets

Coordinates below are 16px gameplay tiles. The checker expands every cardinal
segment into individual native Floor/grass steps with actors present. The real
controller still must execute them, resolve the encounter/battle/text owners,
verify rewards and Continue; this source checker does not replace that proof.

| Existing journey section | Legal targets after this candidate |
| --- | --- |
| From meadow landing `(22,20)` toward bridge | `(22,24)` → `(22,28)` → `(46,28)`; unchanged. |
| Return west from `(46,28)` | **`(21,28)` → `(21,25)` → `(14,25)`**. Old `(20,28)` is now native WALL. Ron still stands `(17,24)` and sights `(17,25)`, so the battle remains intentional. |
| Exact31-tile ordinary return from `(22,20)` | **`(22,25)` → `(15,25)` → `(15,15)` → `(22,15)` → `(22,17)`**. Distances5+7+10+7+2=31. Old `(18,22)` is native WALL. |
| Final west departure after another hop | `(22,20)` → **`(22,25)`** → `(4,25)`. |
| Approved eastern S course from `(46,28)` | `(46,21)` → `(50,21)` → `(50,20)` → `(54,20)` → `(54,24)` → `(60,24)`; shore-v03 unchanged. |

The source report includes every expanded step and the expected rejection of
both obsolete blocked waypoints. The frozen handoff also contains a core
collision/path diagram. The journey fixture's initial position/party/badges/RNG
remain labeled setup; no fixture state change is requested after setup.

Native written-pack preservation, actual GPU comparison and the complete real
journey remain pending. The original shore handoff is unchanged and retained.
This review does not claim whole-route release or earned regional progression.
