# At the Ely League desk

Ely can be reached from the Boundary Waters on Route 13 or from the east through Ely Shore Pass. The clinic and Mart take visitors as soon as they reach town. Both lodge doors lead to the registrar. Signing up requires all eight badges and a restored survey.

## What the player learns

| Visit | What happens | Change at the point of decision |
| --- | --- | --- |
| Eight badges, intake still closed | The north sign used to send everyone to the Boundary Waters shelter. Someone carrying the recovered meter must first visit the Grand Rapids depot. Others may still need the reading from Itasca, the Bemidji officer or the Duluth engineer. | The sign sends the player to the lobby desk. The registrar reads the existing survey milestones and names the next stop. |
| First challenge | The registrar explained the order, backtracking and loss. A player heard where to get supplies only after declining the invitation. No nurse heals the party between Fern and Casey. | The registrar asks whether the player brought medicine before offering the choice. Declining gives directions to the clinic southwest and the Mart south on the other path. |
| Returning Champion | The sign said exiting ended the run. Walking back from Casey to Halden, or between the earlier rooms, preserves the wins. Crossing into the lobby resets them immediately. | The sign explicitly distinguishes returning to earlier rooms from returning to the lobby. Four fresh wins are needed to challenge Casey. |

The Mart sells Hyper Potions, Max Potions, Full Restores, Full Heals and Revives. It also sells Ultra Balls and Max Repels. A supply under the pines holds one Full Restore. The clinic has a nurse and a party PC. The people there still have their own reasons for being in Ely: one leaves spare mittens by the stove, another remembers the father who brought in firewood. The registrar can give the supply warning to every challenger in a single page. [Street and counter access check](../league-journey-v01/check-preparation.py)

Winning against Casey opens the north door to the Hall of Fame. Recording the team completes the run. Continue after the credits returns the player to the Ely clinic. The nearby ranger relays Reed's invitation to the St. Cloud station until that rematch is won. Casey also gives a returning challenger a chance to decline, walk away, or speak to her again. [League return and recovery scenarios](../../../../crates/crystal-bevy/src/bevy_shell/tests/league_return_trip.rs)

## Crystal and Polished Crystal

At Indigo Plateau in Crystal, an optional trainer explains the consequence of losing within steps of the nurse and shop clerk. Polished Crystal keeps those services and the warning alongside more personal encounters, including Yellow offering a Pokémon. Ely spreads its services across town. A short warning at registration gives the player a chance to make the trip while they can still keep their entry. [Crystal's Indigo Plateau lobby](https://github.com/pret/pokecrystal/blob/master/maps/IndigoPlateauPokecenter1F.asm) · [Polished Crystal's Indigo Plateau lobby](https://github.com/Rangi42/polishedcrystal/blob/master/maps/IndigoPlateauPokecenter1F.asm)

The three previous conversations matched both Badge Battles parent archives. On both Ely Challenge editions, the native regression read the sign, walked to the registrar for the Grand Rapids handoff, read the readiness question, declined for supplies, and visited the sign as Champion. All 14 captured 160 × 144 pages matched between editions. A separate controller test began from a five-win fixture, exited the Hall on each lane, saved in Casey’s room, came back, then read the wall register before induction. Both editions passed. [Exact page changes](registration.json) · [Ely dialogue regression](../../../../crates/crystal-bevy/src/bevy_shell/tests/ely_registration.rs) · [Hall and clinic regressions](../../../../crates/crystal-bevy/src/bevy_shell/tests/league_return_trip.rs)
