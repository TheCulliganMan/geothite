# Paras and Parasect sculptures and rigs

The generic eight-limbed insects and flattened mushroom ellipsoids are replaced
by original orange mushroom crabs. Four braced walking legs and two broad,
tapered foreclaws form six insect appendages. Paras has a russet rear shell,
round white eyes with large dark pupils and glints, six forehead beads, an
orange snout, paired ivory mouth claws and two small pink fungal umbrellas.
Parasect has blank pale eyes, a broader muzzle and a large pink fungal dome
that overhangs its head. Golden markings follow the actual stored cap surfaces.
Inset underside disks, radial gills and rooted cream stalks complete the fungi.
The visual references are the official
[Paras](https://www.pokemon.com/us/pokedex/paras) and
[Parasect](https://www.pokemon.com/us/pokedex/parasect) artwork.
Reference pixels are not shipped.

`tools/mushroom_crab_sculpt.py` authors both species.
`tools/mushroom_crab_glb.py` exports canonical skins within all existing limits:

| Species | Parts | Vertices | Triangles | Joints | Accessors | Bytes | Authoring height | Pack height |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Paras | 56 | 19,755 | 31,860 | 11 | 213 | 711,892 | 0.78 | 0.3048 m |
| Parasect | 36 | 13,877 | 22,168 | 10 | 152 | 543,484 | 1.14 | 0.9906 m |

The verified local pack supplies height digits 100 and 303, respectively;
both are classified MUSHROOM. Original pack size is retained. Neutral digests:

- Paras: `9cf3e0d314c5f90ef570f927288a43e2dd32e7421e7d6218202fc1807ec9d82f`.
- Parasect: `ee0b479c17022294ba5cbc8084b4d2312aeb028868773ecd51113da09416838b`.

A yaw-only body and coherent head carry each face. Foreclaws articulate
independently from the front carapace. Four blended leg carriers flex between
the body and fixed walking toes. Each fungus has its own carrier shared by its
cap, golden markings, underside, gills and rooted stalk; the two juvenile fungi
can move separately. Idle, attack and hit retain neutral endpoints and source
cue timing. No terrain, grass-height, size or camera-admission rules change.

Eight Python checks pass across both assets: canonical bytes and schema limits,
closed outward solids, actual normals, fitted gaze/mouth/golden spots on stored
surfaces, exact skin owners, occupied feature roots through motion with a
detached negative control, floor/endpoints/winding/strain/motion, and source-cue
metadata. Front, side and rear views of all three clips were inspected for
both species. Blender generation is byte-identical. Shiny verification is
skipped at the user's request.

Rebuild with:

```sh
python3 tools/mushroom_crab_sculpt.py --out crates/crystal-voxel-view/models/battle_species
```
 Run asset checks with
`python3 -m unittest discover -s tools -p test_mushroom_crab_glb.py`.

Three focused Rust checks pass, covering both runtime species routes, exact
anatomical owners, floor/culling bounds, fixed walking toes, independent claws,
legs and fungal caps, and GPU activation at both verified pack heights. Shared
grounded/volumetric and source-cue pose timing regressions also pass.

The rebuilt native Route 29 Paras encounter against Pidgey passes normal
terrain admission at original pack size. The inspected 3D/2D pair preserves
core frame 2009, checksum 134018625, LCD animation frame 2623 and virtual time
45.983132001. Twin fungi, golden spots, juvenile gaze, mouth claws and digging
foreclaws remain readable with the correct PARAS HUD. Four walking toes meet
the ground. No shader errors were reported. Final ignored evidence uses
`target/pokemon-native-review/paras-normal`.

The native Route 29 Parasect encounter against Sentret also passes normal
terrain admission at original pack size. The inspected pair preserves core
frame 2148, checksum 1893713661, LCD animation frame 2778 and virtual time
48.692868291. The overhanging pink/gold cap, blank pale eyes, broad snout and
tapered foreclaws remain readable with the correct PARASECT HUD and grounded
walking feet. No shader errors were reported. Final ignored evidence uses
`target/pokemon-native-review/parasect-normal`. Both captures use
`CRYSTAL_BATTLE_REVIEW_YAW=-110`, selecting an existing supported angle with
normal scale, support, omission and whole-body/frustum checks. Browser visual
acceptance remains open for both species.
