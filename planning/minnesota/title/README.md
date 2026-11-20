# Pokémon Northstar opening

The revised Northstar pack plays the original Geothite opening movie inside
the game, using its existing native interpreter, artwork, music and timing.
It then stages a three-second Northstar title reveal: the star illuminates,
Kettlurk gathers its current and opens both arms, and the complete NORTHSTAR
name arrives with a lettering shine. Native palette steps reveal the whole
creature together. Its idle performance coordinates a rigid head dip, chest
sway and arm gestures around fixed legs, with circulating body currents and
continuously falling water. The reveal runs once;
returning from the menu keeps the completed composition. The movie keeps its
normal skip controls; A or Start at any point in the title reveal opens
the production New Game/Continue menu. Returning from the menu does not replay
the movie. `GEOTHITE_REDUCED_MOTION=1` starts on a still title; the graphical
browser reads `prefers-reduced-motion`. There is no external video player.

Tamarack's New Game welcome introduces Jiglet through a discovery of her own.
The playable opening gives Mom a clear Pokégear handoff, keeps the three starter
personalities distinct, and carries Reed's case exchange into the first battle
and Tamarack's care afterward. All text lives in `planning/minnesota/story.json`:
`intro_revision` selects the welcome and field labels, and
`dialogue_overhaul.texts` owns their pages. The two existing story-flow overlaps
are synchronized there as well.

The scoped Rust publisher preserves every scene, battle, map layout, creature, existing
runtime asset and audio program. Its only additional runtime file is the opt-in
marker `data/title/northstar-opening-v1.json`. Existing packs without this marker
retain their direct New Game/Continue startup. Other regions retain their own
startup. The terminal clients still enter the playable game directly.

## Build and review

The intro build is `content-packs/minnesota-northstar-intro-v03.crystalpack`,
built from the Headwaters Journey/Mothlight parent. Later selected descendants
retain this opening; resolve `planning/minnesota/world.json` for the current
selection. The title animation runs from these existing pack resources and
does not require republishing their art or changing save identities. It is not a public release;
existing player saves retain their original identity.
The [frozen build recipe](intro-build.json) binds the parent and resulting pack
hashes to the canonical dialogue selection. Use a fresh output path when
repeating the publisher:

```sh
CARGO_INCREMENTAL=0 cargo build --locked -p crystal-assets \
  --example pack_northstar_intro --bin verify_crystalpack
target/debug/examples/pack_northstar_intro \
  content-packs/minnesota-mothlight-battle-cafe-headwaters-journey-v01.crystalpack \
  planning/minnesota/story.json \
  content-packs/minnesota-northstar-intro-v03.crystalpack
target/debug/verify_crystalpack \
  content-packs/minnesota-northstar-intro-v03.crystalpack --audio --desktop
```

The v03 build/readback and all 602 PCM hash, frame-count and loop-range checks
pass. Its SHA-256 is
`b5decf90a6a45cd87549a5f26820962ef4a521bac1246e423ccfba5ae8394afb`;
content identity is
`71d27e7ff029521aa17d18265cebb13e7467e13a28c0a5a80cada5f22e3ee8f0`.
The preservation test restores only selected text and the marker, then requires
complete equality with the parent. The Station Return candidate passed all four
starter approaches, including its new Pouchid aisle route. Canonical native page
bounds and current-pack production stdio Mom/Pokégear/Tamarack/starter-portrait/
refusal checks pass.

The earlier v01 text-identical candidate also passed the earned three-starter
journeys through Reed and Ivy, Mom's repeat/Start/Continue course, the egg
refusal/retry/Continue course, a real rival loss and supplies on return, and all
four lab entrances. These tests use installed dialogue and ordinary controller
inputs; the rival-loss setup begins immediately after the starter gift.
Generated packs, saves, frames and logs stay ignored under `target/` and
`content-packs/`.

Build and launch the native game with the new opening and an isolated save:

```sh
CARGO_INCREMENTAL=0 cargo build --locked -p crystal-bevy --bin crystal-bevy
CRYSTAL_PREVIEW_AUDIO=1 target/debug/crystal-bevy \
  --pack content-packs/minnesota-northstar-intro-v03.crystalpack \
  --save-path target/northstar-intro-20261119/local-game.crystalsave
```

The v03 integrated native regression plays the complete original source movie,
checks its music queue and native scene surfaces, then verifies title motion,
menu cancellation, the complete Tamarack welcome, new-game arrival and Start.
Separate native-key tests cover A/B/Start skip without consuming a second menu
action. A real `GEOTHITE_REDUCED_MOTION=1` initialization skips the movie, retains
identical title pixels and still accepts Start. The original movie and its
cleanup own their existing timing; the Northstar clock only animates title art.
The v03 Continue course also passes, preserving the saved player, map, position
and host clock. The final production executable was launched on macOS: its
original studio/movie scenes, transition into the animated Northstar title,
physical Start input, menu and B return were visually verified in the actual
game window. Whole-workspace formatting and affected-crate Clippy pass, with
remaining repository warnings outside this change. Build output stays retained
while the local game is running.

```sh
CRYSTAL_NORTHSTAR_TITLE_PACK="$PWD/content-packs/minnesota-northstar-intro-v03.crystalpack" \
CRYSTAL_NORTHSTAR_REVIEW_DIR="$PWD/target/northstar-intro-20261119/original-in-game" \
RUST_MIN_STACK=67108864 CARGO_INCREMENTAL=0 \
cargo test --locked -p crystal-bevy --lib northstar_title_and_tamarack_opening \
  -- --ignored --nocapture --test-threads=1
```

The standalone legacy Johto title test still requires its absent historical
`apps/web/assets/data/content-packs/core-modular.crystalpack` fixture. The new
Northstar regression executes the original movie from the verified current pack.

### Animated title follow-up (2026-11-19)

The three-second entrance and coordinated head, chest and arm animation were
verified against `content-packs/minnesota-intro-fen-clearings-v01.crystalpack`
(SHA-256
`73a502444fc02ba16b61a054ff2505b9fedd55a5687a3ad5c0c0cecda869f11e`).
Its original movie/title programs, 103 opening resources, audio and opening
dialogue match the v03 build. This follow-up changes the Rust compositor and
presentation clock; it does not rebuild the pack or migrate saves.

The full original-movie/New Game course, all three native skip buttons,
confirmation during the reveal, menu return, reduced-motion initialization and
Continue pass. Two complete ambient cycles retain planted feet, the readable
prompt, bounded image allocations and unchanged gameplay state. Native captures
show a 4,906-pixel reveal and up to 2,785 changing hero-area pixels. The head
and chest outline checks change 520 and 448 pixels respectively, independently
of material colors. All 37 production captures exactly match the reviewed
compositor frames. Independent visual inspection checked the connected
silhouette, fixed feet, empty torso and readable text at native size and 3×.
The rebuilt desktop client was observed playing the original movie, skipping
into the entrance, completing the title reveal, accepting Start and returning
with B. The local review app remains open; this is not a public or browser
deployment.

Whole-workspace formatting and affected-crate all-target Clippy pass, with
existing repository warnings and no diagnostics in the title modules or test.
Evidence stays ignored under
`target/northstar-intro-20261119/title-performance/`. The shared build cache
remains in use by the running game and other work. The inactive preview helper
and its executable were removed (1.42 MiB recovered); retained frames document
the visual review. The last disk check showed 353 GiB free.

## Historical title-art publication

The earlier Minnesota opening used one still 160×144 native title card: Kettlurk over
muted new Minnesota flag geometry, with an eight-point star, split falls,
original pixel lettering and a permanent Start prompt. The flag reference is the
[Minnesota Secretary of State's official design](https://www.sos.mn.gov/about-minnesota/state-symbols/state-flag).
No copyright/studio splash, Crystal movie or idle attract-mode restart is shown.
Other packs retain their authored source opening.

Professor Tamarack introduces Minnesota, a new field survey and the Winona field
station using her accepted native portrait. The showcase is **Jiglet**, the small
single-hook first stage of the Jiglet → Trollure → Crappeller family. The native
Jiglet portrait and matching cry come from the parent pack. Naming, gender,
clock confirmation, menu input and first-map arrival retain their production
owners. Internal source-compatible catalog IDs remain stable.

The Rust publisher accepts the currently selected Minnesota pack and the
accepted Tamarack source, and requires a fresh ignored output path:

```sh
CARGO_TARGET_DIR=target/northstar-story CARGO_INCREMENTAL=0 \
cargo build --locked -p crystal-assets -p crystal-bevy \
  --bin pack_northstar_title --bin crystal-bevy \
  --features crystal-bevy/location-tester
./target/northstar-story/debug/pack_northstar_title \
  content-packs/minnesota-iron-crosscut-bends-v01.crystalpack \
  planning/minnesota/npcs/tamarack-ice/01-tamarack/native-pixels-v4.json \
  content-packs/minnesota-northstar-title-v03.crystalpack
```

The builder preserves every parent runtime resource, every complete map and all
audio bytes/contracts. It installs only the title image and Tamarack portrait
resources and revises the New Game presentation catalog. It validates native
text widths, the active roster, the resulting pack and its storage round trip.
Packs, captured frames, temporary executables and verification logs stay ignored.

The opt-in graphical input/compositor regression uses the completed pack:

```sh
CRYSTAL_NORTHSTAR_TITLE_PACK="$PWD/content-packs/minnesota-northstar-title-v03.crystalpack" \
CRYSTAL_NORTHSTAR_REVIEW_DIR="$PWD/.local/northstar-title-review/frames" \
RUST_MIN_STACK=67108864 CARGO_INCREMENTAL=0 \
cargo test --locked -p crystal-bevy --lib northstar_title_and_tamarack_opening \
  -- --ignored --nocapture --test-threads=1
```

The test uses real keyboard scheduling, native title/menu/clock/dialogue
compositors, a full idle timeout, menu cancellation and reopening, New Game,
professor/showcase pages, naming, and the first Winona arrival and Start menu.
MinimalPlugins has no audio device; the test acknowledges the cry-completion
fence after its presentation wait. The separate Rust pack verifier checks all
PCM hashes, frame counts and loop ranges. This is local opening verification,
not an audio listening review or a continuous campaign acceptance claim.

Validation notes: all-target Clippy for crystal-assets, crystal-runtime and
crystal-bevy (including location-tester) passes with existing warnings. The new
leaf modules pass rustfmt. Workspace-wide `cargo fmt --all -- --check` still
reports existing formatting differences. The original Crystal New Game/gender/
clock regression passes. Its separate original title attract-timeout assertion
fails against the shared TUI pack in both the current binary and the pre-task
2026-10-31 04:33 controller binary; that existing failure is outside this regional
still-title change. The Minnesota still-title/intro regression passes.

That historical verified pack is `content-packs/minnesota-northstar-title-v03.crystalpack`
(content identity `cccc5efd93832fa2f8353e2f564dd4e1ae70623577d50c4620cd43069edc9b47`).
A fresh replay of the Cargo-built Rust publisher is byte-identical to that pack.
The rebuilt production Bevy shell ran offscreen at 640×576; its title has zero
pixel differences from the reviewed native LCD at 4× nearest-neighbor scale.
All 25 captured title/menu/clock/welcome frames were visually inspected. The final
wording was replayed through the full New Game course and its changed page was
inspected again. All 377 audio programs pass hashes, frame counts and loop ranges.

Launch the retained local client with audio:

```sh
CRYSTAL_PREVIEW_AUDIO=1 ./.local/northstar-title-review/crystal-bevy \
  --pack content-packs/minnesota-northstar-title-v03.crystalpack
```

The retained screenshots are under `.local/northstar-title-review/final-frames/`
and `.local/northstar-title-review/native-title.png`. They are local review
evidence, not shipped assets. The client and all generated packs remain ignored.

The [full fauna/progression successor](../pokemon/acquisition-v01/encounter-progression-v01/README.md)
retains this exact title and welcome through the same publisher. Resolve
`world.json` for the current selection; the title-only pack above remains its
verified stage record.
