# Reading the trail signs

The Detroit Lakes approach, the preserve and Ely already had printed signs. The directions were attached to the ground beside them. At Ely the north sign explains League registration and changes its advice as the player earns badges, restores the survey and becomes Champion. The second sign, near the clinic, tells visitors where to prepare. Each message now opens when the player faces its printed board.

| Place | Where to look | What it tells the player |
| --- | --- | --- |
| Detroit Lakes Woodland Approach | Board on the pine trail, (8,12) | Follow the pines toward Detroit Lakes. |
| Preserve Pine Entry | Board against the trees, (12,10) | The shelter trail is free. Paid Safari check-in is at basecamp. |
| Ely | Upper trail board, (21,3) | Whether the League is open, where to register and what to expect on a return visit. |
| Ely | Clinic-side board, (15,21) | Where to heal, shop and register. |
| Minneapolis | Board west of the visitor center door, (40,58) | Identifies the South Arrival Desk from the southern approach. |
| Route 8 | Plaque on the pine at (22,6), read from its north side | Cedar Inlet offers a bridge or a shore path. |
| Minneapolis | Placard on the brick wall at (55,51), left of the door | Identifies the Game Corner and advertises its games and prizes. |

The South Arrival Desk formerly had its name on bare pavement at (40,60). The board sits two squares north of that spot, leaving the walking lane and the doorstep clear. The Cedar Inlet and Game Corner messages keep their original positions. Their new artwork occupies part of the pine and the brick wall.

Cherrygrove in Crystal and Polished Crystal gives the town, the guide’s house and the trainer tip their own reading spots. The guide’s house sign sits two squares from the door in Polished Crystal. Minneapolis likewise keeps the Arrival Desk’s road sign separate from its entrance, and the Game Corner plaque belongs to the facade it identifies. [Crystal town events](https://github.com/pret/pokecrystal/blob/master/maps/CherrygroveCity.asm) · [Polished Crystal town events](https://github.com/Rangi42/polishedcrystal/blob/master/maps/CherrygroveCity.asm#L8-L22)

## Review and reproduction

The [outdoor reading survey](audit-outdoor-reads.py) groups actual 16×16 player-facing surfaces from the region’s towns and routes by pixel pattern. The Research Choices Intro edition had 330 reading events and Field had 329. Each used 30 distinct patterns. Grouping exposed five readings on bare ground and two directions drawn as ordinary scenery. Some other readings belong on unusual surfaces, such as a fossil, an eddy or the split hoofprint on the Tall Tale Trail. The inventory is limited to ordinary background reads. Trainers, item searches and coordinate-triggered scenes use different interactions.

The [archive and approach checks](check-pack.py) use the exact Research Choices [Field](field-build.json) and [Intro](intro-build.json) parents. The controller checks walk across the old reading positions, face each board, read the message and follow the nearby clinic, League lobby, Arrival Desk and Game Corner doors. The Safari check saves on an open page, continues, and checks that directions never charge a fee. Each controller starts from an explicit local fixture.

The cyclist Sam patrols the Game Corner sidewalk. In the graphical first-visit check, the player reads the plaque and steps east in front of the door. Sam spots the player from two squares away, rides over and starts his battle. A second visitor carries the same Kindlit and has already defeated Sam. That visitor reads the plaque, passes him, enters the arcade and comes back onto the same sidewalk.

The finished editions retain all 329 Field and 330 Intro reading points, now on 29 distinct surfaces. The seven boards each have two checked foot routes. The pine plaque changes 51 pixels and the Game Corner plaque changes 61. Every older metatile in both affected tilesets renders identically to its parent in the morning, day and night palettes. The regional progression check also found paths through all eight badges and the Champion in both editions.

At South Lake, the eight-square walk on the pebble bank produced 119 consecutive frames with a canoe and 110 without. Both sequences matched the preceding Intro edition pixel for pixel. The equipped walker turns on the stones, launches into the narrows, lands and walks east. The unequipped walker can face the lake and continue along the same bank. All four grassy approaches passed between-tick camera checks, and Chris at noon and Kris at night each completed the two-lake portage. The daytime [footing check](../fen-waterline-v01/check-footing.py) found zero mismatches across 85 bank-to-water contacts and 787 walking or paddling strides in each edition. A separate [Route 15 west bank review](terrain-review.md) traces its high shore and the three usable drops into the meadow.

One battle-design finding remains open on Route 7. Five of its eight trainers carry a level-30 Chorip with Disarming Voice, Water Gun, Psybeam and Sludge. Dean and Wren field the same pair of Pokémon in reverse order. Both editions reproduce the overlap.

Use absolute paths for the appropriate parent, a fresh child output and a capture directory.

```sh
export CARGO_INCREMENTAL=0 RUST_MIN_STACK=67108864
cargo run --locked -j1 -p crystal-assets --example pack_ely_challenge -- --trail-signs "$parent" "$child"
uv run --offline --no-project --with cbor2 --with pillow python planning/minnesota/revisions/trail-signs-v01/check-pack.py "$parent" "$child"
export MN_BADGE_TRAILS_PACK="$child" CRYSTAL_INTERIOR_PACK="$child" MN_DIRECTION_BOARD_FRAMES="$frames"
cargo test --locked -j1 -p crystal-bevy --lib -- --ignored --test-threads=1 regional_direction_boards_native_read_walk_and_follow_the_sign ely_sign_and_registrar_explain_the_actual_run_and_survey ely_town_controller_public_signs_resident_four_road_lanes_and_continue safari_public_boards_keep_trails_free_and_continue_the_actual_page
```

In a working tree containing both editions, set `CRYSTAL_NORTHSTAR_CANONICAL_STORY_SOURCE` to the authored story matching the tested edition. The external packs keep their own save identities.
