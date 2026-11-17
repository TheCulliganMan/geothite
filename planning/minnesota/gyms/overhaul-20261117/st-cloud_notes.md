# St. Cloud: reversible airworks

Authoring candidate on exact refinement v02 (`a9f23e3b...f37a9527`). This is a
static design, not an installed room or gameplay verification. Apply source
geometry and the companion contract over the selected room's live scripts.

The wooden training deck, steel east platform and upper control platform form
one airworks. Two slatted crossings replace the old four fixed bands. WYREL
qualifies the player to operate the fans. The lower crossing must blow east to
reach the other bank; the upper crossing must blow west to reach SAVEN. HALLEN
is optional. Each fan has a matching console at each end, so a player on either
bank can restore the direction needed for a return. There is no timer or
invisible fall/reset penalty. Visible native arrows change with the collision.

## Retained room and actors

Retain `MnStCloudGym`, `MN_ST_CLOUD_GYM`, `mn_gym_5`, dimensions9x11 metatiles,
arrival (4,19), exits (4,21)/(5,21), original exterior warp records1-3 and all
four actor positions/identifiers. WYREL (4,15) faces right, radius3; HALLEN
(14,10) faces left, radius2. Explicit direction overrides prevent the general
Minnesota eight-tile sightline rule from widening these authored lanes.

## Controls and state

These are native background controls on solid computer blocks. Coordinates are
walking tiles. Either bank's copy changes the same two exclusive mode flags.
Every router mode requires `EVENT_MN_GYM_5_TRAINER_1_DEFEATED`. `toggle:true` is
intentional: first interaction on a fresh room selects the useful direction.

| Router | Control tile | First mode / flag suffix | Second mode / flag suffix |
| --- | --- | --- | --- |
| LOWER_WEST_BANK | (6,17) | EAST / V2_LOWER_EAST | WEST / V2_LOWER_WEST |
| LOWER_EAST_BANK | (13,17) | EAST / V2_LOWER_EAST | WEST / V2_LOWER_WEST |
| UPPER_WEST_BANK | (6,9) | WEST / V2_UPPER_WEST | EAST / V2_UPPER_EAST |
| UPPER_EAST_BANK | (13,9) | WEST / V2_UPPER_WEST | EAST / V2_UPPER_EAST |

All suffixes above use `EVENT_MN_GYM_5_`. Default lower direction is WEST;
default upper direction is EAST. No mode flags initially set means these row
preimages remain. Opposite mode flags are mutually exclusive. The paired
routers deliberately share flag names; generic old validator uniqueness rules
must not reject that approved shared-control relationship.

Block transitions use metatile coordinates:

| x | y | Off | On | Requires | Receipt suffix |
| --- | --- | --- | --- | --- | --- |
| 4 | 7 | < | > | V2_LOWER_EAST | V2_LOWER_TILE_A |
| 5 | 7 | < | > | V2_LOWER_EAST | V2_LOWER_TILE_B |
| 4 | 3 | > | < | V2_UPPER_WEST | V2_UPPER_TILE_A |
| 5 | 3 | > | < | V2_UPPER_WEST | V2_UPPER_TILE_B |

`court.all = [EVENT_MN_GYM_5_TRAINER_1_DEFEATED,
EVENT_MN_GYM_5_V2_UPPER_WEST]`; `court.any=[]`. The upper crossing itself
physically protects the leader approach; no additional opaque grille is needed.
The old LOWER_CROSSING_OPEN and original gate constants remain allocated but
must not drive the new geometry. Completed victories remain valid for returns
and rematches. Refresh both arrows and forced-walk collision from the same
resolved glyph; never change only the visible arrow.

## Static evidence and limits

All four direction combinations were checked using an authored-cell model of
horizontal forced walking, with actors occupying their real tiles. Only
lowerEAST/upperWEST reaches the leader from the entry. From each of the three
platforms, its local console remains approachable under every direction state;
changing the controls provides the reverse route. Both lower bank consoles,
both upper bank consoles and optional HALLEN have valid approaches. There is no
WATER terrain. Native glyph allocation82 of192 includes both marked wind glyphs.

Whole-room, arrival and leader views for initial, outbound and both-east states
are retained in `target/gym-overhaul-20261117/balance-postgame/v04/st-cloud/`.
The stock source renderer faces all actors south; the installed contract must
use WYREL's right-facing and HALLEN's left-facing overrides. This evidence does not execute
controller inputs, battles, save restoration or native runtime rendering.
