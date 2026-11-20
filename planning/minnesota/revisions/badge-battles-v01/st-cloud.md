# St. Cloud's fifth badge

At seed 1701 the ordinary Pouchid, Kindlit and Birchit parties lost in 11, 10 and 10 turns with the first four badges applied through the normal gym scripts. Saven had two, three and two Pokémon left, respectively. All three parties lead with the starter, choose the largest estimated immediate damage every turn, and send the first available reserve after a knockout. Saven can choose setup, damage or a paid switch. The total alone cannot establish which move caused a knockout. [Trial policy](../../../../crates/crystal-assets/src/badge_battle_trials.rs)

| Opponent | A decision the player can make |
| --- | --- |
| Deepulse, Lv. 28, Psychic/Flying | Nibbin is in the first daytime slot on Route 1. At 18 it becomes Dark/Ground Cacheek, immune to Psybeam. The Pouchid and Birchit parties already have one with physical Dark attacks. Light Screen affects special damage, so it does nothing to stop these hits. Deepulse can still use Wing Attack or Swift. In the Kindlit party, Water/Psychic Chorip resists Psybeam and can lead while Rimepelt waits. |
| Bankwing, Lv. 29, Ground/Flying | Electric moves deal zero damage. Rimepelt learns Icy Wind at 28, and the Kindlit party still has it. Its Ice damage is quadrupled against Bankwing, with a Speed drop on a successful secondary effect. Bankwing can answer with super-effective Dig. The Pouchid and Kindlit parties can switch to Flying-type Flossail during the charge turn. |
| Silohawk, Lv. 30, Dark/Flying | Electric moves work again. A Route 2 Zapsap can evolve into Tapacitor and learn Spark at 24. A Grass-line starter also has an attack here: Slipperina's Fairy Wind hits Silohawk for double damage, although Silohawk's own Wing Attack is super effective in return. |

Rock coverage offers another route. Daytime Tesslarv occupies a 20% slot on Route 1 and a 30% slot at the Minneapolis end of Route 5. A trainer raising one from Route 1 can keep Rock Throw from level 9. The two ordinary audit parties that own its line both forgot Rock Throw under their oldest-move-first rule. Rock hits Deepulse and Silohawk for double damage but hits Bankwing for neutral damage. Evolving Tesslarv into Bug/Psychic Mothosaic at 28 also changes incoming Flying hits from neutral to super effective. Sinknook is a stronger defensive catch against Deepulse, taking zero from Psychic and half from Flying and Normal, but it occupies only the 1% slot in the segment nearest St. Cloud. From the clinic or gym, both south exits at (24,35) and (25,35) lead onto Route 5. The grass at (34,16) is reachable on foot from the northern road entrance. This same patch offers a 20% daytime Tesslarv slot.

The paired retry changes only who walks into the gym first. Both Cacheek parties open with Cacheek, and the Kindlit party opens with Chorip. The other three keep their relative order. Each comparison uses the same levels, moves, health and random seed. The test records the player’s selected moves and opposing species at seed 1701. It uses seeds 1701, 4242 and 9001 for results. [St. Cloud retry](../../../../crates/crystal-assets/src/badge_battle_saven.rs)

| Starter party | Starter first | Alternate first | What happened at 1701 |
| --- | --- | --- | --- |
| Pouchid | 0 wins in 3 | 0 wins in 3 with Cacheek | Saven sent Silohawk into Fairy Wind on the first turn. The starter-first party eventually took one knockout. With Cacheek first the party took one knockout on every seed. |
| Kindlit | 0 wins in 3 | 0 wins in 3 with Chorip | Rimepelt spent two Icy Winds on Silohawk. By the time Bankwing entered, Rimepelt had fainted. Neither opening recorded a knockout across the three seeds. |
| Birchit | 0 wins in 3 | 0 wins in 3 with Cacheek | With Portabeau first, the party took one knockout on each seed. On 1701, Bankwing still had 14 HP when Velvawn came out. With Cacheek first, all three opposing Pokémon survived on each seed. |

The six opening traces at seed 1701 show Saven replacing Deepulse with Silohawk immediately. Across all 18 attempts the small parties won no fights. Changing the lead alone did not solve the matchup for these four-member teams. The tests use immediate damage, no bag items, and the next available partner after a faint. The earlier published trials used story badge flags without applying the combat badge awards. The table above replaces those results.

A Route 5 Bristub evolves at 28 into Normal/Steel Latchquill, with base Defense 115. It resists Wing Attack and Psybeam, takes neutral damage from Faint Attack, and is vulnerable to Bankwing's Dig. Another locally obtainable Rock/Dark partner comes from the 10% Lv. 15 Driptip slot in Route 3's River Pocket Cave. It evolves into Sinknook at 24.

## Returning with a Route 5 catch

The native Quarry Landings Intro trial uses the same Saven party and Route 5 encounter, species, level-up, move and evolution data as the Leech Lake Trainers parent. Each starter team faced Saven on seeds 1701, 4242 and 9001. The level-24 catch carried Headbutt, Tackle, Bind and Defense Curl. Four normal level-up/evolution steps produced a level-28 Latchquill with Bind, Defense Curl, Take Down and Metal Claw. This part models the levels gained rather than measuring the field encounters or time to earn them. [Catch and move comparison](../../../../crates/crystal-assets/src/saven_route_five_catch.rs)

| No-item approach | Pouchid wins | Kindlit wins | Birchit wins | Median turns |
| --- | ---: | ---: | ---: | ---: |
| Original four, starter first | 0/3 | 0/3 | 0/3 | 10 |
| Add the fresh level-24 Bristub in fifth | 0/3 | 0/3 | 0/3 | 12 |
| Add the level-28 Latchquill in fifth | 0/3 | 0/3 | 0/3 | 13 |
| Switch Latchquill into a visible Silohawk | 0/3 | 0/3 | 3/3 | 12 |
| Switch in, use the paid Rollout lesson | 1/3 | 0/3 | 3/3 | 15 |
| Put the same Rollout user first | 0/3 | 0/3 | 1/3 | 13 |

Every scripted switch into Silohawk spent a normal battle turn. For the Birchit team, all three were wins even without a lesson. Pouchid took two knockouts per run with the untutored switch, but never finished Saven. Simply leaving Latchquill until last had two particularly close losses: both parties lost their final Pokémon on the same turn and the player lost the fight. The two Rollout rows replaced Defense Curl with Rollout for ₽500, used it against a visible Silohawk or Deepulse, and respected its native multi-turn lock.

## A practical retry with the town’s medicine

The original four won five of nine fights when given two local Super Potions and allowed to choose replacements, use status and switch into a visible Dig. Adding an untrained fifth member produced the same five wins. Bringing a level-28 Latchquill instead produced six wins and gave the Kindlit party its first victory under this approach. None of those six required Rollout or a TM. The player purchased two Super Potions for ₽1,400 at the normal St. Cloud Mart and spent a full turn for each dose. [Practical retry and purchases](../../../../crates/crystal-assets/src/saven_practical_retry.rs)

| Wind Lesson Intro retry | Pouchid wins | Kindlit wins | Birchit wins | Median turns | Super Potions used across 9 fights |
| --- | ---: | ---: | ---: | ---: | ---: |
| Original four, tactical decisions | 2/3 | 0/3 | 0/3 | 10 | 0 |
| Original four, two Super Potions available | 2/3 | 0/3 | 3/3 | 12 | 15 |
| Fresh Bristub fifth, two Super Potions available | 2/3 | 0/3 | 3/3 | 14 | 15 |
| Evolved Latchquill fifth, two Super Potions available | 2/3 | 1/3 | 3/3 | 15 | 13 |
| No items, switch Latchquill into Silohawk and try Defense Curl then Rollout | 0/3 | 1/3 | 3/3 | 14 | 0 |
| No items, lead with the same Defense Curl and Rollout user | 2/3 | 2/3 | 2/3 | 13 | 0 |

Across the first four rows, the player saw Bankwing dig and spent a turn switching a healthy Flying partner in twelve times. On all twelve, Bankwing returned above ground, its last move was Dig and the arriving partner had exactly its pre-switch HP. The new pre-fight clue describes an opening a player can actually take. The general policy also uses favorable attacks, available sleep or paralysis, forced replacements and actual Shift prompts. Its overall wins reflect those decisions together.

The last two rows use the original simple damage policy with one deliberate setup. The apprentice replaced Bind, preserving Defense Curl alongside the newly taught Rollout. Against Deepulse or Silohawk, the player took a turn to curl before attempting Rollout. A successful curl raises Defense and doubles Rollout damage while that partner stays in. Leading this way won twice for each starter. Switching it into an attacking Silohawk first used another turn and gave different results. A combo can miss, be interrupted, or face a different opponent before the next hit. [Native Defense Curl and Rollout regression](../../../../crates/crystal-core/src/battle/turn_tests.rs)

The most persistent gap was Kindlit. Its original four spent both potions on all three retries and took zero knockouts; a fresh Bristub still yielded zero. The evolved fifth took at least one knockout each time, winning one of three. The seeds represent paired examples of each policy. Additional reserve choices, teams and player decisions can change those outcomes.

## What the game teaches before Saven

The sign outside the St. Cloud gym at (4,28) identifies Saven as a Flying specialist. Inside, the guide at (5,14) explains Wyrel, the two fan settings and Hallen’s optional catwalk. Required trainer Wyrel introduces Deepulse at level 25. On the Quarry Landings archive Saven asks the challenger to adapt as the wind turns. Her registered loss page mentions healing, but a loss here sends the player to the clinic and skips that page. The Wind Lesson editions give Saven a new pre-fight line, “If BANKWING digs, send in a flier.” The player sees it before selecting an action. Flossail is Poison/Flying and appears in a 30% daytime grass slot on both the stretch just south of town (level 22) and the middle section (level 21). Neither edition’s guide names a catch that resists her birds or explains Bankwing’s Electric immunity. The Route 5 and St. Cloud conversations also leave Bristub’s evolution for the player to discover.

In Route 5’s middle section, grass at (27,12) is 14 dry steps from the northern entrance. Its daytime table gives Bristub a 10% slot at level 24 on a native-level encounter roll. Before Saven, Minneapolis’s Mill Tool Workshop can reteach a level-eligible move for ₽500. From the third gym’s exit mat at (64,34), the workshop door at (24,17) is reachable in 63 steps on foot. Its sign is at (22,18). Three steps from the interior arrival bring the player within speaking distance of the apprentice at (5,5). The apprentice requests consent and charges only after teaching succeeds. The script has no badge requirement. Latchquill’s own level-10 move list includes Rollout, making the lesson available as soon as Bristub evolves at 28.
