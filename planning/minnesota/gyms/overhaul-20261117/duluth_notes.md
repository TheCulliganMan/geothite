# Duluth: stop, turn, combine

Authored against the selected `minnesota-refinement-v02.crystalpack`, raw
content identity `a9f23e3bc1dadad74c377d02c23df61dcf9ed70c1b1d5e3918805593f37a9527`.
This is a source design and static review, not a built or selected room and
not gameplay verification. The matching JSON uses the existing Gym schema;
its party data matches the selected pack. Integration preserves installed
teams, badge/TM rewards, return flow and rematches.

Mahogany's actual supplied native map was rendered and inspected as the
visual reference: coherent blue ice, gray rough stops, visible obstacles and
ordinary footing. Its short sequence of directional turns illustrates the
spatial rule used here. This is a new course rather than that map's layout.
[Crystal Mahogany reference](https://guidestrats.com/pokemon-crystal-tm16-icy-wind/).
Every selected glyph comes from the native `elite_four_room` tileset.

## Geometry and actors

- Keep `MnDuluthGym`, `MN_DULUTH_GYM`, 7x10 metatiles; arrival `(6,17)` facing
  down and exits `(6,19)`, `(7,19)`. Preserve the three exterior warp indices.
- Keep Norek at `(8,3)` and the guide at `(10,17)`. Arrival is unoccupied
  FLOOR before callbacks. Norek waits for interaction.
- Livra / `MN_GYM_8_TRAINER_1`: `(3,10)`, down, radius 1. Her dry `(3,11)`
  sightline is reached from the lower rink's west landing.
- Tordel / `MN_GYM_8_TRAINER_2`: `(4,6)`, right, radius 2. The main upper
  crossing lands at `(5,6)` in his sightline; `(6,6)` is also in the ray.
- Anver / `MN_GYM_8_TRAINER_3`: `(2,7)`, right, radius 1. The optional branch
  crosses `(3,7)` and starts his native battle. Required routes avoid it.
- All three trainer IDs, object indices and native sight battles remain.
  Anver is optional; only Livra and Tordel qualify the leader challenge.

## Ice and dynamic controls

`I` is native ICE in all four quadrants. `a` has a rough FLOOR top-left
quadrant and three ICE quadrants; `b` has rough FLOOR top-right. Nothing
recolors ordinary floor into fake ice or changes the engine sliding rule.

1. From the dry arrival, the lower rink's rough stop is `(6,12)`. Turning
   left stops against the west wall at `(2,12)`; up reaches dry `(2,11)`.
   Step right to Livra's `(3,11)` sightline.
2. Livra opens the east landing at metatile `(5,5)`. From this landing the
   upper course offers real slide/stop choices, including rough `(10,6)`
   and `(11,8)`. The west platform contains Tordel and optional Anver.
3. The combined court approach uses rough `(11,8)` -> left to `(7,8)` -> up
   through `(7,5)` once qualified. One explicit upper sequence from `(10,6)`
   is right to wall-stop `(11,6)`, down to `(11,8)`, left to `(7,8)`, up.
   Other valid sliding routes are intentional; stopping on rough ground,
   using walls, and turning are the spatial rule rather than a hidden code.

| Gate | Closed/open source | Requirement and retained event |
| --- | --- | --- |
| East entry, metatile `(5,5)` | `G = elite_four_room:14` -> `. = elite_four_room:2d` | Livra defeated; `EVENT_MN_GYM_8_EAST_ENTRY_OPEN = 2270`. |
| West recovery, metatile `(1,4)` | `V = elite_four_room:14` -> `.` | Norek defeated; `EVENT_MN_GYM_8_WEST_RETURN_OPEN = 2271`. |
| Court, metatile `(3,2)` | `C = elite_four_room:14` -> `.` | Livra AND Tordel defeated; `EVENT_MN_GYM_8_COURT_OPEN = 2273`. |

Keep `EVENT_MN_GYM_8_EAST_RETURN_OPEN = 2272` registered for old saves but
remove its obsolete geometry action. Recompute all active gate states from
the declared win requirements on every TILES refresh, including clearing a
stale derived gate flag on the closed branch. A prior West Return receipt
alone must not open the redesigned west passage before Norek.

The west recovery gate stays closed until the leader is defeated: opening
it immediately after Tordel would bypass the final ice combination. Before
victory, return through the already-open east landing. After victory, the
west dry link gives a short route back through Livra's landing. No attempt
requires restarting a scene or leaving the map to recover from a wrong slide.

## Dialogue and static review

Proposed ASCII dialogue, at most 18 columns per line, is in the JSON for
root to copy into canonical story. The guide explains rough stops; Livra
points to the upper course; Tordel explains the last combination after his
battle. Anver marks the optional lap and accurately describes the exit.
The existing Waterfall/next-route handoff remains in Norek's after-dialogue.

Whole native SourceArt rooms, the original Mahogany reference and 160x144
source crops were inspected under ignored
`target/gym-overhaul-authoring/build-speed/duluth-*`; immutable originals
and render histories are retained. The initial version had an unintended
west bypass into the upper course; the new west recovery gate closes it.
The final composition uses one continuous ice vocabulary, gray stops/gates
and dry blue footing, removing the competing gold gates and narrow parallel
lanes of the previous room.

The static ICE resting-state graph, with actual actor bodies, establishes:

- Initial state: 33 reachable resting tiles; only Livra approachable.
- Livra won: 53 resting tiles; Tordel and Anver approachable, court blocked.
- Required trainers won: 76 resting tiles; Norek approachable, west recovery
  still closed. Anver's victory is not required.
- Leader won: 79 resting tiles; west recovery open.
- Every reachable resting tile in these phases has a path back to arrival.

These are source/collision results, not native controller, battle, save,
production rendering or ordinary-party balance verification.
