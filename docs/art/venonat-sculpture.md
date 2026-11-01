# Venonat sculpture and rig

Venonat's generic stacked insect is replaced by an original continuous fuzzy
purple body, large faceted pink compound lenses, fitted lens glints, a pink
snout and paired ivory fangs. Curved white feelers end in broad pale leaves.
Small purple arms carry ivory palms and three curled claws each. Short ankles
join two broad ivory feet. Three pointed crown tufts complete the silhouette.

`tools/venonat_sculpt.py` authors 33 closed parts, 15,723 vertices and
22,268 triangles. `tools/venonat_glb.py` exports a canonical 646,940-byte GLB
with 10 joints and 154 accessors at authoring height 1.10. The verified local
pack supplies height digits 303 (0.9906 metres), INSECT. Pack size is retained.
The neutral geometry digest is
`b1425a1645ef672eb73a014ed63285503c3f184ddecb3f782c466bea7163d4f3`.
All existing renderer and schema limits remain intact.

A coherent body carrier keeps the eyes, glints, nose, fangs and crown together.
Feelers, arms and claw hands articulate separately. Blended lower body and
ankles allow motion while both feet and their toe creases remain fixed to the
floor root. Idle, attack and hit retain neutral endpoints and source-cue timing.

Eight Python checks pass: canonical bytes and limits, closed outward solids,
actual face normals, fitted glints on the faceted lenses, anatomical owners,
occupied feature roots through motion with a detached negative control,
floor/endpoints/winding/strain/motion, and source-cue metadata. Front, side and
rear poses of all three clips were inspected. Blender generation is
byte-identical. Shiny verification is skipped at the user's request.

Rebuild with `python3 tools/venonat_glb.py`. Run asset checks with
`python3 -m unittest discover -s tools -p test_venonat_glb.py`.

Both focused Rust checks pass, covering runtime routing, exact anatomy owners,
floor/culling bounds, fixed feet, independent feeler/arm/hand motion and GPU
activation at verified pack height. Shared grounded/volumetric and source-cue
pose-timing regressions also pass.

The rebuilt native Route 29 walking encounter against Pidgey passes normal
terrain admission at original pack size. The inspected 3D/2D pair preserves
core frame 2199, checksum 584843726, LCD animation frame 2794 and virtual time
48.732609791. `CRYSTAL_BATTLE_REVIEW_YAW=-110` selects an existing supported
angle with the normal size, support, omission and whole-body/frustum checks.
Compound eyes, feelers, fangs and claw hands read clearly; both broad feet meet
the ground. The matching VENONAT HUD names the modeled Pokémon. No shader
errors were reported. Final ignored evidence uses
`target/pokemon-native-review/venonat-normal`. Browser visual acceptance
remains open.
