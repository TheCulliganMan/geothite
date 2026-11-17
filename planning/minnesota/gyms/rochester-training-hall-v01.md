# Rochester training hall review candidate

This is an **unselected candidate**, not the installed regional gym. It replaces
the rejected planter maze with a timber practice hall: the entrance faces two
practice stations and a railed leader court. Tavel stands at `(4,15)`, Elvik
at `(9,9)`, Varel at `(6,3)`, and the guide at `(10,17)`. There is open space
to approach each trainer and an ordinary return. It intentionally does not
claim a mechanical puzzle: the challenge is the two ordered practice battles.
The room is 7×10 metatiles, with the existing passive arrival `(6,17)` and
both exit carpets `(6,19)` and `(7,19)` unchanged. The layout and native tile
recipes are in [the source](rochester-training-hall-v01.json).

Compare the [complete source room](../renders/gyms/rochester-training-hall-v03/MnRochesterGym-whole.png),
the [native entrance](../renders/gyms/rochester-training-hall-published-v01/native-entry.png),
and [native leader court](../renders/gyms/rochester-training-hall-published-v01/native-court.png)
with the [Crystal and installed-room comparison](crystal-comparison-2026-11-05.md).
The native images use explicit location fixtures at noon, not an earned save.
The court is on the same floor; the railing does not represent elevation.
The earlier source studies and their renders are retained.

The Rust builder accepts the selected `r06_pine_fork_revision` parent by
content identity. It retains the actual city warp records, trainer teams and
progression scripts, replaces only the room's private art and blocks, moves the
four existing actors, and updates three room-specific text bodies. The source
contains the current trainer species for review; the publisher retains the
parent's actual trainer records instead of reconstructing parties. The builder
runs the existing gym collision and walkthrough validator. The additional
existing Rochester progress repair is a separate second stage, because the
selected parent still sends early Elvik interactions to the parcel text and
uses a derived court flag for Varel's check.

```sh
CARGO_INCREMENTAL=0 cargo build --locked -p crystal-assets --bin pack_rochester_training_hall
target/debug/pack_rochester_training_hall \
  content-packs/minnesota-reviewed-maps-v11.crystalpack \
  planning/minnesota/gyms/rochester-training-hall-v01.json \
  content-packs/minnesota-rochester-training-hall-review-v02.crystalpack
target/debug/repair_rochester_gym_progress \
  content-packs/minnesota-rochester-training-hall-review-v02.crystalpack \
  content-packs/minnesota-rochester-training-hall-progress-v01.crystalpack
```

The hall-only review has content identity
`3ccd591f825c4303cfc49a46fce84861bca6c49dd476795fb39515f97ff311ce`;
the repaired child has identity
`1cd76d358609d3eba0ad549db3cf2c9142c60f5ef1b55473a67974a3fb819963`.
The hall-only pack's complete terrain pixels and collision quadrants match the
source. A parent comparison found only `MnRochesterGym` and `mn_gym_1` changed
in the map and tileset catalogs, plus their corresponding compiled indexes and
five private tileset files. Trainer data, all other maps and the audio manifest
match. All 602 hall-only audio programs pass PCM hash, frame-count and loop
checks.

The production controller test
`rochester_training_hall_controller_trainers_badge_and_returns` passed against
the repaired child: it checks early refusal, both trainer wins, the leader's
intermediate refusal, leader victory, one TM, both city carpets, re-entry and
Continue. Its initial setup is an explicit level-100 Mew and completed parcel;
it does not prove earned progression, ordinary-party balance, loss/retry, or
compatibility with existing in-room saves. Inputs use production movement and
battle handling. The selected `world.json` pointer and hosted game remain
unchanged. The art and course still need broader review, and the other seven
Gyms remain to be improved. A release must explicitly handle old in-room
positions and saved block overrides before selecting a replacement.
