# Umbreon sculpture and rig

The generic dark fox is replaced by an original compact charcoal moonlight
companion with long tapered ears, a thick rising tail, red almond eyes and
vertical pupils, a small muzzle and smile, joined shoulders, rounded haunches
and four planted paws. Hollow gold rings fit the forehead and all four outer
limbs. Continuous gold bands wrap both ears and the tail. The visual reference
is the [official Umbreon artwork](https://sg.portal-pokemon.com/pokedex/0197/).
No reference pixels are shipped.

`tools/umbreon_sculpt.py` authors 46 closed parts, 12,563 vertices and 21,084
triangles. `tools/umbreon_glb.py` exports a 497,056-byte canonical GLB with
195 accessors and 13 joints at authoring height 1.05. The neutral digest is
`c627b4d60cb817a8e18596f759e06d97b6b57497c9af7badaea772b215b77e03`.
The verified local pack supplies height digits 303 (0.9906 metres), MOONLIGHT.
Original pack size is retained.

The rig has body/neck/head carriers, independent ears, four blended limbs and
a continuously weighted three-carrier tail. Face and forehead ring stay
coherent; side limb rings remain with their actual shoulder/haunch surfaces.
Paws and toe creases remain on the neutral root. Ear and tail band segments
share exact stored seam positions and matching quantized skin weights. All
six material boundaries stay closed through idle, attack and hit. Clips retain
neutral endpoints and existing source-cue timing.

Ten Python checks pass: canonical bytes/schema limits, closed outward solids,
actual face normals, fitted gaze/pupils/sparkles, anatomical owners, occupied
neck/ear/shoulder/limb/tail roots with a detached negative control, all six
material seams through animation, hollow centers and actual parent-surface
seating for all five gold rings, floor/endpoints/winding/strain/independent
motion, and cue metadata. Front, side and rear poses of all three clips were
inspected. Blender generation is byte-identical. Shiny verification is skipped
at the user's request. Both focused Rust rig/GPU checks pass.

Rebuild with `python3 tools/umbreon_glb.py`. Run asset checks with
`python3 -m unittest discover -s tools -p test_umbreon_glb.py`.

The rebuilt native example passes a Route 29 walking encounter against Sentret.
The inspected 3D/2D pair preserves core frame 3215, checksum 875707673, LCD
animation frame 3941 and virtual time 72.125268584. Terrain admission passes
at original pack size. `CRYSTAL_BATTLE_REVIEW_YAW=-110` selects an existing
establishing angle with normal support, omission, scale and full-body/frustum
checks. The charcoal body, red gaze, hollow gold rings, both ear bands and
full banded tail remain readable, with planted paws and matching UMBREON HUD
labels. No shader errors were reported. Both shared grounded-volumetric/model
and exact source-cue timing regressions pass. Final ignored evidence uses
`target/pokemon-native-review/umbreon-normal`; browser visual acceptance
remains open.
