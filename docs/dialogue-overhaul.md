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
  content-packs/minnesota-current-duluth-cliff-stairs-v03.crystalpack \
  planning/minnesota/story.json \
  content-packs/minnesota-dialogue-overhaul-v19.crystalpack
```

Choose a fresh ignored output filename for each publication. Completed packs
and existing saves are retained. A revised pack has its own content identity;
old saves must not be relabeled for it. The selected regional world pointer must
not advance until the complete acceptance course passes.

The original exact-parent campaign composer builds a historical foundation
before later climate, grass and terrain overlays. Its private final scene step
requires the active composition guard and preserves that foundation's outdoors,
apart from the guarded League turnout. It shares the canonical scenes and prose
with the regional reviser. The public regional reviser still requires an
approved current parent and the complete warm-coast/geography checks before and
after installation. A historical composition is not a current-world candidate.

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
export CRYSTAL_STORY_REVISION_PARENT="$PWD/content-packs/minnesota-current-duluth-cliff-stairs-v03.crystalpack"
export CRYSTAL_WINONA_TEST_PACK="$PWD/content-packs/minnesota-dialogue-overhaul-v19.crystalpack"
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

Historical earned v13 continuations retain their original pack and saves. They
use `CRYSTAL_NORTHSTAR_CANONICAL_STORY_SOURCE` to read the ignored
`.local/dialogue-overhaul/frozen-v13-story.json` review source; current-candidate
checks leave that override unset. The strict frozen-campaign comparison uses
`CRYSTAL_STORY_FROZEN_SOURCE` for that archive, checks its original programs
against the frozen pack, then permits exactly the 50 reviewed prose changes
and three reviewed scene programs: the two dispatch handoffs and the officer's
inspection/reaction. Ellis gives directions at home; reading
the physical desk copy grants the existing evidence event only after its final
acknowledgment. The two final clinic corrections establish where GILLPUP was
found and acknowledge its return home, with shore repairs still needed. All
other event programs and every actor remain unchanged. It does not relabel any
save.

Builds, generated packs, checkpoints, transcripts and capture files remain
ignored. Build/test results and any unresolved acceptance gates are recorded
below only after execution.

## Campaign coverage

The canonical revision contains 279 conversations across 36 maps, with 829
explicit native pages. It covers required handoffs and their refusals, reminders,
losses, repeats and completed-action returns through the ending.
The frozen v18 pack retains 828 pages. The final source adds one clinic return
page acknowledging the shore repairs still needed. The displaced GILLPUP
interaction describes where it was found; the creature leaves after restoration,
and the aide acknowledges its return home.
Those two text corrections are installed in the composed v19 candidate; v18
and its saves remain untouched.

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

The final-source review candidate is v19, identity
`3c48a27c98c412e3e292a5912193b06131fb8a04b70b8695be4de5f65f6ab45f`.
Its SHA256 is
`47970de2d183d3f4aad270dd4e1a3edc7afca237a14060bc331fd165c8c4db32`.
It was published and read back through Rust from the accepted Duluth stairs v03
parent, retaining the Fair building repair. Its 279 conversations have 829 pages;
the two clinic wording changes are the only text differences from v18. An
independent exact comparison confirms all 730 maps retain their scene programs,
actors, events, warps and scene tables, with only the two clinic text labels
changed. Only the accepted Duluth/Fair terrain and art differ; all encounter
catalogs, species, teams, moves, items, event identities and compiled audio are
exact. All four strict Rust cases pass, including source/pack identity,
frozen-campaign program equivalence and rejection of the shifted League cycle.
The parent-anchor case uses the frozen Rainy predecessor; the initial invocation
with historical r15 failed on its older trainer catalog and is retained. All
377 original PCM hashes, frame counts and loop ranges also pass on v19.
The nine clinic-return stages pass on v19 (202.94 seconds), including the
creature leaving after restoration and the aide's completed-action reply. The
rescue NO/Continue/refusal case also passes (37.20 seconds). The extended Save
regression reads the warning at the saved text speed, keeps the final overwrite
question separate, rejects held-A answering and verifies NO leaves the file
unchanged (23.91 seconds). The exact-v19 earned New Game course uses no
progression or level overrides; it uses the frozen v19 canonical text archive
and retains its own saves. It earned four badges, cleared the headquarters,
read Ellis's physical office copy, completed the automatic courier encounter
and depot handoff, freed the St. Cloud worker and won the first R05 trainer.
Its continuation retains that actual checkpoint and the same pack identity. All three
separate production New Game starter paths through Ivy also pass on exact v19
(496.91 seconds), including custom naming and the explicit delivery. Its ending
acceptance remains open.
It is unselected. Existing v18 saves and their earned course remain on v18.

The frozen full-story earned-campaign input is v18, identity
`5cc4f43900cf14f0e165b73f7ba962ddebc9f1d935e57e270d8d01c4a348270a`.
It extends the accepted Rainy River Icefields descendant
(`17c4400d010fa9dbd10c40c9f86ee543da8b2e795bee53956d8518c30ada5472`),
preserving the newer R09/R10 cliffs and upstream, Confluence and Rainy grass.
The canonical campaign has 279 conversations on 36 maps and 828 authored pages.
Forty-eight further conversations clarify evidence custody, technical terms,
pressure gauges, completed actions and directions. Ivy keeps Tamarack's original
readings; the player later sends the copied Itasca notes and pump log. The stair
lock, drill motor and cage motor have separate purposes. Ellis directs the player
to the hidden office copy; the office desk grants its evidence after reading,
with shutdown, enforcer victory and release required. Existing event identities
are retained. Publication/readback and all 377 PCM hash/frame/loop checks pass.
The strict v18 parent/frozen-program comparison passed all four cases, including
canonical page validation and rejection of the historical shifted League cycle.
Fresh v18 production checks also pass all three starters through Ivy (including
custom naming), the physical home/desk handoff with Save/Continue and repeat,
the three missing-prerequisite desk refusals, the officer's 2D inspection/reaction
frames, and the gray holding fence. The original ICE builder's independent
whole-data/audio preservation test also passes. Current production stdio passes
with byte-empty stderr. The rebuilt controller also passes eleven core cases,
including the required Mom/New Bark gate and actual battle-turn/PP regressions;
one unrelated, explicitly ignored mart case was not run. Affected-crate all-target Clippy passes with existing
workspace warnings; no warning points to the new dispatch/report leaf modules.
The full v18 late-story replay passes from declared five-badge/level-65
prerequisites through Brainerd, Itasca, the officer, mine, Grand Rapids, Duluth,
intake, rescue/shutdown/release, clearance, all five League matches, Hall of Fame,
credits and actual postgame Continue (2086.01 seconds). This is chapter-fixture
evidence. A separate exact-v18 unboosted course started at New Game without
checkpoint or level overrides. It earned the first two badges, cleared the
headquarters and enforcer, and rescued Ellis. Its first invocation stopped on
an overbroad test assertion that compared party HP across a poisoned walk;
the check now isolates the desk conversation from ordinary walking damage.
The same saved lineage continues in
`.local/dialogue-overhaul/earned-full-v18-continue.log`, has read the physical
copy, earned the Current and Capitol Badges, won the automatically staged depot
courier encounter and defeated Reed in St. Paul. No progression or party state was granted.
It subsequently earned the Wind Badge and defeated both Lake Gym trainers.
That continuation stopped in a replay-helper assumption requiring a carried
Grass/Electric partner; the actual four-member party has neither type. The
helper now prefers such a partner when available and otherwise trains the
strongest actually carried partner through ordinary encounters. The retained
Gym-exit continuation kept both victories, then lost four ordinary Lake Leader
battles. The replay driver now adds ordinary training between losses instead
of repeating unchanged preparation. No balance, level, party or progression
state is granted by that change. These losses are not an earned ending pass.
The remaining earned-campaign and listening gates stay open; the candidate is
unselected. Shared geography has since advanced beyond its frozen Rainy parent;
final integration must retain those accepted later revisions.
The officer now pauses to inspect the records and visibly reacts before granting
the report. This adds no progression identity or optional-notebook requirement.

The native 2D saved-session review also exercised the real Rodio output device
with `CRYSTAL_PREVIEW_AUDIO=1 CRYSTAL_AUDIO_TRACE=1`. Map music and page-confirmation
effects were prepared, started at volume 1 and completed through that backend;
the ordinary review remains muted unless this explicit audio option is enabled.
All three starter reviews also complete through their matching pack cry and
receipt fanfare, with sound completion before subsequent text and actual
repositioning beside Tamarack. Their 2D frame sequences and device traces remain
in the ignored review directory. The closed/open gray fence captures were also
inspected at the same position and scale.
This establishes device dispatch, not a completed listening review. The browser
review exposed a separate boot-font failure: the clock's native ellipsis was
rejected, leaving a blank or partial image while its text state advanced. Its
extra-font rendering fix now passes both native regressions: all introductory
clock pages have actual text pixels, and the autonomous printer finishes at the
production tick rate without a render error. Original startup compatibility and
the original clock/input cases also pass after correcting the title-only test
helper to let SplashScreen finish before skipping CrystalIntro. The native
script yes/no continuation now preserves the selected answer in a checkpoint
copy, validates its script owner, and restores older saves to their original
default YES. The candidate's unanswered rescue NO/Continue/actual-refusal case
passes. Saving does not change the live script state or progression identities.

The fresh native-Rust 2D browser bundle passes the clock introduction, custom
player naming, Mom's visible approach, held movement rejection during her text,
all six pages, her return, in-game Save and reloaded continuation. The browser
warning/error log is empty and the sound gesture unlocked the AudioContext.
The corresponding current native Mom and fence captures were inspected.
The named production MCP regression and byte-empty stderr regression also pass.
Nineteen fresh browser audio/input tests pass, including first-gesture sound
unlock, remembered deliberate mute and the Minnesota 2D policy.
Browser review additionally exposed an overwrite prompt whose final question was
cut off; its source-page/input-ownership repair passes the complete-question,
60-frame held-A and NO/no-write regression. The original 23-clinic Save/cancel
and PC-transfer/autosave/Continue compatibility cases also pass. That early
bundle predates the repair and selected-answer serialization; it remains
historical evidence rather than final browser acceptance.
The next live browser replay found that the Start Save panel still hid the
first existing-file warning while the input boundary waited for it. The
warning now takes the same field-printer priority as PC messages; the extended
regression asserts that the warning is visible before the final overwrite
question and now passes on v19. The rebuilt browser replay also passes: the
warning prints without another input, held A leaves the final question
unanswered, and NO closes it without saving. A separate deliberate YES saves
the isolated review game; reloading the final bundle restores the same house
scene without repeating Mom. Browser warning/error logs remain empty. Only 2D
and TUI are offered, and a canvas gesture unlocks the actual audio output path.
The first current native captures then exposed an autonomous-printer gap:
shell-owned Save warnings can have neither pending core text nor an existing
printer. Controller settling initialized them, concealing the missing native
frame-loop entry. The frame predicate now includes shell-owned notices and Save
questions; the extended regression uses unheld integrated native frames instead
of controller settling. That regression now passes (44.87 seconds). The fresh
native 2D warning capture shows both complete lines and no premature choice.
The blank capture is retained and is not visual acceptance. The subsequent
final-question capture exhausted its field-page readiness predicate: that
question belongs to the shell's Yes/No owner. Its replacement observes that
owner after ordinary inputs; it does not write a cursor or a save.
The first fresh original-composer replay failed because the public current-world
reviser rejected its older North Shore tileset. The original composer now calls
the private guarded scene step; the current-world reviser's terrain checks and
parent allowlist remain in force. Both fresh original replays succeed: the
691-map canonical foundation produces 704 maps, and the documented 716-map
regional-flavor parent produces 729 maps. The room-life preservation test needs
the latter parent; its invocation against the former correctly failed because
that foundation predates the residents. The strict test is retained and passes
on its documented parent (53.14 seconds). The independent v19 regional-builder
equivalence/preservation check passes again (29.52 seconds), using the accepted
Duluth parent and frozen campaign source.
The later harbor capture exposed a separate cached-snapshot defect: after
`closetext`/`pause`/`opentext`, the first page could finish printing in live state
while the renderer retained the closed window. `Write` now invalidates the core
snapshot rather than retaining it as a presentation-only change. The new native
transition case checks the closed pause, unheld continuation, correct cached
label, visible textbox/glyphs and deferred harbor reward. The replacement native
2D directions capture now visibly shows the engineer's full first page. After
unrelated test API errors were corrected elsewhere in the shared checkout, the
frame case reached the correct cached speech but exposed a stale synthetic key
edge in its minimal app, which omits InputPlugin's edge clearing. The test now
clears the released A edge before its unheld frames. Its retry is queued; the
failed image, compilation and harness invocation are retained.
The final browser Wasm bundle includes this snapshot repair. Native/browser
builds, both affected-crate all-target Clippy checks, named production MCP and
byte-empty stderr checks passed in the final integration run. Later replay and
capture-helper changes are undergoing their own checks. Changed leaf formatting
and whitespace checks pass; workspace formatting still fails first in the
pre-existing `offset-allocator/src/lib.rs:408` diff.
Rust also exported six hash/frame/loop-verified playback references outside the
repository. Tool-audio inspection is unavailable in this session, so those
references are not a completed listening review.

The v17 controller matrix passed all sixteen cases: three earned starters with
Ivy delivery; held/diagonal Mom and depot interception; all four lab and ten depot
arrivals; full-pocket aide refusal/Toss/retry; clinic and harbor return branches;
both League lanes; gray-fence release; courier defeat/Continue/reentry; and the
five text-control, pause, sound and acceleration regressions. Production stdio
also passed with empty stderr. A strict read-only v17/v18 comparison found only
the officer's one changed script and its derived command indices; every actor,
event and authored text page remains exact. Current v18 late-campaign and
frame-level officer checks remain separate gates.

The preceding v16 candidate retains identity
`fdd4ebcbf21e6a1a254aa8fb8a23586b9af3fe64309bb472773a0c3502e5ae9f`
and its original 826-page transcript. Its three native starter journeys,
including a custom nickname, clinic return branches, harbor replies, League
lanes, strict source/pack preservation and all 377 audio checks passed. Its
holding-fence rerun failed because the required capture-directory environment
variable was omitted; that invocation is not counted as a behavioral pass.

The preceding full-story candidate is v15. Its parent is the accepted
`minnesota-current-grass-lower-woods-v01` descendant, preserving the newer
Northwoods, Detroit approach and Ely grass, compact coast, Castorock cave,
walking art, original audio, post-Champion Kindlit research offer, Mille Lacs
grass, Grand Marais partner retry, Itasca habitat grass and Preserve Pine
entry grass and Lower Mississippi Woods bank refinement. The final
candidate has native identity
`76382fb9e27cb07ffc8980fd16efa768be90e6fae9c0b78739453c13e5e78c8a`.
It remains unselected while the complete acceptance course is open. Whole-pack
preservation, all three earned starter journeys through Ivy, all 377 audio
hash/frame/loop checks, both canonical League lanes and the gray holding-fence
controller case and production stdio opening with empty stderr pass on this
identity. A separate exact comparison confirms that every map's
scripts, text bodies, objects and events match the frozen v13 campaign across
all 730 maps. The changes outside that frozen campaign are fence art, accepted
parent grass and one floor metatile at the League entrance. Global outdoor
placement passes with the original 0/0 offsets. v13 earned saves keep their
original identity. The latest preservation log is
`.local/dialogue-overhaul/frozen-campaign-and-preservation-v15.log`.

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
supplied progression or level boosts. It has earned seven badges and completed
the Brainerd ranger handoff and Leech Lake witnesses. Four earlier Lake leader
losses were retained; ordinary R06 training and free clinic care prepared its
successful retry. It has copied the Itasca readings and completed the officer's
comparison without the optional Mankato notebook. It defeated the mine foreman
after retaining its losses, ordinary purchased medicine and a different carried
lead. Its depleted team then lost to the next mine guard. Ordinary R09 battles evolved
its caught Cacheek into Furrowbank at level 36. It subsequently defeated the
Shift and Weigh guards before losing to the Vent guard. The next continuation
starts from that actual clinic recovery, retaining every earned victory. The fourteenth
continuation has recovered the instrument, completed the Grand Rapids handover,
and reached Duluth's eighth Gym with the same earned party. The replay retains
that actual clinic recovery and the earned foreman victory; it never clears the
victory flag to rerun the boss or supplies replacement levels, HP or money.
The driver no longer assumes that a partner knowing Cut must have
received the guide's HM: this earned partner learned Cut naturally.
Actual checkpoints and in-scene saves retain the v13 content identity;
frozen earlier candidates' saves are never relabeled. The continuing run is
recorded in `.local/dialogue-overhaul/earned-full-v13.log`,
`.local/dialogue-overhaul/earned-full-v13-continue.log` and
`.local/dialogue-overhaul/earned-full-v13-continue2.log`; the latest continuation
is `.local/dialogue-overhaul/earned-full-v13-continue14.log`. The earlier mine
loss and its actual native recovery are retained in
`.local/dialogue-overhaul/earned-full-v13-continue3.log` and
`.local/dialogue-overhaul/earned-full-v13-mine-recovery.log`.
The fourth continuation exposed a driver omission for an indoor blackout
checkpoint. Production stdio walked out of that actual clinic save into
Bemidji, retained all seven badges and the actual post-loss money, and wrote
the checkpoint used by the fifth continuation. Its stderr remained empty.
The fifth attempt also lost and retained its native blackout checkpoint. The
sixth exposed the same missing indoor-map dispatch in the test driver; the
seventh continues from that actual recovery after ordinary stdio clinic exit.
The driver now handles the indoor map at both dispatch points. Its battle
input helper also excludes disabled moves instead of repeatedly selecting an
unavailable attack. Its mine choices can accept the native Shift offer and use
carried Grass/Ground attacks against the visible matchup. Reentry checks keep
earned pressure controls, witnessed scenes and trainer victories instead of
assuming every resumed chapter is pristine. These changes affect the replay's
ordinary inputs, not trainer teams, game balance or earned state.
The continuing course is earning CACHEEK's level-36 evolution through ordinary
R09 encounters and free clinic care before another mine attempt. Its actual
training checkpoints retain native EXP, PP, evolution and Save/Continue; no
level, money, HP or progression is supplied outside the game.

The replay found a real League road blocker: the camp's exit at (32/33,47) met
road walls at (32/33,0). The v13/v14 local walking repair uses offset 1 and its
reciprocal -1 to meet road floor at (30/31,0). Both lanes, denial before shutdown,
Continue and repeated production stdio crossings pass, but a subsequent
cross-publisher check found that those offsets contradict the global outdoor
placement cycle through R12. The final publisher now checks complete regional
placement before and after revision. v13/v14 remain frozen review evidence;
v15 extends the adjacent native road floor by one metatile and restores the
canonical 0/0 offsets. Strict preservation, both clearance lanes, denial before
shutdown, the return walk and Continue pass on that repair.
The external historical-pack regression confirms the v14 placement error and
proves that the publisher rejects it before mutation. It passes in
`.local/dialogue-overhaul/shifted-league-cycle-guard-v14.log`.

Native 2D captures of the road, intake encounter, released Kettlurk,
crew response, harbor return, completed Champion dialogue and actual earned
Minneapolis coercion were inspected. Actual earned Itasca and officer handoff
captures were also inspected. The initial full-frame review misread the officer
capture as missing the player. Enlarged crops and exact GPU pixel comparisons
confirm that the native red player is present and unchanged before and during
the conversation, in both the original and rebuilt captures. The actor also
remains present, opaque and stationary for 120 keyboard-driven frames with both
cold and settled starts. Both regressions pass together in
`.local/dialogue-overhaul/earned-officer-actor-and-pixels-v13-final.log`; no rendering
change was needed.
The complete readable campaign transcript remains ignored at
`.local/dialogue-overhaul/complete-campaign-transcript.md`.

Full earned-campaign completion, native/browser listening and normal-speed
audio recordings remain open. PCM verification, an unlocked browser sound
control and successful source dispatch do not prove audible device playback.
The selected regional pack is not advanced by this work.

Affected-crate all-target Clippy completed after the v15 geometry and retry
regressions, with existing workspace warnings. No warning was introduced in
the revised story leaf modules. The final log is
`.local/dialogue-overhaul/final-clippy-v15-v04.log`.
Workspace formatting check reports existing failures beginning in
`crates/offset-allocator`; new leaf modules are formatted. Verification packs,
saves, the full readable transcript and captures remain ignored under
`content-packs` and `.local/dialogue-overhaul`.

## Gray holding fence follow-up

The user's latest instruction replaces the solid cell barrier with a gray
fence. The shared original/final Rust builder copies native Johto Modern metal
rail pixels into a private holding-office tileset. Five room cells and the
existing gate's collision quadrants change; other room cells, original tileset
files, actors, warps, story programs and event identities remain preserved.
The [holding fence contract](../planning/minnesota/revisions/dialogue-overhaul-v01/holding-fence.md)
gives exact source blocks and positions.

On v14, full enclosure/open-gate collision analysis and strict whole-pack
preservation pass. The production controller verifies separate power and
battle prerequisites, refusal, unchanged closed gate until the release text is
acknowledged, Ellis walking out, the open gap, repeat interaction and Continue.
A separate courier fixture loses an actual native battle, recovers, Continues
and reenters the automatic staged encounter. All 377 audio PCM/hash/frame/loop
checks pass on v14. Native closed, release-conversation and open captures plus
a complete source room render were inspected in 2D. These are declared chapter
fixtures, not earned campaign evidence.

## 2D browser review

The browser build compiles only `bevy-shell,fullscreen-scaling`; it does not
compile the voxel feature. The v15 build passed with:

```sh
WASM_BINDGEN="$PWD/.local/dialogue-overhaul/tools/wasm-bindgen-0.2.128-aarch64-apple-darwin/wasm-bindgen" \
  CRYSTAL_PACK=content-packs/minnesota-dialogue-overhaul-v15.crystalpack \
  CRYSTAL_WEB_OUT=target/dialogue-overhaul-2d-web-v15 sh tools/dialogue-2d-web.sh
```

The official CLI version must match the locked Rust wasm-bindgen version
0.2.128. The script checks this and writes only ignored local review output.
Both browser pack inputs are byte-exact copies of the frozen v15 native pack,
SHA-256 `ac0862307380c5481504396c0c7192e02fdfe5a020a4e066e026cfb761a7a5d4`.
All 377 original audio programs pass on that input.
The build log is `.local/dialogue-overhaul/browser-build-v15.log`.
The v15 bundle was served in a fresh local data directory on port 3011 and
cold-started in the hidden browser through title, trainer choice and the native
clock introduction (including the native Zzz text) in 2D. The sound gesture unlocked and the observed browser
warning/error log was empty. This is a startup check, not a browser campaign
or a listening review.
The earlier v13 local review server was started with:

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
