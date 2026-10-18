# Eevee sculpture and rig

The generic fox is replaced by an original compact brown evolution fox with
large pointed ears, fitted deep brown pinnae, a broad dark amber gaze, a small
soft muzzle and a curved smile. A continuous fluffy cream ruff and three chest
lobes sit around the short neck. Four planted paws support short tapered limbs
and rounded hind thighs. A broad raised brush ends in a cream tip with a gently
serrated material seam. The reference is the
[official Eevee artwork](https://sg.portal-pokemon.com/pokedex/0133/).
No reference pixels are included in the asset.

`tools/eevee_sculpt.py` authors 47 closed parts with 10,813 vertices and 19,076
triangles. `tools/eevee_glb.py` exports a 481,024-byte canonical GLB at authoring
height 0.80. The neutral geometry digest is
`dbb03e378a70bab065c75f1b0dd241eb71f6da0bdf126ba5f2289df74407d633`.
The local content pack supplies height digits 100 (0.3048 metres), EVOLUTION.
The original pack size is retained.

The 13-joint rig supplies body, neck and head carriers, two ear carriers,
three tail carriers and four limb carriers. Paws and toe creases stay on the
authoring root while smooth limb weights permit flexion between the fixed feet
and the torso. The two tail materials share the same 24 seam positions and
identical quantized weights, so the cream cut stays closed in every clip.
Idle, attack and hit retain neutral endpoints and existing source-cue timing.

Nine stored-asset Python checks pass: canonical schema and limits, closed
outward solids, actual face normals, fitted gaze and ear markings, anatomical
ownership and four floor contacts, occupied neck/ruff/ear/limb roots with a
detached negative control, the two-color tail seam throughout animation,
floor/endpoints/winding/strain/independent motion, and cue metadata. Front,
rear and side views of all three clips were inspected. Rebuild with
`python3 tools/eevee_glb.py`; run asset checks with
`python3 -m unittest discover -s tools -p test_eevee_glb.py`.
Shiny verification is skipped at the user's request.

Both focused Rust checks, the shared volumetric/model and source-cue timing
regressions, and byte-identical Blender generation pass. The native example
builds with the existing `target/pokemon-models` cache and `opt-level=0`
overrides for crystal-core/assets/voxel-view/bevy.

The production Route 29 walking encounter against Pidgey passes terrain
admission. The inspected 3D/2D pair preserves state at core frame 3039,
checksum 2392749496, LCD animation frame 3708 and virtual time 66.17505725.
The complete ears, fitted gaze, cream ruff and brush tip remain readable at
original pack size. Paws meet the source ground and the HUD names both species.
No shader errors were reported. Browser visual acceptance remains open.
Final ignored evidence uses prefix `target/pokemon-native-review/eevee-normal`.
