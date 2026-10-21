# Geodude sculpture and skin

The [October 20 reference and posture revision](crocodile-geodude-fidelity.md)
replaces the clenched hands, neutral shape, counts and pin described below.

Geodude has an original faceted rock body and eight-joint glTF skin. Its
canonical runtime source is `crates/crystal-voxel-view/models/battle_species/geodude.glb`.
`tools/geodude_sculpt.py` authors the geometry; `tools/geodude_glb.py` supplies
weights, inverse binds and rotation-only idle, attack and hit clips. Battle
and explicit world species identity share the same neutral body.

The [official Geodude artwork](https://sg.portal-pokemon.com/pokedex/0074/)
guides the irregular gray boulder, angular white eyes, narrow pupils, raised
brows, bent arms and clenched hands. Each hand has four knuckles and a thumb.
An initial software review looked too smooth; fewer rock faces, stronger
radial irregularity and flat body normals corrected that material reading.
Face details fit the stored rock triangles and stay rigid with the boulder.
Shininess uses the existing flag and pack palette on the same geometry.

The asset contains 29 closed parts, 12,179 vertices and 15,352 triangles,
544,164 bytes, with floor zero and authoring height 0.76. Neutral digest:
`90658750b47d9380750c88431617ae2860b5f5cdb8e6d1d0af8fb320124ab33d`.
The lowest boulder contact remains on the fixed root. The boulder, shoulders,
forearms and fists have separate joints. Attack braces at 18% of the source
cue, followed by the fists at 28%; hit recoils and idle gives a gentle survey.
The source game retains ownership of movement, timing, projectiles and audio.

```sh
python3 tools/geodude_glb.py
python3 tools/test_geodude_glb.py
CARGO_TARGET_DIR=target/pokemon-models cargo test --locked -p crystal-voxel-view geodude --lib
```

Seven independent stored-model checks pass: exact regeneration and bounded
schema, outward closed topology and normals, anatomy and dimensions,
hierarchy/binds/weights, rigid face, fixed ground contact and deformation
across clips, rigid fist ownership, neutral endpoints and early brace with
follow-through. Native integration and visual results are recorded in
`docs/art/animated-models.md`. Software review alone does not establish
production animation, browser or overworld acceptance.
Seven Python model checks and focused canonical routing pass. The native
client build passed and its embedded catalog reports 234 exact bodies of
251 packed species, leaving 17 missing.

The held native pair `target/pokemon-native-review/geodude-3d.png` /
`geodude-2d.png` was inspected. Receipt: `state_preserved=true`, hash
3139018129, LCD animation frame 45, virtual time 1.117300167 seconds. Rock
facets, angular brows, bent arms and clenched fingers read at source size;
the camera partially occludes the near arm. The 2D reference is the back
sprite. Production materials are warmer gray than the source's green-gray
shading. Native source-cue motion, browser and overworld review remain open;
the broader model/rigging goal remains unfinished.

Both Geodude Rust integration tests pass, including shared canonical ownership,
fixed boulder contact across clips and eight-joint GPU skin activation at
0.4064 m. The freshly compiled test executable passes all 24 active species
rig regressions (one existing audit ignored), all 20 skinning regressions and
the 234-species reusable model check.

The first Rust test attempts encountered concurrent scenery edits: the planter
filter omitted newly added mansion material roles, and a new special-room test
called a missing helper. The filter now preserves those roles with the same
surviving-vertex remap as other attributes; its existing preservation regression
passes. The special-room owner corrected its fixture before the successful
retry. No game content pack or audio was changed by this model work.
