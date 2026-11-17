# Mankato — split quarry terraces

`mankato.json` uses the complete `mankato-ground-terraces-v01.json` schema as its template. Apply it as geometry/dialogue over the selected refinement-v02 room, preserving installed identities, teams, rewards, rematches and exterior warp indices. Copied party fields are template data; the battle owner supplies current teams.

The 8×10-metatile footprint, arrival `(6,17)` and exits `(6,19)` / `(7,19)` are unchanged. Accepted TORVEN art is `SPRITE_MN_LEADER_TORVEN`. The northwest void is replaced by a traversed upper terrace. A rock spine separates it from TORVEN's eastern terrace and reaches into the quarry, making the descent, bend and final ascent visible. The west stair is the only initial climb; native directional ledges change the return route.

## Native interactions

| Actor | Position | Facing / radius | Role |
| --- | --- | --- | --- |
| `MN_GYM_2_TRAINER_1` / CALDREN | `(2,16)` | right / 1 | Body and sightline cover both west-stair approaches `(2,16)` / `(3,16)`. Win opens the lower rail. |
| `MN_GYM_2_TRAINER_2` / JOREL | `(13,8)` | left / 1 | Body and sightline cover the two approach tiles beneath the east-court rail. |
| TORVEN | `(11,3)` | down | Both original trainer wins qualify. |
| Guide | `(9,16)` | down | Explains stairs and one-way lips. |

The machine-readable qualification and facings are in `mankato-contract.json`. The original entrance statue was moved to the west edge in terrain, leaving CALDREN and the two stair approaches on actual FLOOR.

Changed blocks `[metatile x,y,closed glyph,open glyph]`:

- `[1,7,"A","w"]`: requires `EVENT_MN_GYM_2_TRAINER_1_DEFEATED`; receipt `EVENT_MN_GYM_2_LOWER_STAIR_OPEN`.
- `[6,3,"B","u"]`: consumes the root-computed `EVENT_MN_GYM_2_COURT_OPEN`; its requires list intentionally contains only that same flag, so the terrain gate does not overwrite the court predicate.
- A/B are native `tower:0b` WALL rails. `w` is `tower:09` FLOOR; `u` is `cave:09` FLOOR. Visible native stairs use `champions_room:21` and ordinary FLOOR permission.

## Directional ledges and return

- West terrace `(6,6)` → quarry `(6,8)`, using native `johto:56` HOP_DOWN / WALL complementary quadrants. Return descends south, re-enters the west stair and crosses the upper terrace; it must pass `[2,14,4,16)`.
- Court `(10,6)` → quarry `(10,8)`, using native `cave:39`. Ordinary return uses the opened eastern rail at `[12,6,14,8)`.
- Quarry `(10,14)` → lobby `(10,16)`, also `cave:39`; return uses the western stair and initial terrace drop.
- Every reverse step meets a real WALL quadrant. All landings are FLOOR and clear of actor bodies. The northern rock cap at metatile `(4,0)` is deliberately solid: a normal cave top-edge block there left a half-block bypass between the upper terraces.
- Dropping before the second battle never traps the player. The southern lip returns to the entrance; the first opened stair can be climbed again.

All source dialogue fits ASCII/18-character lines. No new item, Strength, Surf, Cut, additional actor, quiz or reward is required.

## Static review

Ignored final whole-room and player-window renders: `planning/minnesota/renders/gyms/overhaul-20261117/first-three/final-static-review-07/mankato-*`. Earlier attempts, including the rejected void and overlap/bypass fixes, remain retained.

A Python model using actual native quadrant tokens checked initial/first-win/fully-open reachability, native two-tile ledge landings, blocked reverse steps, mandatory return passages, actor occupancy and both exits. Exact measured geometric return lengths are in the adjacent ignored `static-review.json`; they exceed the conservative minima declared in source. This is source/static evidence only: no engine/controller execution, gameplay, save access or Cargo build by the map author.
