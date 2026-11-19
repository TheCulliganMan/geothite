# Minnesota campaign battles

Two trainer partners had moves that worked against their own sets. Both baseline editions contain the same two cases.

| Encounter | What the player faced | Change |
| --- | --- | --- |
| Reed, St. Cloud Station (15,16), after becoming Champion | His level-57 Greytag had Return and Quick Attack. The battle engine gives a trainer’s Pokémon 70 friendship, making Return a 28-power Normal attack. Quick Attack has 40 power, the same accuracy and priority over ordinary attacks. | Strength gives Greytag a reliable 80-power Normal attack. It can still use Quick Attack to finish a faster opponent, Low Kick for coverage and Encore to punish setup. |
| Kite, Route 20 icefield (43,27), east-facing sight range 4 | His level-46 Chorip could spend half its HP on Belly Drum. It knew two attacks, Sludge Bomb and Psychic, and the installed move catalog marks both special. The Attack boost helped neither. | Chorip remembers Water Gun, learned at level 22. It benefits from the Rain Dance that Kite’s opening Reedraft already knows. Its other natural Water attack by level 46 is Bubble. Water Gun adds reliable coverage without the power of a taught Surf. |

The rematch is offered after the Champion flag and requires accepting Reed’s challenge. Kite is an ordinary sight trainer on an optional northern road. His neighboring trainers’ Chorip have different sets at lower levels. [Reed’s authored team](../../story-pokemon.json) · [Kite’s authored team member](route-trainer.json) · [Trainer materialization](../../../../crates/crystal-core/src/battle/start.rs) · [Native move effects](../../../../crates/crystal-core/src/battle/turn.rs)

## From the starter to the rematches

The review followed trainer IDs in actual Minnesota map scripts. Local Intro v03 has 301 ordinary trainer bindings and 35 scripted battle bindings, covering 336 teams and 763 party members. Field Headwaters has 335 teams and 758 members. Intro adds the two-member Mothlight Café battle and one extra partner for each of three Route 16 trainers. Every bound member belongs to the active Minnesota roster, every team has a damaging move on each partner, and every numbered route has at least one trainer. Both editions use fixed levels and the competitive battle planner. [Compiled battle checker](check-battles.py)

| Stage | Installed opponents and preparation |
| --- | --- |
| Winona to Mankato | All three starter choices arrive at level 5. Reed opens with one level-3 Crookit, then returns in Rochester with levels 7 and 8. Varel peaks at 12 and Torven at 16. Going straight to Varel leaves a real level disadvantage. The existing experience check gives a solo starter level 7 on the required path, or 9 after the three optional Route 1 wins. [Opening experience audit](../campaign-quality-v01/README.md) |
| Minneapolis to St. Paul | Reed’s relay team peaks at 22. The four-member Dispatch enforcer peaks at 23. Route 4 trainers run 18–19, the Minneapolis Gym peaks at 22, the St. Paul Gym at 25 and depot Reed at 26. The ICE and Gym challenges can be interleaved. |
| St. Cloud to the Iron Range | The Wind, Lake and Pine Gyms peak at 30, 35 and 40. Route 6 trainers run 27–28, Route 7 trainers 30, Route 8 trainers 30–32. The mine foreman and drill-room Reed both peak at 36. The deeper core guards can field level-40 pairs. Reed fields four partners and the player also has the crew and drill problem to solve. |
| Duluth to Ely | Duluth’s Gym peaks at 45, the Boundary shelter chief at 46 and intake Reed at 45. The last section of Route 13 reaches 48, the same level as Fern’s ace. The northern side trips occupy different bands: Routes 14–16 and 18–19 run 30–31, Route 17 runs 35–36 and Route 20 runs 41–46. |
| League and postgame | The four Elite teams have four partners each and peak at 48, 49, 50 and 51. Casey has six and peaks at 53. The eight optional gym rematches peak at 57–60. Reed’s six-member team peaks at 59. |

## League resources

Crystal’s Indigo Plateau asks for 26 knockouts, with each Elite Four member bringing five partners and Lance bringing six. Ely asks for 22 and has a higher opening and final level. Crystal puts the nurse and shop before the run. Ely does the same through its town clinic and Mart. Returning to the League lobby ends the run. [Crystal trainer parties](https://github.com/pret/pokecrystal/blob/master/data/trainers/parties.asm) · [Indigo Plateau services](https://github.com/pret/pokecrystal/blob/master/maps/IndigoPlateauPokecenter1F.asm)

The seven Route 13 trainers pay 3,316 before savings or item modifiers. That covers one Hyper Potion (1,200), one Revive (1,500) and one Full Heal (600) with 16 left. The four Elite trainers pay 1,972 before Casey, but there is no shop inside. A player must bring those supplies across the registration desk. Casey carries a Full Restore and a Full Heal.

An untraded active partner that earns every knockout receives 43,328 EXP across Ely. Each healthy untraded reserve receives 21,658 without holding an Exp. Share, independently of party size. Companions starting exactly at level 48 on the starters’ growth curve can finish at 53 in the active role or 50 on the bench. These are bounds for the stated roles. Fainting, switching the finisher, traded experience and held items change the amounts. Actual win rates, player spending and incoming damage over an uninterrupted run were outside this static review. [Experience distribution](../../../../crates/crystal-core/src/systems/battle_rewards.rs) · [Earlier PP and type-matchup review](../lake-observations-v01/README.md)

Run `check-battles.py PACK --baseline` on either parent to reproduce both defects. Run `check-battles.py CHILD --parent PARENT` to check the corrections and compare the complete trainer catalog.
