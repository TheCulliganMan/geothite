# Bemidji: two clearings in the Northwoods garden

Authored against the selected `minnesota-refinement-v02.crystalpack`, raw
content identity `a9f23e3bc1dadad74c377d02c23df61dcf9ed70c1b1d5e3918805593f37a9527`.
This is a source design and static review, not a built or selected room and
not gameplay verification. The matching JSON uses the existing Gym schema.
Its party data was refreshed from the selected pack; integration preserves
the installed teams, rewards, rematches and guide gift wrapper.

The reference is Eterna's forest gym: trees form the room and reveal separate
clearings, rather than placing plants against an ordinary station interior.
The original routes and Cut choices here are specific to Bemidji.
[Eterna Gym reference](https://bulbapedia.bulbagarden.net/wiki/Eterna_Gym).
Native johto/forest/kanto/park pixels and collision are reused unchanged.

## Geometry and actor contract

- Keep map `MnBemidjiGym`, constant `MN_BEMIDJI_GYM`, 6x10 metatiles.
- Keep arrival `(4,17)`, facing down, and exterior exit tiles `(4,19)`,
  `(5,19)`. Keep all three exterior warp indices/targets.
- Keep Orlis at `(5,3)` and the guide at `(9,17)`. The arrival is ordinary
  unoccupied FLOOR before callbacks. The source room is pine-enclosed; the
  remaining red entrance carpet is the only imported entrance treatment.
- `MN_GYM_7_TRAINER_1` / Moslen: `(4,9)`, down, radius 2. Its reachable
  sightline is `(4,10)` and `(4,11)` in the western clearing.
- `MN_GYM_7_TRAINER_2` / Elvren: `(8,4)`, down, radius 1; `(8,5)` is the
  approach after the northern tree.
- `MN_GYM_7_TRAINER_3` / Brindel: `(2,8)`, right, radius 1. Stepping into
  `(3,8)` takes the optional battle. Moslen can be reached without that tile.
- All three trainers use native battles on sight. Only the leader waits for
  interaction. Preserve original actor identifiers and indices.

## Cut, qualifications and return

| Control | Native position | Behavior and persistent receipt |
| --- | --- | --- |
| Guide HM | `(9,17)` | Preserve the original full-Pack-safe gift wrapper and `EVENT_MN_GYM_7_CUT_HM_RECEIVED = 2302`; never set the receipt on a failed gift. |
| SOUTH_HEDGE | tree `(3,12)`, approach `(3,13)` | `kanto:32` -> `kanto:6d`; seals west target `(3,11)` plus Moslen and optional Brindel. `EVENT_MN_GYM_7_CUT_SOUTH_HEDGE_PRUNED = 2304`. |
| NORTH_HEDGE | tree `(7,6)`, approach `(7,7)` | Same exact native Cut replacement; seals north target `(7,5)` and Elvren. `EVENT_MN_GYM_7_CUT_NORTH_HEDGE_PRUNED = 2303`. |
| Return gate | metatile `(3,6)` | Closed symbol `G = underground:0b`; open `.` = `forest:01`, only after `EVENT_MN_GYM_7_DEFEATED`. Preserve `EVENT_MN_GYM_7_RETURN_OPEN = 2693`. |

Both Cut approaches are reachable from the entrance with both trees closed.
The east passage uses a standing native pine (`o`, kanto:6d) to keep its route
at x=9; this is already a plain WALL/FLOOR block, not an undeclared Cut tree.
Each tree independently seals its target even with the other tree removed
and with all NPC bodies omitted. There are exactly two CUT_TREE quadrants.

Remove the old prerequisite that Moslen must be defeated before Elvren.
The two clearings can be attempted in either order. Orlis requires both
`EVENT_MN_GYM_7_TRAINER_1_DEFEATED` and
`EVENT_MN_GYM_7_TRAINER_2_DEFEATED`; Brindel is not required. Entering Orlis's
clearing after only the northern Cut can reveal him, but the normal leader
qualification reply prevents starting his battle before both wins.

Rebuild pruned blocks and the victory gate in the idempotent TILES callback.
Keep old numeric constants. These are retained tree locations and receipts,
so an already-pruned tree remains pruned after migrating room snapshots.
Do not clear trainer victories, badge/TM receipts or rematch state. The return
gate's closed branch must restore its authored block and clear its derived
gate flag when the leader-win condition is absent.

## Dialogue and review

The JSON contains proposed guide, trainer and leader lines, all ASCII and
at most 18 columns. Root copies the accepted lines to canonical story data;
the JSON alone is not a live dialogue change. The guide teaches the choice
and identifies the optional battle. Orlis retains the Lake Itasca and
Whirlpool handoff. Preserve the gift/retry, reward and rematch script bodies.

Native SourceArt whole rooms and 160x144 crops were inspected under ignored
`target/gym-overhaul-authoring/build-speed/bemidji-*`, including original
pixels, enlarged views and immutable render history. The bright green native
pine boundary now reads as the place itself. Station planters, repeated blue
brick borders and the wide platform floor are removed. The western side
clearing is compact, while the right-hand passage visibly connects the two
choices; the single closed return gate is distinct from the Cut trees.

Static source collision checks establish both sealed Cut targets and all
actor/arrival/exit surfaces. With both trees pruned, `(7,5)` to the arrival
uses 19 walking steps before victory and 15 after the center return opens.
These measurements include NPC bodies. They do not establish native input,
Cut UI, battle behavior, Continue or rendering in a production client.
