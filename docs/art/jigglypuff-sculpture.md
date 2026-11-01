# Jigglypuff shared sculpture and skin

Jigglypuff's existing shared balloon sculpture now has a seven-joint standard
glTF skin. Its sole canonical source is
`crates/crystal-voxel-view/models/actor_props/icon_jigglypuff.glb`.
The legacy icon and explicit battle/world species paths share that asset.
The former JSON was removed; no duplicate species geometry was introduced.
`tools/jigglypuff_sculpt.py` remains the pure geometry recipe, while
`tools/jigglypuff_glb.py` adds weights, binds and source-cue rotations. The
sculpture CLI and actor-builder route now emit the canonical GLB.

The round pink balloon, pointed pillow ears with charcoal linings, short arms,
splayed feet, angled rolled forelock with fitted crease, gentle smile and
large layered blue eyes are preserved. Eye whites, turquoise lower reflections
and ivory glints stay rigid with the body. The original
[official artwork](https://assets.pokemon.com/assets/cms2/img/pokedex/full/039.png)
guides the sculpture. No external geometry, textures or rigs are imported.
Shininess retains the existing flag and pack palette on the same skin.

The original and regenerated float32 neutral digests match:
`be7a901fb200b5528a7ec3fbba34c9944776fdb953d1eb15b69f58cf9419664e`.
Two regenerated ear Y normals differed only in signed zero; canonicalizing
those zero Y components preserved every original float32 geometry bit.
The asset has 22 closed parts, 13,706 vertices, 22,956 triangles and 627,952
bytes. These verified counts replace the stale higher triangle count in the
earlier document. Authoring height remains 0.82 and floor zero; pack dimensions
own physical size.

Feet stay on the fixed root. The body and layered face form one rigid assembly.
Arms, ears and forelock have separate joints; ear linings and forelock crease
stay rigid with their supporting volumes. The first pose audit caught an
inverted tiny arm cap under blended weights. Each stubby arm now rotates as
one rigid volume, preserving cap topology while moving at the shoulder.
Attack braces the body and arms at 18% of the cue; ears and forelock follow
at 28%. Idle sways gently and hit recoils. Source programs retain movement,
audio, projectiles and timing ownership. This does not claim facial morphs
or a reproduced singing sprite sequence.

```sh
python3 tools/jigglypuff_glb.py
python3 tools/jigglypuff_sculpt.py --output target/jigglypuff-review
python3 tools/test_jigglypuff_glb.py
python3 tools/test_jigglypuff_sculpt.py
CARGO_TARGET_DIR=target/pokemon-models cargo test --locked -p crystal-voxel-view jigglypuff --lib
```

Seven stored-GLB checks and all eight existing sculpture checks pass, including
closed topology/shading, exact generation and limits, anatomy, stored-triangle
face fitting and clearance, physical joins, forelock crease, hierarchy/binds,
weights, rigid face/ear/forelock assemblies, planted feet and deformation
through all clips, neutral endpoints and early brace/delayed forelock response.
Native/runtime evidence is recorded in `docs/art/animated-models.md`. Neutral
review alone does not establish production animation, browser or world acceptance.
The first deformation audit caught a tiny arm-cap inversion under blended
weights. Giving each short arm one rigid owner preserved topology while
retaining shoulder motion. The final seven GLB checks and all eight existing
sculpture checks pass. Focused canonical routing passes. The pure sculpture
CLI was executed and its output matches the canonical GLB byte-for-byte,
without JSON. Modified authoring/routing scripts parse. Blender is unavailable;
its batch route was updated and syntax-checked, not executed. The native
client rebuild succeeded.

The native catalog reports 239 dedicated catalog bodies among 251 packed
species, with 12 using shared or fallback bodies outside that catalog. This
is not a claim that only twelve bodies need quality or rigging work.
Jigglypuff already had a detailed shared sculpture; this step adds its rig and
explicit catalog entry while preserving its geometry. Shininess remains the
existing flag and pack palette on the same skin.

The held native pair `target/pokemon-native-review/jigglypuff-3d.png` /
`jigglypuff-2d.png` was inspected. Receipt: `state_preserved=true`, hash
1141753145, LCD animation frame 37, virtual time 0.953778083 seconds. Rounded
ears with dark linings, rolled forelock, layered eyes, short arms and grounded
feet remain readable at source size. Native pink is muted compared with the
source back sprite; the eyes read teal. The source view is its back sprite.
Source-cue motion, facial/singing sprite fidelity, browser and world review
remain open; the full model/rigging goal is unfinished.

Both Jigglypuff Rust integration tests pass: shared canonical icon/species
neutral geometry, fixed feet through clips and seven-joint GPU skin activation
at 0.508 m. The fresh executable passes all 29 active species rig regressions
(one existing audit ignored), all 25 skinning regressions and all 12 actor tests,
including the 239-species reusable model check. Targeted whitespace checks pass.
There are now 22 registered standard species skins. This remains progress
within the full goal, not proof that all 251 species have improved rigs or
accepted production visuals.
