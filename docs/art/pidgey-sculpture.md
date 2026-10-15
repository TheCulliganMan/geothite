# Pidgey sculpture and skin

Pidgey's previously missing exact body is an original small bird sculpture
with a ten-joint standard glTF skin. The sole canonical runtime source is
`crates/crystal-voxel-view/models/battle_species/pidgey.glb`.
`tools/pidgey_sculpt.py` authors its geometry; `tools/pidgey_glb.py` supplies
weights, inverse binds and idle/attack/hit rotations. Generic closed surface
helpers are reused; no Spearow or Pidgeotto body is relabeled. Battle and
explicit world species identity share the same neutral asset.

The [official Pidgey artwork](https://sg.portal-pokemon.com/pokedex/0016/)
guides its cream breast and throat, brown head and short crest, dark cheek
wedges, ivory eyes, pink hooked bill, folded ochre wings, cream flight tips,
three brown tail feathers and pink feet. Each foot has three front toes and
an opposing rear toe, with ivory front claws. Initial neutral inspection
caught breast fit gaps and a missing cream throat; denser breast fitting and
a separate fitted throat corrected those features. Materials remain matte;
shininess retains the existing flag and pack palette on the same skin.

The GLB has 43 named closed parts, 8,404 vertices and 13,064 triangles,
431,212 bytes, with authoring height 0.82 and floor zero. Neutral digest:
`814a75e55669f0361edc7c35026a7a0b112085cda12d4dcc485693bfd1990fb7`.
Pack dimensions own physical size. Pads, front/rear toes and claws stay on
the fixed root; shanks blend into the torso. The fitted face belongs to the
head. Lower bill, paired wings and flight tips, tail fan and crest have their
own joints. Attack braces at 18% of the cue, with feathers and tail following
at 28%; idle surveys and hit recoils. Source programs retain movement, audio,
projectiles and timing ownership.

```sh
python3 tools/pidgey_glb.py
python3 tools/test_pidgey_glb.py
CARGO_TARGET_DIR=target/pokemon-models cargo test --locked -p crystal-voxel-view pidgey --lib
```

Seven independent stored-model checks pass: exact regeneration, bounded
schema, closed outward topology and normals, anatomy/dimensions, hierarchy
and bind identity, normalized weights, rigid face and feather ownership,
fixed toes/pads/claws, floor clearance and strain through all clips,
neutral endpoints and early brace with delayed tail response. Native and
runtime results are recorded in `docs/art/animated-models.md`. Neutral
software review alone does not establish production animation, browser or
world acceptance.
Seven independent Python model checks and focused canonical routing pass.
The native client rebuild passes; its embedded catalog reports 235 exact
bodies of 251 packed species, leaving 16 missing.

The held native pair `target/pokemon-native-review/pidgey-3d.png` /
`pidgey-2d.png` was inspected. Receipt: `state_preserved=true`, hash
2417923469, LCD animation frame 50, virtual time 1.190217375 seconds. Dark
cheek wedge, cream throat/breast, short crest, pink bill and planted feet,
folded cream wing tips and brown tail read at the tiny source size. Native
brown is less saturated than the source sprite's ochre. The 2D reference is
the back sprite with a raised wing; folded neutral/idle model posture does
not reproduce that sprite pose. Native source-cue motion, browser and world
review remain open. This is progress toward the complete appearance/rigging
goal, not full visual acceptance.

Both Pidgey Rust integration checks pass: canonical neutral sharing and fixed
pads/toes/claws across clips, plus ten-joint GPU skin activation at source
size (0.3048 m). The fresh compiled test executable passes all 25 active rig
regressions (one existing audit ignored), all 21 skinning regressions and the
235-species reusable model check. Targeted whitespace checks pass. The broader
model/rigging goal remains active; most exact bodies still require dedicated
rig and full production visual review.
