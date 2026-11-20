# Parties for the badge and League battle audit

All three journeys begin with a level 5 Field Station starter. Each companion comes from the daytime grass table on the road to its listed badge. Percentages are the share of encounter slots once a grass encounter occurs. [Party construction](../../../../crates/crystal-assets/src/badge_battle_parties.rs)

| Starter | Route 1 before Rochester | Fourth member | Route 6 before Brainerd |
| --- | --- | --- | --- |
| Pouchid | Nibbin 2 (30%), Podlet 2 (30%) | Route 2 Pipboom 9 (20%), before Mankato | Bristub 27 (20%) |
| Kindlit | Podlet 2 (30%), Tesslarv 3 (20%) | Route 3 Chorip 12 (30%), before Minneapolis | Pitchwick 26 (30%) |
| Birchit | Tesslarv 3 (20%), Nibbin 2 (30%) | Route 3 Velvawn 14 (20%), before Minneapolis | Pasquib 28 (10%) |

The starter enters badges 1–8 at levels **11, 15, 21, 24, 29, 34, 39 and 44**. Its four companions trail by one, two, one and two levels in the order shown. The same five enter each League fight at **47, 46, 45, 46 and 45**. At that point the Pouchid party consists of Cyprielle, Furrowbank, Parafloss, Prairondo and Latchquill. Kindlit leads to Borealynx, Parafloss, Mothosaic, Choirbog and Pyropine. Birchit leads to Voyagear, Mothosaic, Furrowbank, Shedstag and Vernacape.

The fixture gives each wild partner its capture moves through the game's normal regional encounter rules. It raises each Pokémon one level at a time, accepts normal daytime evolutions, and carries learned moves across them. When four moves are known, it replaces the oldest with the new offer unless doing so would leave four status moves. It checks every acquisition and evolution against the active Minnesota roster. [Wild moves](../../../../crates/crystal-assets/src/wild_challenge.rs) · [Level-up rules](../../../../crates/crystal-core/src/systems/battle_rewards.rs) · [Evolution rules](../../../../crates/crystal-core/src/systems/evolution.rs)

Before each challenge the fixture executes the badge-granting commands from the preceding gym leader scripts. It checks both the progression flags and the badges the battle engine reads for stat bonuses. The original isolated benchmark set only the progression flags. The current tables use the corrected setup.

These are repeatable battle fixtures. They start each matchup healed with fixed DVs of 8 in the four non-HP stats, zero stat experience, and empty held-item slots. Wild partners are sampled at their authored level floors. An actual party can encounter them at higher levels. Time spent training, manually taught moves, bag resources and attrition between League rooms are outside this comparison.
