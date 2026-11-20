# Ely League, from registration to Casey

The untrained comparison enters Fern's room with six healthy partners at levels 45–47. The three starter histories use both early-route partners and a final catch on Route 13. Before entry the challenger buys six Hyper Potions, six Revives and three Full Heals at the Ely counter for ₽18,000. The workshop moves and six held Gold Berries cost an additional ₽2,060–3,060 depending on the party. For scale, the eight first-time gym leaders award ₽10,064 and the eighteen gym students together award ₽21,240 when every prize goes to the player. The fixture assumes the player chose to save enough for the purchases. [Parties and receipts](parties.md)

The same party and random sequence continue after every victory. Eight badge grants from the installed gym scripts set both the story flags and the badge bits used during battle. The game awards experience and prize money, offers level-up moves and checks evolutions. These challengers keep their four prepared moves when offered a replacement. A consumed berry stays consumed. Damage, PP, levels and status carry into the next room. Between rooms the player revives fainted partners, treats status and spends a Hyper Potion when a partner is at or below three quarters of its maximum HP. Treatment follows party order and stops when the relevant supply is exhausted. There is no mid-run shopping or PP restoration. The first-challenge fixture starts after the Boundary Intake resolution with eight badges. It applies the registrar's run status and each defeated trainer's native progress counter. [Battle and shopping regression](../../../../crates/crystal-assets/src/ely_challenge_run.rs)

Two policies use the same preparations and seeds (1701, 4242 and 9001). Both estimate attacks from the current matchup, choose a living replacement after a faint, and may spend one turn switching when a reserve has a clear matchup advantage. Neither reads the opponent's next action. The returning challenger also knows each opponent's previous lead, reorders the party at the door, and can try one status or healing move against each opposing species. Fresh Halden and Casey challenges use the first policy and all six berries, with HP and PP restored at the beginning of each isolated fight.

## What happened in the five rooms

Fraction denominators count parties that actually reached that room. A party that lost to Fern has no recorded result at Flint.

| With the level 45–47 party | Fern | Flint | Veil | Halden | Casey |
| --- | ---: | ---: | ---: | ---: | ---: |
| Own lead, nine starts | 8/9 | 8/8 | 8/8 | 4/8 | 0/4 |
| Scouted opening and support moves, nine starts | 9/9 | 9/9 | 9/9 | 0/9 | — |

In the first policy, the four teams that beat Halden had four, two, one and one conscious partners at the end of his battle. After treating them with their remaining items, they entered Casey's room with four or five conscious partners and 37–57% of the party's total HP. Every team had spent its six Revives and six Hyper Potions. All four lost with Casey's entire team still standing.

The Kindlit run on seed 9001 reached Halden with five Potions and all six Revives still in the bag. It lost with one opponent remaining. The scouted players won all nine bouts in each of the first three rooms and then lost all nine to Halden. Knowing the opening and spending a turn on support sometimes helps an individual fight and can leave the party worse prepared for the next.

| Fresh six against the last two trainers | Pouchid | Kindlit | Birchit |
| --- | ---: | ---: | ---: |
| Halden, wins in three fights | 0 | 3 | 3 |
| Casey, wins in three fights | 0 | 0 | 0 |
| Casey Pokémon knocked out across those three fights, out of 18 | 0 | 3 | 2 |

The six fresh Halden victories ended with two to six partners still standing. The fresh Casey challenge never came within three knockouts of victory. This points to difficulty at Casey even before counting earlier-room attrition for these teams. It does not estimate the success rate of human players. The trace records each partner's species, level, HP, status, held item, total experience and individual move PP at each door, the supplies remaining and the prize for each win.

## An additional Route 13 outing

A third set used the same scouted battle choices and the same six partners after bounded daytime training on Route 13. The player paid for Ely's medicine before leaving and returned with the full receipt, all six original Gold Berries and a healed party. The three teams fought 270 wild encounters, won 187 and blacked out 83 times. Sixteen of the eighteen partners gained one level. Two earned experience without leveling. All earned stat experience. This is a deliberately strenuous comparison. [Actual training opponents, costs and levels](road-training.md)

The trained parties won 9/9 against Fern, 9/9 against Flint, 9/9 against Veil, 5/9 against Halden, and 0/5 against Casey. The five Casey arrivals knocked out two opponents in total. The Pouchid party lost all three to Halden, the Kindlit party beat him twice and the Birchit party beat him three times. Training advanced the game's saved random state. The two scouted sets used the same three divider seeds at the first room, but their initial random registers differed. These runs compare two preparation histories and cannot isolate the effect of gaining a level.

## Casey's choices on seed 1701

The three fresh fights give different examples of how Casey keeps a team together. The combat trace records both visible HP bars and the challenger's choice at the beginning of each turn. Casey carries one Full Heal and one Full Restore. Musketunge holds no item and its four attacks include no recovery move.

- **Pouchid.** Casey switched out the opening Beetynamo on the first turn. Musketunge answered with Iron Head and Hydro Pump. Latchquill's repeated Headbutts and Fortigard's Psychic brought Musketunge to 22/167. Parafloss chose the two-turn Razor Wind. Musketunge returned to 167/167 during that sequence and the challenger lost without a knockout.
- **Kindlit.** Casey switched Taconarch into the opening Ice Beam, withdrew it at 56/180 and later sent it back at the same HP. Musketunge left at 24/167. At another point, Greytag returned at 53/151. Several meaningful attacks landed on opponents Casey could still use later. The fight ended after 19 turns with one opposing knockout.
- **Birchit.** The player opened by switching to Pyropine. Beetynamo hit it with Thunderbolt, then Casey switched Choirbog into Fire Blast. Later Shedstag took Musketunge down to 2/167 with Shadow Ball. At the next decision Musketunge had 115/167 after another Shadow Ball, consistent with the single Full Restore. Casey finished with five partners standing.

Switching took its own turn, party members returned with the HP at which they left, and one finite restore could undo several hits. Those interactions explain specific turns beyond the raw level difference.
