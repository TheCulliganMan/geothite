# Blocking: a hash-only rebase isolates all new grass

A bounded repair with native acceptance is now available. See
[SHARED-REPAIR.md](SHARED-REPAIR.md). The findings below remain applicable to
the bank-only candidate; do not treat its refreshed hashes as a complete river stage.

Checked actual parent:
`content-packs/minnesota-region-polish-duluth-v01.crystalpack`.

All 40 original `before=5` cell preimages still match. However, after applying
those exact 40 grass replacements in memory, **none of the 160 new gameplay
tiles is reachable from an existing dry Surf shore**: 0/80 lower and 0/80
upstream. The completed landward bank clearings in the original fruit-stage
river restoration are absent. Matching canopy cells is insufficient evidence
that the recesses connect to a playable bank.

Do not publish a hash-refreshed `revision.json` alone on that parent. The root
source `revisions/region-polish-v01/mississippi-banks.json` must not be accepted
as a playable refinement merely because it refreshes seven whole-map hashes.
No root files or pointer were changed by this audit.

The unchanged original `revision.json` works after the completed river layouts
are installed. Regenerate the fruit authoring contract on the actual newer
regional parent using `author_fruit_groves.py --base PARENT --output FRESH_SOURCE`,
then use the existing Rust fruit publisher and written review. This preserves
newer Duluth/swamp/route geometry while installing the completed river terrain,
20+20 trainers and safe tree placements. Apply the original bounded bank patch
afterward. Do not transplant the older whole candidate or weaken preimages.

The Rust `refine_mississippi_banks` publisher now also flood-checks all four
walking quadrants of every replacement against actual connected Floor/grass
from legal existing dry Surf shores, including NPC bodies and chunk joins.
A hash-only rebase onto isolated canopy is rejected with the unreachable-bank
error. The previously published and verified complete local delivery remains
`content-packs/minnesota-fruit-ice-compact-v02.crystalpack`.

## Exact shared candidate native failure

`content-packs/minnesota-region-polish-vergloom-events-v01.crystalpack` was
checked with the existing production controller binary, without a new build.
The accepted lower river activity course fails immediately because its dry
launch is `WALL` instead of `FLOOR` (`minneapolis_falls.rs:206`). The course was
explicitly rebound to this candidate hash as a regression replay; that binding
does not certify matching terrain or actors.

A separate direct collision audit of the exact candidate confirms all 40 bank
cells are applied, but 0/80 new lower and 0/80 new upstream grass tiles are
reachable from dry Surf shores. Each route still has only two trainers.
This candidate does not contain the completed river stage required by the
bank refinement. Do not report its river activity checks as passing.

Receipts: `renders/fruit-compact-v01/shared-candidate-vergloom-v01/access-audit.json`
and `.local/river-shared-vergloom-native-v01.log`. Install the completed river
stage on the newer regional parent, then apply the bank patch and rerun the
existing native courses. The guarded publisher's successful and rejected
parent checks are recorded in `renders/fruit-compact-v01/bank-access-guard-v01.json`.

## Isolated source discrepancy and bounded fix

The first failed native course is `MN_RIVER_FUN_LOWER_REED`. Its initial dry
launch is `MnMississippiLowerWoodsC00R00` at walking tile `[50,40]`: the shared
parent has native block05/WALL, while the completed route source has block02/FLOOR.
That map is not among the seven chunks touched by the 40-cell bank patch.
Therefore the first native failure is an original parent stage omission, not
a terrain regression introduced by those 40 cells.

`revision.shared-activities-v01.json` is an exact-parent bounded repair on the
specified shared Vergloom-v01 pack. It restores only the twelve accepted river
rasters and 44 activities, binds touched teams/encounters to the active Minnesota
roster, and retains the 40 already-applied bank cells. Its tree list is empty.
The public Rust `build_mississippi_activity_revision` shares the existing
verified river-stage code with the fruit publisher but forbids trees. It keeps
all other regional geometry, fruit catalog, winter art/terrain and audio intact.

The new `pack_mississippi_activities BASE SOURCE NEW_OUTPUT` CLI performs strict
input/output/readback verification. It requires the exact parent identity and
per-map preimages; the shared root pointer and root-owned source files remain
untouched. The output candidate is
`content-packs/minnesota-region-polish-mississippi-activities-v01.crystalpack`.
This source already retains the 40 bank cells; do not apply the bank patch a
second time afterward. A newer event-only parent requires a new source epoch
with its exact identity and rechecked river preimages, followed by native checks.
