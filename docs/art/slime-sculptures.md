# Grimer, Muk and Ditto sculptures and rigs

`tools/slime_sculpt.py` authors three original flowing slime models.
`tools/slime_glb.py` exports canonical skins used by the runtime, main builder
and model checker. Legacy static JSON assets are removed.

All three retain one closed continuous body and a broad flat puddle underside.
Grimer has a wider tall open mouth, a true annular lip, pale eyes with tiny
pupils, a blue-gray lower tongue, forward arms and rounder, shorter three-finger
hands. Muk has a wider mouth, angled eyes, a broad sagging base, one long arm
and a shorter opposite hand. Both retain supported front and rear folds.
Ditto has fuller upper lobes, a stronger asymmetric side bulge, tiny eyes,
a curved smile with a fitted pink interior, and three soft lavender highlights.
Its upper body remains continuous with its spread base.

The official [Grimer](https://www.pokemon.com/us/pokedex/grimer),
[Muk](https://www.pokemon.com/us/pokedex/muk) and
[Ditto](https://www.pokemon.com/us/pokedex/ditto) artwork informed the anatomy
and colors. No reference pixels or external geometry are shipped.

| Species | Parts | Vertices | Triangles | Joints | Accessors | Bytes | Authoring height | Pack height |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Grimer | 26 | 18,739 | 25,172 | 13 | 146 | 730,324 | 0.96 | 0.889 m |
| Muk | 26 | 16,163 | 21,396 | 13 | 139 | 633,072 | 0.96 | 1.1938 m |
| Ditto | 8 | 9,310 | 13,980 | 5 | 51 | 403,272 | 0.96 | 0.3048 m |

The verified local pack supplies height digits 211 (SLUDGE), 311 (SLUDGE) and
100 (TRANSFORM), respectively. Neutral geometry digests:

- Grimer: `01ccaddd323bf25f60a019cbc0642316ec52af5c7cd650ecc625d54d607c3a03`.
- Muk: `373608073b4024110aaaf4e23d345ca1c07c2fd6a2725f72fe985893224ca8c3`.
- Ditto: `1a3fc7b87714f7c4c5b82db35cecc4fb7bc3d3679f52f0ccf2336727626327e8`.

At least 48 sole vertices remain fixed to the floor root. The continuous body
blends into a torso and coherent head. Grimer and Muk blend their arms into
separate palms, with six independent digit carriers rooted in those palms.
Ditto has two independent lateral lobe carriers; the highlights use the same
position-dependent influences as the body beneath them. Eyes, mouth coatings,
tongues and annular lips follow the coherent head. Rotation-only idle, attack
and hit clips retain neutral endpoints and the existing source-cue clocks.
World sliding preserves the fixed underside and the authored articulation.

Seventeen Python checks pass: canonical bytes and bounded schemas, hierarchy,
identity binds, normalized influences and anatomical owners, actual occupied
arm/palm/finger roots through motion with a detached negative control, fixed
soles, independent local articulation, floor/endpoints/winding/strain, fitted
faces and Ditto highlights through poses, source cues, closed outward geometry,
unit normals, recipe parity and size, anatomy, true annular mouth openings,
body folds and an independent ray-index comparison. Posed clearance checks
follow the authored front faces; closed markings' intentionally embedded back
and edge walls are excluded from that front-surface test.

Existing 20,000-vertex, 40,000-triangle, 64-primitive, 256-accessor and 1 MiB
limits remain unchanged. Five focused Rust checks pass for all three verified
pack-size GPU paths, production routing/owners/floor/culling and independent
appendages, plus 64 phases of world sliding. Front, side and rear poses of all
three clips were inspected for each model. Blender exports are byte-identical.
Shiny checks are skipped at the user's request. Grass height is untouched.

Rebuild and check:

```sh
python3 tools/slime_sculpt.py --output crates/crystal-voxel-view/models/battle_species
python3 -m unittest discover -s tools -p 'test_slime*.py'
```

The rebuilt production client passes normal Route 29 terrain admission for all
three models at original pack size. Every paired 3D/2D capture was inspected:

| Pair | Core frame | Checksum | LCD frame | Virtual seconds |
| --- | ---: | ---: | ---: | ---: |
| Grimer / Pidgey | 2795 | 1195468295 | 3471 | 62.241414707 |
| Muk / Sentret | 2871 | 2640698820 | 3509 | 62.198563916 |
| Ditto / Pidgey | 2671 | 2973483575 | 3340 | 58.788624415 |

All three preserve state, show full silhouettes and the correct species HUDs,
and meet the terrain with their broad bases. Grimer's open mouth and rounded
hands, Muk's asymmetric arms and wide stare, and Ditto's small smiling blob,
asymmetric upper outline and highlights remain readable. No shader errors
were reported. Final ignored evidence uses `grimer-normal`, `muk-normal` and
`ditto-normal` under `target/pokemon-native-review/`.

The captures select the existing supported -110-degree angle while retaining
normal size, source support, scenery omission and whole-body/frustum checks.
Both shared all-species grounded/volumetric and exact source-cue pose timing
regressions pass. Browser visual acceptance remains open.
