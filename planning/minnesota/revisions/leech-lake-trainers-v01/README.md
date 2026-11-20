# Three battles on the Leech Lake trail

Nell, Silas and Wren stand on the northern half of Route 7. Nell has just gotten a branch off the trail, Silas is trying to catch something that dodges every cast, and Wren brought a puck outdoors when the rink closed. In the Trail Signs editions all three carried the same level-30 Chorip with the same four moves. Two more trainers on the road also had that Chorip. Wren even used Dean’s entire party in reverse order.

| Trainer | Party, in send-out order | What the player faces |
| --- | --- | --- |
| Nell, on the clearing trail | Riceling, Velvawn | The little deer can drain HP with Leech Seed and hit physically with Stomp or Take Down. A switch clears the seed, and Take Down costs Velvawn HP. |
| Silas, by the casting bank | Jiglet, Castorock | The Steel/Water lure can use Water Pulse, Metal Claw, Swift and Pursuit. Swift can catch an evasive target. A Ground move threatens both partners. |
| Wren, near the upper loop | Dartduck, Flossail | Dartduck’s feet grip the ground through tight turns. Quick Attack and Low Kick give it a fast physical opening. Flossail follows with Gust, Acid, Cotton Spore and five uses of Giga Drain. |

All six party members are level 30. The [team definitions](parties.json) are applied after the exact Trail Signs [Field](field-build.json) or [Intro](intro-build.json) archive. The original meadow and sandbank authoring inputs remain available to reconstruct the earlier editions. Mara at the reeds and Dean on the causeway still use Chorip.

## What a player can find

Route 6 has a level-27 Velvawn in daytime grass. The southern half of Route 7 has level-20 Jiglet in its Good Rod table. The angler on Route 3 offers that rod. Daytime grass on the southern half also has level-29 Tapduck, which can evolve into Dartduck at the next level. Wren’s Flossail can learn Giga Drain from the ₽3,000 TM at the fifth-floor counter of the Minneapolis department store.

All eight Route 7 opponents have an unobstructed sightline. Their sixteen challenge squares can be reached on foot. The shortest Brainerd-to-Leech Lake crossing is 159 steps. Skirting every sightline takes 169. These distances count movement on the two installed route chunks with occupied squares and one-way terrain respected.

Crystal’s Route 42 places Fisher Tully, Hiker Benjamin and Pokémaniac Shane on the same road with different teams and different sight ranges. Polished Crystal keeps these three characters and their sight ranges, keeps Qwilfish with Tully, and gives Benjamin a Dunsparce. That informed the decision to keep the people on Route 7 and change what three of them send out. [Crystal Route 42](https://github.com/pret/pokecrystal/blob/master/maps/Route42.asm#L325-L327) · [Polished Crystal Route 42](https://github.com/Rangi42/polishedcrystal/blob/master/maps/Route42.asm#L32-L34) · [Crystal parties](https://github.com/pret/pokecrystal/blob/master/data/trainers/parties.asm) · [Polished Crystal parties](https://github.com/Rangi42/polishedcrystal/blob/master/data/trainers/parties.asm)

## Verification

The [regional trainer survey](survey-trainers.py) checks the trainer records, victory flags, opening and closing scripts for all 303 ordinary trainer actors in each edition. It counts 125 trainers on the twenty numbered routes and 18 students across the eight gyms. All the students have positive sight ranges. In the gym rooms, only a leader’s conversation can load a leader battle. The parent editions had one pair of trainers on the same numbered route with matching species, levels, held items and moves when order and trainer-class DVs were ignored. In both revised editions, there are zero such pairs.

The [archive check](check-pack.py) verifies exact move order and PP, active species, local acquisition, actual trainer sight squares and the two crossings. It compares every other party, each map and map registry, encounters, scripts and graphics with its parent. The two editions have separate identities and keep their previous saves intact.

In both editions, the native controller approaches Nell, Silas and Wren on foot. Each challenges the player before they press A. The native battle screen shows Velvawn after Riceling faints, Jiglet on Silas’s first turn, and Dartduck on Wren’s first turn. Wren awards ₽600 once. Talking to her again opens her line about putting down the puck, and the player can walk on. This sight and artwork check supplies a level-48 Borealynx. The [combat comparison](battle-costs.md) uses separate starter-led parties raised to the Brainerd milestone.

At South Lake the camera followed Kris along the stones in both editions. The canoe stayed packed while she walked and turned. Stepping into the lake launched it, and stepping back onto the bank restored her walking sprite. All 119 frames with the canoe and all 110 frames without it matched the Trail Signs captures. The [footing check](../fen-waterline-v01/check-footing.py) also found zero mismatches at 85 bank-to-water contacts and along 787 walking or paddling strides in each edition.

The 39 sampled gym and League fights produced identical results, turn counts and surviving party counts in the two Intro revisions. Both new archives passed the cave approach and story milestone checks.

Use absolute paths for the applicable parent and a fresh external output.

```sh
export CARGO_INCREMENTAL=0 RUST_MIN_STACK=67108864
cargo run --locked -j1 -p crystal-assets --example pack_ely_challenge -- --leech-lake-trainers "$parent" "$child"
uv run --offline --no-project --with cbor2 --with pillow python planning/minnesota/revisions/leech-lake-trainers-v01/check-pack.py "$parent" "$child"
uv run --offline --no-project --with cbor2 --with pillow python planning/minnesota/revisions/leech-lake-trainers-v01/survey-trainers.py "$parent" "$child"
MN_BADGE_TRAILS_PACK="$child" MN_LEECH_PARTNER_FRAMES="$frames" cargo test --locked -j1 -p crystal-bevy --lib leech_lake_trainer_partners_see_the_walker_and_send_their_teams -- --ignored --test-threads=1
```

## Shiner Narrows follow-up

On Route 15’s Shiner Narrows west bank (`MnR15C00R00`), the terminal’s text and painted views point east from (24,10) through (24,17) and southeast from (24,18). Each eastward jump crosses a wall at x=25 and would land in water at x=26, so a walker cannot make it. The three southward drops from (22,18), (23,18) and (24,18) do land on the meadow at y=20. Both terminal views use the same terrain glyph. It can check each indicated direction against the walking permission two squares away, preserving valid map-edge connections. At this bank that would remove eight arrows and turn the corner arrow south while preserving the native artwork and all walking and canoe routes. Both Trail Signs and Leech Lake Trainers editions have these permissions. [Bank survey](../trail-signs-v01/terrain-review.md) · [Shared terrain glyph](../../../../crates/crystal-tui-view/src/snapshot.rs) · [Native landing rule](../../../../crates/crystal-core/src/world/movement.rs)

The [Quarry Landings review](../quarry-landings-v01/terminal-ledges.md) implemented the landing check and verified the corrected cues in both terminal views.
