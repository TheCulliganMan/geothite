# What the Shore Watch sees

On the east road out of Brainerd, a boat’s wake has washed over nesting birds. The dock ranger asks the player to find out whether the parent bird can bring food through the reeds, where the chicks stayed, and whether a boat has room to turn at the beach. Those three observations explain the ranger’s decision to slow traffic past the reeds and have boats turn offshore.

The earlier ranger had dialogue explaining the incident, but the incomplete-report branch bypassed it on every visit. The first conversation now gives the reason for the walk. Coming back early gives a lead for a volunteer the player has actually missed. If all three have spoken, the park ranger sends the player west to the marina. Before earning the badge or after collecting the reward, the park ranger gives direct advice to cast from the far end of the pier facing open water. The earlier referral sent players to the dock ranger in states where that ranger only discussed the badge or boat wakes. Any of the six orders works. Hearing the same observation again costs no progress.

| Where to look | What the volunteer contributes |
| --- | --- |
| Reeds, Route 18 west at (15,33) | A parent bird carrying food misses the gap when a wave arrives. |
| Meadow, Route 18 west at (31,9) | Two chicks are still at the waterline beneath the grass. |
| Beach, Route 18 east at (11,17) | A boat can turn safely farther offshore by the sand. |

The Lake Badge and all three reports qualify the player for one reward at the marina (27,23). Someone carrying no rod or only an Old Rod receives the Good Rod. Someone carrying a Good Rod or Super Rod earns three Lure Balls. Equipping an earlier rod used to complete the errand empty-handed. A player can already obtain a Good Rod in the Route 3 fishing house and a Super Rod in the Route 6 house. These both remain legitimate alternatives to waiting until Mille Lacs. The ranger keeps an unpaid reward if the relevant pocket is full. Previously completed saves keep their receipt.

The six west-shore trainers are ordinary sight battles. Each has a dry approach from the Brainerd entrance and a team at levels 31, 31 and 30. Hal, at (17,35), sees the player immediately to his east. After his battle he directs an interested angler to the Pine Country fishing house south of Brainerd. The volunteers and ranger are separate people, so none of those fights stands in for an observation.

Crystal’s Lake of Rage fisherman holds an earned Elixir when the Pack cannot take it, independently of repeating the feat. Mille Lacs uses the three existing report flags to remember the completed fieldwork while the ranger holds the reward. [Lake of Rage fishing-house script](https://github.com/pret/pokecrystal/blob/master/maps/LakeOfRageMagikarpHouse.asm)

`check-reports.py PACK` verifies the installed branches for 512 combinations of badge, reports, receipt, rods and pocket capacity, all six report orders, 32 park return states, and the walking contact squares for all six battles. Pass `--baseline` with a League Journey parent to reproduce the old reward and clue behavior. The `shore_watch_lure_reward` library test runs the actual grant with a nearly full stack and all twelve Balls slots occupied, then retries after the player has made enough room. These checks use compiled game data and the item system without playing the route.
