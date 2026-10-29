# Minneapolis → St Cloud → Brainerd

This is a versioned chapter and verification record. Its commands, parties,
page counts and results apply to the packs named below. For current narrative,
character roles and completion actions, read [STORY.md](../STORY.md),
[PROGRESSION.md](../PROGRESSION.md) and the
[current composition record](../revisions/current-story-v01/README.md).
Use the current composition path for new publications. Later source changes
need their own pack verification. Historical fixtures do not establish an
uninterrupted ordinary-party campaign.

## Historical chapter record

The local chapter pack is `content-packs/minnesota-northstar-lake-country.crystalpack`:
**538 maps,251 species,251 moves,377 audio programs**. File SHA-256:
`da27b381f69f64d6530383216077de0802c5b9dc42b12c316319cbc5daeb6d50`.
Internal content identity:
`d7ad6d24e6716c7d594044c056046f08cbb57ca0943ea6443e0e422a9b00baae`.
Its exact input is the [courier successor](courier-story.md), content identity
`58a1dc84c01d3c5d7d0c5efb849a024a5c6ece1daadb525a69055a32c55b4047`.
Generated packs, saves, captures and all render generations remain ignored.

## Roads and independent chapters

Minneapolis north `(32,0)/(33,0)` connects to the south R05 chunk
`MnR05C00R02` at `(22,15)/(23,15)`, with offsets `+5/-5`. The city's one
forest block at metatile `(16,0)` becomes road. Two wardens at `(32,2)/(33,2)`
check **Current and Capitol only**. If Current is missing, they name Avrin;
otherwise they name the missing Capitol Badge. Refusal returns the player to
`(x,4)`, and inbound travel remains free. The notebook, ICE dispatch, courier
victory and manifest never become extra prerequisites. R05's three native
chunks and its existing St Cloud south connection remain intact.

St Cloud's two north wardens at `(24,2)/(25,2)` check **Wind only**, naming
Saven on refusal. Native offsets `+4/-4` already join the city to R06 south
`MnR06C00R01` at `(16,39)/(17,39)`. R06's north mouth
`(42,0)/(43,0)` now connects to Brainerd south `(32,47)/(33,47)`, with
Brainerd/R06 offsets `-5/+5`. All inbound checkpoint travel is free.

Six stray R06 edge blocks become native forest: north metatiles `(20,0)` and
`(22,0)`, and south-chunk `(6,19)/(7,19)/(9,19)/(10,19)`. Brainerd's three
unbound future branches receive forest at `(16,0)/(0,12)/(31,12)`.
R07/R15/R18 are not yet connected in this lineage; their destinations are still
named by existing local signs. These visible temporary closures must be
replaced by actual reciprocal links and badge-six checkpoints when published.
They are not a new story or Surf requirement. The entire R05/R06 perimeter is
checked both walking and with Surf, retaining actual internal water seams.

## MS05: the fishers and the official report

The existing outdoor ranger at `(25,19)`, approached from `(25,20)`, now reads
the canonical two pages: “The fish moved on. / The report says” and
“nothing changed. / Ask the fishers.” The actual Lake Badge is required.
Neither saved chapter flag is set while either page still owns input.
The existing native successful `giveitem HM_SURF, 1` then sets
`EVENT_MN_BRAINERD_REPORT` and `EVENT_MN_SURF_GIFT_RECEIVED`. No new flag numbers,
items or boating rules are introduced. The gift and repeat responses now say
to explore the lakes and preserve the testimony, rather than promising the
unconnected north road. Repeats do not issue another HM.

Both city gyms retain their complete existing trainers, puzzles, leaders,
badges and TMs. St Cloud's native free clinic uses exterior return `(6,10)`;
Brainerd's uses `(9,22)`. Their normal recovery, HOME and Fly data stay exact.
The northern story, ICE/courier chapters and Ryan's waterfall cabin stay exact.

## Executed verification

Five controller cases pass on this file:

- Both Minneapolis lanes reject each separately missing city badge with the
  correct message, retain dialogue input ownership and survive Continue.
- Both unqualified inbound returns reach public care; Wind alone opens the
  St Cloud departure and its absence refuses both lanes.
- Native movement from legal floor stops at all nine closed throats. Two R06
  southern caps sit behind existing bank walls, so movement starts at the
  nearest legal floor; asset checks also prove all four capped quadrants block.
- An empty-party/four-prior-badge course walks every R05 chunk to St Cloud,
  receives the Wind refusal and returns to Minneapolis without courier progress.
- One continuous local course walks R05, earns Wind through the native gym,
  walks R06, earns Lake, acknowledges both ranger pages, obtains Surf once,
  continues and walks home. It uses real public care between practice trainers
  and leaders. Initial four badges, position and level45 Mew are explicit setup;
  the existing deterministic local replay divider is also part of the fixture.
  All movement, battle rewards, healing and story progress thereafter use
  `VisibleShellController::press`. This is not an earned New Game campaign.

Two asset cases pass exact written identity, six changed modules reconstructed
without caches, and complete unrelated-data preservation after reversing only
the declared six-map/storage changes. Exactly ten terrain blocks change:
one mouth and nine caps. Only the ranger's report/gift/repeat text changes among
pre-existing script bodies. Every original encounter, gym, event number,
trainer, art asset, audio program and unrelated map remains exact. Duplicate,
altered mouths/offsets, missing caps and an extra courier road gate are rejected.

Actual stdio checks cover the empty-party R05 round trip/Wind refusal, R06 round
trip/Lake refusal and a controller-earned local Lake save's report/gift/repeat.
They open the production Start menu, preserve money/badge counts and exit with
byte-empty stderr. Rust separately verifies the saved flags and HM quantity.

The whole eight-map corridor and four headless native cameras were inspected.
Each noon/down-facing1280×800 source/native comparison is exact:
Minneapolis checkpoint, St Cloud checkpoint, Brainerd south and ranger.
The active-map camera presents its actors and adjacent terrain; the complete
connected scene presents each map's actors. All original full-resolution views,
pre-conversion images, crops, previews and earlier generations remain under
`renders/lake-country-story/`. The broad northern R06 approach and Brainerd's
south road retain their earlier plain straight stretches; this chapter connects
the existing geography rather than claiming a new route-design acceptance.

All377 audio programs pass PCM hashes, frame counts and loop ranges.
Changed-leaf formatting passes. Workspace formatting still fails first at the
unrelated `crates/offset-allocator/src/lib.rs:408`. All-target assets/Bevy Clippy
finishes with existing warnings under the four previously required allowances
below; no new lake-country warning remains. The plan check validates only the
symbolic graph, not campaign traversal or balance.

## Rebuild and replay

```sh
export CARGO_TARGET_DIR=target/northstar-story
export CARGO_PROFILE_WASM_DEV_OPT_LEVEL=0
cargo run --locked --profile wasm-dev -p crystal-assets --bin pack_lake_country_story -- \
  content-packs/minnesota-northstar-courier-story.crystalpack \
  planning/minnesota/playable/lake-country-story.json planning/minnesota/story.json \
  planning/minnesota/routes/MN_R05.json planning/minnesota/routes/MN_R06.json \
  content-packs/minnesota-northstar-lake-country.crystalpack
cargo test --locked --profile wasm-dev -p crystal-assets --test lake_country_story -- --ignored
export RUST_MIN_STACK=33554432
export CRYSTAL_GYM_TEST_PACK="$PWD/content-packs/minnesota-northstar-lake-country.crystalpack"
export CRYSTAL_WINONA_SAVE_DIR="$PWD/.local/lake-country-review-saves"
cargo test --locked --profile wasm-dev -p crystal-bevy --lib lake_country_controller_ -- --ignored
cargo build --locked --profile wasm-dev -p geothite
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_lake_country.py
target/northstar-story/wasm-dev/verify_crystalpack \
  content-packs/minnesota-northstar-lake-country.crystalpack --audio --desktop
```

Affected all-target lint uses
`cargo clippy --locked --profile wasm-dev -p crystal-assets --all-targets -- -A clippy::never_loop -A clippy::bad_bit_mask -A clippy::approx_constant -A clippy::useless_conversion`,
then the same command with `-p crystal-bevy`.

The actual client is
`target/northstar-story/wasm-dev/geothite play content-packs/minnesota-northstar-lake-country.crystalpack`.
The executed local stdio replays use these retained saves:

```sh
.local/minnesota-review-env/bin/python planning/minnesota/tools/check_lake_country_mcp.py \
  --mode r05 --save .local/lake-country-review-saves/winona-lake-local-road-ready-8651-1793091482639902000.crystalsave
.local/minnesota-review-env/bin/python planning/minnesota/tools/check_lake_country_mcp.py \
  --mode r06 --save .local/lake-country-review-saves/winona-lake-local-wind-road-ready-8651-1793091478742770000.crystalsave
.local/minnesota-review-env/bin/python planning/minnesota/tools/check_lake_country_mcp.py \
  --mode ranger --save .local/lake-country-review-saves/winona-lake-local-earned-lake-8651-1793091494832246000.crystalsave
```

The full fresh-start journey, early-party retry/balance, Mankato Cut/cache,
Brainerd's later branches, Bemidji/Itasca report, rail, League and Kettlurk's
set piece remain unfinished. This publication is one connected chapter of that
larger objective.
