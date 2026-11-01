# Rhyhorn sculpture and rig

The current [full-turnaround fidelity review](rhino-larva-fidelity.md) replaces
thin dorsal fins with four thick peaked shields wrapping both flanks. The
original `tools/rhyhorn_sculpt.py` recipe authors a low tapered skull, fitted
brow and red gaze, pointed ears with fitted openings, nasal horn, backed mouth,
two small fangs, stocky supports, full flat feet and twelve blunt claws.
Downloaded game meshes are viewing references only.

`tools/rhino_glb.py` exports 51 logical/runtime parts, 11,214 vertices,
15,636 triangles, 11 joints, 217 accessors and 457,568 bytes. Authoring height
remains 1.11; pack height remains 0.9906 m. The source and packed digest is
`bc4995af2a77d7d299da382bff3dd3b7f05de0563c70fb128b0241ab609c67ae`.

The body carries the shields; the skull carries its face, brow, horn, lining
and fangs. The jaw, ears/openings and short tail have independent controls.
Four supporting leg chains blend between body, leg and ground root. Pads and
matching claws stay fixed in source-cue idle/action/recoil; whole named supports
participate in world walking. All anatomical pivots follow the revised recipe.

The fidelity review records the inspected game/front/back references, original
corrections, executed checks, native coverage and remaining limits. Python
source, skin and independent volume regressions are `test_rhyhorn_sculpt`,
`test_rhino_glb` and `test_rhino_larva_fidelity`. The regular Blender builder
reproduces the canonical asset exactly. No pack rebuild is needed.
