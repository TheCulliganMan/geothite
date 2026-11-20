# Brainerd, the badge teams and the Ely League

A challenger returning from the Brainerd clinic can find Reedraft in the daytime grass on the east road. The optional guide by the gym door describes how it drains Bankmaw. After the Lake Badge, the same person directs the player to the fishing pier. After the ranger's report, the guide names Leech Lake Trail on the north road. A returning Champion gets the rematch conversation. [Brainerd and the middle badges](middle.md)

Veil's Doublehex can use Iron Head after Swords Dance raises its Attack. Halden's Knotitan can paralyze a challenger with Glare, creating an opening for its next teammate. At Norek's return match, Rimeguard uses Ice Punch with its stronger physical Attack. The eight opening teams keep their original species, moves and levels. [Opening badges](early.md) · [Ely and the return matches](league.md)

## Battle evidence

Each of three parties begins with a different starter and recruits companions along routes it could have reached before each badge. The tests assign the moves a real capture gets, level each partner through its evolutions, and send each party through an isolated match against each of the eight leaders, the four Elite Four members and Casey. The opponent uses its actual decisions about moves and switches. The player chooses the attack with the highest expected immediate damage. All three parties enter each League room freshly healed at the same levels. [Acquisition and levels](parties.md) · [Results by fight](results.md) · [St. Cloud retries](st-cloud.md)

A separate Brainerd test compares the original five, adding the local catch as a reserve, and spending a turn bringing it in on Bankmaw. Reedraft has both Giga Drain and Mega Drain at the level-29 encounter floor. The Pouchid party gains a win with the extra catch. Two successful Birchit runs become losses when required to switch immediately. The guide describes the species interaction and leaves that choice to the player. [All 27 Brainerd battles](middle.md)

## Reproduce

Use the corresponding Fen Waterline parent and choose a fresh external output path. The [Field receipt](field-build.json) and [Intro receipt](intro-build.json) record the archive hashes. The archive comparison also follows the clinic-to-grass route, checks the east grass encounter and reads the north sign.

```sh
export CARGO_INCREMENTAL=0 RUST_MIN_STACK=67108864
cargo run --locked -j1 -p crystal-assets --example pack_badge_battles -- "$parent" "$child"
uv run --offline --no-project --with cbor2 --with pillow python planning/minnesota/revisions/badge-battles-v01/check-pack.py "$parent" "$child"
export CRYSTAL_MINNESOTA_BATTLE_TEST_PACK="$child" MN_BADGE_TRAILS_PACK="$child"
for seed in 1701 4242 9001; do
  MN_BATTLE_REVIEW_SEED="$seed" cargo test --locked -j1 -p crystal-assets --lib representative_badge_and_league_battles -- --ignored --nocapture --test-threads=1
done
cargo test --locked -j1 -p crystal-assets --lib -- --ignored --nocapture --test-threads=1 brainerd_east_road_catch_and_paid_switch_against_sivren st_cloud_retries_with_an_already_raised_partner_leading league_and_rematch_moves_change_their_authored_matchups fern_last_listed_baton_pass_returns_to_the_living_earlier_partner
cargo test --locked -j1 -p crystal-bevy --lib badge_battles_brainerd_guide_pages_follow_the_lake_journey -- --ignored --test-threads=1
```

The guide fixture walks up from the entrance, reads each page through normal input and rejoins the main path. Set `MN_BADGE_BATTLES_GUIDE_FRAMES` to a fresh absolute directory to record the approach and the two hint pages. Shoreline geometry, encounters, animation and sprite assets are identical to Fen Waterline in both editions. The [walking and paddling checks](../fen-waterline-v01/README.md) also run against these packs. The four landings include morning light at the north bank and 9 p.m. on the east bank. The east-bank camera follows Chris from (37,18) out onto the lake and back to that square across 81 consecutive native frames.
