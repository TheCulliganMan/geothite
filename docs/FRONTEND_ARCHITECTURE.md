# Alternative frontend assessment

Geothite can share its Rust gameplay backend with another renderer. The initial
separation in this change makes that a direct crate dependency. A complete
replacement UI still requires extracting presentation orchestration from the
current shell; this is not yet a drop-in replacement of every screen.

## Boundaries

| Crate | Responsibility | Reuse outside Bevy |
| --- | --- | --- |
| `crystal-core` | State, battles, movement, scripts, input, saves, replay primitives | Yes; no renderer dependency |
| `crystal-assets` | Compiled pack loading, validation, data and runtime commands | Yes; use the existing pack |
| `crystal-runtime` | Game session, command dispatch, semantic snapshots, save/replay coordination, audio cue resolution | Yes; extracted from the old `crystal-bevy` library |
| `crystal-audio` | Rust synthesis and canonical PCM decoding | Yes; decoded samples and frame loop ranges are device independent |
| `crystal-net` | Multiplayer transport | Yes; session integration still needs host orchestration |
| `crystal-bevy` | Window/input integration, UI sequencing, classic rendering, playback devices | Current frontend |
| `crystal-render-api` | Bevy visual world frame and render scheduling | Bevy world mods only |
| `crystal-voxel-view` | Alternate world view consuming the classic extraction | Still depends on the classic compositor |

The runtime does not depend on Bevy, GPU resources, Rodio, or a tile compositor.
Its existing `image` dependency supports the pack's path-based PNG loaders, not
window creation. The shared runtime implementation and its tests have moved;
there is no second copy of the backend. Consumers should import runtime types
from `crystal_runtime`, rather than `crystal_bevy`.

## Starting another frontend

Depend on `crystal-runtime`, load a verified `.crystalpack`, and construct
`RuntimeGameShell`. `examples/headless_frontend.rs` demonstrates loading,
advancing one input frame, reading semantic state, draining audio cues, and
decoding a packed sound effect without creating a window or audio device:

```sh
cargo run -p crystal-runtime --example headless_frontend -- content-packs/core-modular.browser.crystalpack
```

Use `presentation_snapshot()` for frequent visual updates. Use `snapshot()` at
integrity boundaries where checksums matter; the presentation path deliberately
omits that work. Snapshots expose world objects, menus, text, battle state and
catalogs without requiring Bevy image handles. A custom renderer can choose its
own artwork and geometry from these identities. `CrystalRuntime::runtime_file()`
provides embedded presentation bytes directly; a new renderer need not materialize
the current renderer's path-based asset tree. Simulation collision data must
remain authoritative even if the visuals change.

Translate host input into `GameButton` or the runtime's typed command methods.
Drive simulation independently of display refresh using the shared Game Boy
frame duration. `tick()` advances an overworld input frame; it is not a universal
controller for all menu, script and battle presentation phases. Read the pending
requests and invoke their explicit command/completion methods. Real-time hosts
must also sample the RTC, manage focus, and avoid advancing VBlank twice.
`advance_radio_broadcast()` uses the same authoritative game state and RNG.

A terminal party defeat also needs an explicit completion: call
`complete_battle_loss()` before presenting whiteout and then
`resolve_blackout_to_last_spawn()` at the recovery boundary. The first command
validates that no usable party member remains and commits the shared loss
cleanup; the second retains its independent recovery validation. Phone
registration stops at `RuntimeCompiledScriptBoundary::PhoneNumberPrompt` until
the host supplies the player's acceptance or refusal. Neither transition should
be inferred from a missing sprite or a finished text box.

Gym scripts update badge flags and badge bits together in `crystal-core`, so a
new frontend sees the same awards as field-move checks and badge counts.
`load_save()` reconciles older saves that recorded an award in only one of those
representations. It does not infer an award from a defeated-leader event.
For fallible session edits, `try_session_update()` restores state and journals
on error without cloning the immutable pack; the host still owns rollback of
external side effects.

Drain resolved audio events once per simulation update and fan them out from the
host if multiple consumers need them. Preserve event order, music reset/fade,
SFX priority, cry handling, and `WaitForSoundEffect` completion. Resolve programs
through `RuntimeAudioCatalog`, then use `audio::pcm::decode_program_source()`.
It validates packed compressed/synthesized PCM length and hash and preserves
exclusive loop ranges measured in stereo frames. Cache decoded data outside the
simulation; MIDI synthesis is synchronous and belongs on an audio worker. Native
Rodio output and browser AudioContext/worker integration remain frontend adapters.
The current Bevy adapter uses the same shared PCM decoder.

## Rust and non-Rust frontends

Another Rust renderer can depend on this crate directly. Godot, Unity, or a
JavaScript UI will need a small Rust host plus an FFI, Wasm, or message adapter.
The runtime snapshots are Rust structures, not a versioned JSON/wire protocol;
they are not currently all serializable. Define explicit presentation DTOs and
commands for that adapter instead of exposing the entire mutable game state.
The web server currently provides hosting, clock/session services and multiplayer
relay; it does not offer the full game runtime as a general frontend service.

## Remaining work before a full custom UI

1. Extract the presentation controller from `bevy_shell`: title/new game, menu
   navigation, text progression, battle animation completion, script callbacks,
   and transitions currently coordinate runtime commands there. Keep the typed
   completion boundaries; do not advance game scripts merely because rendering
   skipped an animation.
2. Extract the audio scheduler from `BevyRuntimeShell`, including fade state,
   priority/preemption, pending decode work, and completion signals. The decoder
   is reusable now, but playback behavior is not yet one reusable service.
3. Define world, battle, and UI presentation views based on semantic snapshots.
   The existing `VisualWorldFrame` requires `Handle<Image>` and is populated
   after `ClassicWorld`; simply implementing another consumer still runs the
   tile compositor. Keep that API for existing Bevy mods and build a separate
   frontend directly from the runtime for genuinely independent visuals.
4. Move host-specific persistence, browser asset installation, and multiplayer
   pumping behind explicit adapters as needed. Browser runtime assets currently
   install into a process-global one-time store, which limits multiple packs in
   one browser instance.
5. Prove parity by driving two frontends with the same recorded input/commands
   and comparing authoritative checksums, pending requests, audio order and
   completion. Cover title, overworld, dialogue, menus, battle, save/resume, and
   multiplayer before declaring complete frontend interchangeability.

A custom world view inside Bevy is the smaller project. A completely independent
frontend is feasible with the shared runtime, but the remaining controller and
audio scheduling extraction is substantial. This change provides the reusable
backend and PCM boundary without changing game rules or bundled content.

## Validation

- `crystal-runtime` also type-checks independently for `wasm32-unknown-unknown`;
  its normal dependency graph contains no Bevy, WGPU, Winit, Rodio or CPAL.
- Native Bevy library, binary, tests and examples type-check with
  `location-tester,operation-trace` enabled.
- Browser Bevy binary type-checks for `wasm32-unknown-unknown` with
  `fullscreen-scaling,voxel-view` enabled. Existing browser warnings remain.
- All 22 `crystal-audio` unit tests pass.
- The `frontend_contract` integration test passes using the shipped browser
  pack: two renderer-free sessions agree, failed edits restore state/journals,
  and one program from each music/SFX/cry catalog decodes against packed PCM
  metadata. Synthetic PCM tests cover sample values and exclusive loop ranges.
- The moved runtime suite completes with **199 passed, 40 failed** when run with
  `RUST_MIN_STACK=33554432` and two test threads. The default test-thread stack
  overflowed in a nested-script test. Of the 40 failures, 27 reference missing
  legacy pack/export/disassembly files, five use synthetic species fixtures
  lacking required animation programs, and eight are other assertions/gameplay
  errors. These have not been repaired or suppressed by this refactor; the full
  suite is not green. One source-text RNG assertion also fails against a string
  already present in the base commit (`61f9d301`).

The current Bevy UI has been type-checked, not manually played through for this
change. A full cross-frontend behavior comparison remains future work.

## Modern 3D world direction

The public Astra world-building demonstrations are useful visual references, but
are not evidence of game-rule fidelity. For example, the author's
[3D civilization build comparison](https://github.com/cagrikacmaz/gpt-6-astra-vs-gemini-3-8-flash)
uses a playable scene, camera interactions, screenshots and manual QA; it explicitly
identifies itself as an uncontrolled product comparison. Its world presentation
and camera work illustrate the intended direction for Geothite. The Pokémon
implementation still needs its own behavior and timing evidence.

A modern renderer can author terrain, buildings, vegetation, lighting, materials
and character models while using the existing Rust session for movement,
encounters, scripts, battles, saves and sound programs. The ASM-derived runtime
grid remains the coordinate reference. Doorways, ledges, grass encounter regions,
NPC positions and map connections must map back to that grid exactly. Rendering
may interpolate between committed positions, but must never move the simulation
according to mesh collision or the camera's frame rate. Decorative height,
interiors and obscured geometry require authored decisions: they cannot be
recovered uniquely from the original 2D tiles.

The outdoor New Bark prototype is retained on `feat/new-bark-3d` (`b68cd17b`). It
is not part of the production battle fixes. Before using it as a replacement,
review every exit and doorway, connected-map seams, NPC interaction range,
foreground occlusion, camera controls and mobile frame times. Then extend its
coverage to interiors and additional maps. An attractive outdoor scene alone does
not establish that the complete world is supported.


## Shared picture animation

`crystal_runtime::frontpic_animation` now owns the main/idle picture program
interpreter and its byte-sized timing state. Bevy delegates to that interpreter;
other frontends can consume the same frame index without scene entities or image
handles. Program selection, cry cues and higher-level animation sequences still
belong to the presentation controller. The shared Hall of Fame timing module is
also available, but the visible ceremony is not integrated yet.

Three existing animation regressions pass after extraction, including the
95-call Cyndaquil main/idle source trace, repeat/frame timing and malformed-program
rejection. The source-trace fixture remains external and is supplied through
`CRYSTAL_FRONTPIC_TRACE`; it is not copied into the repo or shipped bundle. This
validates the extracted interpreter, not every animation controller or frontend.
