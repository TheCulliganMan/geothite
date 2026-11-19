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
