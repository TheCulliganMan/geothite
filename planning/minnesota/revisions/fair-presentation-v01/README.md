# Fair Patrols presentation update

This edition combines the reviewed dialogue and native full-cast trainer
portraits with the Fair Patrols release. The dialogue revision affects 318
maps. All 746 compiled map layouts keep their collision, entrances and exits.
The roster, encounter tables and 602 audio programs match Fair Patrols.

Both editions accept 22 previous save identities. An eligible Fair save copies
to the new slot at the same location. On Gunflint Cave 1F, four former island
tiles now restore on the west bank. Route 15 restores a checkpoint from behind
the removed Cut tree to the clearing on its approach side. The copy preserves
each original save and backup. Across 672 affected Lakeside walking squares
in each edition, 656 restored. The other 16 were fully enclosed in the old
layout. Six generated Fair saves also copied at their original coordinates.
Verification used compiled maps and isolated save files.

Build both editions with the exact raw and hosted Fair packs and the [verified
full-cast v03 art pack](../../npcs/portrait-cast-v01/README.md).

```sh
cargo build --locked -j1 -p crystal-assets --example pack_northstar_fair_presentation \
  --bin verify_crystalpack
mkdir -p target/fair-presentation
target/debug/examples/pack_northstar_fair_presentation \
  content-packs/minnesota-fair-release-v01.crystalpack \
  target/fair-release-20261120/hosted.crystalpack \
  content-packs/minnesota-trainer-full-cast-v03.crystalpack \
  planning/minnesota/story.json \
  content-packs/minnesota-fair-presentation-v01.crystalpack \
  target/fair-presentation/hosted.crystalpack
target/debug/verify_crystalpack content-packs/minnesota-fair-presentation-v01.crystalpack --desktop --audio
target/debug/verify_crystalpack target/fair-presentation/hosted.crystalpack --desktop --audio
```
