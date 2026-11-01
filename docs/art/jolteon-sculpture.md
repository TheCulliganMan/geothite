# Jolteon sculpture and rig

The generic yellow fox is replaced by an original lean lightning fox with
long angular legs, four planted paws, sharp upright ears, fitted violet ear
insets and a narrow dark almond gaze with violet irises and white glints.
Swept cheek bristles flank a small yellow muzzle. A continuous twelve-point
white collar and three long white chest needles frame the throat. Eight
occupied yellow back/rump bristles replace the generic brush tail. The visual
reference is the [official Jolteon artwork](https://sg.portal-pokemon.com/pokedex/0135/);
no reference pixels are shipped.

`tools/jolteon_sculpt.py` authors 50 closed parts, 10,892 vertices and 17,840
triangles. `tools/jolteon_glb.py` exports an 11-joint canonical GLB with 213
accessors and 468,260 bytes at authoring height 1.05. Its neutral geometry digest is
`3e154fb857ffb4531d84ef1cf35b3ec944ab4d59bbb5b0da316722aa0f6af022`.
The verified local pack supplies height digits 207 (0.7874 metres), LIGHTNING.
Original pack size is retained.

The rig provides coherent body/quills and head/cheeks/muzzle/gaze carriers,
blended neck and four limbs, independent ears and an independent white collar.
Paws and toe creases stay on the neutral root. Idle, attack and hit clips keep
neutral endpoints and existing source-cue timing. The old JSON adapter is
removed; production rendering, asset reading and the Blender batch entry point
use the same canonical source.

Eight Python checks cover canonical bytes and schema limits, closed outward
solids, actual normals, fitted gaze and ear surfaces, anatomical owners,
occupied neck/collar/ear/quill/limb roots through animation with a detached
negative control, floor/endpoints/winding/strain/independent motion and cue
metadata. Initial eye triangles cut into the head; subdivided fitted caps fix
the intersection. Front, side and rear poses of all three clips were inspected.
Both focused Rust rig/GPU checks pass. Blender generation is byte-identical.
Shiny verification is skipped at the user's request.

Rebuild with `python3 tools/jolteon_glb.py`. Run asset checks with
`python3 -m unittest discover -s tools -p test_jolteon_glb.py`.

The rebuilt native example passes a Route 29 walking encounter against Pidgey.
The inspected 3D/2D pair preserves state at core frame 1357, checksum
3790288244, LCD animation frame 1904 and virtual time 32.880172499. Terrain
admission passes at original pack size. Yellow body and violet gaze, sharp
ears, white collar, rump bristles and grounded paws remain readable; HUD labels
match the 2D state. No shader errors were reported. The two shared regressions
for grounded volumetric models and exact source-cue pose timing pass.
Evidence uses `target/pokemon-native-review/jolteon-normal`; browser visual
acceptance remains open.
