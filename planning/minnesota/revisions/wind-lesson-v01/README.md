# Saven’s Wind Lesson

Saven teaches one response to her Bankwing in the same three pages she already speaks before the first Wind Badge battle.

| Page | First line | Second line |
| --- | --- | --- |
| 1 | SAVEN: Feel that? | The wind turned. |
| 2 | If BANKWING digs, | send in a flier. |
| 3 | Let me see how | quickly you adapt! |

At the open court in St. Cloud Gym, speaking to Saven shows the greeting and waits for confirmation before starting the fight. A challenger returning after a blackout hears it again. Saven’s ordinary trainer battle sends a defeated challenger to the healing center, skipping the registered loss speech. Her guide already spends five pages explaining the two fans and Hallen’s optional catwalk. Crystal’s Violet Gym likewise teaches a type matchup at the gym where it matters. [Violet Gym guide](https://github.com/pret/pokecrystal/blob/master/maps/VioletGym.asm#L243-L261)

Bankwing announces its first turn of Dig by digging a hole. Switching a healthy Flying-type partner into the following Ground attack avoids the hit. There is a local option for each starter. Among daytime grass encounters on Route 5, Flossail has a 30% slot in each of the north and middle sections at levels 22 and 21 respectively. Flossail is already Poison/Flying when caught.

The active source is the middle page of `dialogue_overhaul.texts.MnStCloudGym.MnStCloudGymLeaderSeenText` in [story.json](../../story.json). The earlier Gym Overhaul source retains its historical greeting. Quarry Landings builds independently from its two Leech Lake Trainers parents. The Wind Lesson builder takes either exact Quarry Landings archive and publishes the new page in the map script, global script and indexed text body.

## Verification

Run the comparison with Python, cbor2 and Pillow from the repository root.

```sh
python3 planning/minnesota/revisions/wind-lesson-v01/check-pack.py \
  content-packs/minnesota-quarry-landings-v01.crystalpack \
  content-packs/minnesota-wind-lesson-v01.crystalpack \
  content-packs/minnesota-intro-quarry-landings-v01.crystalpack \
  content-packs/minnesota-intro-wind-lesson-v01.crystalpack
```

Both written editions passed. The checker verified the complete archive delta, the greeting’s place in the leader script, the first trainer battle, the Dig matchup and the Route 5 encounters. Saven retains her three Pokémon, their original moves, items, levels and AI. Her victory, rewards and rematch scripts stay intact.

The native court check reads each displayed page before the battle. In the battle trials, all 12 Flying partners switched into the announced Dig took zero damage from it. A player can catch Flossail before reaching Saven and use that same answer. [Court and dialogue test](../../../../crates/crystal-bevy/src/bevy_shell/tests/saven_court.rs) · [Battle results](../badge-battles-v01/st-cloud.md)

The working checkout selects Intro and the shared registry selects Field. Each archive has its own save identity. Read-only loads verified that the Quarry Landings and Wind Lesson fixtures each resume on their matching archive. The review leaves existing saves in their original slots. [Campaign coverage and remaining judgments](campaign-review.md)
