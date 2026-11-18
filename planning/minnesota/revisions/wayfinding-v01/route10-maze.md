# Route 10 field passages

The western Iron Descent terrace and the Survey Gallery form one optional trip. A player taking the ordinary road can reach the gallery reader and the existing rewards on their first visit. The spruce crown gives Cut a different trip through that same landscape. Rock Smash opens a gallery overlooking a spring the player can already reach on foot.

## The spruce crown

A narrow timber step on the western terrace faces a young spruce. Cut clears that one tree. A snowy trail curls around a stand of mature spruce to a visible Sharp Beak on the western side of the loop. The eastern fork climbs onto a thawed rock shelf above the original ore bench. Two adjacent takeoff tiles on the shelf's southern edge drop onto that bench. A return from inside the crown can use the drop even if the tree regrows.

The on-foot distance from (18, 10) to (24, 7) is 37 movement inputs over the old road and 12 via the cleared crown. The figure counts movement and hops on the static map. Time spent using Cut or battling a trainer is separate. The upper trail and its reward are inaccessible from the lower end of the drop.

## The survey gallery

A cracked rock on the eastern shelf below the visible Escape Rope stands at (40, 6). The native Rock Smash action opens a narrow passage, followed by a larger room in which the player can circle a solitary boulder on either side. Its face at (44, 10) holds a Star Piece. A two-tile ledge takes the player straight down to the dry spring bank. The western takeoff and landing are one hop apart. Walking back to that takeoff from below takes 68 movement inputs with the rock broken. A player resuming inside the chamber after the rock resets can still leave on foot.

The rock uses Crystal's ordinary smashable boulder object and standard script. A party member who knows the move can break it. The gallery has no Rock Smash encounter table. Its existing cave encounters and visible trainers use their original regional teams.

## Crystal references and authoring

Dark Cave's Violet entrance places smashable rocks beside the routes to treasures and other exits. Route 46 uses cliff bands to give the return trip a different shape. Ilex Forest places a Cut tree in a narrow gap between dense rows of trees. All three native reference maps were inspected alongside the released Route 10 terrain. [Dark Cave](https://github.com/pret/pokecrystal/blob/master/maps/DarkCaveVioletEntrance.asm) · [Route 46](https://github.com/pret/pokecrystal/blob/master/maps/Route46.asm) · [Ilex Forest](https://github.com/pret/pokecrystal/blob/master/maps/IlexForest.asm)

The placements in `route10-maze.json` specify a 32-pixel block coordinate, its expected original block, and the four native source quadrants in upper-left, upper-right, lower-left, lower-right order. Each source value is four times the donor block ID plus its quadrant. The builder rejects changes to any existing non-wall collision, a moved or occupied rock target, either ability opening a passage without its prerequisites, or a saved position inside the new passages losing its return. It uses the same Cut replacement and Rock Smash party action as the game.
