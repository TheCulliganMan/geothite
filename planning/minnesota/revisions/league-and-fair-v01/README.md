# Minnesota League and State Fair

Fern, Flint, Veil and Halden now meet the player in their own field sprites and enter battle with their own portraits. Each uses the existing approved 56×56 portrait, its four-color palette and its six 16×16 walking poses. Left-facing poses also supply the mirrored right-facing frames. Each member keeps the battle style associated with their original trainer class.

Flint opens with Bandrake, which can use Sandstorm and Barrier. His level-49 Fortigard now knows Defense Curl, Rock Slide, Rollout and Psychic. A player who sees Defense Curl has a turn to respond before Rollout builds power. A level-matched Minnesota Latchquill can learn Protect and interrupt the chain, and Fortigard can choose another attack afterward. In the matched full-PP fixture against a level-49 Furrowbank showing Slam, the original ace chose Barrier followed by Rock Slide. The revised ace chose Defense Curl followed by Rollout. Against Latchquill and Taconarch it instead chose Psychic. Crystal's Whitney shows a player what happens if Miltank's Rollout keeps connecting. Polished Crystal also pairs Defense Curl with Rollout on a trained Sandslash. Fortigard learns both moves in its own regional level-up list. [Crystal trainer parties](https://github.com/pret/pokecrystal/blob/master/data/trainers/parties.asm) · [Polished Crystal trainer parties](https://github.com/Rangi42/polishedcrystal/blob/master/data/trainers/parties.asm)

Saven's rematch team and Fern's League team both have a Dakotaskip with Baton Pass. The competitive trainer now chooses a living teammate and commits to that choice when selecting the move. The receiving Pokémon takes its place in normal turn order and inherits eligible effects. A trainer with no eligible teammate can choose another move. If forced to attempt Baton Pass alone, it spends PP and the turn proceeds. When an opponent copies a player’s Baton Pass with Mirror Move, the called move can also pass. This uses the first eligible reserve in party order without consuming another random value. Veil’s Prairondo can encounter this situation in the League.

The fair buildings in St. Paul house local services. The public fairgrounds lie west of Route 5, north of Minneapolis. Eight signs and conversations now distinguish these destinations. The civic guide keeps her story-dependent answers. Her directions to Union Depot also match its location. The four crossing points are the St. Paul west exit, the west end of Route 4, the Minneapolis north exit and the west entrance from Route 5. Both road lanes lead to the fair welcome desk and back, through the city's ordinary road checks.

Casey's invitation to a returning Champion now describes the rematch the player can actually accept. The Hall of Fame registration, return to Ely and the first challenge still use their existing programs. The eight Gym rematches are a separate postgame circuit. In the reviewed pack, Tamarack promises five partners per rematch and each leader brings five. Every guide has an accessible talking square and return carpet. The guides track all eight wins, hold the Leftovers when the Pack is full and award them once.

## Build and check

Field builds on Northern Trails. The selected local Mothlight builds on its Battle Café child so the Lodge cook and daily challenge carry forward. The Field and Mothlight receipts record the pack hashes and identities are tracked in the world selection.

```sh
src=planning/minnesota/revisions/league-and-fair-v01
parent=content-packs/minnesota-mothlight-battle-cafe-v01.crystalpack
child=content-packs/minnesota-mothlight-battle-cafe-league-and-fair-v01.crystalpack
# For Field use the input and output in field-build.json.
export CARGO_INCREMENTAL=0 RUST_MIN_STACK=67108864
cargo run --locked -j1 -p crystal-assets --example pack_league_and_fair -- "$parent" "$child"
for audit in check-fair check-league check-champion; do
    uv run --with cbor2 --with pillow python "$src/$audit.py" "$parent" "$child" || exit 1
done
export CRYSTAL_MINNESOTA_BATTLE_TEST_PACK="$PWD/$child" MN_LEAGUE_FAIR_PACK="$PWD/$child"
cargo test --locked -j1 -p crystal-core --lib baton_pass
cargo test --locked -j1 -p crystal-assets --lib competitive_baton_pass_tests -- --ignored --test-threads=1
cargo test --locked -j1 -p crystal-assets --lib flint_league_tests -- --ignored --nocapture --test-threads=1
cargo test --locked -j1 -p crystal-bevy --lib elite_four_portraits_and_walking_frames_resolve_from_installed_classes -- --ignored
```

A second build reproduced each final pack byte for byte. Independent comparisons covered all 743 Field maps and 745 Mothlight maps, including every terrain block, warp and map edge. The nine changed rooms are the four Elite Four chambers, Casey’s chamber and the four locations with revised fair directions. The final Mothlight comparison also covered all 877 trainers, including the Café cook.

The portrait test uses the native battle and overworld renderers. Set `MN_LEAGUE_RENDER_DIR` to save their still frames for inspection. The battle fixtures exercise the trainer planner and turn resolver directly. They measure defined choices and outcomes on installed teams. Campaign difficulty and the pacing of a complete played League run remain unmeasured.

## Other reviewed areas

The outdoor cave audit rechecked all 14 bound outdoor openings in the selected Northern Trails edition against six Crystal entrances. It examined the rock face above and beside each opening in the original map, script and field-move states. The Route 20 fork reaches the Granite Passage and Voyageurs Ice landings, with a return from each. At the Sump, the red wheel and pipe lead to the Pump Gallery hatch, whose ladder leads back to the same work area. The League revision's preservation check compares terrain and warps across every map in both editions.

Casey still uses the stock Lance field sprite and the stock Champion portrait. The previous accepted Minnesota cast ends at Halden. A distinct Casey would need newly authored field and battle artwork.

The [campaign coverage ledger](coverage.md) records evidence and open questions for each story stage and side quest.
