# Concise dialogue review

This revision reviews the Fair release's 3,639 referenced Minnesota passages,
including towns, trainers, services, optional discoveries, the main story and
return conversations. It rewrites 1,625 passages across 318 maps and five welcome
texts. The changed existing map passages use 4,294 pages instead of 7,295.
The two new south-warden replies add five pages.

Tamarack gives a shorter introduction and a concrete reason for the Rochester
delivery. Reed identifies himself and his search for work before discussing the
case; the case's contents remain uncertain until Tamarack opens it. Ellis, Jo,
Voss and Hale receive context when introduced. Evidence handoffs distinguish the
ring left on the desk from the stamp the player copies. Town and trainer replies
keep their local details without the long anecdote endings.

Winona's south warden has a separate conversation before and after the starter.
Both road triggers retain their existing checks, movement and input handling.
The St. Paul and Safari pairs offer different information. The 22 old, removed
road checkpoints are not restored.

`story.json` owns the pages in `dialogue_overhaul.texts`. Its
`dialogue_clarity_revision` selects the final publication labels; current story
overlays contain matching wording. Historical revision inputs remain frozen.
The Rust publisher replaces only those text bodies, the welcome text, and the
south warden's conversation wiring. It preserves existing text terminators and
unrelated game data, including Johto/Kanto, parties, art and audio.

## Build and review

The exact parent is the selected Fair release:
`content-packs/minnesota-fair-release-v01.crystalpack`, SHA-256
`18003b91597173f1d98937a0f6cc2fa82cfd2c5b3612c9803cd5f615acad2a73`.
Its hosted counterpart is `target/fair-release-20261120/hosted.crystalpack`.
Both parent content identities are checked by the publisher.

From the repository root, choose a fresh ignored output path:

```sh
CARGO_INCREMENTAL=0 cargo build --locked -j1 -p crystal-assets \
  --bin revise_northstar_dialogue_clarity --bin verify_crystalpack
target/debug/revise_northstar_dialogue_clarity \
  content-packs/minnesota-fair-release-v01.crystalpack \
  planning/minnesota/story.json \
  content-packs/minnesota-dialogue-clarity-v04.crystalpack
target/debug/verify_crystalpack \
  content-packs/minnesota-dialogue-clarity-v04.crystalpack --desktop --audio
target/debug/geothite play content-packs/minnesota-dialogue-clarity-v04.crystalpack \
  --save target/dialogue-clarity-20261120/manual.crystalsave
```

For the hosted edition, run the same publisher over the exact hosted parent and
write `target/dialogue-clarity-20261120/hosted-v04.crystalpack`.

This is an **unselected local review candidate**, not a public release. Existing
sessions and saves stay on their original packs. The candidate deliberately
omits the Fair release's migration policy, whose target identity would be wrong
for this revision. Test it with a new save; do not relabel existing saves.

## Verification

The publication regression checks every selected text against the authored
pages, verifies distinct warden branches, and compares the remaining game data
with the parent. The page regression checks all canonical native text.

```sh
CRYSTAL_DIALOGUE_CLARITY_PARENT="$PWD/content-packs/minnesota-fair-release-v01.crystalpack" \
  CARGO_INCREMENTAL=0 cargo test --locked -j1 -p crystal-assets --lib \
  dialogue_clarity_preserves_game_and_installs_every_reviewed_page -- --ignored
CARGO_INCREMENTAL=0 cargo test --locked -j1 -p crystal-assets --lib \
  canonical_dialogue_fits_native_pages
python3 planning/minnesota/tools/check_dialogue_overhaul_mcp.py \
  --client target/debug/geothite \
  --pack content-packs/minnesota-dialogue-clarity-v04.crystalpack
```

Python review tools require Pillow and cbor2. Review logs, controller saves and
native page sheets stay ignored under `target/dialogue-clarity-20261120`.
A full dialogue reading and exact installed-page comparison are distinct from a
complete earned campaign playthrough.

The final raw candidate's SHA-256 is
`f4a3a31897bd005983e83d26af9b46e540520701c65cef90fd192d446113712c`;
the hosted candidate's is
`adfe5087d4ef607bb2bd9af7882e841705c7da5538b08e8435f8fd0d10caba67`.
Both pass pack verification. All 1,625 selected passages match the installed
pages exactly. The preservation regression passes, including all 602 unchanged
audio hashes, frame counts and loop ranges.

Production-controller checks pass for the opening, all three earned starter
choices, Reed's first battle and Ivy's delivery; both wardens before and after
the starter; Tamarack's milestone guidance and declined egg; and the Brainerd
and Bemidji gym directions. The fossil hunter's conversation, choices and full
return route also pass. Milestone-return, gym and fossil checks use declared test
fixtures; the three starter journeys begin at New Game. These are targeted
journeys, not a full earned campaign replay.

The native renderer also passes read-only captures of 19 reached opening pages
and ten warden pages without advancing the conversation. Representative
Tamarack, river-reading, guard and Brainerd pages were visually inspected at
160×144. These are production-renderer captures from saved controller states;
they are not screenshots of a running desktop window. Static sheets separately
check the installed font/frame layout for the opening, guards and handoffs.

Run the external-pack controller regressions with the repository's larger test
thread stack:

```sh
export CRYSTAL_WINONA_TEST_PACK="$PWD/content-packs/minnesota-dialogue-clarity-v04.crystalpack"
export RUST_MIN_STACK=33554432
CARGO_INCREMENTAL=0 cargo test --locked -j1 -p crystal-bevy --lib \
  winona_wardens_explain_partner_requirement_and_remain_after_starter -- --ignored
CARGO_INCREMENTAL=0 cargo test --locked -j1 -p crystal-bevy --lib \
  overhaul_three_earned_starters_and_ivy_delivery -- --ignored
CARGO_INCREMENTAL=0 cargo test --locked -j1 -p crystal-bevy --lib \
  story_review_tamarack_ -- --ignored --test-threads=1
CARGO_INCREMENTAL=0 cargo test --locked -j1 -p crystal-bevy --lib \
  fossil_hunter_cliff_conversation_and_route_remain_accessible -- --ignored
```

Formatting, scoped diff checks and `cargo clippy --locked -j1 -p CRATE
--all-targets` pass for `crystal-assets` and `crystal-bevy`. Existing lint warnings
remain elsewhere; none point to the dialogue publisher or changed dialogue tests.

Cleanup removed 373.5 MiB of temporary review files and superseded candidates,
plus the publisher's disposable Cargo output; the volume retains 337 GiB free.
Shared Cargo output stays in place for the other work and running native client.
No separate target directory or new incremental cache was created.
