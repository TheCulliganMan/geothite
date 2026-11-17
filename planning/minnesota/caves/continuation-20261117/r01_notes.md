# Driftless Hollow: stone alcove and ledge return

This single continuation iteration gives the eastern extension a recognizable
optional discovery. The through route stays on the broad lower floor. At its
northeastern fork, a narrow stone spur separates a higher pocket: walk around
the spur's northern tip, search the lone stone, then hop west from the ledge
at its foot. The return rejoins the lower route beside the eastern exit.

The stone contains the **existing Escape Rope**, at the same `(36,9)` event
coordinate with the same `EVENT_MN_HIDDEN_R01CAVE1F` collection receipt. The
reward gains a visible native landmark rather than another item or an extra
visible item ball. Native A interaction searches the immediately faced
background event before collision fallback; Itemfinder uses that event's
coordinate and receipt independently of its terrain. Standing at `(35,9)` and
facing right therefore searches the actual rock. The existing hidden Repel
at `(20,10)` remains beneath the central fossil ridge.

That Repel's ordinary floor needs a spoken clue instead of another repeated
stone prop. Its target is directly below the ridge's southwest corner: the
solid mass spans `x20..25, y7..9`, with `(20,10)` its first dry square below the
western tip. Nils's canonical post-battle reply recalls circling the ridge three times
and seeing a dented can under its west lip. That gives an early player a reason
to search the sheltered floor. The player may also find the Repel independently.
The four pages fit the native textbox.

## Exact source delta

The `MnR01Cave1F` entry in `rooms.json` retains the original refinement
preimage. The combined builder installs this revision directly on that parent.

Five metatiles change, using zero-based `(column,row)` coordinates:

| Position | v03 block → new block | Purpose |
| --- | --- | --- |
| `(17,3)` | `02 → 29` | Native rock caps the separating spur; the northern entry goes above it. |
| `(17,4)` | `02 → 08` | Solid west-facing wall separates the alcove from the lower through route. |
| `(18,4)` | `0a → 29` | The existing Escape Rope coordinate `(36,9)` becomes the native lower-left rock quadrant. |
| `(17,5)` | `0a → 3b` | Native west-facing ledge allows the two return jumps. |
| `(18,5)` | `2e → 0a` | Dry floor connects the alcove to the ledge while retaining its eastern wall. |

The source waypoint `(36,9)` moves to its adjacent search approach `(35,9)`;
new waypoints `(35,6)`, `(35,10)` and `(33,10)` record the entrance and ledge.
The installed hidden event itself **does not move**. The old background event
is retained rather than duplicated in `hidden_items`.

## Choice, preservation and measured return

Nils remains at `(28,5)`, facing LEFT with radius 3, using the same two legal
level-7 partners and battle receipt. His visible upper-gallery challenge can
be accepted or bypassed. Neither the new alcove nor either entrance requires
entering his ray. This preserves his dialogue about walking around the central
ridge, which remains a complete circuit.

The western fossil/HM guide, original Antidote, original climbing routes,
Lamp Gallery and self-return warps, both exterior entrances, map dimensions,
arrival `(9,10)`, palette, music, encounters and story scripts remain intact.
No actor, reward, event ID, gate or party is added or moved.

Distances below count movement edges: a native two-tile ledge hop is one edge.
They start at the Escape Rope's west-side search approach `(35,9)`.

| Destination | v03 shortest return | New ledge return | New route retraced without ledge | New outward route |
| --- | --- | --- | --- | --- |
| Eastern exit `(31,13)` | 8 | **7** | 14 | 14 |
| Western clear arrival `(9,10)` | 29 | **30** | 35 | 35 |
| Upper fork `(33,7)` | 4 | **5** | 6 | 6 |

The concrete gain is seven saved edges when leaving the new alcove toward the
eastern exit, or five toward the west, compared with retracing its entrance.
The new western return is one edge longer than the formerly open v03 floor;
this is a purposeful optional pocket, not a claim that every old route became
shorter. The ordinary lower crossing is preserved.

## Static review

`target/driftless-east-continuation/review.py` reads the actual v03 pack's native
collision quadrants, graphics, palette and installed actor sprites. It checks
ordinary movement, UP_WALL entry/departure and native HOP_LEFT transitions in a
read-only graph. It creates no session, controller or save.

All six combinations of the Antidote present/collected and Nils at `(28,5)`,
`(27,5)` or `(26,5)` retain every warp, waypoint, actor approach and hidden-item
approach. All 251 reachable resting positions before collection, or 252 after,
can return to the clear arrival. Both `(35,10) → (33,10)` and
`(35,11) → (33,11)` jumps succeed in the static graph; walking east from either
landing is blocked by the intervening wall. Forbidding all three active
trainer trigger squares still reaches every authored waypoint, entrance and
hidden-item approach, with 248 reachable positions.

The whole room and actual 160-by-144 native camera crops were reviewed beside
v03. The new stone, separating spur, northern entry and ledge landing are
visible together in the reward camera; the native rock boundary encloses the
pocket. Evidence remains ignored under `target/driftless-east-continuation/`,
including `whole-before-after.png`, `reward-before-after.png` and
`static-review.json`. Root owns final compiled-pack validation. No Cargo,
gameplay, client, controller, deployment or commit ran for this source pass.
