# St. Paul and State Fair games

The public fair has a prize-name error. The ring-toss host at (17,24) in `MnStateFairgrounds` promises a **CORN STICK**. After a win, the Pack shows **SWEET CORN**, a food that restores 20 HP. The award itself uses the correct item, `FAIR_CORN_STICK`. Three installed dialogue pages use its old name: `MnFairGameInviteText`, `MnFairGameBagText` and `MnFairGameWinText`. These cover the invitation, the full-Pack reminder and the successful handoff. [Canonical fair dialogue](../../story.json) · [Food names and effects](../../../../crates/crystal-assets/src/state_fair_modpack.rs)

## Getting there and back

St. Paul’s Quiz Pavilion is a local annex. Its door at (25,15) leads to a host at (5,3), reachable four steps from the indoor arrival. The public midway is west across Route 4 to Minneapolis, north onto Route 5 and west at its fair turning. Both lanes work at each of the four road crossings in both directions. Talking to the Minneapolis road officers opens their crossings. The fair’s grounds board points back to the east exit. [Road and annex checks](../league-and-fair-v01/check-fair.py)

With all 255 fairground actors occupying their home squares, both east gate lanes reach all 39 staffed counters, including 32 shops. From the southern entry square (95,65), the walks to the midway quiz, ring toss, prize clerk and care challenge take 125, 117, 77 and 59 steps respectively. Each return is the same length. All ten St. Paul annex doors and the three public halls have accessible outward and return approaches. The two exit carpets in each of those 13 interiors resolve to the correct outdoor doorstep. [Floor plan and counter locations](../../zones/state-fairgrounds.json)

## Awards and repeat visits

| Activity | What the installed scripts do |
| --- | --- |
| St. Paul Quiz Pavilion | Yes, no, yes answers the Mint Berry, Potion and Lure Ball questions and earns one SilverPowder. Wrong answers restart the quiz. A full Pack leaves the prize available and the host explains that the same questions must be answered again. |
| Midway stamp rally | The barn volunteer (5,3), dairy demonstrator (3,1) and midway host (9,24) supply three distinct stamps in any order. The dairy teaches the answer to the midway butter-carving question. A wrong answer points back there. At (21,64) the prize clerk checks all three stamps and gives one Rare Candy. A full Pack keeps all three stamps intact. |
| Ring toss | Each accepted free throw draws one of two outcomes. Declining or missing permits an immediate retry. A win with a full Pack remains claimable on return without another throw. |
| 4-H care question | The host at (37,62) asks whether to stop and check a partner that sits down at the show gate. Yes earns one Berry. A wrong answer or a full Pack allows another attempt. |

The static trace used the selected Trail Discoveries Intro archive (`07c75a8928bc`). It checked all eight answer patterns at the annex, all six stamp orders, both toss outcomes, successful and refused item handoffs, and repeats after each reward. Each worked in all four combinations of survey restoration and League Champion flags. The final combination includes the normal Champion return. Movement and reward scripts were examined without running gameplay. [Release identity](intro-build.json) · [Fair reward programs](../../../../crates/crystal-assets/src/state_fair_scripts.rs)
