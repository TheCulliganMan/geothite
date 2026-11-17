# Rochester — choose your practice lane

`rochester.json` uses the complete `rochester-training-hall-v01.json` schema as its template. This is a geometry/dialogue overlay proposal for the selected refinement-v02 pack, not a replacement historical recipe. Preserve installed trainer identities, teams, badge/TM claims, Champion rematches and exterior warp indices. The copied party fields are template data; the battle owner's current teams remain authoritative.

The room is tightened from 7×10 to 6×9 metatiles (12×18 walking tiles). Arrival is `(6,15)` and exits are `(6,17)` / `(7,17)`. Preserve the three existing warp indices and their exterior destinations while updating these coordinates. Accepted VAREL art is `SPRITE_MN_LEADER_VAREL`. The whole room becomes a timber training court with genuine native `tower:09` timber flooring, a small leader court framed by the existing native pillars/posts, one entry rail and two visibly separate practice lanes. Native solid timber posts (`tower:1b`) replace the former black blocking fill. The right lane is shorter and has ELVIK; the left loop requires no second battle.

## Native interactions

| Actor | Position | Facing / radius | Role |
| --- | --- | --- | --- |
| `MN_GYM_1_TRAINER_1` / TAVEL | `(5,14)` | right / 2 | Required introduction. His body and sightline cover the two approach tiles `(6,14)` and `(7,14)` beneath the sole entry rail. |
| `MN_GYM_1_TRAINER_2` / ELVIK | `(9,8)` | left / 1 | Optional short lane. His body and sightline cover its two walking tiles; the west loop avoids both. |
| VAREL | `(6,3)` | down | Court requires TAVEL only. |
| Guide | `(9,15)` | down | Describes the choice before entry. |

The machine-readable court and sightline contract is `rochester-contract.json`. Remove the old ELVIK qualification from VAREL's later repair branch; keep the parcel prerequisite and native leader/reward logic.

Changed block contract is `[metatile x, y, closed glyph, open glyph]`:

- `[3,6,"G","."]`: `EVENT_MN_GYM_1_TRAINER_1_DEFEATED` opens the timber entry rail. Receipt: `EVENT_MN_GYM_1_V2_INTRO_RAIL_OPEN`.
- `G` is native `tower:0b` with four WALL quadrants; `.` is native `tower:09` with four FLOOR quadrants.
- No ordered controls, optional reward, additional actor, item requirement or reset is introduced.

All source dialogue is ASCII and at most 18 characters per line. The guide and TAVEL's after-battle speech explain the short battle lane and quiet loop explicitly; ELVIK remains optional even if the player reaches the court by the loop first.

## Static review

Final ignored images are under `planning/minnesota/renders/gyms/overhaul-20261117/first-three/final-static-review-07/rochester-*`. Both closed/open whole rooms, native originals, oriented actors and 160×144 arrival/choice/leader crops are retained, with every earlier render preserved.

A Python check using the pack's native quadrant collisions found the initial rail seals the court, TAVEL's sightline cannot be bypassed at the entry, and the leader remains reachable with ELVIK's body and sightline treated as blocked. Both exits and all actor positions are legal. The open room has 116 reachable walking tiles with actor bodies retained. These are source/static findings; no game, client, controller, save, Cargo build or native gameplay test was run by the map author.

The earlier beige court was rejected after the Rust builder exposed `tower:28` / `tower:29` as undefined all-FF metatiles. The planning renderer had silently aliased glyph 255 to 127. Both source references are removed; the final court uses defined timber FLOOR art. Earlier renders remain preserved as rejected evidence. Every native art reference in all three room recipes now passes explicit block-length, palette-index and physical 2bpp bounds checks before rendering; no fallback aliases are accepted for referenced blocks.
