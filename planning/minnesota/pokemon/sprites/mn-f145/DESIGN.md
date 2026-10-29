# Verdigrim — Copper Howl Pokémon

**Steel/Poison · legendary · MN_319 / MN_F145 · Minnesota native 2D.**

Verdigrim is an imposing wolf formed from an ancient mass of native float copper.
Rounded copper lobes merge into huge shoulders, a deep chest, nugget paws and a
heavy tail. Green weathering crust covers broad surfaces; smooth reddish copper
is exposed on the forehead, shoulders, knuckles and tail. Tall ears and a strong
canine muzzle make the wolf readable at native size.

The material reference is glacial float copper. The University of Minnesota's
[mineral guide, page 35](https://conservancy.umn.edu/bitstreams/7452b0d6-8260-4e78-9b7a-a22cf0b0070a/download)
records rare native copper in Minnesota glacial drift and its rose/copper-red,
brown and green weathering colors. The giant living wolf and its powers are
original game fiction. A specimen photograph was inspected as a material-only
reference; its URL and hash are recorded in the import manifest, and its local
copy remains in the ignored render archive.

Minnesota timber wolves and Ojibwe wolf-as-companion teachings inform its
character. The [1854 Treaty Authority](https://www.1854treatyauthority.org/management/biological-resources/wildlife/wolf-research)
documents Ma’iingan and northeastern Minnesota wolves; its
[2016 climate plan, section 4.11.3](https://www.1854treatyauthority.org/images/ClimateAdaptationPlan_Final-July_2016-optimized%281%29.pdf)
describes the companion relationship. Verdigrim has its own fictional name.
The copper coat, poison, Team Lode excavation and Kettlurk rivalry are invented
for the game, rather than attributed to an Ojibwe story.

Team Lode strips glacial overburden from its worksite. The buried copper mass
opens its eyes and howls; a huge forepaw crushes the test line. Poison leaks
from its coat and turns the local water green. Verdigrim braces its paws and
tail across the channel that Kettlurk uses to return diverted water.

Stats are **110 / 120 / 130 / 60 / 80 / 100**, total **600**, in
HP/Attack/Defense/Speed/Special Attack/Special Defense order. It is genderless,
cannot breed and has no evolution. Slow growth and catch rate 3 support its
legendary role. Normal native inks are white, copper, deep green and black;
the shiny uses pale oxidized metal and violet seams.

Crunch at level 40 threatens Kettlurk through its Ghost weakness. Poison Jab at 44
provides reliable physical STAB; Iron Tail at 50 and Sludge Bomb at 57 complete the
proposed level 60 encounter. Steel/Poison does not confer Water or Ghost immunity.
Ground and Fire retain ordinary counterplay.

[verdigrim-story.json](../../verdigrim-story.json) authors the reveal and
containment sequence: the excavation exposes Verdigrim, green water spreads,
Kettlurk's return stalls, and the player stops the drill, closes the contaminated
outlet and opens a clean bypass. Crew clearance finishes the scene; capture is
optional. These map scripts and water-state transitions are not installed in the
playable story yet. Exact placement requires the selected world geometry.

The selected source is [front-back.imagegen-v4.png](front-back.imagegen-v4.png),
created with built-in ImageGen using [the exact prompt](prompt-v4.txt).
V1's dragon-like creature, v2's feline and v3's pointed copper-fur wolf remain
saved with their prompts. V4 follows the user's float-copper direction.
The reviewed 56×56 front and 48×48 close rear preserve the ears, muzzle, huge
forepaws, rounded mineral masses and green/copper distribution with four inks.
Native variants and the original source are retained by the Rust converter's
immutable render archive. Only the v4 manifest is registered.

The [six-pose overworld sprites](../../overworld/20261027-v1/sources/mn_f145/README.md)
are accepted in normal and shiny form, with source-backed pupils and eight
inspected production terrain views. Use `SPRITE_VERDIGRIM`.

Its original score in `cry-designs.json` uses a rising wolf howl, an unequal
low companion, brief throat rasp and a closing growl. Runtime `CRY_MN_319` is
installed and verified in the separate asset review pack. It lasts 1.524 seconds,
with 33,596 stereo PCM frames at 22,050Hz; the complete 225-voice gallery has unique
PCM. Scoped runtime decoding, corruption rejection, slow playback, exact WAV
agreement and preservation checks pass. Listening acceptance remains open; the
rendered file is `~/Music/Geothite/minnesota-cries/verdigrim-runtime-v1/VERDIGRIM.wav`.
The base-cry field in the species profile is a compiler compatibility reference;
the original-cry overlay supplies the authoritative runtime binding.
See [the full story handoff](STORY-HANDOFF.md).

Native conversion uses the repository's Rust converter:

```sh
cargo run --locked -p crystal-assets --example minnesota_sprite_sheet -- \
  --manifest sprites/mn-f145/manifest.json
```

The actual native conversion and visual inspection passed. The existing Rust
balance audit reviews 225 active species with zero progression findings, including
22,500 default move sets, 636 engine battles and 123 evolution boundaries.
The selected-family compiler fixture passed exact v4 sprite provenance and
profile-reference checks and wrote an ignored 472-species review pack.
It uses the completed Fairy v2 base; it does not select a regional map revision.
`content-packs/verdigrim-catalog-review-v02.crystalpack` reopens with 471 maps,
472 species and 270 moves. All 377 existing audio programs retain matching PCM
hashes, frame counts and loop ranges. The original howl is installed in the successor asset review pack;
listening acceptance remains open.
This is catalog and asset
validation, not evidence of a playable reveal or an earned regional encounter.
Source inputs stay in the repository; generated exports, verification packs and
listening WAVs remain ignored or external. No game selection or deployment changes.
