# Pokémon battle review and retained terrain coordinates

The October 21 Tyranitar field and studio review exposed a separate native
battle-capture failure. Both Route 29 walking inputs and the real Falkner
trainer interaction reported no checked terrain anchors. Those failed
screenshots were not accepted as model evidence.

A fresh headless Falkner reproduction with `CRYSTAL_LOADING_TRACE` identifies
the failed condition as `atlas or grid geometry differs`. At the actual source
pose, Violet Gym (4, 1) facing right, the published terrain is a 120 × 120 grid
starting at (-48, -48). Its retained 2D viewport starts at (0, 0). Inferring
symmetric padding around that 20 × 18 viewport instead gives (-50, -51).

`visual_world_grid_window` deliberately aligns the optional renderer's window
to complete 4 × 4 source metatiles. The renderer records that window in
`RenderedViewport::visual_world_grid_origin` and publishes it in
`VisualWorldFrame::grid_origin`. A witness must compare those actual retained
identities; the old viewport-padding calculation no longer describes the
published grid. The one-frame walking revision lag seen in the earlier logs
was not enough to explain the failure: this stationary trainer also fails.

The static, fishing/scripted-trainer and walking/Surf readers now share
`retained_encounter_grid`. Native builds read the retained texture, dimensions
and metatile-aligned origin together. Classic builds retain their original
fixed-halo calculation. Both paths still require a retained viewport.

Map identity and bounds, texture, grid extent/origin, tile size, viewport,
terrain revision, player identity, facing, displacement and original step
correspondence remain checked. A missing, cold or mismatched frame still
fails; the edit introduces no later-frame repair or acceptance fallback.

This investigation does not change Pokémon geometry, physical sizes, pack
contents, gameplay state or admission thresholds. Native visual acceptance
still requires an actual checked source frame, admitted scenery, the intended
embedded model, matching 2D/3D state and inspection of the resulting images.

## Executed regression and native checks

The existing
`scripted_trainer_anchor_preserves_continuous_blocked_approach_receipt`
regression fails before the correction with
`the real off-center field must supply checked anchors`. The same test passes
afterward, retaining the actual blocked-approach displacement through the
authored Falkner dialogue and battle start. No test tolerance or fixture state
was changed to obtain that result.

```sh
CARGO_TARGET_DIR=target/pokemon-models \
CRYSTAL_RENDER_TEST_PACK="$PWD/content-packs/text-tui.crystalpack" \
cargo test --locked --jobs 2 -p crystal-bevy --lib --features location-tester \
  --config profile.dev.package.crystal-bevy.opt-level=1 \
  scripted_trainer_anchor_preserves_continuous_blocked_approach_receipt \
  -- --nocapture
```

All 52 native-feature `anchor` regressions pass in the resulting
`crystal_bevy-d50e607812df6c41` test executable. They include static encounters,
fishing, walking/caves, Surf, trainer sight and scripted trainers, plus cold,
stale, wrong-grid, missing-actor, off-step and one-shot evidence rejection.
Their session/journal assertions keep presentation observation separate from
gameplay mutation. The executable and result identities are retained in the
ignored review gallery.

```sh
CRYSTAL_RENDER_TEST_PACK="$PWD/content-packs/text-tui.crystalpack" \
target/pokemon-models/debug/deps/crystal_bevy-d50e607812df6c41 \
  anchor --nocapture --test-threads=1
CARGO_TARGET_DIR=target/pokemon-models cargo build --locked --jobs 2 \
  -p crystal-bevy --features location-tester --example immersive_battle_3d \
  --config profile.dev.package.crystal-bevy.opt-level=1
```

The classic configuration separately passes all 47 applicable `anchor`
regressions with `voxel-view` and `location-tester` absent. These are two
configuration-specific runs, not 99 distinct behavioral tests. The classic
command was:

```sh
CARGO_TARGET_DIR=target/pokemon-models \
CRYSTAL_RENDER_TEST_PACK="$PWD/content-packs/text-tui.crystalpack" \
cargo test --locked --jobs 2 -p crystal-bevy --lib \
  --config profile.dev.package.crystal-bevy.opt-level=1 \
  anchor -- --nocapture --test-threads=1
```

The rebuilt native executable has SHA-256
`607ea8aa8ea2a145cbe999ac98800876d8c223939a5b03496801defe0ab3a0fe`.
Its exact compressed canonical Tyranitar and Cyndaquil payloads were verified.
Three headless, muted 1000 × 700 production captures now preserve their actual
2D/3D state and report admitted scenery with `source=[]`, `fallback=None`:

| Case | Actual models | Input |
| --- | --- | --- |
| Route 29 paced walking | Tyranitar / Sentret | Twenty `RALA` repetitions, `WALK_STEPS=1` |
| Route 29 held walking | Tyranitar / Pidgey | Twenty `RALA` repetitions, `WALK_STEPS=0` |
| Falkner scripted trainer | Cyndaquil / Pidgey | Sixty `A` presses |

The driver uses `CRYSTAL_RENDER_TEST_WALK_INPUT=synthetic`,
`CRYSTAL_RENDER_TEST_HOLD_BATTLE=0`,
`CRYSTAL_RENDER_TEST_HOLD_MOVE_MENU=1` and
`CRYSTAL_ENCOUNTER_TRACE=1`. The two walking captures use
`--walking-encounter`; the trainer uses `--falkner-encounter`. All screenshot
processes exit successfully. The walking logs still contain the coherent
preceding source revision, now with `anchored=true`; the origin correction
does not waive revision or original-step checks.

The receipt guard caught that the Falkner fixture supplies Cyndaquil directly
and ignores the generic party-species setting. Its image is labeled as that
fixed trainer fixture, not Tyranitar evidence. The fixture, trainer party,
authored scripts and controller progression were preserved.

All six original PNGs were inspected. Tyranitar's full body, upright supports,
low tail and fitted armor remain visible in the two outdoor battle views at
its unchanged 2.0066 m pack height. The indoor pair shows the real Cyndaquil
fixture and source-sized Pidgey against the Gym floor. The classic move menu
covers the player's back picture: these pairs establish matching state;
the earlier pack pictures and complete studio orbit supply the separate
likeness coverage. This does not establish Cyndaquil shape fidelity, every
animation phase, shiny, browser or full-roster acceptance.

The [Tyranitar review](tyranitar-fidelity.md) and its ignored gallery,
`target/pokemon-fidelity-review-20261021-tyranitar/index.html#native-battles`,
retain the new pairs, actual admission receipts, before/after regression
results, compact baseline rejection and executable/model identities. The
gallery now contains 29 selected PNGs and seven paired-state receipts.

The 271 battle-folder assets match the initial file snapshot; the five
actor-space canonical sources match exact compressed payloads in the initial
native executable. All 276 canonical model identities remain unchanged.
The audit skill records both the terrain-origin check and the need to read
each fixture's actual party setup. Its validator passes. Task-local scratch
is removed after retaining selected evidence; packs and build caches remain.
Nothing is staged or committed. Broader model improvement remains active.
