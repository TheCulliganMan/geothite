# Town returns

## The crew book in Duluth

At Union Hall a boy looked for his mother in the night crew book. The heading was all he could find. The tally keeper upstairs at the Ore Warehouse has her name, Alma Lind, and the names of her coworkers. You can meet the keeper or the ground-floor hall host first. Upstairs in the hall, the recorder copies all three names. Read the open book on the west side of the table to see Alma between Nora Berg and Gus Ward. The pages and center fold use four original 8×8 tiles drawn in the table’s existing four shades. [Book drawing](crew-book.json) · [Table placement](../../../../crates/crystal-assets/src/northstar_crew_book.rs)

Recording the names and collecting the thank-you Ether are separate events. When the Pack is full, the book still changes, the hall host and warehouse clerk recognize the delivery, and the recorder holds the Ether. A later visit collects it once. A preexisting reward receipt also opens the written book. The old map, actor, item and reward identities are retained. [Return conversations and custody checks](check-returns.py)

## Rochester and the restored water

Early in the story, the player can find Loobit and Ivy at the Rochester clinic. During the rescue, Ivy has left and the aide keeps Loobit there. After the survey restores the water, the aide tells you she carried the basin down to the bank. Loobit slipped out, dove and surfaced behind her. The Pokémon has gone from the clinic. Before its release, talking to it uses Loobit’s original cry. The review resolved the actor and sound through the engine, synthesized its 20,305 stereo frames and matched the original sample hash. Crystal and Polished Crystal’s Olivine lighthouse scenes likewise let the recovering Pokémon, its cry and the person beside it tell the same story. [Wildlife and starter review](wildlife.md) · [Crystal’s lighthouse](https://github.com/pret/pokecrystal/blob/master/maps/OlivineLighthouse6F.asm) · [Polished Crystal’s lighthouse](https://github.com/Rangi42/polishedcrystal/blob/master/maps/OlivineLighthouse6F.asm)

The Intro edition now carries the four resident conversations already present in Field. St. Cloud’s stonecutter reacts after the worker is freed. The river watcher, Duluth dock hand and island artist describe their partners again when the water returns. At the State Fair the ring-toss host promises Sweet Corn, and a winning throw counts even when the Pack is full. [First visits, return visits and prize checks](check-fair-residents.py)

## Harbor Gym

The west gate that Norek opens used to give a longer route to the exit. The new skating lane turns at two rough patches. After the badge, slide west from the central rough patch, south past Livra and east at the last stop. From Norek’s west, north and east sides, leaving now takes 9, 11 and 13 directional inputs. Each route saves five inputs. Before the badge the original ice course, two required sight battles and optional third opponent remain in place. [Room design, route comparison and return tests](dungeon-review.md)

## Build and verify

The [Field receipt](field-build.json) and [Intro receipt](intro-build.json) name the exact external parent and expected output for each edition. Choose a fresh path for the result. Both have their own pack and save identity.

```sh
src=planning/minnesota/revisions/return-visits-v01
edition=intro
parent=$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["parent_pack"])' "$src/$edition-build.json")
mkdir -p target/town-return-review
child="$PWD/target/town-return-review/$edition.crystalpack"
CARGO_INCREMENTAL=0 cargo run --locked -j1 -p crystal-assets --example pack_town_returns -- "$parent" "$child"
uv run --with cbor2 --with pillow python "$src/check-preservation.py" "$parent" "$child"
uv run --with cbor2 --with pillow python "$src/check-returns.py" "$child" --parent "$parent"
uv run --with cbor2 --with pillow python "$src/check-fair-residents.py" "$parent" "$child"
uv run --with cbor2 --with pillow python "$src/check-wildlife.py" "$child"
uv run --with cbor2 --with pillow python "$src/check-harbor-gym.py" "$child" --parent "$parent"
export MN_TOWN_RETURNS_PACK="$child" MN_RETURN_VISITS_PACK="$child"
export CARGO_INCREMENTAL=0 RUST_MIN_STACK=67108864
cargo test --locked -j1 -p crystal-assets --lib clinic_return_loads_matching_actor_cry_and_restored_room -- --ignored --test-threads=1
cargo test --locked -j1 -p crystal-assets --lib harbor_exit_runs_on_reward_reentry_and_continue -- --ignored --test-threads=1
cargo test --locked -j1 -p crystal-assets --lib crew_book_and_held_ether_survive_same_pack_continue -- --ignored --test-threads=1
```

The comparisons cover six changed rooms in Field and eight in Intro. All previous event addresses, the 64 original house metatiles at morning, day and night, the original audio, encounters, teams and field moves compare equal. Verification used compiled game content, native renders, the archive’s own font and frame, and focused engine tests. The [campaign ledger](../league-and-fair-v01/coverage.md) records the wider review.
