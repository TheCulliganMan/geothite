# The lake observations and the road to Ely

Detroit Lakes and Lake Itasca now give the same answer about Route 19: the road is open. Walking north through the old Detroit gatehouse used to trigger a six-badge warning on two center squares. Stepping to one side bypassed the check. The entrance sign now names the route. The Lake Badge belongs to the ranger’s optional Max Repel reward. The ranger describes the remaining observations using the pond, the flower bend and the pines. Three native flower blocks show the plants mentioned on the observation board. [Route 19 review and coordinates](route19.md) · [Town and cabin](../../towns/MnDetroitLakes.json)

## Returning to the north shore

After hearing both pump times, the Duluth engineer remembers the call and directs the player to the crew. After the landing scene, the shelter assistant and radio likewise remember where the player stopped. Returning through the hatch restores the controls and actors for that stage of the rescue. The exterior sign describes the open Route 13 trail. Grand Marais’s lamp, the Safari and catching Kettlurk are optional. The separate Devil’s Kettle encounter requires both release and restoration. The opening, iron mine and north-shore quest summaries now name the partners the game gives, Finn as the source of the meter and the intake crew as the final restoration scene. Historical milestone labels and flags remain available to older builders. [Playable progression](../../PROGRESSION.md) · [Harbor, six intake states and Kettle](check-harbor.py)

On Duluth's outer float, a fisherman whose line has snagged points out the water along the west edge. Search one pace north of the southwest corner for a Nugget. Once it is collected, he returns to his conversation about fishing with his father. Crystal’s Route 32 uses a tree recess and a small patch of grass to distinguish two search squares. Here the water beside the deck looks like the rest of the bay, so a line cast from the dock supplies the clue. [Native Duluth clue](duluth-float-clue.json) · [Crystal Route 32](https://github.com/pret/pokecrystal/blob/master/maps/Route32.asm)

The hidden-item audit found 57 separate rewards on 47 Minnesota maps in each edition. Of those, 47 can be reached on foot, seven require water travel and one each requires Cut, Rock Smash or Whirlpool. The static search begins at real entrances and checks the facing square, return route, exact native event and one-time flag. It leaves the player’s acquisition of those field abilities and the amount of room in the Pack outside its scope. [Region-wide hidden-item check](check-hidden.py)

## The League desk and Casey

With all eight badges and an unfinished survey, the Ely registrar identifies the player’s next handoff. A trainer carrying the meter goes to the Grand Rapids technician. Someone carrying the two pump times goes to the Duluth engineer. Once Hale has been beaten, the player needs the shelter radio, and after the call the crew needs help through the shelter’s east door. Badge-specific advice still takes precedence when a badge is missing.

Casey used to say “How about a match?” and start the battle with either answer button. Returning Champions can now say yes or take a moment to prepare. Declining in the chamber preserves four wins and permits an immediate retry. Exiting to the lobby or returning to the clinic after a loss resets the run. Prior championship and survey records persist. All five rooms retain spent HP, PP and supplies across their normal entry and Continue callbacks. Crystal and Polished Crystal also use local door state to mark League progress. [League decision check](check-league.py) · [Native chamber and refusal tests](../../../../crates/crystal-assets/src/northstar_league_return_tests.rs) · [Crystal League registration](https://github.com/pret/pokecrystal/blob/master/maps/IndigoPlateauPokecenter1F.asm) · [Polished Crystal Champion chamber](https://github.com/Rangi42/polishedcrystal/blob/master/maps/LancesRoom.asm)

## The five-battle resource curve

The five League teams have 22 Pokémon, with levels rising from 47 to 53. A fixed-roll calculation used each level-50 starter's naturally learned attacks, carrying its remaining PP forward across all five teams. Cyprielle spent 45 PP and finished 20 opponents. Voyagear spent 55 PP and finished 19. Borealynx spent 60 of 70 PP and finished all 22, but nine opposing attacks in the sample could knock it out from full health in one hit. For example, Flint’s Fortigard can deal 181 with Rock Slide against Borealynx’s 142 HP. The same move matchup is safer for a teammate: the largest sampled direct hit from Fortigard against Musketunge was 34 of its 150 HP.

A level-34 Pikeman can be caught by Surf on Route 9 and evolves at 44. At level 50, that Musketunge covered the remaining opponents for 9 PP beside Cyprielle or 12 PP beside Voyagear. Ely’s shop sells Hyper Potions (1,200), Revives (1,500), Full Heals (600) and Max Potions (2,500), and the clinic restores PP before registration. The calculations use fixed successful hits against fresh opponents, with no badge boosts, held items, critical hits, switches or incoming HP attrition. They measure move coverage and PP demand. They cannot establish a probability of winning the League. [Native damage and PP check](../../../../crates/crystal-assets/src/northstar_league_curve_tests.rs)

## Verification

Set `parent` and `child` from the [Field receipt](field-build.json) or the [Mothlight receipt](mothlight-build.json). The builder requires a fresh output path. Both editions descend from their own Floodwatch parent.

```sh
src=planning/minnesota/revisions/lake-observations-v01
export RUST_MIN_STACK=67108864 CARGO_INCREMENTAL=0
cargo run --locked -j1 -p crystal-assets --example pack_lake_observations -- "$parent" "$child"
uv run --with cbor2 --with pillow python "$src/check-loop.py" "$parent" "$child"
uv run --with cbor2 --with pillow python "$src/check-harbor.py" "$child"
uv run --with cbor2 --with pillow python "$src/check-hidden.py" "$child" --parent "$parent"
uv run --with cbor2 --with pillow python "$src/check-league.py" "$child"
uv run --with cbor2 --with pillow python "$src/check-preservation.py" "$parent" "$child"
MN_LAKE_OBSERVATIONS_PACK="$PWD/$child" cargo test --locked -j1 -p crystal-assets --lib lake_observations_ -- --ignored --test-threads=1 --nocapture
```

The two archive comparisons cover all 743 Field and 745 Mothlight maps. Exactly ten rooms changed in each. All art and animation data, wild encounters, items, trainer parties, map entrances and audio are preserved. The native flowers are placed on ordinary walkable ground. [Whole-pack scope](check-preservation.py)
