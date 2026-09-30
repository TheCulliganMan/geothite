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

Move effects now consume the existing source animation interpreter's current
frame: BGP/OBP palette state, battler displacement, screen shake and the actual
live object OAM/frameset output. The arena uses ten reused billboard objects
(the source slot limit), with the source art, colors, mirror direction, clipping
and trajectory. The original object runtime and decoded bundle are shared with
the classic renderer; F3 does not start another interpreter or advance a tick.
Generic move particles/lunges are suppressed whenever this source frame exists.
The 32-particle pool remains only for non-source send-out/HP-loss presentation.

The source palette remaps the nearest source shade of each authored model color.
Neutral frames restore the original mesh colors. This is a 3D adaptation, not
an assertion of pixel-identical original battler silhouettes. Current horizontal
source scanline offsets also bend modeled battler geometry; removing the source
buffer restores the cached original positions.
The arena darkens/brightens at the same palette writes; the compact original
HUD remains readable and is never flashed. Model geometry is cached by species;
only the two active actor meshes are uploaded and recolored on palette changes.
Leaving the battle, opening an owning full-screen page or disabling 3D hides all
ten source objects and thirteen reused effect volumes; returning to a neutral
frame restores materials, colors and model positions.

Full source flashes are the default. F4 toggles Full/Reduced in native play;
`BevyShellConfig.battle_reduced_flashes` sets the initial choice. The preview also
accepts `--reduced-flashes`. Reduced mode caps model/arena palette contrast to
12% and uses the same current object frame with neutral OBP registers. Source
frames, object trajectories, durations, sounds, commands, HP and PP are identical.
This setting applies to the modeled view; F3 restores the original classic view.

### Verified Shadow Ball source

The ignored `core-modular.browser.crystalpack` script selects BGP `$1b` on source
frame 1, inverting battler shades and darkening the arena background. The same
frame spawns the blue-palette Barrage Ball object at `(64,92)` with the source
`WAVE_TO_TARGET` callback. Frame 33 spawns `BALL_POOF` at `(132,56)`; the script
returns at frame 57. It does not request a repeated `FLASH_*` background effect,
so the renderer does not invent a strobe. The pack's source sound remains owned
by the production animation. No extracted script or art is committed.

### Psychic and Hyper Beam in depth

Psychic uses the pack's actual `PSYCHIC_M` identity and `BattleAnim_PsychicM`
script. Eight `WAVE` objects begin at source frames 1, 9, 17, 25, 33, 41, 49 and
57. Their current callback/OAM positions and changing sizes drive violet 3D
rings beside the original source art. The original alternating hues and rotating
horizontal sine buffer continue through frame 160; the wave effect stops at
161 and the script completes at 165. The same current buffer bends the modeled
battlers. Three quiet violet rings visualize that sustained distortion around
the target. Violet volumes are an original 3D interpretation; no target lift,
extra attack duration or black strobe is invented.

Hyper Beam uses `HYPER_BEAM` / `BattleAnim_HyperBeam`. Actual beam segments start
at frames 1, 5, 9 and 13, with a tip on frame 13. These current source segments
drive joined golden 3D volumes; the source framesets retain their own lifetimes
and deletion. Screen shake, inverted flashes, gray/yellow palette cycling,
source art and sound remain owned by the existing interpreter. There is no
additional windup or predicted impact. The script completes at frame 61;
visible HP loss remains the only impact/recoil cue, and recharge remains a
production battle rule. Full/Reduced changes volume opacity and flash contrast,
never source timing, geometry trajectories or gameplay.

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

The `immersive_battle_shadow_ball_preserves_actual_source_palette_and_object_frames`
regression compares every source frame 0–56 with the actual classic object
renderer, including art handles and positions. It verifies the inversion and
poof boundaries, no premature impact/future poof, and unchanged authoritative
state. Separate tests cover delayed repeated flash writes, full/reduced color
limits with identical source frames/poses, bounded object/entity/material counts,
F3 retirement and neutral/interrupted restoration. Include `location-tester` in
the `crystal-bevy` test features to run the legal-TM/controller preview regression.

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

Add `--shadow-ball` for a separate disposable session with a level-40 Gengar.
The real TM item mutation teaches `TM_SHADOW_BALL` into the first move slot,
validating compatibility and assigning the pack's PP; no fabricated move is
injected. Select FIGHT and the first move normally, then acknowledge the used-move
message. Example capture: `--shadow-ball --record output/shadow-ball-full
--record-on-move --seconds 15`; repeat with `--reduced-flashes` for the reduced
comparison. These options never load or write a user save.

`--psychic` instead uses level-20 Kadabra and the real `TM_PSYCHIC_M` mutation;
`--hyper-beam` uses level-20 Raticate and `TM_HYPER_BEAM`. Both TMs are verified
against the pack's compatibility table and occupy the first move slot. Move
fixture flags are mutually exclusive. Raticate's Normal attack faces the
unchanged level-20 Rock-type Sudowoodo, allowing the real recharge turn to be
shown without altering enemy HP or combat rules. Recording examples:
`--psychic --record output/psychic --record-on-move --seconds 20` and
`--hyper-beam --record output/hyper-beam --record-on-move --seconds 25`.
Acknowledge the ordinary turn messages to display recharge. At the initial
Hyper Beam command menu only, this disposable developer fixture resets the
RNG state to `(add: 0, sub: 0)` and supplies 16,384 DIV replay samples from the
existing test LFSR (seed `0xc5af`, polynomial `0xb400`). This makes accuracy and
damage repeatable for capture; it is developer input, not an emulation of live
hardware timing. The production accuracy, damage and recharge routines still
run unchanged. Psychic and normal play retain their existing divider source.
No fixture loads or writes a user save.

The default battle lighting uses inexpensive soft contact shadows that follow
presented battlers and their faint/send-out visibility. For a measured A/B test,
set `CRYSTAL_BATTLE_SHADOWS=on` to additionally enable directional shadow maps.
The default remains the same on hardware and software adapters; it does not
silently claim the software-driver capture rate as the hardware rate. Use
`--measure output/battle-measure --seconds 20` for update timing without PNG/GPU
readback, then compare the same camera and session with `--record` separately.
