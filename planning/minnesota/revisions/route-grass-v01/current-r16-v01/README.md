# R16 Red River Prairie — existing-layout review, not yet accepted

The currently selected R15 pack already contains a dense, functional R16.
Review its actual native terrain rather than adding grass to meet a quota.
Both clipped exterior maps match canonical layout exactly:1152metatiles,
1016grass walking quadrants. Source/native drift is zero. Original10trainers,
resources, river/ditch bridges, railings, pollinator basin and field identity
remain intact. All native trainer/wild teams use12active original
Minnesota species; the exact species set is in active-Minnesota-species-proof.json.

Weighted ordinary walking with actual actor occupancy shows6required grass steps
on the59-step hayfield ascent, a dry native southbound ledge return, and4required
grass steps each direction through the cottonwood neck on the main route. The
layout already meets the requested outward-grass/dry-return pattern. The possible
extra gate cell was only measured in memory; no revised layout or pack was written.
Canonical R16, its terrain, and world selector remain untouched.

The whole native route has been inspected in2D. A fresh native hayfield-bank
camera shows the hay, readable dry path and native ledge. A cottonwood-neck camera
is being captured. Render originals are retained under current-r16-v01.

Existing full geometry/seams/bridges/barriers/interactions/resources/Continue and
native trainer battle/PP/victory/party/flag Continue cases both pass on the selected
pack. Two additional populated regressions verify the existing grassy climb and
dry return, and an actual hayfield encounter/nativeRUN/Continue. Their initial
build v01 stopped in a shared Safari FishingError conversion compile error. The
conversion was already fixed by another author while being inspected; no core
edit was made by this task. That build is confirmed terminal101. Build v02 is
running in the owned2-job target/route-grass cache; the queued4case replay waits
for that exact build. An active build must not be restarted on observation timeout.

Accept R16 only after fresh4case replay, production stdio, actual2D review and
scoped lint/format checks. This will be a checksum-bound read-only review record,
not a terrain revision or a new selected pack. Preserve any incoming habitat/
cyclist selection; if R16 native encounters/programs change, refresh the review
against that actual parent. R17-R20 and other outdoor/story scope remain open.

The fresh cottonwood-neck native camera has now also been inspected. Its field
grass spans the actual neck between forest and rail closures, with dry main
trails on either side. A separate geometric audit removes all pickup blockers
and then all actor blockers: the hayfield ascent still requires6grass steps
in both cases. Its habitat price is terrain-based and persists after pickups.

## Accepted existing layout

Fresh4case native replay passes all geometry/seam/bridge/barrier/resource and
trainer/PP/victory/Continue cases plus the populated grassy climb/dry ledge return
and actual hayfield encounter/nativeRUN/Continue. Affected all-target Clippy and
scoped formatting pass with existing warnings. Production stdio passes15calls
from the reached hayfield save, walking(53,15)to(52,15), opening Start/Pack and
Items/Balls, then closing to overworld; stderr0. The exact same selected pack
already passed377audio hashes/frame/loop checks; its SHA was rechecked unchanged.

Acceptance is recorded in acceptance.json and scope inventory r16-v01. There
is no terrain revision, new pack, canonical dossier/layout change or selector
change. All730maps stay byte-for-byte within the same verified pack. The review
adds no world metadata, keeping the pending habitat/cyclist publication window
stable. North Shore remains ice-free. R17-R20 and wider scope remain open.

## Current habitat-parent regression

The habitat/cyclist child is selected on R15. R16 native modules and encounter
tables remain exact. All4native cases and15stdio calls (stderr0) were rerun on
that actual selected child, with fresh reached saves; all377audio programs were
verified again. See current-habitat-parent-recheck.json for exact identity.
R16 acceptance therefore remains current; terrain/selector stay untouched.
