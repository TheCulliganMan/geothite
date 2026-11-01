# Bounded repair for the missing shared river stage

The omission is confirmed on the selected shared lineage: its rivers had two
trainers each and lacked the completed landward banks. The first failed course
starts at `MnMississippiLowerWoodsC00R00 [50,40]`, where source block02/FLOOR
was missing and the parent retained block05/WALL. This map was not touched by
the 40-cell bank patch. Refreshing bank hashes alone cannot install the missing
terrain or activities.

The verified bounded repair is
`content-packs/minnesota-region-polish-mississippi-activities-v01.crystalpack`.
It composes directly on the exact shared Vergloom-events-v01 parent. It restores
the 12 completed river modules, retains all 40 bank cells, and installs 36 added
trainers, six pickups and two medics. Each route now has 20 trainers. There are
29 grass components and 952/792 grass gameplay tiles. All traversable grass is
reachable from actual dry Surf shores with NPC bodies included.

Exact output SHA256:
`3e2fc6def5c7326129b656ee07ff9df7e0831b0c1b483607f42f107a6ef19941`.
Content identity:
`f9a465a42c2f8bb03ed3f85ee0b1e69bd8556a8b17b5adb3c759bfc2f4f5cc20`.

`pack_mississippi_activities` shares the existing verified river-stage code with
the fruit publisher, requires an empty tree list, and validates exact parent and
river preimages. Original river water and outer boundaries are explicitly
preserved. Complete inverse equality proves all 679 other maps and all unrelated
catalog data remain exact, including newer Duluth/swamp/routes and event work.
Fruit catalog, winter terrain, native art and audio remain exact. Only the river
stage manifest is appended.

Native acceptance passes on that exact v01 repair: all 44 added activities,
wild grass encounters, real Surf launch/dismount, battles/PP/XP/money, supplies,
healing, repeated interaction and Continue. Three existing native tests passed
in 47.08 seconds. The level 80 PEPIVERN, initial badges and starting bank are
explicit fixtures, not earned campaign progression. No renderer/export build
was needed for those native checks.

Receipts:

- `renders/river-shared-activities-v01/residual-v01.json`
- `renders/river-shared-activities-v01/written-v01/proof.json`
- `renders/river-shared-activities-v01/written-v01/river-controller-proof.json`
- `renders/river-shared-activities-v01/all-grass-access-v01.json`
- `.local/mississippi-shared-stage-native-v01.log`

The exact v02 event parent is now present. All 12 of its river terrain preimages
match. `revision.shared-activities-v02.json` is a fresh exact-parent contract
for root composition, with identical river target rasters/activities. The v02
pack and shared pointer have not been overwritten. Compose with:

```sh
target/river-fun/wasm-dev/pack_mississippi_activities \
  content-packs/minnesota-region-polish-vergloom-events-v02.crystalpack \
  planning/minnesota/connections/mississippi-bank-refinement-v01/revision.shared-activities-v02.json \
  content-packs/minnesota-region-polish-mississippi-activities-v02.crystalpack
```

That v02 command has not been executed here; the successful native acceptance
above is bound to the published v01 repair. Use the fresh source with the
written reviewer and rerun the existing native river tests on the v02 child.
The source already retains all 40 bank cells; do not apply the bank-only patch
again afterward. The full older fruit/ice candidate is not an integration input.

## Meaningful preservation and why the parent omitted the stage

The selected shared event-v02 parent has no `minnesota-mississippi-fun-v1`
manifest, zero `MN_RIVER_FUN_` actors, and two trainers per reach. The earlier
working fruit/river sibling has the manifest, all 44 activity actors and twenty
trainers per reach. The earlier activity verification belonged to that separate
unselected branch; it was not evidence that the canonical shared R10/R09 branch
already contained those activities. This is a missing composition stage, not
a deletion by the canopy-only bank refinement. The first failing launch chunk
was untouched by the bank refinement, independently confirming that diagnosis.
Receipt: `renders/river-shared-activities-v01/stage-lineage-diagnosis.json`.

The bounded river stage preserves the original four trainer objects/scripts,
route levels, original water quadrants, exterior mouths and all connections.
It adds the previously missing 36 trainers, six supplies and two medics while
restoring the hand-authored dry banks and grass. Touched teams and river wild
encounters use the active Minnesota roster. The 40 authored bank cells remain
grass and become reachable through the restored landings. No map dimensions,
rooms or regional coordinates change.

Complete inverse equality against the exact selected parent proves all 679
non-river maps and unrelated catalogs remain byte-identical, including newer
Duluth, North Shore coves, R06/R07, swamp, Vergloom and R09 work. Existing
Minneapolis badge/Waterfall gates and bridge maps lie outside these twelve
chunks and remain exact. The fruit catalog, all winter terrain, native runtime
art files and compiled audio remain exact. The shared world selection is left
to the integration owner. No older whole fruit/ice pack is transplanted.

## Exact final root v03 acceptance

The root-owned `minnesota-region-polish-final-v03.crystalpack` now has a fresh
native run on its actual hash, rather than only transferred acceptance.
All three existing river tests pass in 125.59 seconds, covering every one of the
44 added activities plus wild encounters. The reached-save fixtures are retained
under `renders/river-shared-activities-v01/final-root-v03/native-saves/`.

Actual final SHA256:
`67fb7ee40c022f0de78b561bbb144767d1483fe97a7dba9a2e6ec111d8c3360e`.
The native proof also checks exact river-module, added trainer-team and wild-table
parity with the previously verified sibling. A separate complete inverse audit
against the exact event-v02 parent confirms all 679 non-river maps and unrelated
catalogs, native art, compiled audio, fruit catalog and winter terrain remain
exact. Only the river-stage manifest is added. Receipt:
`renders/river-shared-activities-v01/final-root-v03/native-and-preservation-receipt.json`.
Native log: `.local/mississippi-final-root-v03-native.log`.

No root output or pointer was written by these final checks.
