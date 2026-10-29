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
  content-packs/minnesota-current-fossils-repaired-r15-v01.crystalpack \
  planning/minnesota/story.json \
  content-packs/minnesota-dialogue-overhaul-v03.crystalpack
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
unless the source's held-button acceleration applies. Audio cues fence printing
until playback finishes. Renderer-neutral settling completes the device fence
without changing the script's required acknowledgement.

The existing Fast/Mid/Slow character delays and saved options remain in force.
Acceleration cannot also acknowledge a newly completed page or answer a later
choice. Dialogue diagnostics remain opt-in through `CRYSTAL_DIALOGUE_TRACE`;
production terminal stderr must stay empty.

## Acceptance course

External pack prerequisites are read at runtime. None are embedded in tests.

```sh
export CRYSTAL_STORY_REVISION_PARENT="$PWD/content-packs/minnesota-current-fossils-repaired-r15-v01.crystalpack"
export CRYSTAL_WINONA_TEST_PACK="$PWD/content-packs/minnesota-dialogue-overhaul-v03.crystalpack"
export CRYSTAL_RENDER_TEST_PACK="$CRYSTAL_WINONA_TEST_PACK"
export CRYSTAL_WINONA_SAVE_DIR="$PWD/.local/dialogue-overhaul/saves"
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

## Verification results

Candidate v03 has 730 maps and content identity
`c4db1f521c05c7abdd4489e5da8eea3bee028593cb3e297c3d511ab790c9fe0d`.
Its parent is the selected fossil/progression descendant, identity
`53917ea37309b35a3eff89340608f9e118dc568346a4fc258ce328a092e883af`.

Passed: three earned starter journeys through Ivy's delivery; four declared
lab carpet arrivals; ten declared depot arrivals; source text controls,
30-frame pauses, 20-frame paragraphs, two five-frame scroll stages, held-key
acceleration, ordered non-silent sound playback preparation; strict preservation
of geography, catalogs, original event identities and unrelated rooms; all 377
original audio PCM hashes/frame counts/loop ranges; original Mom/gate and battle
controller regressions; production MCP movement/Start and clean-stderr checks.
Native 2D captures of Mom, Tamarack and Reed's placed case were inspected.
The new stdio opening replay also checks exact pages and movement locks:

```sh
.local/minnesota-review-env/bin/python \
  planning/minnesota/tools/check_dialogue_overhaul_mcp.py \
  --client target/northstar-story/debug/geothite \
  --pack content-packs/minnesota-dialogue-overhaul-v03.crystalpack
```

The full earned campaign is still under validation. Its first replay stopped in
the existing driver's night recruitment strategy after two badges; the revised
driver is replaying from New Game with ordinary captures and no level boosts.
Native/browser listening and complete normal-speed scene recording remain open.
No candidate has been promoted.

Affected-crate all-target Clippy completed with existing workspace warnings;
introduced warnings were corrected. Workspace formatting check reports existing
failures beginning in `crates/offset-allocator`; new leaf modules are formatted.
Verification artifacts remain ignored under `.local/dialogue-overhaul`.
