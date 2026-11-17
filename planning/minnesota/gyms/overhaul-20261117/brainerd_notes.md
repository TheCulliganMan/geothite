# Brainerd: two-bank dock routing

Authoring candidate on exact refinement v02 (`a9f23e3b...f37a9527`). This is a
static design, not an installed room or gameplay verification. Keep current
parties, native casts, progression, rematches and circuit scripts when applying
this source over the existing room.

LORLA remains the first required match. The winch then offers a real route
choice: CENTER reaches BRAVEL's training pier; beating him opens the leader
court. LEFT provides the alternate leader approach without that optional
battle. A second visible winch on the north landing removes the compulsory trip
back to the south control. Both controls operate the same floating walk. The
middle landing is widened to permit passing around its trainer and reaching
the north winch. The compact room stays5x8 metatiles.

## Geometry and actors

Retain `MnBrainerdGym`, `MN_BRAINERD_GYM`, `mn_gym_6`, arrival (4,14), exits
(4,15)/(5,15), original exterior warp records1-3, guide (7,14), LORLA (2,10) and
SIVREN (5,3). BRAVEL moves (2,7) -> (4,6), preserving his identifier/slot.
LORLA faces right, radius1; BRAVEL faces down, radius2. Use explicit direction
overrides. BRAVEL remains optional individually; the chosen route determines
whether his win is needed for the court.

## Control and qualification contract

SOUTH_WINCH at (6,13) and NORTH_WINCH at (1,7) use the same native two-choice
menu (`toggle:false`). CENTER and LEFT communicate the selected route without
requiring a blind second press. Both modes require
`EVENT_MN_GYM_6_TRAINER_1_DEFEATED`. Their fresh mutually exclusive state flags
are `EVENT_MN_GYM_6_V2_WALK_CENTER` and `EVENT_MN_GYM_6_V2_WALK_LEFT`. Keep all
old flag numbers, including the former routing names, allocated and unchanged.

The court predicate is:

- all: `EVENT_MN_GYM_6_TRAINER_1_DEFEATED`
- any: `EVENT_MN_GYM_6_V2_WALK_LEFT`, `EVENT_MN_GYM_6_TRAINER_2_DEFEATED`

Compute existing COURT_OPEN from that predicate before applying the court
bridge. Do not require BRAVEL to select CENTER: that would create a circular
prerequisite. Each console stands on a bank outside every moving block.

| x | y | Off | On | Requires | New visual receipt suffix |
| --- | --- | --- | --- | --- | --- |
| 2 | 4 | C | v | LORLA win + V2_WALK_CENTER | V2_CENTER_WALK_OPEN |
| 1 | 4 | B | v | LORLA win + V2_WALK_LEFT | V2_LEFT_WALK_OPEN |
| 3 | 2 | A | j | EVENT_MN_GYM_6_COURT_OPEN | V2_COURT_OPEN |

Coordinates above are metatiles. Receipt suffixes use `EVENT_MN_GYM_6_`.
Source guide, prompts and after-lines are all ASCII, at most18 columns, ready
for canonical story integration. Existing completed victories retain access.

## Native water protection

Use `wooden_port_deck` and the explicit `buoy_quadrants` from the JSON. They
place actual native floating-barrier pixels and BUOY collision at every exposed
walk/water boundary. In addition to the older room's rails, the new source
protects the south pier, both closed bridge gaps, the widened middle landing,
the right pool edge and the water alongside the leader dock. Open `v` retains
a one-tile walking strip with buoy rails, rather than an exposed water flank.
Do not replace the buoy declaration with an invisible wall or blanket ban on
Surf/canoe. The visible physical boundary provides the protection.

## Static evidence and limits

Initial, CENTER, LEFT and CENTER-after-BRAVEL states passed an authored-cell
reachability check with actors blocking their cells. LORLA and the south winch
are accessible initially; the north winch and BRAVEL become
accessible on either open route. The leader is inaccessible initially and on
unqualified CENTER, but accessible on LEFT or qualified CENTER. Both exits
remain reachable. Across every reachable state, the model found zero
walkable-to-WATER adjacency edges, closing mounting bypasses. Native glyph
allocation verified below192. No game, controller, battle or save fixture was executed.

Whole-room, arrival and leader views of all four states are retained in
`target/gym-overhaul-20261117/balance-postgame/v04/brainerd/`. Earlier drafts and
reference sheets are retained alongside them. The source renderer faces actors
south; installed LORLA must use the contract's right-facing override. Her
one-tile sightline prevents approach movement from occupying the LEFT route's
single south landing at (3,10), preserving the optional bypass. Migration
must translate old dock block overrides and BRAVEL's old position deliberately.
