# Charmander dedicated sculpture and skin

Charmander replaces its missing exact battle body with an original orange
lizard sculpture and eleven-joint standard glTF skin. The sole canonical
runtime source is `crates/crystal-voxel-view/models/battle_species/charmander.glb`.
`tools/charmander_sculpt.py` authors geometry; `tools/charmander_glb.py` authors
its skin and source-cue clips. No icon or evolved-species body was relabeled.
Explicit world species identity shares the exact neutral body.

The inspected [official Charmander artwork](https://sg.portal-pokemon.com/pokedex/0004/)
guides the rounded orange head/muzzle, blue eyes and glints, small nostrils,
open gentle grin and short teeth, cream belly, three orange digits per paw,
strong haunches, six ivory foot claws and a long curved tail. The tail fire
uses the established generic folded-flame geometry helper from
`cyndaquil_sculpt.py`, fitted to Charmander's own tail; the Cyndaquil body was
not imported. Five closed ember/orange/gold fire layers keep the tip legible
from both sides. The first model exceeded the runtime vertex limit because
small eye patches were over-tessellated. Reducing their subdivision density
preserves the silhouette and belly fit while meeting existing bounds.
Shininess remains the existing flag and source-pack palette on the same body.

The final GLB has 43 named closed parts, 13,250 vertices and 21,086 triangles.
Floor minimum is zero and neutral height is 1.00 authoring unit; the pack owns
physical size. Neutral identity digest:
`6ae03caf697cdaa11acbbb4d2bc90af86f0db04ecf02b64c7e7342a5aed72bfa`.
The root fixes both pads and six foot claws. Torso/head and separate arms/haunches
brace early. Three tail joints preserve its bend; a child flame joint adds
restrained folded-fire flicker and delayed settling without detaching the fire
socket. Face/teeth belong to the head, and fingers follow their paw joints.
Idle lasts 3.8 seconds; attack braces at 18% of the source cue with tail/fire
settling at 28%; hit recoils. All keys are rotations. Game programs still own
movement, projectiles, audio, damage and timing.

```sh
python3 tools/charmander_glb.py
python3 tools/test_charmander_glb.py
CARGO_TARGET_DIR=target/pokemon-models cargo test --locked -p crystal-voxel-view charmander --lib
CARGO_TARGET_DIR=target/pokemon-models cargo test --locked -p crystal-voxel-view species_rig --lib
```

Seven independent stored-model checks pass, covering exact regeneration,
bounded schema, closed outward anatomy, normals, distinct features, binds/
weights, rigid face, fixed pads/claws, floor clearance, deformation, tail/fire
chain binding, neutral endpoints, early brace and delayed fire response.
Native/runtime verification and critique are recorded in
`docs/art/animated-models.md`; neutral software review alone does not establish
native source-cue motion or browser acceptance.

Canonical routing passes. The fresh native build embeds the final asset and
its catalog reports 232 exact bodies out of 251 packed species. The held
3D/2D pair was inspected at `target/pokemon-native-review/charmander-3d.png` /
`charmander-2d.png`; its receipt confirms unchanged battle state (hash
3584115452, LCD frame 47). The face, cream belly, curved tail and claws remain
readable at the pack's scale. The source view is its back sprite. Decoded
front/rear idle/attack/hit poses were also inspected in `charmander-poses.png`.
The folded flame is thin from the native camera angle; volume/readability,
native source-cue motion, browser and overworld review remain open.

Both Cargo Charmander integration tests pass, proving canonical neutral sharing,
planted pads/claws throughout clips and eleven-joint GPU activation at source
size (0.6096 m). The freshly compiled test executable in
`target/pokemon-models/debug/deps/` also passes all 22 active rig tests (one
existing migration audit ignored), all 18 skinning tests, the 232-species model
loading check and both camera regressions after the unit-scale fixture repair.
This is progress toward the full model/rigging goal, not complete visual acceptance.
