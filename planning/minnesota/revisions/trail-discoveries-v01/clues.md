# Hidden discoveries on the Minnesota trails

The installed region has 57 hidden items across 47 maps. Fourteen are in towns, 23 on routes, 18 in caves and two underground in the Iron Range mine. All 57 have an unoccupied adjacent search position with a path back to a map entrance. Forty-seven are reachable on foot, seven require water travel, and one each requires Cut, Rock Smash and Whirlpool.

Crystal’s Route 32 registers four readable signs and two separate search squares, one holding a Great Ball and the other a Super Potion. Minnesota’s prairie posts and Cedar Ridge make the same distinction visible on the map. [Original Route 32 events](https://github.com/pret/pokecrystal/blob/f91d74b9c383da9a36f69f79c366852b5d0525e9/maps/Route32.asm#L895-L901)

## Two misleading landmarks

**Birch grove, Route 8.** The Birch Walker at (39,39) used to send the player to a flower bed. The Miracle Seed is among two fallen cones in a thawed clearing under the pines at (35,42) in the southern route chunk. Its original two-bit art shows the cones on ordinary ground. The Walker now remembers finding a thumb-sized seed among the cones south of him. The conversation after the pickup still returns to his ordinary story. From the Leech Lake entrance it takes 21 steps to the east side of the cones. From the standing square south of the Walker it takes 93. Both paths run in both directions.

**Cedar Ridge, Route 13.** The Hyper Potion in the snowy nook at (38,32) used exactly the same sign graphic as the three readable trail markers. Approaching the fourth sign offered only the hidden pickup. It now shows two stones balanced in the snow. The new artwork occupies the single solid square. The surrounding three quarters of the block keep their original snow pixels and footing. The three actual signs keep their artwork and messages. From the Boundary Waters trail, the player can reach the square immediately south of the cairn in 41 steps and walk back the same way.

## Prairie Farm Road noticeboards

The Antidote at (18,34) is tucked against a pair of fence posts. Checking that surface exposed a separate problem on Route 2. At five real noticeboards the text interaction sat on the matching pair of posts beside the panel. Walking up to the letters and pressing A missed the directions. Each interaction now sits one square east, on its own printed panel. The Antidote stays on the fence.

| Board | Printed face | Reading position | Closest tested public entrance |
| --- | --- | --- | --- |
| Rochester | (57,13) | (57,14), face north | 2 steps from Rochester |
| Lind Farm | (15,15) | (15,16), face north | 39 steps from Mankato |
| Nursery Bank | (51,19) | (51,20), face north | 14 steps from Rochester |
| Tool Shed | (49,37) | (49,38), face north | 36 steps from Rochester |
| Mankato | (5,33) | (5,32), face south | 7 steps from Mankato |

The three prairie trainers still see the same five squares. Tess sees (52,21). The north-facing trainers see (23,18) and (23,17), and (47,36) and (47,35). Those squares remain accessible from the public road.

## Discoveries that already work

- On the mine’s haulage floor, the weigh worker at (14,7) describes a heavy ball under a sample just east of him. The target really is the rock one square east and one north, at (15,6). It can be searched from ordinary floor.
- The Granite Passage scout describes a dry lip across the seep. Its Full Restore at (11,19) requires Surf. Two search positions are available once the player crosses.
- St. Paul’s walker gives the far northeast bank as the location of his BlackGlasses. The searchable landing is northeast of him, separated from the main bank by water. Duluth’s angler points to the open edge of the separate float where the Nugget can be reached.
- In Former Shores, two story observations and a Potion share the appearance of native cave rock. The separate waterline, pillar and solitary rock occupy different terrain arrangements. Their shared material makes sense in that chamber.

The [clue check](check-clues.py) reads the actual archive and inventories every background-item event, its reward, persistent receipt, surrounding collision and standing positions. It checks the native pixels of the cones and cairn, the five noticeboard events and the three trainers. Pass a pack path, add `--baseline` for Intro v03 or Headwaters Journey, and use `--parent` for a parent-to-candidate comparison. `--gallery DIRECTORY` produces 57 player-scale terrain crops. The access check starts from public map connections or real doors, carpets, cave mouths and ladders, with authored actors present. It excludes passive arrival records on ordinary floor. This establishes local searchability with each named field move available. Earning that move and travelling the complete campaign involve additional game states.
