# Squirtle dedicated sculpture and skin

Squirtle replaces its missing exact battle body with an original blue turtle
sculpture and nine-joint standard glTF skin. The sole canonical runtime source
is `crates/crystal-voxel-view/models/battle_species/squirtle.glb`.
`tools/squirtle_sculpt.py` authors geometry and `tools/squirtle_glb.py` authors
its skin and source-cue clips. No icon or evolved-species body was relabeled.
The existing generic ribbon helper is reused for fitted plate joins, but the
body, shell and tail are authored for Squirtle. Explicit world species identity
shares the same immutable neutral body.

The inspected [official Squirtle artwork](https://sg.portal-pokemon.com/pokedex/0007/)
guides the round blue head, maroon eyes/glints, broad muzzle, small smile,
cream chest plates, brown domed carapace, rolled ivory rim, short rounded arms,
three-digit paws, grounded toes and solid curled blue tail. A neutral review
caught a hollow-looking tail curl; filling its center corrected that silhouette.
The six-sided shell seam is one closed fitted annular solid, while branch
seams follow actual stored triangles. Materials are matte. Shininess retains
the existing flag and source-pack palette on the same geometry and rig.

The final model has 45 named closed parts, 17,298 vertices and 26,948 triangles,
within existing runtime limits. Floor minimum is zero and neutral height is
0.96 authoring units; source-pack dimensions own physical size. Neutral digest:
`222004eaea9a5593380331ab51391d1ddfdcda105fc35c11ebe53835e4e341d7`.
The root fixes both pads and six toes. Shell, rim, plate joins and chest all
share the torso joint, preserving their attachment as one rigid assembly.
Head, arms and haunches respond separately; two tail joints supply delayed
curl follow-through. Face stays rigid on the head and fingers follow each paw.
Idle lasts four seconds; attack braces at 18% of the source cue and the tail
follows at 28%; hit recoils. Keys contain rotations only. Source game programs
still own movement, audio, projectiles, damage and timing.

```sh
python3 tools/squirtle_glb.py
python3 tools/test_squirtle_glb.py
CARGO_TARGET_DIR=target/pokemon-models cargo test --locked -p crystal-voxel-view squirtle --lib
CARGO_TARGET_DIR=target/pokemon-models cargo test --locked -p crystal-voxel-view species_rig --lib
```

Seven independent stored-model checks pass, covering exact generation, bounded
schema, closed outward solids, normals, anatomy, binds/weights, rigid face,
coherent shell ownership, tail-chain binding, fixed pads/toes, floor clearance,
deformation, neutral endpoints and early brace/delayed tail response.
Native/runtime results and critique are recorded in `docs/art/animated-models.md`.
Neutral software review alone does not establish native source-cue animation or
browser acceptance.

Focused canonical routing passes. The native build embeds the final asset and
its catalog confirms 233 exact bodies out of 251 packed species. The held native
3D/2D pair was inspected at `target/pokemon-native-review/squirtle-3d.png` /
`squirtle-2d.png`; its receipt confirms unchanged battle state (hash 2100597877,
LCD frame 36). Shell, plate joins, maroon eyes, planted blue limbs and filled
tail curl read at source size. The source view is its back sprite. Decoded
front/rear idle/attack/hit poses were inspected at `squirtle-poses.png`. Native
blue reads teal relative to the source sprite, a remaining color-fidelity limit.
Native source-cue motion, browser and overworld review remain open.

Both Cargo Squirtle integration tests pass, proving canonical neutral sharing,
fixed pads/toes across clips and nine-joint GPU activation at source size
(0.508 m). The freshly compiled test executable in
`target/pokemon-models/debug/deps/` also passes all 23 active rig tests (one
existing migration audit ignored), all 19 skinning tests and the 233-species
reusable model check. A transient moved-value error in an unrelated interior
constructor blocked the first test build; its current source checks material
role length before consuming the surface, and the retry passed. The complete
appearance/rigging goal remains unfinished.
