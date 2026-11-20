# The challenger's turn at Ely

The original challenger valued Super Fang as half the target's current HP when choosing an attack. Its substitution rule instead treated Super Fang as a one-power move. It also paid the Minneapolis apprentice to restore Magnitude to Prairondo, but the single-hit damage preview cannot give one range for Magnitude's seven possible powers. That left the move out of the attack ranking. On the recorded fresh Casey approach, Prairondo chose two-turn Dig against Water/Steel Musketunge. The new challenger estimates Magnitude at its median native power of 70 when choosing. The attack itself still draws its power in the battle engine. [Original Casey trace](sequence.md) · [Challenger comparisons](../../../../crates/crystal-assets/src/ely_challenger_tactics.rs)

Parafloss's Razor Wind takes two turns and has 75% accuracy. Its usual alternative, Sludge, cannot damage Musketunge, whose second type is Steel. The new choice considers an immediate finishing hit, actual move priority, the opposing HP bar and Spikes when replacing a partner. It also uses the default Shift question after the trainer announces a replacement. A voluntary switch during a turn still gives the opponent an action. When the player has no eligible reserve, the challenger waits for the actual send-out.

Each fresh Casey comparison starts with the same six partners, levels, purchased berries, ₽18,000 receipt, leads, moves and three divider seeds. One challenger keeps the Bag closed. The other can spend its six Hyper Potions, six Revives and three Full Heals during the fight. Using a medicine consumes one purchased item, targets a specific partner and gives Casey the usual action that turn. A separate nine-attempt comparison carries the same receipt through all five rooms. Until the final door, that challenger saves Hyper Potions for partners missing at least half their HP and treats the greatest deficit first. [Paired and full-circuit fixtures](../../../../crates/crystal-assets/src/ely_challenge_run.rs)

## Outcomes with one shopping trip

These native combat outcomes use the Minnesota Intro Finder Lesson edition and Casey’s first-challenge team. Each column retains its original starter's five traveling partners and one Route 13 recruit. Each cell covers the same three divider seeds, 1701, 4242 and 9001.

| Challenge | Pouchid | Kindlit | Birchit |
| --- | --- | --- | --- |
| Fresh Casey, Bag closed, wins out of three | 0 | 0 | 0 |
| Fresh Casey, paid medicine available, wins out of three | 0 | 0 | 1 |
| Fresh Casey, triage medicine rule, wins out of three | 0 | 0 | 2 |
| Full circuit with medicine, consecutive victories by seed | 3, 3, 4 | 4, 4, 3 | 4, 4, 4 |
| Full circuit with triage, consecutive victories by seed | 3, 3, 4 | 4, 4, 4 | 4, 4, 4 |

With the initial medicine rule, Birchit’s seed-9001 win took 33 turns. Two partners survived. The player spent one Hyper Potion and five Revives, leaving five Hyper Potions, one Revive and all three Full Heals. Each of the six full-circuit challengers who reached Casey ran out of both Hyper Potions and Revives by the end of that fight. The other three lost to Halden. An observed weakness in the initial item rule was reviving another partner on consecutive turns while the opponent continued knocking them out. The triage rule gives the active or a weakened reserve a chance to recover HP between Revives. It changes only the item decision. It retains the same team, stock and legal Shift offers. In the fresh Casey fights, Birchit won on 1701 in 44 turns and on 9001 in 41 turns, finishing each battle with one partner standing. Pouchid and Kindlit each failed to knock out an opposing partner in all three of their fresh triage attempts. The additional medicine decisions helped Birchit’s particular team.

Seven of the nine full-circuit triage entries reached Casey, compared with six using the initial medicine rule. Pouchid and Birchit each knocked out three of Casey’s six on seed 9001. None of the eighteen modeled starter entries cleared all five rooms with the single receipt. A fresh Casey win shows that the final team can be defeated. Achieving that result with the party and supplies left after four wins remains an open balance question.

## First registration after an earned campaign

The October 31 V16 campaign began at New Game with no supplied party, levels, badges or items. Two continuations used its exact native saves after a Lake training limit and a later blackout. The first League registration records eight badges, no Hall entry, ₱17,337, one Ultra Ball, six Hyper Potions, one Revive, two Full Heals and an Awakening. None of the six carried an item. The checkpoint was taken before its first League battle.

| Eligible partner | Level | Total experience | HP / Attack / Defense / Speed / Special stat experience |
| --- | ---: | ---: | --- |
| Boomruff | 37 | 54,096 | 8,422 / 9,443 / 8,512 / 8,081 / 6,664 |
| Shedstag | 38 | 56,972 | 3,601 / 4,068 / 3,679 / 3,186 / 2,898 |
| Barricant | 41 | 72,236 | 8,558 / 9,502 / 8,646 / 8,012 / 7,077 |
| Parafloss | 39 | 63,956 | 7,846 / 8,854 / 7,948 / 7,432 / 6,276 |
| Borealynx | 51 | 130,126 | 17,972 / 20,033 / 18,063 / 17,225 / 14,710 |

At level 51, Borealynx had 162 HP, 124 Attack, 128 Speed and 136 Special Attack. The native formula at the same level and DVs with zero stat experience gives 145 HP, 107 Attack, 112 Speed and 121 Special Attack. The observed difference was smaller for the other partners. Shedstag gained 5 HP and 6 Attack. Barricant gained 9 HP and 9 Attack. Four of these partners were below the modeled level-45–47 party. Its higher-level starter knew Ice Beam, Headbutt, Icy Wind and Quick Attack at registration. Its actual move choices are relevant to matchups even when its level is higher.

The sixth member was level-41 Mudmender, whose family has since retired. The player's boxes were empty. The current-pack trial reconstructs the five eligible species with their earned DVs, experience, five stat-experience counters, six stored battle stats, original trainer and all twenty saved moves. All five species retain the same base stats. Boomruff, for example, had 107 max HP at the checkpoint. Recalculating with all its accumulated stat experience would have credited 108 HP before its next level-up. Every move must belong to the current species' level-up list at the saved level or its current machine compatibility. The model spends the saved wallet at today's Ely counter, buying five additional Ultra Balls, five Revives, two Hyper Potions and one Full Heal for ₱16,500. It then attempts the first daytime Route 13 encounter, with real ball consumption, wild turns and capture rewards, and visits Ely's nurse before entering. This models an additional outing and spending decision. The historical player carried Mudmender into the League. [Current reconstruction and original checksums](../../../../crates/crystal-assets/src/ely_earned_party.rs)

The original V16 compiled archive SHA-256 begins `b1bb11e4de04` and the first-registration save begins `5a1f42f637b5`. Both are read-only inputs. The fixture validates the complete pinned files, reads the original archive's embedded identity, and decodes the save with the native reader against that identity. The current League receives a fresh fixture. The old run’s maps, flags and saved state are retained in their original files.

The Route 13 outing produced a level-46 Pyropine. The third turn used the first Ultra Ball, which caught it. Five Ultra Balls remained. The party took eight Hyper Potions, six Revives, three Full Heals and ₽837 into Ely. It kept the captured moveset and its five travelers’ saved moves. Two of the three modeled entries reached Casey. On 1701, the party defeated one of Casey's six. On 4242 it spent its last Hyper Potion and Revive during Halden's battle, then fell to Casey in four turns. On 9001, Fern defeated it in fourteen turns with three of her partners still standing. These three seeds sample one historic player's party and a modeled new sixth partner. They cannot establish a human completion rate.
