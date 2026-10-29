# Mankato → River Valley → Minneapolis

This is a versioned chapter and verification record. Its commands, parties,
page counts and results apply to the packs named below. For current narrative,
character roles and completion actions, read [STORY.md](../STORY.md),
[PROGRESSION.md](../PROGRESSION.md) and the
[current composition record](../revisions/current-story-v01/README.md).
Use the current composition path for new publications. Later source changes
need their own pack verification. Historical fixtures do not establish an
uninterrupted ordinary-party campaign.

## Historical chapter record

[Complete connected source view](../renders/river-story/whole-connected-city-valley.png) ·
[Native Mankato checkpoint](../renders/river-story/native-mankato-north.png) ·
[Native Minneapolis arrival](../renders/river-story/native-minneapolis-south.png) ·
[Native southern bank closure](../renders/river-cities/native-r03-south-bank.png)

The ignored `content-packs/minnesota-northstar-river-story.crystalpack` contains
538 maps and all eight current gyms. Its internal content identity is
`f98c5446760288142f6e56727f910b585d9dc67cbd9eaec5674c4bc542960821`.
The final file SHA-256 is
`01944a98d5381de24bee3469f0f426a16a08c4e9bf96c4ec891655f2d2a21a85`.
It extends the exact Prairie Road chapter, identity
`f9853ad3269c91f53871f50ce766e4d59b8228bcfdbbeded44243ccad48981be`.
These are content identities, not file hashes.

The two existing River Valley chunks now connect Mankato's northern forest mouth
to Minneapolis's southern street. Two wardens check the actual second badge
when leaving Mankato. Both lanes refuse unqualified outbound travel, explain
Torven's Prairie Badge requirement, own input during dialogue, and return the
player one step. Inbound travelers can always return to town. Guard visibility
reconciles afterward, including after Continue. The optional Prairie notebook
is independent of the second-badge gate.

| Mouth | City core edge | Route core edge | Native city / route offsets |
| --- | --- | --- | --- |
| Mankato north ↔ R03 south | `(24,0)`, `(25,0)` | `MnR03C00R01 (18,15)`, `(19,15)` | `3 / -3` |
| Minneapolis south ↔ R03 north | `(32,47)`, `(33,47)` | `MnR03C00R00 (20,0)`, `(21,0)` | `6 / -6` |

Mankato's wardens stand at `(24,2)` and `(25,2)`, with outbound triggers one
tile inside and inbound triggers one tile outside. Native `ENGINE_HIVEBADGE`
is the existing engine slot for the authored Prairie Badge. No new event flag,
badge, item, money or party member is awarded by this overlay.

Only four modules change: the two towns and two R03 chunks. One native forest
metatile in each town becomes ordinary road; the new reciprocal records retain
Mankato's existing R02 connection and R03's internal connection. Six visible
south-bank forest caps close unintended walking/Surf exits at global metatiles
`(7,31)`, `(8,31)`, `(10,31)`, `(11,31)`, `(13,31)` and `(18,31)`.
The complete walking/Surf component now reaches only the declared route mouths;
the wardens cannot be bypassed. Other R03 terrain, encounters, actors, cave doors
and all gym courses stay exact. Existing story
programs, event numbers, trainer catalogs, source files, audio, HOME and recovery
remain unchanged. Cached and reconstructed split-payload modules agree.

## Executed publication and verification

```sh
CARGO_TARGET_DIR=target/gym-interiors cargo \
  --config 'profile.wasm-dev.package.crystal-assets.opt-level=0' \
  run --locked --profile wasm-dev -p crystal-assets --bin pack_river_cities -- \
  content-packs/minnesota-northstar-prairie-story.crystalpack \
  planning/minnesota/playable/river-city-links.json \
  planning/minnesota/routes/MN_R03.json \
  content-packs/minnesota-northstar-river-story.crystalpack

CARGO_TARGET_DIR=target/gym-interiors cargo \
  --config 'profile.wasm-dev.package.crystal-assets.opt-level=0' \
  test --locked --profile wasm-dev -p crystal-assets --test river_cities -- \
  --ignored --test-threads=1

RUST_MIN_STACK=67108864 CARGO_TARGET_DIR=target/gym-controller \
CRYSTAL_GYM_TEST_PACK="$PWD/content-packs/minnesota-northstar-river-story.crystalpack" \
CRYSTAL_WINONA_TEST_PACK="$PWD/content-packs/minnesota-northstar-river-story.crystalpack" \
CRYSTAL_MINNEAPOLIS_TEST_PACK="$PWD/content-packs/minnesota-northstar-river-story.crystalpack" \
cargo --config 'profile.wasm-dev.package.crystal-assets.opt-level=0' \
  --config 'profile.wasm-dev.package.crystal-runtime.opt-level=0' \
  --config 'profile.wasm-dev.package.crystal-bevy.opt-level=0' \
  test --locked --profile wasm-dev -p crystal-bevy --lib \
  river_cities_controller -- --ignored --nocapture --test-threads=1

.local/minnesota-review-env/bin/python \
  planning/minnesota/tools/review_river_cities.py

target/gym-interiors/wasm-dev/verify_crystalpack \
  content-packs/minnesota-northstar-river-story.crystalpack --audio --desktop
```

Two asset cases pass: exact preservation and uncached reconstruction, plus
rejection of repeated publication, changed badge prerequisites and incorrect
mouths/offsets. All eight reciprocal edge projections are checked by the builder.
Five current production-controller cases pass. The three gym/connection cases
cover:

- Both unqualified gate lanes, dialogue input ownership, denied repeat after
  Continue, and unqualified inbound return after saving outside the wardens.
- Both qualified lanes with only prior badge fixtures: no gym2 victory event
  or notebook is required. Crossing, Continue and returning work in both lanes.
- One continuous local course: enter Mankato's real facade, defeat Caldren,
  Jorel and Torven through the terrace puzzle, earn badge2/TM, walk the complete
  R03, enter Minneapolis's real facade, defeat Lenvi/Jossa and solve its adjacent
  switches, earn Avrin's badge3/TM, then walk back to Mankato. Native attacks spend
  PP; rewards and terrain states survive Continue.

The additional existing Minneapolis story course also passes on this successor:
a separate level45/two-badge fixture walks R03 to the dispatcher, wins the ICE
battle, acknowledges the rescue and evidence, then walks back and Continues.
The first attempt omitted `CRYSTAL_MINNEAPOLIS_TEST_PACK`; supplying that required
input allows the case to run and pass. It grants neither the third badge nor
the optional Prairie notebook. This remains a separate local story fixture.
The fifth case places the player at each of the six former south-edge exits and
proves repeated native movement stops at the visible forest cap.

The continuous course starts with **one explicit Mankato placement, the first
badge and a legal level100 partner**. Nothing is teleported or granted afterward.
Wild encounters use the native RUN menu; trainer sight is avoided through legal
walking paths. This proves the connection and consecutive gym mechanics, not
ordinary-party balance or a fresh starter journey.

The Prairie Road input was separately rechecked: its two preservation/rejection
cases and three local controller cases pass. Those cover Rochester's Care gate,
R02 travel, notebook prerequisites/acknowledgement, Mankato room returns and
Continue. Its separate earned New Game case was not rerun in this step.

The earlier `minnesota-river-city-links-v1` candidate, content identity
`49bc32fc6c6c867a21934e14713e0ad03a038b1a81cd1c119e1ff8bd8304f991`,
remains available as `minnesota-northstar-river-cities.crystalpack` for historical
comparison. Its tests passed, but it lacked the six southern boundary caps.
Version2 above supersedes it; the current builder rejects modified/missing caps.

The complete connected source composition and matched native city frames were
inspected. Both current1280×800 native frames match their source/player reference
exactly (1,024,000 pixels each), at noon with south-facing initial fixtures.
The southern closure camera was also inspected. Earlier640×576 city frames
match too (368,640 pixels each). An outdated
renderer produced an earlier incorrect Minneapolis frame; rebuilding with
`--features location-tester` fixed it. These 640×576 captures document the first
candidate's unchanged city mouths; the current reviewer also emits a wider
1280×800 view under `renders/river-story`. That rejected frame is retained alongside
the originals and conversions. Blank parts of the connected sheet represent
unmapped space; the composition invents no connecting terrain.

Both current1280×800 native entrance frames were also inspected and match their
source/player references exactly:0/1,024,000 differing pixels each. The wider
Mankato camera includes the capped southern river boundary. The complete
composition compares both source town mouths and every R03 block, permitting
only the six declared caps.

Actual stdio now plays the entire Mankato/R03/Minneapolis round trip, visits the
office's still-locked record, and opens Start after returning, with unchanged
badges/money and byte-empty stderr. Its save deliberately has two badges and an
empty party. A second actual stdio run loads the controller-earned enforcer win,
declines/retries the separate release, reads the record, visits the dispatcher
at home and opens Start. Both processes exit cleanly. Exact replays from the
repository root:

```sh
.local/minnesota-review-env/bin/python planning/minnesota/tools/check_river_story_mcp.py \
  --save .local/river-story-review-saves/winona-river-local-road-ready-97498-1793086557567743000.crystalsave
.local/minnesota-review-env/bin/python planning/minnesota/tools/check_minneapolis_mcp.py \
  --pack content-packs/minnesota-northstar-river-story.crystalpack \
  --save .local/river-story-review-saves/ms03-enforcer-win-97611-1793086629871148000.crystalsave
```

The same builder, asset cases, five controller cases and client build were run
with `CARGO_TARGET_DIR=target/northstar-story` and
`CARGO_PROFILE_WASM_DEV_OPT_LEVEL=0`. Controller inputs additionally set
`CRYSTAL_MINNEAPOLIS_SAVE_DIR` and `CRYSTAL_WINONA_SAVE_DIR` to the absolute
`.local/river-story-review-saves` directory. These are retained local review
saves, not a newly earned complete game.

All377 audio programs pass PCM hashes, frame counts and loop ranges. Actual
stdio also passes on this successor from the exported empty-party/two-badge
fixture: Mankato → both R03 chunks → Minneapolis's dispatch office → Mankato,
locked-record interaction and the production Start menu, with unchanged
money/badges and byte-empty stderr. The generic core clean-stderr regression
also passes independently. The exact additional invocation was:

```sh
.local/minnesota-review-env/bin/python \
  planning/minnesota/tools/check_river_story_mcp.py \
  --client target/gym-controller/wasm-dev/geothite \
  --save .local/river-story-review-saves/winona-river-local-road-ready-97611-1793086627555421000.crystalsave
```

The save is explicit local setup, not an earned journey. Changed Rust leaves pass
rustfmt. Workspace formatting retains unrelated differences beginning in
`offset-allocator/src/lib.rs:408`; full clippy is blocked by the existing
`crystal-core/src/multiplayer/mod.rs:1332` always-false bit mask. Scoped all-target
lint passes for both affected crates with the four existing allowances
`never_loop`, `bad_bit_mask`, `approx_constant` and `useless_conversion`, with
no warnings in the changed chapter modules. This is not an unsuppressed
workspace lint pass.

All eight selected Minneapolis regressions pass on the final successor: four
ICE/service/outcome cases and four switch-gym prerequisite/loss/persistence
cases. One old exit helper initially asserted the review hall after a city-bound
carpet. It now reads the actual native exit/arrival records, executes the same
production inputs, and retains the exact hall-coordinate assertion when the
room is in the hall. The three prairie local cases also pass on this successor.

## Remaining journey

R02 and R03 now have actual city mouths. The later
[fourth-gym approach](gym-four-approach.md) also opens the R04 east road with
the Current Badge/dispatch-record checkpoint and a continuous third/fourth
gym course. Minneapolis's north exit, remaining city bindings and gates,
St Paul's courier, rail, later chapters and
the complete earned badge journey still need integration/acceptance. All eight
local gym courses were previously verified together; this successor additionally
proves the consecutive second/third courses. It does not establish completion
of the region or user visual approval.
