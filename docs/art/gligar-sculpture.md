# Gligar sculpture and standard skin

`tools/gligar_sculpt.py` authors the anatomy and `tools/gligar_glb.py` exports
canonical `models/battle_species/gligar.glb`. The species builder now emits this
skin and removes duplicate static JSON. The model has 32 closed parts, 17,422
vertices and 26,768 triangles. The normalized 1.06 span and floor origin remain
unchanged. The local pack's height 307 means 3′7″ (1.0922 m).

Actual local Crystal front/back sprites and normal palettes were inspected.
Source RGB555 pink (25,6,18) becomes sRGB8 (206,49,148), violet (8,7,20) becomes
(66,57,165), with white and black. The sails and tongue use violet, replacing
the previous muted blue sails and pink tongue. Geometry helpers perform the
only conversion to linear material colors. Historical part labels retain
“blue,” “ivory” and “pink tongue” for migration continuity.

The head, fitted eyes/pupils/glints, mouth and fangs share one owner. Ears and
tongue have their own joints; tongue and fitted crease remain coherent. Both
feet and talons stay fixed to the identity root. The body and legs blend from
fixed lower support to the torso. Each arm, sail and upper spar share the same
position-based shoulder/pincer weight field, keeping their attachments coupled.
Complete crescent pincers remain rigid, preserving their actual open centers
and jaws. A staged tail chain reaches the venom bulb and independently owned
stinger. Rotation-only idle/attack/hit clips have neutral endpoints and delayed
pincer/tail response, with authoritative battle cues owning playback timing.

The original boundary-only sail caps had long, thin triangles that inverted
under motion. Species-local interior support and constrained Delaunay flips
retain every original sail boundary position and the analytic depth/thickness,
while improving the cap mesh. Widening the pincer blend reduces sail strain.
The tongue was also narrowed and flattened after neutral inspection, retaining
its upper overlap with the head and mouth. The other 28 parts preserve exact
stored f32 positions, normals and triangle order from the previous static asset.
The two revised sails and tongue/crease are the only geometry changes.

The GLB is 756,232 bytes, with 179 accessors and 15 joints, within existing
loader limits. Its rest digest is
`9d3742959ab097a791488894bd78c071e6ace9c2dcb6c41b7dbd7e81ced1c10d`.
All eight stored-skin and ten migrated anatomy checks pass: canonical
regeneration, bind identity, source colors, fitted owners, coupled attachment
chains, grounded support, neutral endpoints, winding/edge strain, tongue
proportions, closed topology, actual pincer gaps and volumetric/surface support.
The focused Blender builder produces byte-identical GLB with no static JSON.
Final front and earlier refined rear software previews were inspected. Both
final production Rust checks pass: GPU skinning at the pack height, and
off-grid sampling for planted support, coherent owners and culling bounds.
The final native build and matched normal Gligar/Pidgey review pass: core 2025,
checksum 1364756158, LCD 2637, time 46.529815542, state preserved. Open pincers,
visible sails, fitted face, smaller violet tongue and grounded pads/talons remain
readable with floor contact shading. The curled tail is partly hidden in this
composition; its full rest shape was inspected in the rear software preview.
Native rest-pose/rig/color integration is accepted. Exact source-frame
face/wing motion, field gaits and browser review remain open. Shiny-specific
checks and captures stay skipped at the user's request.
