# Route 20: upper grotto loop and seep cache

Authored against `content-packs/minnesota-gym-cave-overhaul-v02.crystalpack`.
The current `MnR20Cave1F` is **14 × 12 metatiles**, not the retired 9 × 8 layout
in older dossiers. Its final authoring entry is `MnR20Cave1F` in `rooms.json`.

## Composition and discoveries

The existing two-mouth cleft, west supply shelf, upper scout gallery, opposite
stairs, westward drops and enclosed seep stay recognizable. Twenty-one native
metatiles change. A new upper eastern loop wraps around an attached rock mass:
players can turn north from the upper gallery toward the lone-rock cache, then
continue down the eastern edge and rejoin the lower gallery. It offers an actual
second connection rather than a dead corridor ending at a prize. The direct
stair remains the shorter through route.

The seep gains a small dry foothold within its southwest wall. This gives the
previously decorative enclosed pool a concrete optional destination. The main
mouth-to-mouth route and existing rewards remain accessible without Surf.

| Discovery | Gameplay tile | Item | Access |
| --- | --- | --- | --- |
| `MN_CAVE_R20_UPPER_CACHE` | `(22,4)` | MAX_REVIVE ×1 | Dry upper loop; face the lone native rock itself |
| `MN_CAVE_R20_SEEP_CACHE` | `(11,19)` | FULL_RESTORE ×1 | Surf-only dry foothold; water approach `(12,19)` |

The upper item declares `target_surface: "rock"`: its BGEVENT_ITEM occupies
the visible WALL quadrant at `(22,4)`, with reachable ordinary-FLOOR approaches
on all four sides. Native A examines the directly faced background-event tile
without requiring it to be walkable (`world/session.rs`,
`check_interaction_checked`); this makes searching the rock discover the item.
It replaces the draft floor target `(23,4)`, which inspecting the rock would
miss. No terrain or previously published item coordinates change.

The second item declares `requires_surf: true`, with `(11,19)` also listed in
`optional_water_waypoints`. Both use native hidden-item programs and independent
new earned flags; neither has a visible item-ball actor. Existing hidden REVIVE
at `(15,15)` and HYPER_POTION ball at `(5,5)` retain their programs and receipts.

Native Johto references inspected: Union Cave B1F's branched supply landings and
water-separated return opportunities, and Mt. Mortar 1F Inside's attached rock
masses, side pockets, stairs and optional Max Revive. The latter's actual source
also has `hiddenitem MAX_REPEL, EVENT_MOUNT_MORTAR_1F_INSIDE_HIDDEN_MAX_REPEL`.
This design uses the supplied `cave` graphics and forced NITE palette; no copied
layout, invented ladder, new art catalog or external artwork is introduced.

## Preserved contract and migration

Normalized SHA256 of exact parent `{blocks,objects,events,scripts}`, sorted JSON
keys with compact separators:

`656d33b3c249ae668d9cdb36b464067b064ca36106ec1403e0260ce134c721c8`

- Safe ordinary-FLOOR arrival: `(5,20)`, facing down.
- Warp 1 remains `(5,21)` → `MnR20C00R00`, target warp 2.
- Warp 2 remains `(23,19)` → `MnR20C01R00`, target warp 2.
- Original dimensions, cave tileset, native NITE palette, music, map attributes,
  all three actors/IDs/order/positions, scripts, trainer data and existing events
  are retained. `actor_positions` is empty; `trainers` is empty.
- Retained item actor owns `EVENT_MN_R20_ITEM_01`.
- Retained native Wayfinder trainer owns `EVENT_MN_EXTRA_R20_WAYFINDER`.
- Retained hidden item owns `EVENT_MN_HIDDEN_R20CAVE1F`.
- Scout is an ordinary persistent script actor with event flag `-1`.
- No Strength/Cut actor, coordinate event, block-change program or hide-only
  collectible ownership needs carrying through a room-snapshot reset.

Reentry may clear this room's old block/object snapshots after the shared idle
checkpoint checks, while preserving these flags and all unrelated gameplay
state. Original item/trainer receipts prevent reward or battle replay. If the
player was inside this room, `(5,20)` is clear in base geometry before callbacks.

## Static checks and visual review

`target/cave-continuation-20261117/r20/review.py` reads the exact supplied pack
and the authored rows. It expands actual native collision quadrants, directional
UP_WALL entry/departure masks and retained HOP_LEFT drops into a read-only graph.
It does not instantiate a game session or run the script engine.

The retained regional trainer turns and uses radius 8 with terrain/body
occlusion. Every possible approach endpoint was checked, rather than leaving
its body at its authored origin `(21,17)`:

`(20,17), (21,15), (21,16), (21,17), (21,18), (21,19)`.

For each endpoint, both the occupied and collected item-ball states pass:

- both mouth tiles, all dry waypoints, every actor approach and existing hidden
  item approach remain reachable;
- the new upper cache targets the actual native rock and has four dry approaches;
- the new seep cache is reachable with Surf and has no dry approach;
- every reached dry and Surf position has a route back to the arrival;
- no reached Surf position touches any outer map edge;
- all used native metatiles, palette indices and physical glyph addresses exist.

There are 253 dry / 283 Surf-reachable static positions with the item ball
present and 254 / 284 after collection for each trainer endpoint. These are
source-geometry checks, not proof of battle outcomes or runtime input behavior.
The integration owner's native Rust geometry validator remains authoritative.

Inspected retained renders under `target/cave-continuation-20261117/r20/`:
`before-after.png`, `candidate-3x.png`, `upper-loop-4x.png`, `seep-cache-4x.png`,
`plain-west-passage-4x.png`, `east-arrival-4x.png`, and `shelf-choice-4x.png`.
Native 160×144 crops and full-resolution originals have immutable render
histories. A trial lower-passage protrusion was rejected after visual review
because its wall seam weakened the original rock face; the final passage keeps
its original natural boundary. The loop and secluded water landing supply the
new decisions and discoveries.
