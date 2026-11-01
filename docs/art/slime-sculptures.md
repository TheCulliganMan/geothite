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
| Muk | 26 | 16,163 | 21,396 | 13 | 139 | 633,076 | 0.96 | 1.1938 m |
| Ditto | 8 | 9,310 | 13,980 | 5 | 51 | 403,272 | 0.96 | 0.3048 m |

The verified local pack supplies height digits 211 (SLUDGE), 311 (SLUDGE) and
100 (TRANSFORM), respectively. Neutral geometry digests:

- Grimer: `efd59abdeaebc370eabfeac2aa8ec133ddfd0106e2114b787d776b6d38d06511`.
- Muk: `60179e88f045db6353cea68824224ae5e1ffa2db3b802e91910cb5949452f75d`.
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

## Continuous fold roots

A later front/side/rear review found broken-looking folds on both sludge
bodies. The former sparse depth samples left real gaps: a decoded posed-mesh
audit found 27 nonpositive root sections in Grimer and 46 in Muk out of 810
sections per species. The worst gaps were 0.000205 and 0.000316 authoring units.

The six folds per model now use `tools/surface_strokes.py`. It smooths the
original XY paths and fits each ring to the actual supporting body triangles.
The same round radii remain, with each center 35% of its radius above the
surface. Only these twelve folds' positions and normals change. Body and face
geometry, complete bounds, materials, hierarchy, inverse binds and decoded
animation keys are preserved. Ditto's GLB remains byte-for-byte unchanged.

Nineteen candidate and installed anatomy and skin checks pass. The added stored-triangle
oracle requires an occupied root and an exposed front at fifteen cross-sections
of every fold through idle, attack and hit at three times each. Both previous
assets fail; deliberately floating and buried copies fail as well. Fresh
windowless GPU front/side/rear views of the installed models were inspected.
These normalized studio views supplement the production captures below.

All three exporters reproduce the installed bytes on Python 3.9.6 and 3.14.6.
Grimer's tongue supplies an explicit `fsum` centroid to retain two reviewed
positive-zero coordinates across those versions. The former implicit `sum`
changed only their sign bits on Python 3.9; Muk and Ditto already matched.
This source correction changes no canonical GLB bytes.

## Refit production review

Both installed models were reviewed in four fresh, muted, windowless native
2D/3D pairs: close front views in `PlayersHouse2F` and oblique views after
ordinary walking in New Bark Town. The reviewer contains all 276 current
canonical compressed GLBs; source and executable hashes remained unchanged.

| Species / view | Core frame | Checksum | LCD frame | Virtual seconds |
| --- | ---: | ---: | ---: | ---: |
| Grimer front | 80 | 3737093225 | 80 | 2.995400958 |
| Muk front | 54 | 41366297 | 54 | 1.303911042 |
| Grimer outdoor walk | 806 | 1109307622 | 833 | 15.269307292 |
| Muk outdoor walk | 727 | 2696047226 | 750 | 14.559234709 |

Every accepted pair reports `state_preserved=true` and the expected companion
identity. The open outdoor path was chosen from the pack's real walkable floor
and NPC occupancy. Each walk exercises 64 input/render samples. Both puddles
remain supported, with their creases attached through the shown poses. Room
walking trials put Muk too close to the floor edge and were replaced by these
outdoor views. A nearby building partially occludes its unchanged raised arm;
the studio angles retain full arm inspection. The classic world view does not
draw the additional 3D companion, so these pairs establish matching game state
and placement rather than a new source-sprite likeness assessment.
