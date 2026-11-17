# Mankato ground terraces — combined review candidate

This **unselected** candidate adapts the earlier
[sand-terrace design](mankato-terraces.md) to the later R06 regional lineage,
after the Rochester training hall and its separate progress repair. It
replaces the selected room's wooden floor with sandy ledges and a distinct
darker lower arena. Wooden gates mark the two existing locked approaches;
native stairs stay visible beyond them. The three one-way drops and their
ordinary return routes retain the existing coordinates and rules. The real
trainer and leader positions, badge/TM scripts and city doors are unchanged.

Inspect the [complete room](../renders/gyms/mankato-ground-terraces-v01/MnMankatoGym-whole.png),
[lower gate open](../renders/gyms/mankato-ground-terraces-v01-lower-open/MnMankatoGym-whole.png),
[both gates open](../renders/gyms/mankato-ground-terraces-v01-court-open/MnMankatoGym-whole.png),
[native entrance](../renders/gyms/mankato-ground-terraces-published-v01/native-entry.png)
and [native leader court](../renders/gyms/mankato-ground-terraces-published-v01/native-upper.png).
Source event views and native camera positions are explicit fixtures, not an
earned campaign. The whole source is
[mankato-ground-terraces-v01.json](mankato-ground-terraces-v01.json).

The guarded Rust publisher checks the parent identity, room and actor/warp
contracts, and the preexisting gate commands. It installs the private native
art, retargets the four `changeblock` operands to the new gate art, and retains
all other installed scripts. The guide's revised explanation retains the
regional Cut directions. Source team examples match the installed Minnesota
parties; publication preserves the parent's actual trainer records.

```sh
CARGO_INCREMENTAL=0 cargo build --locked -p crystal-assets --bin pack_mankato_ground_terraces
target/debug/pack_mankato_ground_terraces \
  content-packs/minnesota-rochester-training-hall-progress-v01.crystalpack \
  planning/minnesota/gyms/mankato-ground-terraces-v01.json \
  content-packs/minnesota-two-gyms-review-v01.crystalpack
```

A rebuild with the final source guard was byte-identical to the tested pack;
a deliberately altered first-gate prerequisite was rejected before writing output.
The review pack identity is
`d74cd46e8cbabd2c8f011b7782585ff092955cfd88c46664a48c28184518c988`.
The exact native terrain pixels and collision quadrants match the source. A
comparison against the immediate parent finds only `MnMankatoGym` and its
`mn_gym_2` private tileset changed, together with the relevant compiled
map/script indexes and five private art files. All other maps, all trainer
records, the actual Mankato warp records and the audio manifest match. All
602 audio programs pass their PCM hash, frame-count and loop checks.

The literal production-controller course passed with the existing explicit
Bayleef 19 / Pidgey 17 party and prior Care Badge fixture. It earned both
trainers and the leader, performed the three southward hops and reverse-denial
checks, retreated before Jorel, continued, and used both actual city carpets.
Three additional production-controller cases passed the first-badge refusal,
loss/recovery with the ascent closed, and full puzzle/return persistence;
they use strong or losing explicit fixtures. The Rochester training-hall controller test also passed on the combined
candidate. These local setups do not prove
an ordinary Minnesota-party balance or a continuously earned regional journey.

No selected-world pointer, hosted pack or user save changed. Saved positions
and map block overrides inside revised gyms require explicit migration before
selection. Rochester's design remains under review, and the other six Gyms
remain to be improved and tested.
