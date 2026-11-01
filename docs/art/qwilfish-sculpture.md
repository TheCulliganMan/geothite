# Qwilfish sculpture and standard skin

`tools/qwilfish_sculpt.py` authors the anatomy and `tools/qwilfish_glb.py` exports
its canonical `models/battle_species/qwilfish.glb`. The species builder uses this
standard GLB and retires the duplicate static JSON. All 36 parts retain the
previous asset's exact f32 positions, normals and triangle order: 9,014 vertices
and 13,644 triangles. The normalized 0.91 reference span and floor origin are
unchanged. The local pack's height 108 means 1′8″ (0.508 m) in production.

Two closed hemispheres share the complete equator of one inflated ellipsoid.
Fourteen rooted spines project in three dimensions. Pointed eyes, fitted pupils,
pursed lips, small pectoral fins and their V folds retain their authored support.
A curved caudal peduncle reaches a rounded paddle with fitted centers and two
oblong spots on each face. Historical part names retain “teal,” “cream” and
“pink” for migration continuity; actual colors now follow the local Crystal
front/back sprite: RGB555 navy (4,6,16), yellow (27,29,11), white and black.
The lips are yellow, and both fitted paddle centers use source white. Geometry helpers perform the sole conversion from sRGB to
linear material colors.

Seven joints use the shared runtime skinning path. The inflated body, face and
all spines share one rigid owner. Each pectoral fin and its fitted fold blend
from the body to their own joint. Three tail stages bend the peduncle, while the
complete spotted paddle follows its final joint rigidly. Idle, attack and hit
clips have an identity root, neutral endpoints and rotations only. Yaw-only
motion preserves every vertex's floor height; delayed tail motion follows the
initial body/fin stroke. Authoritative battle cues own playback timing. These
clips do not yet claim exact reproduction of the source sprite frames or field
gaits.

The 378,948-byte asset uses 142 accessors, within existing loader limits. Its
rest digest is `84580db6a0528140ae8fb939c5808b542d23f52df996fb2eb072b8352d1dd6d1`.
Focused Blender species-builder output is byte-identical and emits no duplicate
static JSON. The independent stored-GLB and migrated anatomy suites pass all 14 checks:
canonical regeneration, bind identity, source colors, rigid attachment owners,
fin/tail weights, neutral endpoints, floor height, winding, edge strain, shared
body seam, closed geometry and fitted feature support. Both focused production Rust checks pass: GPU skinning at the pack height,
and off-grid animation sampling for coherent attachments, floor height and
culling bounds. The initial native build and matched normal pair passed integration checks,
but inspection exposed yellow paddle centers where the source reads white.
Both centers are now white; a material-only audit proves the binary buffer,
nodes, skin, clips and schema unchanged. Final canonical/palette checks pass.
The earlier 14 Python, two Rust and Blender results predate this two-assignment
material correction. The white-center native build and matched normal review pass (core 1456,
checksum 3488417035, LCD 2095, time 36.573381584, state preserved). The paddle
is partly hidden by the body in this composition; its visible center is white.
This accepts native integration, with exact motion and all-angle fidelity open. Shiny-specific verification is
skipped at the user's request.
