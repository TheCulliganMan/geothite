# Ninetales sculpture and rig

The generic pale fox is replaced by an original elegant cream fox with a longer
muzzle, sharp red gaze, warm fitted ear insets, a full cream mane, long chest
point, swept cheek fur and three swept crown wisps. Joined shoulders, rounded
haunches and four planted paws support nine flowing plumes. Seven rise into a
high fan; two lower tails sweep forward. Each has a continuous golden end.
The visual reference is the
[official Ninetales artwork](https://sg.portal-pokemon.com/pokedex/0038/).
No reference pixels are shipped.

`tools/ninetales_sculpt.py` authors 56 closed parts, 14,231 vertices and
23,784 triangles. `tools/ninetales_glb.py` packs these into 46 runtime parts,
250 accessors and a 662,324-byte canonical GLB with 20 joints at authoring
height 1.30. The verified local pack supplies height digits 307
(1.0922 metres), FOX. Original pack size is retained.

The source geometry digest is
`f435a002995098a49bd288d358210163951d154a7098a145dfeb064d2bd40777`;
the packed runtime digest is
`d3c5cac139c338a855ec83edbb1e3df15cbfddb68f7affd4a59f6564301173c3`.
Matching golden tip materials and crown wisps share runtime primitives.
Declared ranges restore every source part in its original order. Each packed
vertex retains its source part's actual skin assignment; different tips keep
their own carrier. Per-part ranges select each tail carrier before its weights are calculated;
all renderer/schema limits remain intact.

The rig provides body/neck/head carriers, independent ears, a shared tail-fan
carrier, nine independently blended plume carriers and four blended limbs.
Face, cheek fur and crest remain coherent with the head. The mane and chest
point move with the neck carrier. Paws and toe creases remain on the neutral
root. All nine cream/gold interfaces share exact stored positions and matching
quantized weights through idle, attack and hit. Neutral endpoints and existing
source-cue timing are retained.

Ten Python checks pass: canonical bytes/schema limits and lossless source
recovery, closed outward solids, actual normals, fitted eyes/ear insets,
anatomical owners, occupied roots with a detached negative control, all nine
plumes articulating independently, all nine material seams, floor/endpoints/
winding/strain/independent motion, and cue metadata. Both focused Rust rig/GPU
checks pass, including each packed tip's carrier, exact material seams,
floor/culling bounds and planted paws. Front, side and rear poses of all three
clips were inspected; Blender generation is byte-identical. Shiny verification
is skipped at the user's request.

Rebuild with `python3 tools/ninetales_glb.py`. Run asset checks with
`python3 -m unittest discover -s tools -p test_ninetales_glb.py`.

The rebuilt native Route 29 walking encounter against Sentret passes terrain
admission at original pack size. The inspected 3D/2D pair preserves core frame
3435, checksum 2149984195, LCD animation frame 4096 and virtual time
74.372243042. `CRYSTAL_BATTLE_REVIEW_YAW=-110` selects an existing establishing
angle with normal support, omission, scale and full-body/frustum checks.
The long muzzle and red gaze, cream mane and crest, high plume fan and lower
forward tails, golden ends and grounded stance remain readable with matching
NINETALES HUD labels. No shader errors were reported. Both shared grounded-
volumetric/model and exact source-cue timing regressions pass. Final ignored
evidence uses `target/pokemon-native-review/ninetales-normal`; browser visual
acceptance remains open.
