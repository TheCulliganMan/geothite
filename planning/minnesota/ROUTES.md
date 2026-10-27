# Minnesota route authoring index

Use the exact [world](world.json), [story](story.json), route dossier and explicit
layout together. Extents below reflect current source, including proposals
awaiting native publication. A source render is not production input evidence.
[STATUS.md](STATUS.md) and each linked runbook distinguish current native checks
from superseded captures and unfinished regional progression.

Every dossier has matching `routes/MN_Rxx.layout.json`; its core coordinates,
actors/teams, speech, events, room exits and route ports are implementation
contracts. One metatile is four 16px gameplay tiles. Clipped chunks are at most
32×24 metatiles. Read native collision quadrants before placing actors or
claiming a fence/rail/tree blocks travel.

| Route / exact runbook | Planned regional connection | Current source extent | Story / required conditions |
| --- | --- | --- | --- |
| [MN_R01 Driftless Trail](routes/MN_R01.md) | WINONA → ROCHESTER | 32×20; 1 chunks; 2 rooms | Q01 / EVENT_MN_STARTER_RECEIVED |
| [MN_R02 Prairie Farm Road](routes/MN_R02.md) | ROCHESTER → MANKATO | 30×22; 1 chunks; 2 rooms | Q02 / BADGE_1 |
| [MN_R03 Minnesota River Valley](routes/MN_R03.md) | MANKATO → MINNEAPOLIS | 24×32; 2 chunks; 1 rooms | Q02 / BADGE_2 |
| [MN_R04 Twin Cities River Crossing](routes/MN_R04.md) | MINNEAPOLIS → ST PAUL | 28×18; 1 chunks; 0 rooms | Q03 / BADGE_3 |
| [MN_R05 River Bluffs North](routes/MN_R05.md) | MINNEAPOLIS → ST CLOUD | 32×56; 3 chunks; 1 rooms | Q03 / BADGE_3, BADGE_4 |
| [MN_R06 Pine Country Road](routes/MN_R06.md) | ST CLOUD → BRAINERD | 32×44; 2 chunks; 1 rooms | Q04 / BADGE_5 |
| [MN_R07 Leech Lake Approach](routes/MN_R07.md) | BRAINERD → LEECH LAKE | 32×48; 2 chunks; 0 rooms | Q04 / BADGE_6 |
| [MN_R08 Northern Lake Circuit](routes/MN_R08.md) | LEECH LAKE → BEMIDJI | 40×48; 4 chunks; 0 rooms | Q05 / BADGE_6 |
| [MN_R09 Iron Range West](routes/MN_R09.md) | BEMIDJI → GRAND RAPIDS | 64×36; 4 chunks; 1 rooms | Q06 / BADGE_7 + EVENT_MN_HEADWATERS_REPORT |
| [MN_R10 Iron Range Descent](routes/MN_R10.md) | GRAND RAPIDS → DULUTH | 48×20; 2 chunks; 1 rooms | Q06 / BADGE_7 + EVENT_MN_DEPOT_CLEARED |
| [MN_R11 North Shore Trail](routes/MN_R11.md) | DULUTH → GRAND MARAIS | 32×72; 3 chunks; 1 rooms | Q07 / all eight badges + EVENT_MN_LIGHTHOUSE_SIGNAL |
| [MN_R12 Gunflint Trail](routes/MN_R12.md) | GRAND MARAIS → BOUNDARY WATERS | 64×56; 6 chunks; 2 rooms | Q07 / all eight badges + EVENT_MN_LIGHTHOUSE_SIGNAL |
| [MN_R13 League Approach](routes/MN_R13.md) | BOUNDARY WATERS → ELY | 32×64; 3 chunks; 1 rooms | Q08 / all eight badges + EVENT_MN_SURVEY_RESTORED |
| [MN_R14 Headwaters Trail](routes/MN_R14.md) | BEMIDJI → LAKE ITASCA | 48×32; 4 chunks; 0 rooms | Q05 / BADGE_6 |
| [MN_R15 Central Lakes West](routes/MN_R15.md) | BRAINERD → DETROIT LAKES | 32×18; 1 chunks; 1 rooms | SQ02 / BADGE_6 |
| [MN_R16 Red River Prairie](routes/MN_R16.md) | DETROIT LAKES → MOORHEAD | 48×24; 2 chunks; 0 rooms | SQ02 / BADGE_6 |
| [MN_R17 Border Pines](routes/MN_R17.md) | BEMIDJI → INTERNATIONAL FALLS | 32×56; 3 chunks; 2 rooms | SQ03 / BADGE_7 |
| [MN_R18 Mille Lacs Shore](routes/MN_R18.md) | BRAINERD → MILLE LACS | 40×24; 2 chunks; 0 rooms | SQ04 / BADGE_6 |
| [MN_R19 Prairie Forest Link](routes/MN_R19.md) | LAKE ITASCA → DETROIT LAKES | 32×30; 2 chunks; 0 rooms | SQ01 / BADGE_6 |
| [MN_R20 Voyageurs Border Loop](routes/MN_R20.md) | INTERNATIONAL FALLS → BOUNDARY WATERS | 64×32; 4 chunks; 1 rooms | SQ03 / all eight badges + EVENT_MN_LIGHTHOUSE_SIGNAL |

The Boundary Waters preserve is a separate [Safari proposal](playable/boundary-waters-safari.md),
with a free story basecamp. Its four candidate preserve chunks are excluded from
the registered world inventory. R12/R13/R20 route access must remain independent
of paid expedition budgets.

Current worthwhile route features include directional ramp/drop returns,
necessary protected crossings, grass that changes route choice, distinct shore
turnouts, real rooms/services and Cut gates that seal an actual target or useful
link. No quota requires every route to contain each feature. Keep the varied
sizes and identities; inspect the weakest through camera as well as landmarks.

The [connected progress atlas](renders/atlas/minnesota-current-connected.png)
composites actual maps at one scale. Teal requires exact current source/native
terrain and actor agreement. Amber includes source revisions whose older pack
is historical. Dashed hub links reserve travel that remains to be implemented.
