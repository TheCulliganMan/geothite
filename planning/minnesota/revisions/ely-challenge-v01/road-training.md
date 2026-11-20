# Training on Route 13

Each of the six travelers takes a turn in daytime grass just north of Ely. The comparison begins after the team's one ₽18,000 purchase. Six Hyper Potions, six Revives and three Full Heals stay in the Bag. The trainer's purse is empty, so ordinary blackout money loss cannot consume money intended for supplies. [Original parties and equipment](parties.md) · [Ely purchases and League trials](../../../../crates/crystal-assets/src/ely_challenge_run.rs)

The five resting partners wait in Ely's PC. The trainer puts the traveling partner's Gold Berry away at the clinic, fights every encountered opponent, and takes the normal clinic recovery after a loss. A surviving partner also visits the nurse when below three-quarters HP, affected by status, or out of PP on a move. The trainer gives the berry back at the final clinic visit, withdraws the next partner before depositing the current one, and eventually restores the original order. Training uses no purchased treatment.

Each partner stops after 12 wins or 16 encounters. An encounter comes from the native daytime grass roll at (34,23) on southern Route 13, including the ordinary wild level variation, DVs, and moves. The wild Pokémon uses the game's own move selection. The player's solo partner chooses the highest expected direct damage of its legal moves. Only a surviving knockout earns experience. Stat experience follows each defeated species' actual base stats. Level-up offers can replace Psybeam or Petal Dance with Psychic, Faint Attack with Crunch, Magnitude with Earthquake, and Sludge or Razor Wind with Sludge Bomb if the partner knows the corresponding move. Other offers are declined.

The test logs each opponent's species, level and awarded XP, and each partner's level change, total XP, stat experience, whiteouts and clinic visits. One trained team per starter is shared across the three League seeds. Walking time, time spent at the PC and time spent at the counter are outside the model. [Native training fixture](../../../../crates/crystal-assets/src/ely_road_training.rs)

## Measured preparation

Values below come from the installed Ely Challenge Intro edition. Each entry gives wins over encounters, experience received and level on return to Ely.

| Party slot | Pouchid team | Kindlit team | Birchit team |
| --- | --- | --- | --- |
| 1 | Cyprielle 8/16, 7,071 XP, Lv47 → 48 | Borealynx 12/16, 13,478 XP, Lv47 → 48 | Voyagear 12/14, 11,032 XP, Lv47 → 48 |
| 2 | Furrowbank 12/15, 10,204 XP, Lv46 → 47 | Parafloss 8/16, 10,590 XP, Lv46 → 47 | Mothosaic 8/16, 6,400 XP, still Lv46 |
| 3 | Parafloss 8/16, 10,335 XP, Lv45 → 46 | Mothosaic 11/16, 9,057 XP, Lv45 → 46 | Furrowbank 11/16, 9,544 XP, Lv45 → 46 |
| 4 | Prairondo 12/12, 10,783 XP, Lv46 → 47 | Choirbog 10/16, 7,238 XP, Lv46 → 47 | Shedstag 12/16, 10,978 XP, Lv46 → 47 |
| 5 | Latchquill 12/14, 9,486 XP, Lv45 → 46 | Pyropine 12/12, 11,080 XP, Lv45 → 46 | Vernacape 12/15, 10,911 XP, Lv45 → 46 |
| 6 | Fortigard 6/16, 3,896 XP, still Lv46 | Latchquill 9/16, 8,414 XP, Lv45 → 46 | Pyropine 12/12, 12,646 XP, Lv46 → 47 |

| Starter | Total encounters | Wins | Blackouts | Nurse visits beyond blackout recovery |
| --- | ---: | ---: | ---: | ---: |
| Pouchid | 89 | 58 | 31 | 34 |
| Kindlit | 92 | 62 | 30 | 42 |
| Birchit | 89 | 67 | 22 | 37 |

Sixteen partners gained one level. Fortigard and the Birchit team’s Mothosaic earned experience and stat experience without crossing their next level threshold. Cyprielle learned Psychic in place of Petal Dance. All three crews reached the desk with their original medicine and held berries. The number of blackouts and repeat clinic visits makes solo training expensive in time. These are automated battle outcomes under the stated policy.
