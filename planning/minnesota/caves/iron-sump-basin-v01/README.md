# Iron Range flooded sump

The complete 20×12-metatile floor replaces the old warehouse aisles with a
flooded western drift, two proper rock climbs and separate east workings.
The original pump pocket remains sealed until both native pressure circuits
are off. All five shaft pairs, both guards and the original story programs
retain their exact identities and positions.

[Whole floor](../../renders/iron-sump-basin-v517/published-audit-v521/after-2x.png) ·
[Before and after](../../renders/iron-sump-basin-v517/published-audit-v521/whole-before-after.png) ·
[Native player cameras](../../renders/iron-sump-basin-v517/native-v522/MnIronSump-production-cameras.png) ·
[Implementation plan](plan.json)

Selected local pack: `content-packs/minnesota-iron-sump-basin-v02.crystalpack`.
[Acceptance](../../renders/iron-sump-basin-v517/acceptance-v531.json) and
[selection proof](../../renders/iron-sump-basin-v517/post-selection-v531.json)
bind its exact parent, pack, source and native evidence. The original camera
captures use v01; v02 preserves all mine modules and render inputs exactly.

All unlabeled coordinates are 16px walking tiles. The western Haulage arrival
is `(4,19)`. The bank bends around a 38-cell basin to two separate climbs and
the upper Ventilation shaft. Ordinary stairs30 at metatile `(2,3)` use column4;
right-corner36 at `(6,3)` use column12. Both climbs use y8→7→6→5 and the ordinary
reverse. The sump guard stays at `(14,10)`, approached from `(13,10)`.

The basin is optional. Cast from `(12,12)` facing left, or use native Surf to
enter `(11,12)`. A visible solid rock lip blocks the southern exit at `(11,15)`;
all outer perimeter cells are wall. Surf cannot cross to the eastern workings,
bypass the pressure gate or escape the map. The native Minnesota lake fishing
group replaces the old generic shore reference.

The service loop reaches the eastern floor through Ventilation. The drain
guard stays at `(25,17)`. The grille at metatile `(16,3)` blocks the pump pocket
until both `EVENT_MN_IRON_HAUL_PRESSURE_OFF` and
`EVENT_MN_IRON_VENT_PRESSURE_OFF` are set by their original controls.
The gate read tile is `(32,7)`, approached from `(32,8)`; the original callback
opens block13. Later drill, test, shutdown and rescue flags are not granted.

| Pair | Sump active / passive | Other map active / passive |
| --- | --- | --- |
| West haul | `(5,18)` / `(4,19)` | Haulage `(7,4)` / `(6,5)` |
| Upper west | `(9,4)` / `(8,5)` | Ventilation `(5,20)` / `(4,21)` |
| East haul | `(31,18)` / `(30,19)` | Haulage `(29,22)` / `(28,23)` |
| Pump access | `(35,4)` / `(34,5)` | Pump Gallery `(5,18)` / `(4,19)` |
| Service return | `(25,10)` / `(24,11)` | Ventilation `(29,4)` / `(28,5)` |

Closed free floor counts include actual guard bodies: west136, east117,
pump pocket13. Opening the gate joins the east and pocket into134 cells;
the west still requires its actual shaft network. This floor adds no trainer,
item, reward or mandatory field-move requirement.

## Source and fast review

`before.json` freezes the installed source, `after.json` contains the complete
reviewed floor and native quarter recipes, and `contract.json` binds the entire
original module and its ten warp records. Selection uses `current.json` and the
active `world.json` source index after final verification. The old
`lairs/iron-workings/sump.json` is a campaign seed; publishing it alone does not
reproduce this floor.

```sh
PYTHONPATH=.local/ely-backroute-python python3 planning/minnesota/tools/review_iron_sump.py \
  --pack content-packs/minnesota-iron-sump-basin-v02.crystalpack \
  --source planning/minnesota/caves/iron-sump-basin-v01/after.json \
  --output planning/minnesota/renders/iron-sump-live --watch
```

The measured warm source loop took 0.026 seconds. It retains full originals,
2× views and rejected passes. The first lower shore corners produced floating
boulders; private blocks43/44 now combine the upper native water-edge half with
the lower native rock-face half, and matching WATER/WATER/WALL/WALL collision.
The alias keeps all original graphics and palettes. Renderer behavior explicitly
uses the original cave animation so the basin moves at native timing.

## Publication and validation

Build `rebuild_iron_sump_basin` with the pinned toolchain under the shared Cargo
slot. Its exact arguments are `PACK SHA256 BEFORE AFTER CONTRACT FRESH_OUTPUT`.
The output must be fresh and ignored. Reopen with strict native validation.
`audit.py --pack PACK --output FRESH` independently compares the whole parent,
all native shore quarters/collision, original guard teams and their active
regional evolution closure, fishing species and unrelated content.

The three ignored `iron_sump_` controller tests require
`CRYSTAL_WINONA_TEST_PACK`, a fresh `CRYSTAL_IRON_SUMP_REVIEW_DIR`, and separate
`CRYSTAL_NORTHSTAR_EVENTS_SAVE_DIR`. They test exact animation frames and clock
schedules; all four pressure flag states with physical gate denial and Continue;
and the original full pressure course, guard battles with move PP changes,
all five shaft pairs both directions, both climbs and all 38 WATER cells.
Local setup supplies the declared regional party, eight badges and field tools
before first input. It does not establish earned regional access or balance.

`check_mcp.py --pack PACK --save REACHED_BANK_SAVE --client CLIENT
--verifier READONLY_NATIVE_SAVE_HELPER --output FRESH` independently casts the
registered rod, proves a real regional wild battle, uses Run, launches Surf,
checks the rock lip, continues in Surf in a new process, dismounts and traverses
all five pairs before returning through the real mine mouth. It reads native
saves without mutating them, records every RPC, and requires empty stderr.
The first rejected replay overpressed A after a failed cast and inadvertently
accepted Surf. Its recording is retained; the corrected retry waits for the
actual overworld state before casting again.

Ten actual native camera comparisons have zero pixel differences, including
ordinary bank passages, both climbs, guards and the closed pump access.
The complete floor and player camera sheet were visually inspected. Final
acceptance, parent preservation and selection receipts live under
`renders/iron-sump-basin-v517/`. Ventilation and the pump/drill floors still need
their own quality work; this revision does not complete the broader cave goal.
