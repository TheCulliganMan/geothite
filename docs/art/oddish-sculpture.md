# Oddish sculpture and skin

Oddish's missing exact body is an original round blue bulb with two root feet,
five cupped green leaves and a twelve-joint standard glTF skin. Its sole
canonical runtime source is
`crates/crystal-voxel-view/models/battle_species/oddish.glb`.
`tools/oddish_sculpt.py` authors its anatomy; `tools/oddish_glb.py` supplies
weights, inverse binds and source-cue clips. The generic closed botanical leaf
helper is reused for blade and raised midrib construction. The arrangement,
body, face, feet and weights are authored specifically for Oddish.

The [official Oddish artwork](https://sg.portal-pokemon.com/pokedex/0043/)
guides the broad blue bulb, tiny red eyes and ivory glints, open happy mouth
with pink tongue, two blue root feet and five pointed green leaves. The first native review exposed a narrow leaf silhouette from the side:
spreading the blades through depth and lowering the outer leaves corrected
the fan in a side-angle software review. Final native verification is recorded
below. Matte materials preserve the simple silhouette. Battle and explicit world species
identity share one neutral source. Shininess retains the existing flag and
pack palette, using identical geometry and rig.

The GLB has 19 named closed parts, 8,865 vertices, 14,620 triangles and 441,232
bytes, with floor zero and authoring height 1.00. Neutral digest:
`b138445f423d89daff929c82b90cc4950b9a9534b53d767887664b65e1ada7ab`.
Pack dimensions own physical size. Both root feet stay on the fixed root;
the bulb blends out of its lowest contact area into the body joint. The fitted
face remains rigid on that joint. Each leaf has a base and tip joint; blade
and midrib share identical longitudinal weighting. Joint pivots use the exact
same normalization transform as the authored geometry. Attack braces at 18%
of the cue and leaf tips follow at 28%; idle sways and hit recoils. Source
programs retain movement, audio, projectiles and timing ownership.

```sh
python3 tools/oddish_glb.py
python3 tools/test_oddish_glb.py
CARGO_TARGET_DIR=target/pokemon-models cargo test --locked -p crystal-voxel-view oddish --lib
```

Seven independent stored-model checks pass: exact regeneration, bounded
schema, closed outward solids and normals, anatomy and dimensions, hierarchy
and inverse-bind identity, normalized weights, rigid face, fixed feet, floor
clearance and strain throughout every clip, five complete leaf chains,
neutral endpoints and early brace with delayed leaf-tip response. Native
integration and visual evidence are recorded in `docs/art/animated-models.md`.
Neutral software review alone does not establish native animation, browser or
world acceptance.
Seven independent Python model checks pass on the final asset, including
fixed feet and deformation throughout all clips. A depth-span assertion was
added and verified to guard the wide fan from a side view. Focused canonical
routing passes. The final native rebuild passes; its embedded catalog reports
236 exact bodies out of 251 packed species, leaving 15 missing.

The first production review exposed a narrow upright tuft when the mostly
planar leaves were viewed from the battle camera. Spreading the five blades
through depth and lowering the outer leaves corrected that silhouette. The
final held pair `target/pokemon-native-review/oddish-3d.png` / `oddish-2d.png`
was inspected. Receipt: `state_preserved=true`, hash 3978529836, LCD animation
frame 37, virtual time 0.947527042 seconds. The fan is wider from the production
view, with blue bulb, small red eye and rooted feet visible at source size.
Some blades are edge-on or occluded from this camera. The 2D reference is the
back sprite. Native bulb is bluer, and leaves darker, than the source sprite's
gray-green bulb and bright leaves. Native source-cue motion, browser and world
review remain open. The broader appearance and rigging goal remains active.

Both final Oddish Rust integration checks pass: canonical neutral sharing and
fixed root feet across clips, plus twelve-joint GPU skin activation at source
size (0.508 m). The freshly compiled executable passes all 26 active species
rig regressions (one existing audit ignored), all 22 skinning regressions and
the 236-species reusable model check. Targeted whitespace checks pass. No
pack, audio program or unrelated game content was changed. The full model and
rigging goal remains unfinished; most exact bodies still need dedicated rigs
and complete production visual acceptance.
