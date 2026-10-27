# Brainerd → Leech Lake → Bemidji

The local chapter pack is `content-packs/minnesota-northstar-leech-shore.crystalpack`:
**541 maps,251 species,251 moves,377 audio programs**. File SHA-256:
`03c9d073629c60debc66368f07a4170d0b305026c84fd3d5fdcfd96f95c60a97`.
Internal content identity:
`43b3d65bda72a8904cfad17781cc0da408045c64d91ee22fb5c6b3b1634006db`.
Its exact input is the [lake-country successor](lake-country-story.md), content
identity `d7ad6d24e6716c7d594044c056046f08cbb57ca0943ea6443e0e422a9b00baae`.
Packs, saves, captures and every render generation remain ignored.

## A complete shore stop

Leech Lake was absent from the native input, rather than an already implemented
town needing only connections. The new hand-authored32×24 shore has a bent
forest path, two timber cabins, a short public fishing pier and a separate lake
launch. The large lake stays visibly enclosed by native shore/forest. Main
travel, witnesses and free care need neither Surf nor the ranger report.

The pier has two walking rows, `(28..40,21)/(28..40,22)`, north/south post rails
at y20/23 and a closed end at x41. The fisher at `(35,21)` reports empty nets
against last year's published catch. The lake launch is separate: FLOOR
`(31,29)` faces WATER `(32,29)`. The shore walker at `(27,29)` distinguishes
public access from claimed badge authority. Inside the fisher's cabin, a resident
kept dates after ICE asked him to stop writing. These are optional testimonies;
they grant no item, badge or chapter flag.

| Room | Actual door | Safe exterior return | Passive room arrival | Both carpet exits |
| --- | --- | --- | --- | --- |
| Shore Care Cabin | `(11,13)` | `(11,14)` | `(10,13)`, record3 | `(10,15)/(11,15)` |
| Fisher House | `(9,35)` | `(9,36)` | `(5,6)`, record3 | `(5,7)/(6,7)` |

The nurse is behind the real counter at `(7,3)`; approach `(7,5)` across counter
`(7,4)`. Native HealParty restores HP/PP for free and blackoutmod selects the
actual new care cabin. Group78 contains exterior1/care2/fisher3. Only the shore's
own exterior and care recovery records are added; original HOME, recovery,
Fly data, event numbers and encounters stay exact. The existing native Leech
Lake geographic area is reused when the seven-bit landmark table is full.

## Four reciprocal crossings

| Town edge | Route edge | Town direction and offset |
| --- | --- | --- |
| Brainerd `(32,0)/(33,0)` | R07 south chunk `(20,47)/(21,47)` | north `+6` |
| Leech Lake `(32,47)/(33,47)` | R07 north `(20,0)/(21,0)` | south `+6` |
| Leech Lake `(32,0)/(33,0)` | R08 south-left chunk `(26,47)/(27,47)` | north `+3` |
| Bemidji `(24,47)/(25,47)` | R08 north-left `(40,0)/(41,0)` | south `-8` |

Every route reciprocal uses the opposite direction and negated offset. Sixteen
native lane projections are checked. Brainerd and Leech Lake's north wardens
check **Lake only**, with both lanes, directional refusal and free unqualified
inbound return. The ranger's Surf/report flags and courier completion are not
road conditions.

Brainerd's previous north forest cap becomes road. Eleven native forest caps
close unrelated edges: R07 `(11,0)` and south-chunk `(11,23)`; R08 north-left
`(0,17)/(0,18)` and south-left `(11,23)/(12,23)/(14,23)/(15,23)`; Bemidji
`(12,0)/(0,12)/(23,12)`. The two R08 west-bank cells were actual unbound lawn,
found by the complete perimeter check. Both walking and Surf graphs cover all
R07/R08 chunks, including R08's clipped east column. Existing internal water
and land seams stay intact. Bemidji's R09/R14/R17 and Brainerd's optional
branches remain visibly closed until their real bindings are published.

## Executed evidence and limits

Six production-controller cases pass:

- Both Lake refusal lanes in both towns, dialogue ownership, Continue, both
  unqualified inbound lanes and access to free public care.
- Empty-party/six-prior-badge R07/shore/R08/Bemidji/clinic travel and return,
  fisher testimony and actual closed-pier collision, without ranger/courier
  or headwaters progress.
- A deliberately injured/depleted partner receives actual free HP/PP care.
  Both real carpets of both cabins return safely, with dialogue and Continue.
- One continuous local course walks both roads, receives native HM01, plays
  the complete existing Pine gym/Cut walkthrough, loses the first leader
  attempt using neutral Psychic attacks, receives ordinary clinic recovery and
  money loss without a badge/TM, continues and retries using its existing Fire
  move. The actual win awards Pine/Giga Drain, persists and returns to Brainerd.
  Prior six badges, initial level45 Mew with Cut, position and deterministic
  local replay divider are setup. Cut teaching is not earned by this fixture.
- Native Surf mounts at the separate launch, reaches the closed east coast,
  survives Continue and dismounts back onto the launch. Main travel stays dry.
- A local Magikarp uses public care, then actually loses a route encounter.
  Native blackout returns to the new care cabin with restored HP/PP and reduced
  money. No story progress is granted, and Continue preserves the recovery.

Two asset cases pass exact written identity, all nine new/changed modules with
caches removed, only twelve declared old terrain changes, unchanged original
objects/scripts and complete unrelated-catalog preservation. All original gyms,
story chapters, Ryan's waterfall cabin, art and compiled audio remain exact.
Duplicate publication, active-carpet arrivals, a nurse on the counter, a broken
pier rail/end, an unbound edge and an extra Surf road gate are rejected.

Actual stdio passes three local replays: the empty-party road/cabins/pier/clinic
round trip; controller-earned Pine Continue, exit, care and gym re-entry; and
the actual Pine-loss Continue at public care. All open Start, preserve their
money/badge counts and exit with byte-empty stderr. The Pine path planner reads
the compiled replacement blocks for its already earned Cut state, preserving
their wall quadrants. It modifies only a planner copy; real client movement
must reach the exit. The live pack, save and story flags are never rewritten.

The complete connected nine-map corridor and whole cabins were visually
inspected. Native noon/down-facing pier and care-porch1280×800 references match
exactly, as does the complete fisher room at4×,640×512. Native care-counter and
arrival views were inspected against the full source room. Initial stacked
facade choices, an excessive pier and broad empty yards were corrected before
publication. One initial pier comparison differed by27 pixels at the player's
lower leg beside the posts; that view remains archived. An adjacent upper-lane
capture matches exactly. No native pixels were edited. Every original, crop,
conversion, rejected pass and preview remains under `renders/leech-shore/`.

All377 PCM hashes, frame counts and loop ranges pass. Changed-leaf rustfmt
passes. Workspace formatting retains the pre-existing offset-allocator failure
at line408. All-target assets/Bevy Clippy finishes with existing warnings under
the four established allowances below; no new shore-module warning remains.
The symbolic plan check now contains164 planned maps, including the two rooms;
that count is not evidence of earned campaign traversal.

Itasca's actual marker map, R14, the acknowledged Bemidji report and its R09
gate remain next. Bemidji's officer still has ordinary comparison dialogue,
without granting the report. The full fresh-start campaign, early-party balance,
Mankato Cut/cache, optional branches, rail, League and Kettlurk remain unfinished.

## Rebuild and replay

```sh
export CARGO_TARGET_DIR=target/northstar-story
export CARGO_PROFILE_WASM_DEV_OPT_LEVEL=0
cargo run --locked --profile wasm-dev -p crystal-assets --bin pack_leech_shore -- \
  content-packs/minnesota-northstar-lake-country.crystalpack \
  planning/minnesota/towns/MnLeechLake.json planning/minnesota/towns/MnLeechLake.layout.json \
  planning/minnesota/routes/MN_R07.json planning/minnesota/routes/MN_R08.json \
  content-packs/minnesota-northstar-leech-shore.crystalpack
cargo test --locked --profile wasm-dev -p crystal-assets --test leech_shore -- --ignored
export RUST_MIN_STACK=33554432
export CRYSTAL_GYM_TEST_PACK="$PWD/content-packs/minnesota-northstar-leech-shore.crystalpack"
export CRYSTAL_WINONA_SAVE_DIR="$PWD/.local/leech-shore-review-saves"
cargo test --locked --profile wasm-dev -p crystal-bevy --lib leech_shore_controller_ -- --ignored
cargo build --locked --profile wasm-dev -p geothite
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_leech_shore.py
.local/minnesota-review-env/bin/python planning/minnesota/tools/review_leech_binding.py
target/northstar-story/wasm-dev/verify_crystalpack \
  content-packs/minnesota-northstar-leech-shore.crystalpack --audio --desktop
```

All-target lint runs with
`cargo clippy --locked --profile wasm-dev -p crystal-assets --all-targets -- -A clippy::never_loop -A clippy::bad_bit_mask -A clippy::approx_constant -A clippy::useless_conversion`,
then the same command with `-p crystal-bevy`.

```sh
.local/minnesota-review-env/bin/python planning/minnesota/tools/check_leech_shore_mcp.py \
  --mode roads --save .local/leech-shore-review-saves/winona-leech-local-road-ready-11230-1793093714567591000.crystalsave
.local/minnesota-review-env/bin/python planning/minnesota/tools/check_leech_shore_mcp.py \
  --mode pine --save .local/leech-shore-review-saves/winona-leech-local-earned-pine-11230-1793093735201068000.crystalsave
.local/minnesota-review-env/bin/python planning/minnesota/tools/check_leech_shore_mcp.py \
  --mode loss --save .local/leech-shore-review-saves/winona-leech-local-pine-loss-11230-1793093730603912000.crystalsave
target/northstar-story/wasm-dev/geothite play content-packs/minnesota-northstar-leech-shore.crystalpack
```
