# Immersive battle presentation

The optional modeled view now includes a dedicated 3D battle arena. The same
production `VisibleShellController` still owns FIGHT / PKMN / PACK / RUN,
selection, PP, HP, damage, status, switching, capture, rewards, dialogue and
return to the map. There is no renderer input handler or alternate turn engine.
F3 / the existing 3D setting changes presentation only.

## Presentation boundary

`capture_presented_battle` runs immediately after `render_playfield` selects its
current snapshot, including a retained battle-message scene. It publishes the
species in the actual active party slot, visible transformation, and the current
source images. It never requests a fresh authoritative snapshot. A resolved turn
can be ahead of its dialogue; the renderer cannot reveal that future state.

`VisualBattleFrame` in `crystal-render-api` is read-only presentation data. Its
bounded cues come from existing visible animations: moves, HP-loss impact,
fainting, sending out, withdrawing, and ball throws/deflection. A move cue is not
evidence of a hit. Only visible HP loss causes impact/recoil. Capture cues expose
only the current visual phase, never a predicted success or a new outcome.
Animation completion does not call gameplay code.

## Compositing and interruptions

- The 3D arena, its camera, lighting and effects use render layer 29
- The modeled overworld uses layer 31 and turns off during battles
- The classic camera continues to render HUD, text, menus and fade overlays on 0
- Original HUD glyphs are compactly re-anchored to backed top-corner panels;
  original command/dialogue sprites are docked below the combatants. No values
  are recreated. Their source transforms are retained and restored on F3
- The redundant seven-column “What will…” main-menu prompt is omitted because
  it splits long nicknames; the active name remains readable in its HUD and all
  four production choices are retained
- Only the replaced white battle canvas, battler art and animation objects are
  parked on hidden layer 30
- Source HUD eraser rectangles also hide the same underlying source-coordinate
  HUD regions, so temporarily erased or fainted HP displays do not reappear
- Trainer portraits/sliding introductions retain their original presentation;
  the 3D arena takes over after those existing presentation stages finish
- The regular party, item and summary surfaces retain their original controls
- Title, encounter transition and post-capture Pokédex paths explicitly retire
  a retained arena even when they return before normal scene extraction
- Disabling 3D immediately restores the complete classic battle layers

## Art scope and honest fallback

Exact species meshes are shared with the authored actor catalogs. The renderer
reports `BattleViewStatus.modeled_species` separately from `source_art_species`.
Generic icon families are never substituted for an unmodeled exact species.
Unknown species retain the live game-pack front/back art as a billboard in the
arena. Shiny, Substitute and Minimize currently retain their correct source art
rather than receiving an inaccurate normal-palette species model.

Original procedural environments include meadow, forest, cave, water,
interior and ice. Their editable palette source is
`art/battles/arena-palettes.json`; their deterministic mesh generator is
`arena_mesh` in `crates/crystal-voxel-view/src/battle_view.rs`. The same source
contains the small original capture-ball mesh. Arena trees reuse the original
editable New Bark tree kit. No ROM, content pack or extracted game data is
committed. `tools/validate-battle-art.py` validates the palette source and source
boundary without needing a game pack. Authored species sources/generators are
listed in the actor catalog documentation.

Effects use a fixed pool of 32 particles. Arena meshes are rebuilt only when
context changes and retired meshes are removed. Species meshes are cached by
exact identity. Camera motion is intentionally slight to keep the production
HUD and choices readable. Move effects are elemental presentation classes,
not a claim of individually authored choreography for every move.

## Verification

Run source validation and pure render tests:

```sh
python3 tools/validate-battle-art.py
cargo test -p crystal-render-api battle
cargo test -p crystal-voxel-view battle_view::tests
```

With the ignored external pack available, run the actual controller and bridge
regressions; a screenshot alone does not establish battle playability:

```sh
CRYSTAL_RENDER_TEST_PACK="$PWD/content-packs/text-tui.crystalpack" \
  cargo test -p crystal-bevy --features voxel-view immersive_battle
CRYSTAL_RENDER_TEST_PACK="$PWD/content-packs/text-tui.crystalpack" \
  cargo test -p crystal-bevy --features voxel-view \
  renderer_neutral_controller_exposes_battle_commands_and_executes_a_turn
```

Native smoke coverage must include a displayed battle, selecting FIGHT and a
move, PP/HP mutation and retained dialogue, repeated choices, party/cancel,
items/cancel, F3 off/on, faint/switch, capture/Dex, and return to the overworld.
Pack availability and native results belong in the verification report; they
must not be inferred from the pure mesh tests above.

## Native disposable preview

```sh
cargo run -p crystal-bevy --features location-tester,fullscreen-scaling \
  --example immersive_battle_3d -- \
  content-packs/core-modular.browser.crystalpack
```

This explicitly opted-in developer fixture starts a fresh Route36 session with
normal-palette Cyndaquil and Totodile, the Pokédex flag and a small test bag, starts the production
Sudowoodo scripted encounter, and uses `VisibleShellController` to complete its
intro. It never loads or writes a user save. Arrows, Z/Enter, X and F3 retain the
normal controls. Add `--screenshot output/battle.png` for a native GPU readback;
`--second output/battle-classic.png` captures the other view in the same session.
`--live` with paired screenshots keeps the game running after capture for input
checks. Recordings use `--record output/battle-recording --seconds 20`. Add
`--record-on-move` to arm recording until the first actually presented move.
It never drives input; a 180-second no-move timeout fails instead of producing
an empty successful clip. The first observed cue progress is saved without
inventing pre-roll frames.

The default battle lighting uses inexpensive soft contact shadows that follow
presented battlers and their faint/send-out visibility. For a measured A/B test,
set `CRYSTAL_BATTLE_SHADOWS=on` to additionally enable directional shadow maps.
The default remains the same on hardware and software adapters; it does not
silently claim the software-driver capture rate as the hardware rate. Use
`--measure output/battle-measure --seconds 20` for update timing without PNG/GPU
readback, then compare the same camera and session with `--record` separately.
