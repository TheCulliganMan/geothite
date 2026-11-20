# Halden's fourth room

Veil tells a challenger with three wins that Halden will give them a minute. Halden now asks whether they're ready. Declining leaves the trainer in the room with their party, money and three wins. They can open the Pack, use their own medicine, save, and talk to him when ready. Halden remembers the earlier conversation. [Room and service checks](counter.md)

On the first challenge Halden brings Knotitan, Marshrack, Pikeman and Barbelisk. Marshrack is a Grass/Water species found on Route 13. At level 50 it knows Mega Drain, Bubblebeam, Stomp and Growth by level-up. Previously the second slot was a level-50 Musketunge with the same four moves the challenger would see again on Casey's level-52 Musketunge. The postgame Halden keeps his original team. Both editions retain Casey's six partners and all five rooms' original victory, loss and door behavior.

## Consecutive challenges

Each of the three starter parties entered at levels 45–47 with six paid Hyper Potions, six Revives and three Full Heals. The same party and remaining medicine went into the next room after each win. Each party used the same three seeds before and after the change. [Party sources](../ely-challenge-v01/parties.md) · [Battle decisions](../ely-challenge-v01/tactics.md)

| Starter | Reached Casey before | Reached Casey after | Circuit wins before | Circuit wins after |
| --- | ---: | ---: | ---: | ---: |
| Pouchid | 1/3 | 3/3 | 0/3 | 0/3 |
| Kindlit | 3/3 | 3/3 | 0/3 | 0/3 |
| Birchit | 3/3 | 3/3 | 0/3 | 1/3 |

Birchit's winning party finished Casey's battle in 26 turns with three partners standing. Across the nine attempts all three Pouchid runs now got past Halden. One other Pouchid run left five of Casey's six standing, and two Kindlit runs left all six standing. The signal is that the fourth room presents a different matchup and an attainable path through all five rooms. These fixed teams and seeds give only a small sample of how players might fare.

A separate comparison reconstructed five partners from the archived first-registration save and a Route 13 catch. That party reached Casey twice on each edition and lost at Fern on the third seed. On seed 4242 it defeated two of Casey's partners after Marshrack's introduction. It previously lost that fight with all six opponents standing. The original save remains bound to its original pack. [Earned party and outing](../ely-challenge-v01/tactics.md) · [Badge machines](machines.md) · [All nine postgame fights](postgame.md)

## Reproduce on a local pack

Use the matching Finder Lesson parent in the [Intro receipt](intro-build.json) or [Field receipt](field-build.json). The builder requires a fresh output path and rereads the resulting archive.

```sh
export CARGO_INCREMENTAL=0 RUST_MIN_STACK=67108864
cargo run --locked -j1 -p crystal-assets --example pack_ely_challenge -- --ely-return "$parent" "$child"
uv run --offline --no-project --with cbor2 --with pillow python planning/minnesota/revisions/ely-return-v01/check-pack.py "$parent" "$child"
MN_BADGE_TRAILS_PACK="$child" cargo test --locked -j1 -p crystal-bevy --lib halden_readiness_ -- --ignored --test-threads=1
CRYSTAL_MINNESOTA_BATTLE_TEST_PACK="$child" cargo test --locked -j1 -p crystal-assets --lib -- --ignored --test-threads=1 ely_first_challenge_uses_the_installed_halden_party_before_casey ely_triage_challengers_spend_one_receipt_across_the_league postgame_rematches_field_the_announced_parties_and_finish_all_nine_battles
```
