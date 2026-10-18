# Gastly sculpture and rig

The generic ball, red eyes, tooth-bar grin and separate cloud blobs are replaced
by an original dark spherical ghost face, broad slanted white eyes, tiny black
vertical pupils, a pink crescent mouth and two ivory fangs. A continuous,
irregular purple gas cloak surrounds the orb, with nine tapered comma curls
distributed across its depth.
The silhouette and face follow the [official Gastly artwork](https://sg.portal-pokemon.com/pokedex/0092/).
No reference pixels are included in the asset. The vapor uses closed opaque
papercraft geometry consistent with the existing model renderer.

`tools/gastly_sculpt.py` authors the geometry; `tools/gastly_glb.py` exports
the canonical asset. Its 19 closed parts contain 18,911 vertices and 26,448
triangles in 809,624 bytes. Authoring height is 1.05. The neutral geometry digest
is `ed3adabb2b737302e42e576b7a43375e42045406d494e4beaed7acee1546ce32`.
The local content pack supplies height digits 403 (1.2954 metres), classification
GAS. Existing Float presentation supplies the intended hovering clearance.

The 11-joint skin has a coherent face carrier, a separate halo carrier and eight
angular gas-sector carriers. Adjacent sectors blend around the entire halo and
its curling wisps. Yaw-only gas deformation preserves the authoring floor while
the orb turns separately. Idle, attack and hit clips use the existing source-cue
timing and neutral endpoints.

Eight stored-asset Python checks pass: canonical bytes and limits, closed
outward solids, actual face normals, fitted face clearance, face/gas ownership,
occupied orb/halo/curl roots with a detached negative control, floor/endpoints/
winding/strain/independent motion, and source-cue metadata. Front and rear
software views of all three clips were inspected. Rebuild with
`python3 tools/gastly_glb.py`; run the asset checks with
`python3 -m unittest discover -s tools -p test_gastly_glb.py`.
Shiny verification is skipped at the user's request.

The first native side view exposed a thin planar mist silhouette. The final
halo has a turbulent 0.34-unit depth before normalization, with curls spread
forward and backward through the cloak. The revised side view was inspected
before rebuilding the native client.

The final asset passes both focused Rust checks, the shared volumetric/model
and source-cue timing regressions, and byte-identical Blender generation.
The native example builds with the existing `target/pokemon-models` cache
and `opt-level=0` overrides for crystal-core/assets/voxel-view/bevy.
The production Route 29 walking encounter against Sentret passes terrain
admission. The inspected 3D/2D pair preserves state at core frame 2110,
checksum 2652738907, LCD animation frame 2734 and virtual time 47.96444296.
The side view retains vapor depth and curls around the dark orb; the visible
eye and fang remain attached, the intended hover has a ground shadow, and
the HUD names both species. No shader errors were reported. Browser visual
acceptance remains open. Final ignored evidence uses prefix
`target/pokemon-native-review/gastly-normal`.
