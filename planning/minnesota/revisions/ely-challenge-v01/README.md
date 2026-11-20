# Preparing for the Ely League

The sign outside the League now sends a challenger whose survey is unfinished to the registrar. The desk can identify the particular missing handoff. Before asking a qualified challenger to enter, the registrar explains that the five rooms have no nurse. A returning Champion can read exactly when a retreat ends the run. [The desk, sign and native pages](ely.md)

The five-room test buys medicine once in Ely and carries the same six Pokémon forward, including damage, status, move uses, levels and consumed berries. A second comparison starts Halden and Casey with a fresh party to distinguish the opponent from the cost of the earlier fights. The three different teams and both approaches to the first room are documented. [Party preparation](parties.md) · [Results by chamber](sequence.md)

An earlier set of 117 isolated first-challenge fights used the story flags for earned badges. The battle engine reads a separate badge record. The corrected fixture executes the preceding gym leaders' own badge-granting commands and checks both representations. The new comparison includes the 27 Brainerd retries, the 18 St. Cloud openings and the two before-and-after League matchups. [Corrected badge and League table](../badge-battles-v01/results.md) · [Brainerd](../badge-battles-v01/middle.md) · [St. Cloud](../badge-battles-v01/st-cloud.md)

The regional review also traced the survey through the intake and the voyage back to the International Falls clinic to collect the reward. It covers all 19 towns, 59 hidden items, 303 sight trainers and 24 milestone completions. A further battle comparison uses actual Route 13 encounters before returning to Ely. [Regional coverage and remaining cases](coverage.md) · [Intake and clinic check](check-milestones.py) · [Route 13 training](road-training.md)

The older story builder now follows the explicit room that owns the six shelter conversations. It also accepts the existing Duluth engineer’s 31-line conversation across multiple pages. Each line still has to fit the native 18-column text window.

## Build and reproduce

Use the matching Badge Battles parent from the [Intro receipt](intro-build.json) or [Field receipt](field-build.json). The builder writes and rereads a fresh external pack. The preservation check compares both full archives, including all maps, trainers, art, collision, audio, and script tables.

```sh
export CARGO_INCREMENTAL=0 RUST_MIN_STACK=67108864
cargo run --locked -j1 -p crystal-assets --example pack_ely_challenge -- "$parent" "$child"
uv run --offline --no-project --with cbor2 --with pillow python planning/minnesota/revisions/ely-challenge-v01/check-pack.py "$parent" "$child"
export CRYSTAL_MINNESOTA_BATTLE_TEST_PACK="$child" MN_BADGE_TRAILS_PACK="$child"
cargo test --locked -j1 -p crystal-assets --lib -- --ignored --nocapture --test-threads=1 ely_prepared_challengers ely_scouted_
cargo test --locked -j1 -p crystal-bevy --lib -- --ignored --test-threads=1 ely_sign_and_registrar_explain_the_actual_run_and_survey ely_hall_can_be_revisited_from_both_exits_before_recording league_return_trip_
```

Set `MN_ELY_CHALLENGE_FRAMES` to an absolute directory to keep the native Ely sign and registration pages. The Hall test starts with five wins before induction, leaves the Hall on either lane, saves in Casey's room, returns and opens the physical register. It checks the party, bag and prize money before the first record is accepted.
