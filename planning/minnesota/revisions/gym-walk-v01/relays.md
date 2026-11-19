# The feeder and fan controls

A player entering Minneapolis sees two columns of breaker switches beside LENVI. The first panel they can stand in front of is five steps from the door, at (4,16) facing north. On Badge Trails, neither column answered from that side. The only working switch was on the back at (5,14). LENVI described the switch as being above the trainer, even though the cabinet runs alongside the trainer. The revised room connects all four exposed switch squares on the west cabinet and the three exposed squares on the east cabinet. A locked feeder names the opponent who can unlock it. Both trainer victories and both powered bridges still matter.

St. Cloud draws a screen and a dial side by side at each of four fan desks. Only one half of each desk answered. At the first desk the guide occupied (7,14), the sole standing square directly in front of its right-hand dial. The guide now stands at (5,14), a single step from a conversation stance at the entrance. Both visible controls lead to that desk's original EAST/WEST menu. The solid backs of the desks, both fan runs, and HALLEN's optional shortcut retain their original behavior.

In Crystal's Vermilion Gym, each of the 15 individually drawn bins answers when examined. Polished Crystal uses those same visible targets for its working electric-lock puzzle. Minneapolis and St. Cloud keep their local devices and puzzle rules and follow the same convention for where a player can press A. [Pokémon Crystal room](https://github.com/pret/pokecrystal/blob/master/maps/VermilionGym.asm) · [Polished Crystal room](https://github.com/Rangi42/polishedcrystal/blob/master/maps/VermilionGym.asm)

| Installed device | Switch squares, in player coordinates |
| --- | --- |
| Minneapolis west breaker | (5,14), (4,14), (4,15), (5,15) |
| Minneapolis east breaker | (8,7), (9,6), (9,7) |
| St. Cloud lower west and east | (6,13) and (7,13), (12,13) and (13,13) |
| St. Cloud upper west and east | (6,5) and (7,5), (14,3) and (15,3) |

Run `relays.py --baseline BADGE_TRAILS_PACK` for either original edition and `relays.py GYM_WALK_PACK` for its successor. The checker starts at each public arrival, follows native quadrants, respects trainer bodies and their sight approaches, runs installed control and tile callbacks, and finds both an outward and a return route. For St. Cloud it checks the EAST crossing and WEST return with HALLEN's entire unchallenged sightline excluded, then checks a second visit that earns the catwalk. The reported walking counts treat a forced conveyor ride as one directional input. This is a compiled-data check. The native entrance captures and complete rooms from the reviewed Intro pack are retained under `target/gym-walk-20261119/relays/`.
