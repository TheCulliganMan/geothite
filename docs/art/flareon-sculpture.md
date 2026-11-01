# Flareon sculpture and rig

The generic orange fox is replaced by an original compact flame fox with a
full scalloped cream collar, broad curled cream plume and swept cream forehead
flame with two seated wisps. Rounded orange cheeks, deep blue gaze and white
eye glints, charcoal ear insets, a short smiling muzzle, rounded haunches and
four planted paws complete the silhouette. The visual reference is the
[official Flareon artwork](https://sg.portal-pokemon.com/pokedex/0136/).
No reference pixels are shipped.

`tools/flareon_sculpt.py` authors 46 closed parts, 11,495 vertices and 20,424
triangles. `tools/flareon_glb.py` exports a canonical 512,692-byte GLB with
209 accessors, 14 joints and authoring height 0.97. The neutral digest is
`2bdb14269978d1eaed0163530f5ecd9a966b45b5501db0927542151ec15268b5`.
The verified local pack supplies height digits 211 (0.889 metres), FLAME.
The original pack size is retained.

The skin uses body/neck/head carriers, independent ears, a separate collar
carrier, four blended limb carriers and a continuously weighted three-carrier
plume. The entire cream tail is one closed solid with gentle longitudinal
fur ridges. The forehead flame stays coherent with the head and gaze; the
collar owns all three chest lobes. Paws and toe creases stay on the neutral
root. Idle, attack and hit retain neutral endpoints and source-cue timing.
The old JSON adapter is removed; production rendering, Python asset readers
and the Blender batch entry point use the same canonical source.

Eight Python checks pass: canonical bytes/schema limits, closed outward
solids, actual face normals, fitted eyes and ear insets, anatomical owners,
occupied neck/collar/ear/limb/plume roots with a detached negative control,
floor/endpoints/winding/strain/independent motion, and source-cue metadata.
Both focused Rust rig/GPU checks pass. Blender generation is byte-identical,
and front, side and rear poses of all three clips were inspected. The native
example rebuild passes. Shiny verification is skipped at the user's request.

Rebuild with `python3 tools/flareon_glb.py`. Run asset checks with
`python3 -m unittest discover -s tools -p test_flareon_glb.py`.

The initial native view exposed a red cast in the orange body material. The
body/face palette was corrected against that production result; geometry,
weights and motion are unchanged. The final material passes all eight Python
checks and both focused Rust checks, with byte-identical Blender generation.
Both shared volumetric-model and exact source-cue timing regressions also pass.

The final native Route 29 walking encounter against Pidgey passes terrain
admission at original pack size. The inspected 3D/2D pair preserves core frame
3535, checksum 3189029028, LCD animation frame 4259 and virtual time
80.209025832. `CRYSTAL_BATTLE_REVIEW_YAW=-110` selects an existing establishing
angle while retaining terrain support, omission, scale and full-body/frustum
checks. The orange body, blue gaze, dark ear insets, cream forehead tuft,
collar and full plume remain readable, with planted paws and matching FLAREON
HUD labels. No shader errors were reported. Final ignored evidence uses
`target/pokemon-native-review/flareon-normal`; browser visual acceptance
remains open.
