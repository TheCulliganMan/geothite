# Lake Pepin and the three wakes

The artist sketches from the grassy west bank. In both Town Returns parents they pointed players to the end of a dock, although the ground beneath them was grass, and the nearby read occupied an ordinary square of open water. Across the river, the boat skipper named a reed bed where the native map displays pines. Their old reply sent players back to all three observations even when one remained.

In the revised scene the artist points out a curl of foam beside them. The clue is legible from the bank and the river remains open to boats. Once the player has seen the wake, the artist explains how to find the skipper. His directions identify the first pine south of him, where the water-side observation takes place. The river bends north and west to a small bay cut into the wooded north bank. The skipper keeps that bearing available if the player returns after the comparison.

| Stop | Installed location and approach | What counts |
| --- | --- | --- |
| Sketch | Western Lower Woods, artist at (22,10) | The drawing is recorded when its last page is acknowledged. On a later visit the artist points to the foam or the skipper. |
| Eddy | Same bank, face east from (23,10) to (24,10) | The water stays navigable. The two new 8-pixel graphics animate an original four-frame curl at the same tempo as the surrounding water. |
| Skipper | Eastern Lower Woods, (50,16) | The three observations can arrive in any order. He asks about a missing observation and compares their evidence only when all three are present. |
| Pine | South of the skipper, examine (50,20) | The dry approach from (50,19) explains the need for water travel. From (49,20) in the water the player can see the fin pass beneath the pine branches. |
| Inlet | Northeast Lower Woods, face north from (24,22) | After the comparison, the actual active Barbelisk surfaces. Its level-55 battle asks for consent. Declining leaves it in view. An uncaught departure clears it for this visit. A later physical reentry restores it, while capturing it keeps the inlet empty. |

`check-pepin.py PACK` reads the written archive. `--baseline` checks the corresponding Town Returns parent instead. The audit follows the eight clue combinations, compares the actor and background reads to their compiled commands and checks both directions through the existing terrain with the resident and trainer squares occupied. The route between the people is 166 steps with water travel. The pine-to-inlet trip spans 71 uninterrupted water steps each way. Neither distance includes random battles. The separate native `owned_canoe_reaches_pepin_pine_observation_without_a_surf_partner` check takes the ordinary owned canoe from the east bank and reads the actual pine event. The original encounter, catch flag and new-visit callback remain in force.
