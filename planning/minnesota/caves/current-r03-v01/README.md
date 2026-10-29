# R03 bank cave: fitted rock and dry walker approach

This revision improves the complete 8×6-metatile bank cave on the selected
704-map story build. Solid native rock replaces the black void, and a small
enclosed seep meets the west bank. The passage turns north around an attached
granite nose before doubling south into the original Great Ball alcove. This
is a short optional discovery, not another large dungeon or a regional shortcut.

The authoritative [native contract](native.contract.json) binds the exact story
parent, both room-source hashes and the actual exterior pairing. Rust publishes
the ignored candidate `content-packs/minnesota-r03-bank-seep-v02.crystalpack`.
Only29cave blocks, border01→2e, native block/attribute mirrors and identity
change. All703other complete maps, story programs, roster, encounters, original
resources, art and audio remain exact to the704-map story parent.

## Playable contract

All coordinates below are16px gameplay tiles, except the named seep metatile.

| Feature | Native position and behavior |
| --- | --- |
| Exterior entrance | MnR03C00R00(8,37), CAVE, active warp1 |
| Safe exterior return | (8,38), FLOOR, passive warp2; waiting/revisiting cannot reenter |
| Interior mouth | (7,9), LADDER, warp1; clear arrival (7,8) |
| Walker | (5,5), original three-page `jumptextfaceplayer`; dry approach (5,6) |
| Seep | Metatile(1,3), native3e; four enclosed WATER cells, no open map edge |
| Bend | Floor course (6,5)→(6,3)→(7,3)→(8,3); native UP_WALL(8,2)/(9,2) blocks further north |
| Granite nose | WALL(8,5) and WALL(9,8); reward approach must go around its top |
| Reward | Original GREAT_BALL×1 at(12,5), approach(11,5), `EVENT_MN_R03_ITEM_02` |
| Return | Ordinary dry route around the same nose to(7,8), then the original mouth |

No new story condition, battle, trainer party, Cut gate, staircase, healing
service or random encounter is added. Original dialogue/pickup presentation
owns input. The collected body disappears, its floor becomes walkable, and
Save/Continue retains the quantity and event without granting another ball.

## Render history and fast iteration

- [Whole-room before/after](../../renders/r03-current-bank-v01/published-v02/whole-before-after.png)
- [Six native player cameras](../../renders/r03-current-bank-v01/native-v02/MnR03Cave1F-production-cameras.png)
- [Exact preservation audit](../../renders/r03-current-bank-v01/published-v02/receipt.json)
- [All nine installed caves at one native scale](../../renders/r03-current-bank-v01/all-nine-whole-v03/all-nine-public-caves.png)
- [Final combined proof](../../renders/r03-current-bank-v01/verified-v02/receipt.json)

`source-v00` freezes the original cave. `source-v01` is the rejected first
seep: it occupied the original walker approach(5,6). Its publication, failed
controller logs and native renders remain retained. `source-v02` moves that
seep one metatile west and preserves the dry approach; this is the canonical
source and v02pack. `source-v03` retains the unused south-bank alternative.
Do not publish that alternative by filename order.

The generic read-only audit reuses installed native art and checks actor-aware
ordinary floor, enclosed water, actual mouth pairs and complete normalized pack
equality. The measured warm room render/check is0.120seconds after pack load.
Source changes remain independent from compilation and native publication.
Every output epoch is fresh; keep originals, phase comparisons and rejected
renders. See [PUBLICATION.md](PUBLICATION.md) for the executed commands.

The original R03 production test passes on v02, including all three fully
printed walker pages, native roof directionality, quantity/event/input locks,
passive return, reentry, collected floor and Continue. Six complete640×576
native frames compare with zero differences. All377audio PCM hashes, frame
counts and loop ranges pass. A fresh Rust replay produces byte-identical output.

The complete nine-cave geometry test also needs the story's exact R09 overlay:
its lower shaft is a sealed2dblock until the foreman is defeated, then the
native callback opens a ladder. Its two added lair records are not outdoor
mouths. The compatibility check must certify that exact source/native delta,
prove the closed entrance and passive arrival, and continue to test every
original cave mouth. Never restore the old open floor to make a test pass.

`world.json` selects `r03_bank_seep_revision`, retaining every prior sibling
record, including the current R06audit. The final combined course passes on
this exact704-map pack: all nine caves,68reached camera checkpoints, stairs,
short hops/reverse blocks/ordinary returns, pickups, Continue and every
original outdoor mouth. The mine's closed story-shaft and passive arrival are
also checked. The original R03interaction case passes again on the final test
build, and two actual stdio processes pass with byte-empty stderr. The separate
earned New Game opening passes through Reed, delivery and real egg refusal/
retry/Continue. Ely-only snowfall also passes its existing presentation test.

The source and final reached native views were inspected. The accepted
[outdoor atlas](../../renders/r03-current-bank-v01/atlas-v02/full-state-native-current.png)
retains97maps in one seam component, with zero overlaps or placement cycles.
This revision does not complete the broader regional map goal, passenger
trains, absent hubs or an uninterrupted earned campaign.
