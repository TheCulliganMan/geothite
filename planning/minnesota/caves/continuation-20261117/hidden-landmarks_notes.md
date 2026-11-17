# Search the visible cave landmark

The Route 10 and Route 12 entries in `rooms.json` refine two discoveries from
the local v02 review pack. Both use native hidden items with their original
collection receipts.

| Room | Change from v02 | Why |
| --- | --- | --- |
| Ore Line, `MnR10Cave1F` | Move `MN_CAVE_ORE_NUGGET` from floor `(26,5)` onto the existing rock `(27,5)`. No metatile changes. | Facing the obvious rock should find the Nugget; targeting the floor beside it makes that natural search fail. |
| Gunflint, `MnR12Cave1F` | At zero-based metatile row 2, column 6, replace native cave block `02` with `28`. Move `MN_CAVE_GUNFLINT_ELIXER` from `(12,5)` onto its rock `(12,4)`. | The upper shelf gains a visible reason to investigate its western end. The floor at `(12,5)` remains the approach and navigation waypoint. |

Both items declare `target_surface: "rock"`. The Nugget's native block `2b`
contains `[FLOOR, FLOOR, FLOOR, WALL]`; the Elixir's block `28` contains
`[WALL, FLOOR, FLOOR, FLOOR]`, ordered top-left, top-right, bottom-left,
bottom-right. These are the existing native stone graphics and collision
quadrants, not invisible walls or new prop art. All other rows, item quantities,
item IDs, event IDs, arrivals, warps, actors and trainer teams are unchanged.
The canonical item token remains `ELIXER`, matching the native item catalog.

The runtime checks the immediately faced background-event square before its
collision fallback (`crystal-core/src/world/session.rs`,
`check_interaction_checked`). It does not require the event square to be
walkable. `find_itemfinder_hidden_item` in `systems/field_items.rs` searches
uncollected `BGEVENT_ITEM` coordinates and their exact hidden-item receipts
without a terrain restriction. Thus the player can stand beside the rock,
face it and press A; Itemfinder can still detect it. Keep the existing
`EVENT_MN_CAVE_ORE_NUGGET` and `EVENT_MN_CAVE_GUNFLINT_ELIXER` receipts so an
already collected local-review item cannot be awarded again.

## Static evidence

The native-art review uses the exact v02 pack, forced NITE palette, authored
rows and installed actor sprites. It produced whole rooms and 160-by-144
before/after camera crops under `target/northstar-hidden-landmarks/final/`.
Both whole rooms and the enlarged camera comparisons were inspected.

A limited graph using native collision quadrants and fixed installed actor
positions confirms all four neighboring floor approaches and paths back to
the clear arrival:

| Rock | Reachable adjacent floor tiles | Shortest approach from arrival |
| --- | --- | --- |
| Ore Line `(27,5)` | `(27,4)`, `(28,5)`, `(27,6)`, `(26,5)` | 32 steps |
| Gunflint `(12,4)` | `(12,3)`, `(13,4)`, `(12,5)`, `(11,4)` | 24 steps |

The production cave validator separately owns exhaustive trainer approach
positions and collected-item states. These authoring checks do not establish
battle outcomes, runtime playthrough acceptance, publication or deployment.
No controller, game session, client, Cargo build or gameplay inputs were used.

The other four original discoveries remain useful terrain rewards: Driftless's
sheltered ridge toe, Ore Line's spring rim, and the two dry Voyageurs ice
landings. Their navigation landmarks already exist. Adding more stones or
moving those events would change the intended floor-search and ice-landing
behavior without as clear an improvement.
