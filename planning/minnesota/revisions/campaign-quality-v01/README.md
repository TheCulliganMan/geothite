# Campaign directions, gym prizes, and hidden items

Field and Mothlight both descend from the [Crystal cave faces](../crystal-cave-faces-v01/README.md). Their parents contain different amounts of the earlier Lakeside and Route 7–11 work. The builder checks the installed data and keeps the original Field lessons and Rochester art.

## Changes for the player

- At the Boundary intake, Finn accounts for the radio call, the first approach and the battle with Reed. Trying a control early gives directions for the first unfinished step. Defeating Reed, stopping the drill, draining the sump and opening the bypass still require their original actions.
- The St. Paul sign directs walkers northwest to Union Depot and northeast to the Capitol. At the northern bend of the North Shore Trail, the sign points east into Grand Marais. In Mothlight the Lake Itasca sign now identifies the western trail to Detroit Lakes. The street and trail checks start on legal sign-reading squares.
- In Mothlight each gym leader has the opportunity to demonstrate the move in their TM. Ten active regional species can learn Steel Wing, including Patchebee, and nine can learn ThunderPunch. Tavel at the Rochester gym has his Timekeeper art. All 18 ordinary gym trainers initiate on sight. The player speaks to each of the eight leaders.
- Mothlight also gains the missing regional handoffs, advice for the first badge absent at the Ely desk, and Mom’s homecoming after the championship. Her repeat visit retains healing and the home return point.
- Five existing hidden items have local clues, including the quartz seam at the Survey Mine route and the fallen pine cones on Route 8. Both editions use the surrounding terrain’s native background colors through morning, day and night.

The Route 30 sign in Crystal names the two places a trail joins. At a decision point the separate sign for Mr. Pokémon’s house tells the player which way to go. Polished Crystal retains those two roles when it moves the signs to fit its revised map. We used the same distinction at the North Shore bend. [Crystal Route 30](https://github.com/pret/pokecrystal/blob/master/maps/Route30.asm) · [Polished Crystal Route 30](https://github.com/Rangi42/polishedcrystal/blob/master/maps/Route30.asm)

## Build and verify

From the repository root, select the release prefix below and the corresponding fresh output. Use a single Cargo job. Both parent and child packs are external, and both build recipes record their exact SHA256.

```sh
stem=content-packs/minnesota
# For Mothlight use stem=content-packs/minnesota-mothlight
parent="$stem-crystal-cave-faces-v01.crystalpack"
child="$stem-campaign-quality-v01.crystalpack"
src=planning/minnesota/revisions/campaign-quality-v01
CARGO_INCREMENTAL=0 cargo run --locked -j1 -p crystal-assets --example pack_campaign_quality -- "$parent" "$src" "$child"
uv run --with cbor2 --with pillow python "$src/check-journey.py" "$child"
uv run --with cbor2 --with pillow python "$src/check-preservation.py" "$parent" "$child"
uv run --with cbor2 --with pillow python "$src/check-gyms.py" "$parent" "$child"
uv run --with cbor2 --with pillow python "$src/check-intake.py" "$parent" "$child"
uv run --with cbor2 --with pillow python "$src/check-clues.py" "$parent" "$child"
MN_CLUES_PARENT="$PWD/$parent" RUST_MIN_STACK=67108864 CARGO_INCREMENTAL=0 \
  cargo test --locked -j1 -p crystal-assets --lib route_clues_preserve_rewards_flags_and_native_itemfinder -- --ignored
```

The journey check covers all 256 badge combinations, both survey states and both registration decisions. The intake check follows 28 control and crew responses per edition. The hidden-item checks cover the five clues and compare all 57 rewards and collection flags. Native data and static scene captures support these results. Campaign-wide story pacing, trainer difficulty, and the wider exploration review are ongoing.

## Findings on the selected Mothlight build

These observations are specific to the [Mothlight build receipt](mothlight-build.json). They describe the game after this revision and give later revisions concrete cases to check.

| Area reviewed | What a player can encounter | How to check a future change |
| --- | --- | --- |
| Route 9 mine, on the way back from Finn | The pump-gallery panel at (25,17) correctly distinguishes the isolated feed from the stopped motor. After the player stops the bit, the panel describes boots on the ladder. Both exits from the drill core return here. | Follow both branches inside the isolated-feed response. Check the panel before isolation, with the bit running and after the motor stops. |
| International Falls and the Voyageurs watch | The clinic ranger and the town notice direct the player east. International Falls has one public road, which leaves south onto Route 17. The border arrival checks the two western entry tiles of Boundary Waters, all eight badges, the harbor signal and clinic registration. Players can register with seven badges. | Trace the trip from the clinic to the arrival tiles and back. Check the ranger's answer when the player has seven badges, when the harbor signal is missing, and when the PP Up is ready or the Pack is full. |
| Inner Basin | Three survey dials at (9,10), (18,14) and (11,18) look like ordinary repeated rock. The outlet lever at (14,20) does too. A player looking for the next mechanism has to guess which rock to examine. | Compare each control with its neighbors at native scale. Preserve its reachable reading square and the outlet opening after all three settings are correct. |
| Gunflint spring and Driftless Lamp Gallery | The spring's original solid pillar at (28,16) has become four floor squares. Players can dismount on it and search the Revive from land. In the Lamp Gallery the fossil at (6,4) describes the east shelf, where the Rare Candy sits. | The original pillar block at (14,8) allows two water-side searches and preserves all 470 dry squares reachable from the entrance and all 112 water squares. Check the fossil text against the northwest shelf in the 24-by-20 room. |
| Trainers on Routes 1–20 | All 125 regular trainers have clear sightlines. Of those, 122 can see a square reachable on foot from a route entrance. Three optional opponents on Routes 6, 8 and 13 overlook pockets opened by Cut. The shortest crossings of Routes 1 and 2 can bypass every trainer. | On Route 2, Tess at (51,21) currently faces away from the Rochester road. Facing east for one square gives the player a visible battle and a walk around her of equal length. Two squares would close that choice. |

The story review traced all 24 authored milestones. It also found that Moorhead's planned prairie trainer circuit and Nugget differ from the installed gauge-board errand and Full Heal. The opening battle review calculated experience for a single level-5 starter with no wild victories. The required wins over Reed and Tavel bring that starter to level 7 before Varel's level-10 and level-12 partners. Adding the optional gym trainer brings the starter to level 8. Winning all three optional Route 1 battles instead brings it to level 9. These numbers establish the available training choices. Actual battle difficulty still needs separate evaluation.
