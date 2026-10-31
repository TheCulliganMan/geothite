# Pokémon Northstar opening

Minnesota now opens directly at the production New Game/Continue menu, without
a title card, splashes, movie or Start prompt. B remains in that menu. This
runtime rule applies to Minnesota content regardless of pack filename or title
art; existing packs need no rebuild. Continue, options and new-game setup still
use their production flows. Other regions retain their authored startup.

The opt-in `northstar_title_and_tamarack_opening` regression now checks immediate
menu entry, idle/B behavior and the Tamarack welcome through Winona arrival.
`minnesota_direct_menu_preserves_continue` checks the immediate menu with a real
save, summary cancellation and loading the saved trainer and position. Both use
`CRYSTAL_NORTHSTAR_TITLE_PACK`; the Continue case also supports older Minnesota
packs without regional opening art.

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

The selected final local pack is `content-packs/minnesota-northstar-title-v03.crystalpack`
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
