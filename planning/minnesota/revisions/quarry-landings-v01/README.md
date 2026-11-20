# Mankato quarry landings

The first trainer opens the west stair in Mankato Gym. It leads to an eight-square rock lip overlooking the entrance. On the Leech Lake Trainers editions, two of its drops failed. The guide stood on the landing below column 9, and the right-hand statue stood below column 13.

The guide now stands one square farther south on the plank floor and faces the arrival aisle. Both statues sit flush with the outer walls. The right statue and adjoining planks use the gym’s existing metatiles. All eight drops lead onto the connected lower floor. The marked lower escape at column 10, the west quarry drop and the court return retain their original positions. The trainer at the west stair still sees the player at (3,16), and the upper trainer still watches (12,8).

## Archive checks

Run from the repository root with Python, cbor2 and Pillow. Supply each archive after its corresponding parent.

```sh
python3 planning/minnesota/revisions/quarry-landings-v01/check-pack.py \
  content-packs/minnesota-leech-lake-trainers-v01.crystalpack \
  content-packs/minnesota-quarry-landings-v01.crystalpack \
  content-packs/minnesota-intro-leech-lake-trainers-v01.crystalpack \
  content-packs/minnesota-intro-quarry-landings-v01.crystalpack
```

The checker compares the full archive, the guide’s native object record, the event stream and the encoded map. It applies the installed gate programs’ actual tile replacements to trace the room before the first win, after it, and after both wins. It checks the entrance approach to the guide, each individual landing, reverse movement and access to the leader.

## Regional survey

`survey-ledges.py PACK [PACK ...] --details` inventories explicit ledge permissions and their two-square cardinal landings. The report describes each map’s initial terrain and fixed standing actors. Story scripts, walking NPCs and acquired field moves can change the local route through a map.

In the parent Field edition, 42 of 355 Minnesota maps contain 174 marked ledge squares. Intro has 42 of 357 maps and 170 marked squares. Each edition has one corner with two directions. At (24,18) on Route 15 the southward drop reaches the meadow, while its eastward face overlooks water. Crystal encodes this corner as separate right and down inputs. [Original player movement and ledge direction table](https://github.com/pret/pokecrystal/blob/master/engine/overworld/player_movement.asm#L342-L378)

Both written Quarry Landings archives passed the comparison. Each row counts cardinal inputs from marked squares, so the Route 15 corner contributes two.

| Edition | Directions examined | Open on foot before → after | Fixed actors before → after | Solid walls before → after | Water before → after |
| --- | ---: | ---: | ---: | ---: | ---: |
| Field | 175 | 161 → 163 | 4 → 3 | 1 → 0 | 9 → 9 |
| Intro | 171 | 157 → 159 | 4 → 3 | 1 → 0 | 9 → 9 |

Every other landing retains its coordinates, terrain classification and fixed actor. The three remaining occupied landings are on the Grandstand and Minneapolis Music Room stages. All nine water landings are east-facing directions above Route 15’s high bank. Both editions retain their open low shoreline.

## Walking the quarry

In both editions the native controller entered from Mankato, read the guide from the west, fought Caldren and Jorel on sight, and climbed the stairs their victories opened. It took all eight drops in one visit and walked back out to the street. The supplied level-40 Borealynx isolates the geometry from battle difficulty. The two formerly blocked landings each have a separate save after the jump. [Quarry regression](../../../../crates/crystal-bevy/src/bevy_shell/tests/quarry_landings.rs)

The terminal also checks the landing before it draws a ledge. The Route 15 west bank is visible as ordinary ground in the browser's painted and text views, with three southward drops at the meadow. A separate native test follows the original Crystal ledge in Fuchsia City. [Route 15 findings and references](terminal-ledges.md)
