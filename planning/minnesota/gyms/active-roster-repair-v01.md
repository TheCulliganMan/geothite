# Eight-gym review: active-roster repair

The first combined review pack still contained eleven slots from retired
families in Mankato, St. Cloud and Brainerd. The explicit replacements in
`active-roster-repair-v01.json` keep each slot's level, item and DVs. They use
active Minnesota species and level-up moves legal at that level. Mankato uses
Castorock, Burrbud and Sodcalf; St. Cloud uses Vanechick and its evolved
Silohawk; Brainerd uses Pikeman, Paddarray and Pinglet. These are deliberate
team changes, and their ordinary-party balance needs review.

The guarded Rust publisher reads the canonical roster and curation, rejects
retired replacements, illegal level-up moves and overdue level evolutions, and
checks every `MN_GYM_` trainer after repair. The ignored child pack is
`content-packs/minnesota-eight-gyms-active-roster-review-v01.crystalpack`,
identity `ba3b067a92192dad36d9b8884296dc036653d2acf3e3eb22325cef8b3ee0fd73`.
Compared with its parent, all maps, species, items, event constants and audio
are unchanged. Only the declared species and moves in seven trainer records
change; their other fields remain exact. All 602 PCM hashes, frame counts and
loop ranges pass. The initial-state atlas in
`../renders/gyms/eight-gym-active-roster-review-v01/installed-all.png` renders
the actual compiled maps and actors, without simulating gate states.
The repair rejected deliberately retired, illegal-move, stale-slot and
overdue-evolution inputs before writing output.
Every `loadtrainer` in the eight compiled gym rooms names one of the audited
`MN_GYM_` trainer records.

On this combined pack, fixture-based controller courses passed for all eight
gyms. Additional cases covered Mankato and St. Cloud losses and badge gates,
Brainerd's canoe/Surf barrier, and Duluth's new ice turn and retreat. These
start from explicit local fixtures and do not prove a continuous campaign.

```sh
cargo run --locked -p crystal-assets --bin repair_gym_active_roster -- \
  content-packs/minnesota-eight-gyms-review-v01.crystalpack \
  planning/minnesota/gyms/active-roster-repair-v01.json \
  planning/minnesota/pokemon/roster.json \
  planning/minnesota/pokemon/roster-curation.json \
  content-packs/minnesota-eight-gyms-active-roster-review-v01.crystalpack
```

This is a review child, not the selected world or a deployment. The strong
fixture-based controller replays do not establish a continuously earned
campaign or representative Minnesota-party balance. Automatic client save
migration, remaining regional integration and user visual review are still open.

The selected local parent is `minnesota-reviewed-maps-v11.crystalpack`,
content identity `834d4be07c45079fdc81f1777856a37058d064b4188f548e31616684436da5e3`.
A direct parent/child audit found reused block IDs with different collision in
every gym: Rochester 13, Mankato 6, Minneapolis 2, St. Paul 8, St. Cloud 3,
Brainerd 19, Bemidji 9 and Duluth 8. Minneapolis changes from 5×9 to 7×11
metatiles and St. Paul from 7×10 to 9×12; actors also move in Rochester,
Minneapolis and St. Paul. Bounds validation alone therefore cannot establish
that an old saved override or actor position is safe. No user save conversion or
selection has been performed.

The Rust `migrate_gym_review_save` command now converts a verified primary save
from that exact parent to a separate slot for either this review pack or the
canoe-dialogue successor. It validates both packs and identities, rejects
pending scenes and unrecognized overrides, rebuilds gym gates through their
callbacks, remaps known actors and handles the canoe shop's changed tiles and
display. It reads the original primary directly, leaving both it and its backup
untouched; a corrupt primary is rejected even if a backup exists. The destination
and its backup must be absent. A complete validated save is staged on the same
filesystem and published with a no-overwrite hard link. Filesystems without
hard-link support return an error. The destination begins with one primary copy;
the ordinary game save operation subsequently creates its normal backup.

For a local review, build the command and run it with the exact pack paths and
separate existing-source/new-destination save paths:

```sh
cargo build --locked -p crystal-runtime --bin migrate_gym_review_save
target/debug/migrate_gym_review_save \
  content-packs/minnesota-reviewed-maps-v11.crystalpack \
  content-packs/minnesota-eight-gyms-canoe-dialogue-review-v01.crystalpack \
  /absolute/path/to/original.crystalsave /absolute/path/to/new-review.crystalsave
```

The process-level test invokes that executable against real packs and a local
Rochester fixture, verifies that the target runtime resumes the new save, and
rejects a second write. Additional tests cover the eight rooms, shop collision,
saved actor state, unknown changes, occupied backup and damaged primary. A
separate controller test plays a trainer battle before saving and continues to
finish Rochester with an explicitly strong test party. These checks do not
integrate migration into browser, TUI or desktop save selection, and they do not
prove a full earned campaign or normal-party balance.

An additional fresh New Game on the canoe-dialogue successor now reaches and
completes the redesigned Rochester Gym with an earned party. The driver chose
the installed starter, played the opening and delivery, defeated Route 1 and
Gym trainers, caught a second partner, trained through ordinary encounters,
defeated Reed and Varel, and saved and continued after the Care Badge. It set no
level boost or progression flags; its starter finished at level 13. It used
ordinary bought Potions and three in the leader battle. One earlier attempt lost
to the second Gym trainer, and another blacked out during training. The passing
checkpoint is retained under the ignored `target/gym-earned-canoe-review-v01/`
directory for continued review. This is one party and one run, not a balance
guarantee or earned evidence for the other seven Gyms.

Continuing that same saved party through the R02 trainers and the Mankato
notebook reached the redesigned second Gym. It defeated both Gym trainers;
the lead ended the second fight at 1 HP and recovered at the real clinic.
Four bounded attempts at Torven then lost. A separate replay of the actual
post-trainer save showed a level-16 Rimepelt and level-12 Nibbin with no Potions
or Super Potions against Castorock, Burrbud and Sodcalf. The battle driver used
attacks and did not attempt Nibbin's Sand-Attack or recruit a better matchup.
This is a reproducible concern for that party and resource state, not proof that
the leader is unbeatable or that changing its team would be the right repair.
The real post-trainer checkpoint is retained in the same ignored review
directory for strategy, recruitment and balance follow-up.

A further replay from that same checkpoint led with Nibbin, used two
Sand-Attacks through the ordinary battle menu, then switched to Rimepelt. On
the third attempt it defeated all three of Torven's Pokémon without Potions,
earned the Prairie Badge and saved after leaving the Gym. The party and items
were unchanged at the start of the replay; ordinary battle RNG varied. This is
an earned example of a viable strategy, not a general balance guarantee.
The continuation toward Minneapolis then stopped because the existing
Cut-partner test assumes R02 has a daytime Cut learner. This pack's actual
R02 table has none in daytime; the earlier R01 table contains Pinit during the
day and Dewdant at night. A continuation from the actual Prairie victory save
returned through the connecting routes, caught a Cut-compatible partner on
R01 with an owned Ball, returned to Mankato, obtained the Cut HM, and reached
Minneapolis. It also earned the Good Rod and defeated actual R03 trainers. The
same-pack Minneapolis arrival and its backup are retained in the ignored review
directory.

From that arrival, the same unboosted party trained on R03 to level 23, then
completed the ICE headquarters, defeated Reed and Voss, freed the dispatcher
and returned him home. The lead reached level 28 through those battles. The
actual headquarters scene checkpoints and dispatcher-home save remain in the
ignored review directory. Continuing from that dispatcher-home save, the same
party defeated both trainers, operated the electrical station's real switches,
beat Avrin and saved outside the third Gym with the Current Badge. Its lead was
level 28–29 after the intervening training and story fights; this establishes
the route and victory for this party, not a representative balance range. A
continuation from that real save handled the R04 trainer's input-owned approach,
won the battle and reached St. Paul; both the post-Gym and St. Paul arrival saves
are retained.

The same party completed the courier and manifest handoff, crossed St. Paul's
reviewed Gym pads, defeated its two trainers and Brannic, and earned the
Capitol Badge at level 31. It visited the canoe shop through ordinary input,
received the canoe and Surf HM, and returned to Minneapolis with its four
badges and evidence intact. Continuing from that save, it crossed R05, played
the St. Cloud worker scene, and won the reviewed airworks Gym and Wind Badge at
level 33, consuming four carried Super Potions against Saven. These are real
continuations of the same party, though its levels reflect extensive optional
training and the intervening story battles. The post-Wind save is retained.
The onward replay resumed from the post-Wind save, defeated both R06 trainers,
picked up the Super Rod and reached Brainerd at level 33. The arrival save
retains the same five badges and story evidence. The party used the native
Brainerd clinic, shifted the pier with the physical winch and defeated both
dock trainers through controller input. The second post-battle save supports
replaying the leader approach separately. The first earned Sivren attempt
reached Paddarray after using all five carried Super Potions and ended in a
native blackout. The recovery and subsequent R06 training saves are retained
for continued balance verification. After 27 training outings, the same party
used the native clinic and beat Sivren with level 31 Pitchwick, level 31
Cacheek, level 40 Borealynx and level 30 Choirbog. It earned the Lake Badge and
one Rain Dance TM and walked back out to Brainerd. The win required all five
carried Super Potions. This is one earned route through the gym and involved
substantial optional training. Continuing from the Gym return, the party
acknowledged the ranger's installed report, saved both report flags and
confirmed the existing Surf HM remained at one copy after speaking again.
