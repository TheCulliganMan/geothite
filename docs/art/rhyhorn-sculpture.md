# Rhyhorn sculpture and rig

`tools/rhyhorn_sculpt.py` authors the heavy gray barrel, lowered wedge muzzle,
short rooted nasal horn, pointed ears, fitted red gaze, armored lower jaw,
four stocky legs, knee cuffs, broad feet and twelve supported ivory claws.
Four thick dorsal plates intersect the barrel, and the forehead crest remains
seated in the head. No external geometry is imported.

`tools/rhino_glb.py` exports the canonical `rhyhorn.glb`: 46 parts,
14,488 vertices, 25,368 triangles, 11 joints, 178 accessors and 520,208 bytes.
Authoring height remains 1.11; the verified pack height is 0.9906 m.
The neutral geometry digest is
`e6ad87610f6a2b99b8e1780fddffabfb97166fdb1c8d50278ca6e765dcbc0188`.

The skin carries the body and dorsal armor together, the head and face
coherently, and gives the jaw, both ears and short tail independent joints.
Four complete leg chains retain fixed floor pads and toe claws. Idle, attack
and hit clips use the existing source-cue clocks, with neutral endpoints.
World walking uses the four authored support chains. The dedicated builder,
asset reader and runtime registry use the GLB; the obsolete static JSON is
removed. The generic rock builder continues to reject both rhinoceroses.

All 31 combined Python checks for Rhyhorn and Rhydon pass, including closed
outward geometry, fitted markings, occupied attachment roots through motion,
weights, local joint articulation, floor contact, winding and strain. Four
focused Rust checks pass for their GPU paths at verified pack sizes, runtime
articulation and world support chains. Front, side and rear poses of all three
clips were inspected. Blender exports are byte-identical. Shiny checks are
skipped at the user's request.

The rebuilt native Route 29 Rhyhorn/Pidgey 3D/2D pair was inspected at identical
state: frame 4472, checksum 3264865042, LCD frame 5203 and virtual time
97.399773957. The complete armored silhouette, red gaze and species HUD are
readable; all four pads meet the terrain. Normal admission at the original
pack size passes with the existing -110-degree review angle. No shader errors
were reported. Ignored evidence is `target/pokemon-native-review/rhyhorn-normal`.
Both shared geometry and source-cue timing regressions pass. Grass height is
untouched. Browser visual acceptance remains open.
