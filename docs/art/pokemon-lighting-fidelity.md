# Outdoor Pokémon sky-fill correction

The prior [rhino/larva review](rhino-larva-fidelity.md) exposed almost black
native back faces despite readable studio geometry. The missing indirect
illumination hid shoulders, haunches, armor, markings and tail volume. This
revision corrects that shared outdoor lighting problem; roster-wide shape
improvement remains active.

## Cause and correction

The existing procedural sky stores normalized sRGB colors. Bevy 0.20's
`EnvironmentMapLight` consumes physical radiance before camera exposure.
Using intensity 1 beside the world's 3000-lux sun supplied negligible sky fill
to `StandardMaterial` Pokémon. The world sky now supplies 600 cd/m² multiplied
by those same clock-driven sky colors. Directional shading remains visible.

The palette-preserving terrain shader already owns its light-energy response
and does not apply PBR camera exposure to that response. Its indirect diffuse
and reflection samples now divide out the actual view-environment intensity,
retaining normalized sky color. This keeps the PBR correction from multiplying
painted terrain and water reflections. Enclosed rooms and explicit Dark maps
retain zero outdoor environment intensity.

The two cached cubemaps retain their handles, mip structure and five-minute
refresh cadence. The authored models, physical heights, rigging, animation,
base pigment, zero emission, sun and global ambient settings retain their
existing contracts. All 276 canonical GLBs are byte-identical to audit start.

## Executed verification

Twelve focused Rust tests pass. They cover actual camera/environment
publication, retained image handles, all cube faces/mips, source-frame identity,
fixed indoor/dark exclusion, clock continuity, sun/night energy, unchanged idle
light settings, and Pokémon's lit material/grounding-shadow contract. A new
regression decodes stored diffuse sky pixels and checks their physical fill
against the direct sun, including dawn/night separation.

The exact reviewed shader text and canonical Rhyhorn, Rhydon, Larvitar and
Totodile gzip streams were found in both native executables. The revised shader
and the same canonical models were also verified in the tested unit executable.
The baseline native executable was preserved before editing the shader.

```sh
CARGO_TARGET_DIR=target/pokemon-models cargo test --locked --jobs 2 \
  -p crystal-voxel-view --lib --no-run \
  --config profile.dev.package.crystal-voxel-view.opt-level=0
CARGO_TARGET_DIR=target/pokemon-models cargo build --locked --jobs 2 \
  -p crystal-bevy --features location-tester --example render_at_location \
  --config profile.dev.package.crystal-bevy.opt-level=1
```

The ignored gallery records the twelve exact test names and executable,
shader and selected canonical-model hashes. Offscreen native review uses
`CRYSTAL_RENDER_TEST_HEADLESS=1`, `CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image`,
`CRYSTAL_PREVIEW_MUTE=1`, actual pack dimensions and the production controller.

## Readability measurements and review scope

Four daytime back views use identical before/after static placement, facing,
camera and original model dimensions. The following fixed body regions are
sampled in the actual native PNGs; masks selected from the baseline exclude
background and differently colored markings. Values are median display luma
on a 0–255 scale, not physical brightness ratios.

| Model | Before | After |
| --- | ---: | ---: |
| Rhydon gray back | 5.8 | 59.4 |
| Rhyhorn rear barrel | 6.4 | 66.6 |
| Larvitar green back | 25.4 | 75.0 |
| Totodile blue back | 20.2 | 67.1 |
| Rhydon back at 22:00 | 6.0 | 7.0 |

The backs now expose real volume, color boundaries and appendage roots.
Directly lit obliques retain fitted faces and dimensional shading; the night
control retains the established dark blue treatment. Fixed window/trim, blue
roof and paving regions in the Rhydon rear control are byte-identical before
and after. This is a bounded source-palette check, not whole-frame pixel
identity: NPC timing and other live art work can differ.

Daytime obliques use NewBarkTown `(7,12)`, hour 12, the real `RR` keyboard route,
rotation 1/pitch 35 and zoom 3 for Rhydon or 5 for the other three. The two
executables can finish those timed inputs at different positions and poses;
these shots establish qualitative readability. They are not the basis of the
fixed-region measurements.

Static rear controls use EcruteakCity, rotation 4/pitch 35: Rhydon `(20,27)` at
zoom 1; Rhyhorn `(20,25)` at zoom 2; Larvitar and Totodile `(20,25)` at zoom 5.
Rhydon's front/back controls are repeated at hour 22. Each retained native/2D
pair independently preserves position, clock, facing and game state. The
classic field omits the added companion, so it proves state and presentation,
not a visible back-sprite comparison. The prior sculpture reviews retain the
actual front/back sprite and full game-orbit evidence.

Separate lake controls use NewBarkTown `(15,8)`, rotation 0/pitch 45, zoom 0,
at hours 12 and 22. The actual water surface is visible and retains its sky
reflection, blue depth and dark night treatment. Moving waves, foliage and NPCs
make these qualitative controls, not whole-frame pixel comparisons.

Selected evidence is ignored at
`target/pokemon-fidelity-review-20261020-lighting/index.html`: twelve before/after
comparison sheets, 48 original native/classic PNGs, 24 paired-state receipts,
the exact test results, asset-identity proofs and bounded pixel measurements.
The personal `game-model-fidelity-audit` skill now distinguishes insufficient
production sky energy from a geometry/pigment defect and calls for checks of
the source-painted shader's separate light-energy convention.

This accepts the sampled outdoor lighting correction. It does not establish
browser, shiny, every-map, every-animation or full-roster visual acceptance.
The shared world-camera change does not alter the separate battle camera.
No pack rebuild, model import, source-pigment edit or geometry edit was needed.
The frozen reviewer executable and scratch captures are removed after keeping
the selected gallery. Shared packs and build caches are preserved. Nothing is
staged or committed.
