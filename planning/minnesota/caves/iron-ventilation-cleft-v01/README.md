# Iron Range ventilation cleft

Status: **verified and selected local review**. The selected pack is
`content-packs/minnesota-iron-ventilation-cleft-v03.crystalpack`.
[Acceptance](../../renders/iron-ventilation-v534/acceptance-v558.json) and
[selection proof](../../renders/iron-ventilation-v534/post-selection-v558.json)
bind its exact parent, source index and native evidence. The earlier
[unselected candidate snapshot](verified-candidate.json) is retained as history.

The complete 18×13-metatile ventilation floor replaces the checkerboard
facility and storage aisles with a raised western shelf, a contiguous rock fin
with two walkable branches, and an eastern service cleft. Vern, both LODE
guards, the original valve and all four shaft pairs retain their exact native
positions and programs.

[Whole floor](../../renders/iron-ventilation-v534/audit-v548/after-2x.png) ·
[Before and after](../../renders/iron-ventilation-v534/audit-v548/whole-before-after.png) ·
[Native cameras](../../renders/iron-ventilation-v534/native-v548/MnIronVentilation-production-cameras.png) ·
[Three connected mine floors](../../renders/iron-ventilation-v534/connected-floors-v557/connected-native-floors-original.png) ·
[Implementation plan](plan.json)

Unlabeled coordinates are 16px walking tiles. The west Sump arrival is `(4,21)`.
The native30 climb at metatile `(4,4)`, column8, uses y10→9→8→7 and its ordinary
reverse to the upper Haulage shelf. The original vent guard stays at `(10,10)`.
The western fin provides two actual paths, checked through `(5,13)`, `(5,17)`,
`(9,17)` and `(9,13)`; its rock center `(7,14)` stays solid.

The original pressure console is read at `(13,15)` from `(13,16)`. Refusing its
native YES/NO offer leaves `EVENT_MN_IRON_VENT_PRESSURE_OFF` unset. Accepting
opens the grille at metatile `(9,6)` to floor07. The actual passage runs from
`(17,13)` through `(18,13)` and `(19,13)` to `(20,13)`. With pressure live, the
rightward step from `(17,13)` stays blocked and the eastern floor is unreachable
on this level. The original service network still provides the intended way
around to the western control.

The east corner36 climb occupies metatile `(14,3)`, column28, using y8→7→6→5
and its reverse. The shift guard stays at `(29,8)`; Vern stays at `(25,15)`,
approached from `(24,15)`. The open crosscut makes the return between these
workings direct, while both mine valves remain required for the separate
Sump pump gate. No later drill or rescue flag is granted.

| Pair | Ventilation active / passive | Other map active / passive |
| --- | --- | --- |
| Lower west | `(5,20)` / `(4,21)` | Sump `(9,4)` / `(8,5)` |
| Upper west | `(7,4)` / `(6,5)` | Haulage `(27,4)` / `(26,5)` |
| Lower service | `(27,20)` / `(26,21)` | Haulage `(13,12)` / `(12,13)` |
| Upper service | `(29,4)` / `(28,5)` | Sump `(25,10)` / `(24,11)` |

## Authoring and quick preview

`before.json` freezes the installed facility terrain. `after.json` contains the
whole authored rock floor. `contract.json` binds the complete original module,
all eight warp records and actor anchors. Selection is recorded separately in
`world.json` and its source index after verification. The old
`lairs/iron-workings/vent.json` is a campaign seed, not this current terrain.

```sh
PYTHONPATH=.local/ely-backroute-python python3 planning/minnesota/tools/review_cave_source.py \
  --pack content-packs/minnesota-iron-ventilation-cleft-v03.crystalpack \
  --source planning/minnesota/caves/iron-ventilation-cleft-v01/after.json \
  --output planning/minnesota/renders/iron-ventilation-live --watch
```

The warm source loop measured 0.027 seconds and preserves every source,
original-resolution image and 2× review. The first preview had an overly broad
western floor; the retained second pass adds the connected rock fin and two
usable branches. Native publication and controller review remain separate.

## Native implementation and evidence

Build `rebuild_iron_ventilation_cleft` using the pinned toolchain and shared
Cargo slot. Its arguments are `PACK SHA256 BEFORE AFTER CONTRACT FRESH_OUTPUT`.
Output must be fresh and ignored. The executed current-parent publication and
publisher hash are in `renders/iron-ventilation-v534/publication-v539.json`.
The final R13-parent publication command and successful production checks are
in `renders/iron-ventilation-v534/final-checks-v548.json`.

The private `mn_iron_vent_rock` alias copies the installed native mine graphics
and palettes exactly. Numeric07 is replaced with the existing native floor0d
and its four FLOOR quadrants. This preserves the original facility callback's
meaning: the ordinary cave07 contains a ladder and would create an unintended
shaft if used without the private override. Every other private metatile and
all existing source art remain exact. The original upper-right active shaft
quadrants retain all four unchanged shaft pairs. The dry room's unused fishing
reference becomes the active Minnesota lake group.

`audit.py --pack PACK --output FRESH` independently compares the complete
current parent, terrain, private floor pixels/collision, original trainer
parties and active regional evolution closure. It checks all other maps,
existing runtime files, audio and programs remain exact and creates the whole
floor review images. Packs and rendered output remain ignored.

The ignored controller case `iron_ventilation_controller_...` requires
`CRYSTAL_WINONA_TEST_PACK` and fresh `CRYSTAL_IRON_VENT_REVIEW_DIR`, plus separate
`CRYSTAL_NORTHSTAR_EVENTS_SAVE_DIR`. It begins from declared local chapter
access at the original mine mouth; subsequent travel, guard victories with
move PP changes, control choices and story flags use production input only.
No pressure or guard defeat flags are preseeded. The reached declined-valve
save feeds the independent stdio replay, which uses a read-only native save
verifier and records all calls. Local setup does not establish earned regional
arrival, campaign balance or a full story playthrough.

The final-parent controller case passed. The independent replay passed 780
recorded RPC calls with empty stderr. All 377 audio PCM hashes, frame counts and
loop ranges match. All-target Clippy passed; new Rust leaves pass formatting.
Full-workspace formatting retains existing unrelated differences beginning at
`offset-allocator/src/lib.rs:408`.

The eleven native camera positions cover ordinary passages, both stairs,
arrivals, the western fin, guard, valve, Vern and crosscut. All final frames have
zero pixel differences from native source references. The complete room,
actual cameras and three-floor shaft overview were visually inspected.
The overview retains one native scale; matching A–H letters identify real
warps between floors, not walking seams. Its eight pairings are checked against
the actual pack and unchanged `network.json`. Rebuild it with
`review_iron_workings_network.py --output FRESH`; the default reads the selected
pack from `world.json`.

The pump floor subsequently received its own
[native maintenance-gallery revision](../iron-pump-gallery-v01/README.md);
the drill core remains quality work. The later
[North Shore footbridge revision](../../revisions/north-shore-footpath-v01/README.md)
clarifies all seven crossings and separately verifies the current qualified
Duluth–Grand Marais round trip without Surf. That repair changes bridge
readability; the earlier report did not reproduce a missing dry collision link.
