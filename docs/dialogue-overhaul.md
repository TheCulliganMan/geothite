# Minnesota dialogue overhaul

The revision replaces remote opening conversations with staged scenes, gives the
survey errand an explicit introduction and handoff, and rewrites the required
campaign conversations as authored native pages. It retains current geography,
the active Minnesota roster, existing progression identities and original audio.

## Source and publication

`planning/minnesota/story.json` → `dialogue_overhaul` is the final authority for
the revised text labels. Each page has one or two lines of at most 18 native
ASCII glyphs. The Rust authoring compiler rejects overflow; it never silently
reflows these pages. Historical chapter text is intermediate source. The current
story composer applies this revision after the chapter and rival overlays.
`refresh_northstar_dialogue` rejects an already revised pack, preventing an old
text-only overlay from restoring obsolete prose.

The standalone Rust publisher requires an explicitly reviewed current parent:

```sh
export CARGO_TARGET_DIR=target/northstar-story
cargo build --locked -p crystal-assets --bin revise_northstar_story
target/northstar-story/debug/revise_northstar_story \
  content-packs/minnesota-current-marais-partner-mille-v02.crystalpack \
  planning/minnesota/story.json \
  content-packs/minnesota-dialogue-overhaul-v13.crystalpack
```

Choose a fresh ignored output filename for each publication. Completed packs
and existing saves are retained. A revised pack has its own content identity;
old saves must not be relabeled for it. The selected regional world pointer must
not advance until the complete acceptance course passes.

## Presentation contract

The shared production controller remains authoritative. Authored movement,
coordinate events, conversations, menus, battle turns and acknowledgements use
the same input path in the graphical client and MCP.

Field text retains typed paragraph, continuation, inline pause and sound cues.
Paragraphs clear the box and wait 20 source frames. Continuations move the
carried line upward in two five-frame stages. Inline text pauses wait 30 frames,
unless the source's held-button acceleration applies. These boundaries follow
the [pinned Crystal text implementation](https://github.com/pret/pokecrystal/blob/3bc8daa4173e96a7f4011dad3922eb6fa5dad5c6/home/text.asm#L859). Audio cues fence printing
until playback finishes. Renderer-neutral settling completes the device fence
without changing the script's required acknowledgement.

The existing Fast/Mid/Slow character delays and saved options remain in force.
Acceleration cannot also acknowledge a newly completed page or answer a later
choice. Dialogue diagnostics remain opt-in through `CRYSTAL_DIALOGUE_TRACE`;
production terminal stderr must stay empty.

## Acceptance course

External pack prerequisites are read at runtime. None are embedded in tests.

```sh
export CRYSTAL_STORY_REVISION_PARENT="$PWD/content-packs/minnesota-current-marais-partner-mille-v02.crystalpack"
export CRYSTAL_WINONA_TEST_PACK="$PWD/content-packs/minnesota-dialogue-overhaul-v13.crystalpack"
export CRYSTAL_RENDER_TEST_PACK="$CRYSTAL_WINONA_TEST_PACK"
export CRYSTAL_WINONA_SAVE_DIR="$PWD/.local/dialogue-overhaul/saves"
export CRYSTAL_NORTHSTAR_EVENTS_SAVE_DIR="$PWD/.local/dialogue-overhaul/scene-checkpoints"
export RUST_MIN_STACK=67108864
cargo test --locked -p crystal-assets --lib authored_dialogue
cargo test --locked -p crystal-assets --lib dialogue_revision_preserves -- --ignored
cargo test --locked -p crystal-bevy --lib overhaul_text_program
cargo test --locked -p crystal-bevy --lib overhaul_three_earned -- --ignored --test-threads=1
cargo test --locked -p crystal-bevy --lib overhaul_printer -- --ignored
cargo test --locked -p crystal-bevy --lib overhaul_text_confirmation -- --ignored
cargo test --locked -p crystal-bevy --lib overhaul_earned_campaign -- --ignored --test-threads=1
```

The full campaign test forbids level boosts and externally supplied chapter
checkpoints. Local chapter fixtures supplement that earned course; they do not
replace it. Publication also requires the existing controller/MCP/clean-stderr
regressions, all 377 audio PCM/hash/frame/loop checks, and inspection of native
headless 2D views plus native/browser audio playback. The user's latest direction
limits this overhaul's visual implementation and acceptance to 2D.

Builds, generated packs, checkpoints, transcripts and capture files remain
ignored. Build/test results and any unresolved acceptance gates are recorded
below only after execution.

## Campaign coverage

The canonical revision contains 279 conversations across 36 maps, with 815
explicit native pages. It covers required handoffs and their refusals, reminders,
losses, repeats and completed-action returns through the ending.

| Chapter | What the conversation establishes |
| --- | --- |
| Winona → Rochester | Tamarack, Ivy, a partner, the sealed original readings, explicit receipt and delivery. |
| Minneapolis → St. Paul | ICE coercion, Ellis and Jo, separate rescue and dispatch evidence, the courier's automatic depot encounter and Mina's next destination. |
| Brainerd → Itasca → Bemidji | Habitat damage, the missing meter, an independent reference record and the officer's comparison of contradictory logs. |
| Mine → Grand Rapids | Diverted equipment, pump testing, actual meter recovery, custody, and the technician's readable log copy. |
| Duluth → Boundary Waters | Pump equipment, the intake plan, Kettlurk, habitat consequences, the crew and directions to the relay. |
| Intake → League | Separate battle, evidence transmission, drive shutdown, visible release, crew response and clearance; registration, five opponents, Hall of Fame, credits and returns. |

The optional Mankato notebook remains optional. Closed Safari admission stays
closed. Ivy's clinic aide is a distinct speaker; Ivy leaves for the intake when
summoned, and advice follows the latest completed investigation action. Harbor
refusals distinguish missing badges from missing evidence. Completed relay and
harbor replies describe the shutdown and released creature as completed actions.

## Verification results

The current full-story candidate is v13. Its parent is the accepted
`minnesota-current-marais-partner-mille-v02` descendant, preserving the newer
Northwoods, Detroit approach and Ely grass, compact coast, Castorock cave,
walking art, original audio, post-Champion Kindlit research offer, Mille Lacs
grass and Grand Marais partner retry. The final
candidate has native identity
`16ff1b62e0ffc81017c888bd77337bb0fb63d9e780cbf66f2faca529d09e4f48`.
It remains unselected while the complete acceptance course is open. Whole-pack
preservation, all 377 audio hash/frame/loop checks and the production stdio
opening replay with empty stderr pass on this identity.

The v13 acceptance runs pass all three earned New Game starter journeys through
Ivy, continuous and diagonal depot interception under the production 2D keyboard
schedule, and both lanes of the repaired League road. The production stdio
opening and repeated road crossings retain byte-for-byte empty stderr. All 377
original audio programs pass their PCM hashes, frame counts and loop ranges.

The v12 full scene suite passes four lab approaches, ten depot arrivals,
full-pocket aide refusal, native Pack toss, retry and once-only receipt; held
and diagonal input into Mom's scene; source text controls, 30-frame pauses,
20-frame paragraphs, two five-frame scroll stages, acceleration and ordered
non-silent playback preparation. Nine cumulative clinic return fixtures prove
distinct actors, correct advice, movement locks, no accidental rewards and
Save/Continue. Harbor prerequisite/return and Boundary finale fixtures assert
battle, evidence, drive, visible Kettlurk escape, crew response and clearance
separately, including exact actors and deferred final flags. These scenes are
unchanged in v13 apart from the separately verified reciprocal road offsets.

The v13 declared late-story course has completed Brainerd, Leech Lake, Bemidji,
Itasca, the expanded mine and rescue, Grand Rapids custody, Duluth and the
compact coast, the chief, evidence transmission, local shutdown, visible
release, crew response, repaired road, Ely care and shopping, registration,
all five League battles, the Hall of Fame autosave, credits and actual postgame
Continue. It began with an explicit five-badge party fixture, then used ordinary
inputs and native checkpoints without further state grants. This proves the
connected later chapters; it does not establish an earned New Game campaign or
battle balance. The completed continuation log is
`.local/dialogue-overhaul/late-story-v13-league-retry.log`.

The independent v13 earned course starts from production New Game, uses
ordinary catches, training, menu actions and battles, and forbids externally
supplied progression or level boosts. It has earned four badges and reached
St. Cloud. Actual checkpoints and in-scene saves retain the v13 content identity;
frozen earlier candidates' saves are never relabeled. The continuing run is
recorded in `.local/dialogue-overhaul/earned-full-v13.log`.

The replay found a real League road blocker: the camp's exit at (32/33,47) met
road walls at (32/33,0). The revision restores the canonical one-metatile
connection offset so it meets the existing road floor at (30/31,0), with
reciprocal -1 on return. All terrain cells stay exact. Both lanes, denial before
shutdown, Continue and repeated production stdio crossings in both directions
pass on v13. Native 2D captures of the road, intake encounter, released Kettlurk,
crew response, harbor return, completed Champion dialogue and actual earned
Minneapolis coercion were inspected. The complete readable campaign transcript remains ignored at
`.local/dialogue-overhaul/complete-campaign-transcript.md`.

Full earned-campaign completion, native/browser listening and normal-speed
audio recordings remain open. PCM verification, an unlocked browser sound
control and successful source dispatch do not prove audible device playback.
The selected regional pack is not advanced by this work.

Affected-crate all-target Clippy completed after the final League checkpoint
driver correction, with existing workspace warnings. No warning was introduced
in the revised story leaf modules. The final log is
`.local/dialogue-overhaul/final-clippy-v13.log`.
Workspace formatting check reports existing failures beginning in
`crates/offset-allocator`; new leaf modules are formatted. Verification packs,
saves, the full readable transcript and captures remain ignored under
`content-packs` and `.local/dialogue-overhaul`.

## 2D browser review

The browser build compiles only `bevy-shell,fullscreen-scaling`; it does not
compile the voxel feature. The executed build command was:

```sh
WASM_BINDGEN="$PWD/.local/dialogue-overhaul/tools/wasm-bindgen-0.2.128-aarch64-apple-darwin/wasm-bindgen" \
  CRYSTAL_WEB_OUT=target/dialogue-overhaul-2d-web-v13 sh tools/dialogue-2d-web.sh
```

The official CLI version must match the locked Rust wasm-bindgen version
0.2.128. The script checks this and writes only ignored local review output.
The local Rust server was started with:

```sh
CRYSTAL_DATA_DIR="$PWD/.local/dialogue-overhaul/browser-v13-data" \
  target/northstar-story/debug/crystal-web-server \
  --dir target/dialogue-overhaul-2d-web-v13 --host 127.0.0.1 --port 3009
```

The browser replay exposed two real startup defects: the native 2D composite
image upload plugin was installed only with the voxel feature, and OakSpeech
rejected the regional GILLPUP portrait because it expected WOOPER. The upload
plugin is now unconditional; the showcase uses the pack's portrait and its
matching embedded cry. The regional portrait regression passes. The v13 Rust-only browser was rebuilt
and cold-start checked through the native clock and GILLPUP showcase in 2D,
with the sound control unlocked and no browser errors observed. The earlier v04
hidden browser replay passed fresh New Game, naming, the rendered home, Mom's
approach and return, Save and reloaded Continue. A 16 MiB Wasm stack fixes the
former name-confirmation overflow. Sound readiness alone does not prove audible
output; browser listening and a normal-speed audio recording remain open.

## Browser publication — 2026-10-29

The v05 story is now [live on Geothite](https://geothite.ryanculligan.com/?view=2d),
using the completed Cedar warmth/R20 parent. The
[deployment record](deployment.md#northstar-native-2d-story-release-2026-10-29)
gives exact input/runtime hashes, the final image, public online Chromium/mobile
WebKit opening checks, the hosted-lineage fix and rollback commands. Minnesota
is native 2D only; TUI remains available. Native downloads retain their previous
verified content through a pinned pack path. Full earned-campaign balance and
listening review remain separate acceptance work.
