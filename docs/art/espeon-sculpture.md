# Espeon sculpture and rig

The generic lilac fox is replaced by an original slender sun cat with broad
triangular ears and dark blue fitted pinnae, a silver-violet almond gaze,
a fitted ruby forehead lens and sparkle, swept sensory cheek fans, a small
feline muzzle, joined shoulders, rounded haunches and four planted paws.
A long curved tail ends in two separately curled tips, joined at an occupied
smooth hub. The visual reference is the
[official Espeon artwork](https://sg.portal-pokemon.com/pokedex/0196/).
No reference pixels are shipped.

`tools/espeon_sculpt.py` authors 44 closed parts, 10,809 vertices and 19,164
triangles. `tools/espeon_glb.py` exports a 484,268-byte canonical GLB with
208 accessors and 15 joints at authoring height 1.00. The neutral digest is
`051135bb83bc3ca1a337c80500fa056725f8b3fa59436c1b5267d195d4d5eb4e`.
The verified local pack supplies height digits 211 (0.889 metres), SUN.
Original pack size is retained.

The rig uses coherent body/shoulders/haunches and head/face/gem/cheek carriers,
a blended neck and four limbs, independent ears, three tail carriers and two
independently blended fork-tip carriers. The tail hub stays with the final
stem carrier; both fork roots share that carrier before fading into their
own tip joints. Paws and toe creases remain on the neutral root. Idle, attack
and hit retain neutral endpoints and existing source-cue timing.

Nine Python checks pass: canonical bytes and schema limits, closed outward
solids, actual face normals, fitted gaze/gem/ear surfaces, anatomical owners,
occupied neck/shoulder/ear/limb/stem/hub/fork roots through motion with a detached
negative control, floor/endpoints/winding/strain/independent motion, separate
tip articulation relative to the hub, and cue metadata. Front, side and rear
poses of all three clips were inspected. Blender generation is byte-identical.
Both focused Rust rig/GPU checks pass. Shiny verification is skipped at the
user's request.

Rebuild with `python3 tools/espeon_glb.py`. Run asset checks with
`python3 -m unittest discover -s tools -p test_espeon_glb.py`.

The rebuilt native example passes a Route 29 walking encounter against Pidgey.
The inspected 3D/2D pair preserves core frame 4069, checksum 2848246175, LCD
animation frame 4787 and virtual time 93.811969876. Terrain admission passes
at original pack size. `CRYSTAL_BATTLE_REVIEW_YAW=-110` selects an existing
establishing angle with normal support, omission, scale and full-body/frustum
checks. The lilac body, ruby lens, violet gaze, broad ears, cheek fans and both
fork tips remain readable, with planted paws and matching ESPEON HUD labels.
No shader errors were reported. Both shared grounded-volumetric/model and exact
source-cue timing regressions pass. Final ignored evidence uses
`target/pokemon-native-review/espeon-normal`; browser visual acceptance remains
open.
