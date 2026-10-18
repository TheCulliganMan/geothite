# Omanyte and Omastar sculptures

The canonical battle meshes now use `tools/omanyte_sculpt.py` and
`tools/omastar_sculpt.py`. The main battle generator dispatches those recipes
before Blender batching. Neither species is rebuilt from the old shared toy
recipe. These are original, static sculptures, with +Y up, front +Z and a
grounded root. They contain no game pixels or imported external geometry.

The inspected official [Omanyte artwork](https://sg.portal-pokemon.com/pokedex/0138/)
and [Omastar artwork](https://sg.portal-pokemon.com/pokedex/0139/) informed the
anatomy. Omanyte has a ribbed cream nautilus shell, a fitted side spiral on
both flanks, large white/black eyes and ten tapered blue tentacles. Its eight
lower tentacles meet the floor. Rib channels stop before the coil region,
so they no longer run through and obscure the spiral. The unfinished candidate
had 56,544 triangles; the installed sculpture has 18,848, below the existing
22,000-triangle limit.

Omastar has independently posed adult anatomy: six rooted shell spines,
yellow eyes with black slits, four raised tentacles and six spreading/curling
tentacles. Its mouth has a recessed dark well and four closed ivory beak
quadrants. The cross between them is empty volume. The installed sculpture
has 21,292 triangles.

The installed meshes pass 23 focused stored-mesh checks. These cover closed
outward surfaces, face/normal agreement, recipe parity, scale, grounding,
budget, fitted gaze visibility, rooted anatomy and uncrossed tentacles/shell
details. Omastar's open-cross test rejects a deliberately filled beak.
Front, oblique, side and rear software previews were inspected. The default
Lugia recipe remains byte-for-byte equal to its canonical sculpture after
adding optional resolution arguments to the shared paddle helper; its parity
regression passes. The canonical model-layout check also passes.

Fresh native image-target captures completed for both species in normal and
shiny appearances using the actual `text-tui.crystalpack`. All four pairs
reported preserved state. The four 3D images and two normal 2D reference
images were individually inspected. The 3D bodies are
visible, grounded and distinct. The 2D capture path obscures/crops the reference
sprites, so these pairs do not establish clean 2D fidelity acceptance. Full
shiny material acceptance, ordinary-window/browser views and animation remain
open. These static sculptures do not complete the full Pokémon appearance goal.

The current loaded-model Rust checks passed: two exact-model tests and five
canonical-body/routing tests, including all 225 registered bodies. Both final
model fingerprints were verified in the actual reviewer and test executables.
The new wild-species controller regression passed with the external pack:
it preserves the requested lead, retains the source Sudowoodo encounter,
executes a selected move, observes its PP decrement and exposes battle dialogue.

The broader Bevy `species` filter returned 16 passes and three failures.
The existing Vance preview fails save-reference validation because its saved
`EVENT_BEAT_BIRD_KEEPER_VANCE` disagrees with an empty compiled trainer flag.
The other failures are a missing legacy pack path in the nickname regression
and an invalid intro timing oracle in the evolution regression. None was
reported as passing or bypassed. The explicit wild-species review mode keeps
the Vance fixture intact and uses the existing authored wild battle through
the same production controller.

The broad `python tools/check-battle-species.py --allow-pending` audit fails
on the existing Aerodactyl model's 31,516 triangles. A direct count of the
current canonical JSON bodies finds 89 models at or above 22,000 triangles.
The limit has not been relaxed. The two nautilus replacements pass their
own geometry and budget checks; this does not certify the rest of the roster.

Reproduce the focused checks from the repository root:

```sh
python -m unittest discover -s tools -p 'test_oma*_sculpt.py' -v
python tools/check-model-layout.py
cargo build --locked --profile wasm-dev --jobs 2 --target-dir target/pokemon-native \
  -p crystal-bevy --features location-tester --example immersive_battle_3d
CRYSTAL_RENDER_TEST_PACK="$PWD/content-packs/text-tui.crystalpack" \
  cargo test --locked --profile wasm-dev --jobs 2 --target-dir target/pokemon-native \
  -p crystal-voxel-view -p crystal-bevy --features crystal-bevy/location-tester \
  --lib immersive_wild_species_preview_retains_requested_body_and_real_battle
```

On this Windows host, the Vulkan control panicked while choosing a swapchain
present mode. DirectX 12 successfully captured the bodies. The verified command
in PowerShell is:

```powershell
$env:WGPU_BACKEND = 'dx12'
$env:CRYSTAL_PREVIEW_MUTE = '1'
$env:CRYSTAL_RENDER_TEST_CAPTURE_TARGET = 'image'
& target/pokemon-native/wasm-dev/examples/immersive_battle_3d.exe `
  content-packs/text-tui.crystalpack --wild-species OMANYTE --size 1000x700 `
  --screenshot target/omanyte-3d.png --second target/omanyte-2d.png
```

Repeat with `OMASTAR`, distinct output names, and `--shiny` for the variants.
The saved pairs were 1500×1050 at the host's 1.5 scale. A native shader-review
binding also needed a mutable Bevy asset guard to compile the current checks;
that one-line repair does not change model geometry or gameplay.

The review example's `--catalog` option reads the actual supplied pack and
prints dedicated-body coverage and missing names without opening a window or
exporting a duplicate game catalog. Temporary captures, references and logs
belong under ignored `target/` and are not deliverables.

A fresh neutral software roster review inspected all 225 dedicated bodies in
ten sheets. Existing refinement priorities include Kabuto/Kabutops, the Eevee
and fox family, and Mew/Mewtwo. The pack-based catalog reports 26 missing bodies.
Neither the roster load tests nor a single neutral angle approves every model.

The later [canonical nautilus skins](nautilus-skins.md) add independent shell,
gaze, tentacle and adult beak controls, correct the concave polar caps and reuse
the canonical tentacle hinges for world crawling. Current captures use the
headless image path and never open a window.
