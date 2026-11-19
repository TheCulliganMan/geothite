# Brainerd to Mille Lacs by Route 18

The main road runs east from Brainerd to Mille Lacs. The two fishing piers lie to the south. Farther east, the Surf sign marks a public launch, and the beach volunteer stands near the park entrance. Crossing the two route maps on foot takes 87 steps in either edition.

Across the lake, a Revive sits beside a pair of pines and a flower patch. Follow the western sand to come ashore, or land beside the low brown rocks on the eastern side. The south point and two mainland coves share a shallow blue edge. An owned canoe launches when the player walks from shore into the lake and returns to walking when they land. Surf also reaches the islet.

Crystal’s Lake of Rage provided the reference for a destination visible across open water. Cianwood provided the example of a low, walkable shore. Polished Crystal's Lake of Rage places another optional reward and trainer near its shore. Route 18 keeps its own map, rewards and four-frame lake ripple. [Crystal’s Lake of Rage](https://github.com/pret/pokecrystal/blob/master/maps/LakeOfRage.asm) · [Polished Crystal’s Lake of Rage](https://github.com/Rangi42/polishedcrystal/blob/master/maps/LakeOfRage.asm)

## Movement and art checks

Run `check-route.py PARENT CHILD` with matching League Journey and Shore Watch archives. The check follows both walking lanes through the Brainerd, route and park connections. All 11 previously checked public boarding points remain usable. The lake has 1,798 connected water squares. The islet has 67 reachable dry squares with the Revive present and 40 pairs of adjacent land and water squares where a player can step ashore or relaunch. Both editions retain all 272 encounter-grass squares, two ledge squares, the original Cut and Headbutt squares, and the outer buoys.

The shoreline regression covers 400 dry and wet squares around both coves and the entire islet. It reads the soles and hulls from the two walkers and both canoe sprites, including their six native poses and mirrored strides. The bottom-center pixel at `(8,15)` in each 16×16 sprite is the physical contact point. On a southbound launch that point enters the water on the first two-pixel step. On a northbound return it reaches dry ground on the last step. The same rule handles bank crossings to the east and west.

All 85 open shore contacts in that area support the mounted or walking pose at each two-pixel movement tick, in both directions. The 635 adjacent pairs on the same surface account for another 9,525 intermediate positions where both soles or the canoe keel stay supported. These checks also run separately as `check-route.py --footprints CHILD`. The full comparison verifies all four phases of the 14 new waterline glyphs and the morning, day and night pixels of all 150 inherited blocks.

The production camera views for comparing terrain are the Revive islet at `MnR18C00R00 (60,36)` and the east beach at `MnR18C01R00 (6,18)`. The walk and canoe regression records the player approaching from two squares inland, launching, paddling and returning at the western mainland, southern islet, eastern beach and Mille Lacs picnic cove.
