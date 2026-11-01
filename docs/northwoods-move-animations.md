# Northwoods move animations

The Northwoods animation overlay gives fourteen imported attacks their own
authored Gen II programs and preserves the five Minnesota signatures. It is
applied after the completed Fairy v2 pack. Move facts, effects, learnsets,
species, the type chart, runtime files and compiled audio stay unchanged.

The implementation lives in `crates/crystal-assets/src/northwoods_move_animations.rs`
and its `northwoods_move_animation_programs.rs` leaf module. New stationary
objects reuse native 2bpp tiles, framesets, palettes and sound assets. The
production command compiler, object VM and battle rasterizer play the programs;
the website uses their native 160×144 output.

| Move | Authored action |
| --- | --- |
| Water Pulse | Successive water rings travel to the target. |
| Aqua Jet | A short water burst leaves a fast wake and splash. |
| Ice Shard | A solid shard advances and breaks into frost. |
| Energy Ball | Leaves gather around an outward-moving green orb. |
| Drain Punch | A fist strikes, then green motes return to the user. |
| Signal Beam | Alternating green and blue segments follow a zigzag. |
| Iron Head | A metallic ram charges and sheds impact sparks. |
| X-Scissor | Two slashes cross over the target. |
| Fairy Wind | Curls and sparks sweep from user to target. |
| Disarming Voice | Sound arcs advance and strike the target. |
| Dazzling Gleam | Diamond rays expand around the target. |
| Play Rough | Alternating palms and fists surround the target. |
| Moonblast | A charged moon projectile arcs into a burst. |
| Poison Jab | A pointed thrust scatters venom-colored impact drops. |

These attack graphics do not promise an optional secondary effect. Drain
Punch alone includes the return motion that represents its guaranteed drain.
The Fairy attacks use damaging target-directed sequences rather than healing
or accuracy-status donor programs.

## Build

Packs remain ignored and outside the shipped website. The presentation overlay
requires the completed 270-move Fairy pack, refuses a second application, and
checks the written pack identity after publication to disk.

```sh
CARGO_BUILD_JOBS=4 CARGO_TARGET_DIR=target/agent-northwoods-publish \
cargo build --locked -p crystal-assets \
  --bin pack_northwoods_animations --bin export_minnesota_pokedex_web \
  --config 'profile.dev.package.crystal-assets.opt-level=0' \
  --config 'profile.dev.package.crystal-core.opt-level=0'

target/agent-northwoods-publish/debug/pack_northwoods_animations \
  content-packs/northwoods-fairy-v2.crystalpack \
  content-packs/northwoods-animated-v2.crystalpack
```

The verified v2 output contains 471 species and 270 moves. Its SHA-256 is
`e61681219987003565ead09fa91b993201322b7f7bcc98eb6a41a96309b23bd5`.
The retained v1 draft uses a smaller Ice Shard projectile; v2 uses a broad blue
native 2bpp shard. All earlier review renders remain available.
Both overlay tests pass, including exact gameplay/audio preservation and
rejection of repeated application. Build logs and retained visual work live
under `planning/minnesota/renders/moves-animation-20261027-v1/`.

## Verification and retained captures

Production controller tests pass for all fourteen imported attacks and all five
signatures on animated v2. Legal regional learners select the move through
FIGHT, spend real PP, damage the target and queue the exact authored program.
The separate five-Fairy controller regression also passes. Both overlay tests
pass against the completed Fairy base.

Root and independent visual review accept all fourteen imported presentations.
The small first-draft Ice Shard remains in `review-v2/`; its accepted correction
is in `ice-review-v3/`. Failed harness captures and their diagnostics are
retained too. The corrected harness constructs the production battlers and HUD
fresh for every frame.

The native capture regression checks both battle sides with normal and shiny
actors, nonempty custom OAM, distinct visible effect frames, exact native
mirroring, fixed effect palettes, at most 40 object pieces and 10 per scanline,
binary alpha, three visible inks per object and complete cleanup. APNG decoding
must reproduce every captured native frame at the production Game Boy clock.
Per-variant keyframe JSON retains the effect hash, frame counts and authored
contact indices; the website's idle still copies that exact contact PNG.
Older capture directories without metadata retain their frame-zero fallback.

Scoped formatting and exporter tests pass. Full-workspace formatting still
fails on existing `offset-allocator/src/lib.rs:408` formatting; Clippy stops on
the existing `crystal-core/src/multiplayer/mod.rs:1332` `bad_bit_mask` error.
Those unrelated failures remain in the verification logs.

For a fresh capture directory, the production regression can be rerun with:

```sh
CRYSTAL_RENDER_TEST_PACK="$PWD/content-packs/northwoods-animated-v2.crystalpack" \
CRYSTAL_RENDER_TEST_HEADLESS=1 CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image \
NORTHWOODS_ANIMATION_RENDER_DIR="$PWD/planning/minnesota/renders/moves-animation-fresh" \
CARGO_BUILD_JOBS=2 CARGO_TARGET_DIR=target/agent-northwoods-animation \
cargo test --locked -p crystal-bevy --lib \
  northwoods_moves_render_native_frames_both_sides_and_cleanup \
  --config 'profile.dev.package.crystal-bevy.opt-level=0' \
  --config 'profile.dev.package.crystal-bevy.debug=0' -- --nocapture
```

`NORTHWOODS_ANIMATION_MOVES` optionally selects comma-separated move IDs for
disjoint capture batches. Unknown, empty and repeated IDs fail validation.
Every output directory must be fresh, so a rerun cannot overwrite retained work.
`NORTHWOODS_ANIMATION_REVIEW_ONLY=1` produces player-normal design-review clips;
those clips do not establish the complete four-variant verification.

## Published website

Release `20261027-northwoods-12` is live at
<https://geothite.ryanculligan.com/pokedex/moves.html>. All nineteen catalog
cards and 141 genuinely learned extension-move cards use the packed learnsets.
The 76 animations retain 5,484 native frames, 76 contact strips and 76 uniform
filmstrips. All nineteen player-normal effect hashes differ; each clip has at
least seven distinct effect frames. The final 76 APNGs and 76 contact stills
pass exact capture-byte checks.

Actual browser controls pass at 1280px and 390px, including side/shiny/Play/Stop,
download parity, single-active playback, visible mobile Moves navigation,
real APNG motion and Pokémon-page previews. Native Psychic move aliases display
the canonical Psychic badge without changing packed data. All 2,038 public
exact-byte checks and six health/private-route checks pass. Existing service
configuration is preserved. All 1,252 sprite PNGs match the separately approved
sprite publication11 byte-for-byte.

The final web integration suite, thirteen focused exporter tests and scoped
formatting pass. Release11 remains the local checkpoint before the Psychic
display-label correction; its animation assets and browser code are identical
to release12. The unregistered bat proposal is separate from this publication.

```sh
target/agent-northwoods-publish/debug/export_minnesota_pokedex_web \
  content-packs/northwoods-animated-v2.crystalpack \
  planning/minnesota/pokemon target/minnesota-pokemon/native \
  planning/minnesota/pokemon/species-profiles.json \
  target/minnesota-pokedex-web/pokedex/releases/20261027-northwoods-12 \
  /pokedex/releases/20261027-northwoods-12 \
  planning/minnesota/renders/moves-animation-20261027-v1/captures

POKEDEX_WEB_RELEASE=target/minnesota-pokedex-web/pokedex/releases/20261027-northwoods-12 \
POKEDEX_ANIMATION_SOURCE=planning/minnesota/renders/moves-animation-20261027-v1/captures \
node web-client/pokedex/app.test.mjs

bash tools/deploy-pokedex.sh 100.101.29.133 \
  target/minnesota-pokedex-web/pokedex/releases/20261027-northwoods-12 \
  20261027-northwoods-12
```

Use a new release ID for any further export or publication. Build products,
packs and verification output stay ignored; retained render sources are archived.
