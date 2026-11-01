# Metapod and Kakuna sculptures and rigs

The static cocoon recipes now produce original, articulated canonical GLBs.
Metapod has a clearer projecting face, hooked crown, rear point, curved lower
tail and three fitted tail segments. Rounded ivory crescents and dark pupils
replace the rectangular eyes. Sloping green lids, cheek shields and seated
seams complete the pale green chrysalis. Kakuna has a rounded golden crown,
triangular dark eyes with small ivory glints, broad folded arm plates, a raised
central shield and three tapered abdominal seams. Its shield uses structural
cross-sections at the actual shell's profile breaks, avoiding thin overlapping
coatings and split highlight fans. The visual references are the official
[Metapod](https://www.pokemon.com/us/pokedex/metapod) and
[Kakuna](https://www.pokemon.com/us/pokedex/kakuna) artwork.
Reference pixels are not shipped.

`tools/cocoon_sculpt.py` authors both species. `tools/cocoon_glb.py` supplies
the canonical skins and strict authoring readers. All existing limits remain:

| Species | Parts | Vertices | Triangles | Joints | Accessors | Bytes | Authoring height | Pack height |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Metapod | 15 | 10,664 | 15,788 | 4 | 80 | 455,168 | 0.80 | 0.7112 m |
| Kakuna | 14 | 11,069 | 16,336 | 5 | 80 | 478,004 | 0.80 | 0.6096 m |

The verified local pack supplies height digits 204 and 200, respectively;
both are classified COCOON. Original pack size is retained. Neutral digests:

- Metapod: `6b3fdcfefd6a88f8417852ca46a6d144a9d4d35c5e5e712b3c42c8b61576b36f`.
- Kakuna: `2174b2bfbb87197fe326bd5fd82a05a06dc5539f943618d45a086dcb10d1f805`.

A neutral floor root fixes the lowest shell points. Blended shell and head
carriers give small cocoon motions while the complete gaze stays coherent with
the head. Metapod has an independent blended tail fold. Kakuna has separate
folded-arm carriers shared by each plate and its seam. Idle, attack and hit use
restrained rotation-only motion, neutral endpoints and existing source-cue
timing. No terrain, grass-height, size or camera-admission rules change.

Eleven Python checks pass across both assets. These cover canonical bytes and
schema limits, closed outward solids, actual normals, physical bounds, fitted
faces and plates on stored surfaces, Metapod's continuous bent profile, exact
skin owners, fixed lower shell points, seated fold roots through motion with a
detached negative control, floor/endpoints/winding/strain/articulation, and
source-cue metadata. Shiny verification is skipped at the user's request.

Rebuild with:

```sh
python3 tools/cocoon_sculpt.py --output crates/crystal-voxel-view/models/battle_species
python3 -m unittest discover -s tools -p 'test_cocoon*.py'
```

Three focused Rust checks pass, covering both runtime species routes, exact
head/fold owners, fixed lower-shell contacts, floor/culling bounds, independent
head/tail/arm-fold motion and GPU activation at verified pack heights. Both
shared grounded/volumetric and source-cue pose timing regressions pass.
Front, side and rear poses of all three clips were inspected for both species;
Blender generation is byte-identical.

The rebuilt native Route 29 Metapod encounter against Pidgey passes normal
terrain admission at original pack size. The inspected 3D/2D pair preserves
core frame 2421, checksum 1062851264, LCD animation frame 3031 and virtual time
53.275264209. The hooked crown, projecting face, sleepy gaze, rear point and
segmented lower tail remain readable with the correct METAPOD HUD. The lower
shell tip meets the ground. No shader errors were reported. Final ignored
evidence uses `target/pokemon-native-review/metapod-normal`.

The native Route 29 Kakuna encounter against Sentret also passes normal terrain
admission at original pack size. Its inspected pair preserves core frame 2578,
checksum 1752407553, LCD animation frame 3260 and virtual time 57.848109083.
The rounded crown, triangular gaze, folded arms, clean central ridge and
abdominal bands remain readable with the correct KAKUNA HUD and planted lower
shell tip. No shader errors were reported. Final ignored evidence uses
`target/pokemon-native-review/kakuna-normal`. Both captures select an existing
supported angle with `CRYSTAL_BATTLE_REVIEW_YAW=-110`, retaining normal scale,
support, omission and whole-body/frustum checks. Browser visual acceptance
remains open for both species.
