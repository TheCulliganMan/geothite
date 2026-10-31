# Iron Range haulage rock galleries

The complete 18×14-metatile haulage level now uses native rock galleries,
raised workings and a visible metal crosscut shutter. The previous warehouse
floor and crate aisles are replaced. Ruth, both LODE guards, all five shaft
pairs, the pressure control and every original story program remain intact.

[Whole floor](../../renders/iron-haulage-rock-v490/published-audit-v507/after-2x.png) ·
[Before and after](../../renders/iron-haulage-rock-v490/published-audit-v507/whole-before-after.png) ·
[Native cameras](../../renders/iron-haulage-rock-v490/native-v501/MnIronHaulage-production-cameras.png) ·
[Exact implementation plan](plan.json)

All coordinates below are 16px gameplay tiles. Metatile coordinates are labeled.
Both sides stay separated while haulage pressure is live. The original service
loop reaches the far workings through Ventilation and Sump. Closing the valve
at `(29,11)`, approached from `(29,12)`, opens the crosscut at metatile `(9,8)`.
That creates a direct return between the galleries. The separate ventilation
valve must also be closed before the original sump gate admits the pump gallery.
These controls do not grant later drill or rescue progress.

| Shaft | Haulage active / passive | Connected map active / passive |
| --- | --- | --- |
| Mouth | `(5,22)` / `(4,23)` | Drill Approach `(25,5)` / `(24,5)` |
| Lower haul | `(7,4)` / `(6,5)` | Sump `(5,18)` / `(4,19)` |
| Upper crossing | `(27,4)` / `(26,5)` | Ventilation `(7,4)` / `(6,5)` |
| East drop | `(29,22)` / `(28,23)` | Sump `(31,18)` / `(30,19)` |
| Service loop | `(13,12)` / `(12,13)` | Ventilation `(27,20)` / `(26,21)` |

Every native index and target remains exact. The west native30 climb occupies
metatile `(5,4)`, walking column10; the east corner36 occupies `(13,4)`, column26.
Both use y10→9→8→7 and the ordinary reverse. Ruth remains at `(5,15)`, the
weigh guard at `(14,7)`, and conveyor guard at `(30,13)`. Native sight, battles,
PP use, defeat flags and postbattle dialogue retain their original implementation.
The room adds no creature, trainer, reward, water or field-move requirement.

The closed galleries contain 170 west and 136 east free ordinary floor cells
with all three actual bodies present. Opening the shutter joins all 310 cells.
The boundary is solid native rock. Every source image, rejected pass and native
capture remains under `planning/minnesota/renders/iron-haulage-rock-v490/`.

## Source and fast review

`before.json` freezes the installed warehouse terrain. `after.json` is the
reviewed rock floor and exact native art recipes. `contract.json` binds the
complete original module and unchanged ports. `current.json` identifies the
selected source after verification. The legacy
`lairs/iron-workings/haulage.json` remains a historical campaign seed; building
that seed alone does not reproduce this current terrain. Resolve `world.json`
and its current source index before further work.

```sh
PYTHONPATH=.local/ely-backroute-python python3 planning/minnesota/tools/review_iron_haulage.py \
  --pack content-packs/minnesota-iron-haulage-rock-v03.crystalpack \
  --source planning/minnesota/caves/iron-haulage-rock-v01/after.json \
  --output planning/minnesota/renders/iron-haulage-rock-live --watch
```

The loop loads native art once, retains immutable complete originals and 2×
views, and checks both pressure states with actual bodies. Measured rendering
took 0.057–0.084 seconds after initial pack/art loading. Source rendering does not
publish or replace production input review. The initial disconnected arrival
and furniture-like barrier remain retained as rejected passes.

## Native publication and checks

Build `rebuild_iron_haulage_rock` with the pinned Rust toolchain under the shared
Cargo slot. Its arguments are `PACK SHA256 BEFORE AFTER CONTRACT FRESH_OUTPUT`.
The executed final-parent command and hashes are in
`renders/iron-haulage-rock-v490/publication-v504.json`. Output stays ignored.
All 731 other complete maps, original programs/events/actors, existing art/audio
and unrelated tables remain exact. The five new native-art files belong solely
to `mn_iron_haulage_rock`; the original cave tileset is unchanged.

The private alias imports the native control and metal grille. Its shaft moves
complete cave0b quadrants with their collision so the original upper-right
active stairs remain valid. Private block13 remains ordinary floor, preserving
the unchanged pressure callbacks. The unused fishing reference is corrected
from the old generic shore group to the Minnesota lake group. No new fishing
bank is claimed in this dry level.

`audit.py --pack PACK --output FRESH` independently compares the complete pack,
every native art recipe, the alias collision mirror, the regional guard teams
and active evolution closure. It writes review images and receipts only.
Run it with the same Python environment above.

The ignored controller case
`haulage_rock_controller_guards_pressure_network_all_five_pairs_and_continue`
requires `CRYSTAL_WINONA_TEST_PACK`, a fresh `CRYSTAL_HAULAGE_ROCK_REVIEW_DIR`
and a separate `CRYSTAL_NORTHSTAR_EVENTS_SAVE_DIR`. It follows the original
multi-floor pressure course, refusal/retry, both valves, the two-circuit sump
gate, actual guard battles, both climbs and all five pairs in both directions.
Later drill/rescue flags stay unchanged. Local partner/position/badge/supply
setup is declared in `plan.json`; it does not establish earned regional access
or campaign balance.

`check_mcp.py --pack PACK --client CLIENT --save REACHED_DECLINED_VALVE_SAVE
--output FRESH` independently replays the reached control, physical denial,
refusal/retry, actual crosscut, every pair, clean stderr and fresh-process
Continue. Its walking replans after real dialogue notices release movement.
The first replay exposed an older helper that kept sending movement into an
owned text window; that failed recording is retained.

The native camera sheet was inspected. All nine full-frame comparisons have
zero differences. The later parent only adds the retained Route 6 work;
the haulage module and its private native art remain identical to those captures.
Selection and final verification receipts record the actual pack and source
bindings. Broader cave work remains active, including the other mining floors.
