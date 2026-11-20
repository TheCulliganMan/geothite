# The Winona Itemfinder lesson

The research aide gives the Itemfinder to a trainer who has caught 25 different Minnesota species and already collected the ten-catch award. The aide remembers hearing it beep in a cave, explains how to face a square and search it, and mentions the SELECT shortcut. The tool works underground as well as on trails. Crystal's Ecruteak Itemfinder giver also explains what to do when it responds. [Crystal’s Ecruteak house](https://github.com/pret/pokecrystal/blob/master/maps/EcruteakItemfinderHouse.asm)

A researcher who still has a Poké Ball to find in Winona hears a clue to the grass beneath the riverside fir east of the clinic. The clinic door is at (17,7) and the hidden ball is at (25,8). A player who found it earlier gets the tool and its ordinary instructions and can leave immediately. All five Pokédex award tiers now use the aide’s own full-pocket response. Tamarack keeps his line when offering the Master Ball.

The runner accepts an Ely Challenge archive and a fresh external destination. It builds and rereads the child. The Winona practice ball is on dry ground one step from the river. You can walk over it, press A from beside it, or carry the canoe farther east into the water. The previous river tiles put an eight-pixel strip of stone inside the first navigable water square. During the launch the native camera showed the keel on brown gravel. The revised landing puts the stone strip on the bank and leaves the first navigable square animated from its edge. The crossing test follows the walk, launch, paddle and return, sampling the actual ground beneath the keel on every mounted frame.

The independent comparison checks the entire archive, including the town and station, other maps, collision, trainer teams and audio. [Field receipt](field-build.json) · [Intro receipt](intro-build.json)

```sh
cargo run --locked -j1 -p crystal-assets --example pack_ely_challenge -- --finder-lesson "$parent" "$child"
uv run --offline --no-project --with cbor2 --with pillow python planning/minnesota/revisions/finder-lesson-v01/check-pack.py "$parent" "$child"
MN_BADGE_TRAILS_PACK="$child" cargo test --locked -j1 -p crystal-bevy --lib -- --ignored --test-threads=1 winona_itemfinder_aide_ ely_hidden_items_survive_full_pockets_and_continue ely_shift_
CRYSTAL_CANOE_TEST_PACK="$child" cargo test --locked -j1 -p crystal-bevy --lib -- --ignored --test-threads=1 winona_walk_over_the_riverside_ball
```
