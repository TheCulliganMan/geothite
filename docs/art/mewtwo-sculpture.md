# Mewtwo genetic psychic cat and planted skin

> The October 20 [reference and leg-profile revision](upright-trio-fidelity.md) supersedes the former posture, tail shape and digest below.

`tools/mewtwo_sculpt.py` replaces the old all-purple, round-eyed generic cat
with an original pale genetic body, purple pelvis and heavy curved tail,
guided by [the official Mewtwo artwork](https://sg.portal-pokemon.com/pokedex/0150/).
The narrow fitted violet gaze, long muzzle, blunt cranial horns, back-neck
arch, three long fingers with rounded contact tips, powerful haunches and
three-pad feet distinguish it from Mew. No reference pixels or third-party
geometry are imported.

`tools/mewtwo_glb.py` exports the canonical standard glTF skin at
`crates/crystal-voxel-view/models/battle_species/mewtwo.glb`. The old static
JSON is removed; the common Blender builder regenerates identical bytes.
The model has 52 closed parts, 15,722 vertices, 27,880 triangles, 17 joints
and 227 accessors in 633,436 bytes, within existing renderer limits.
Neutral/source SHA-256:
`ef903cf6d3fbd2c2b7ed2d8777e1d91726c56940e30f77aaa767e5e50eceb521`.
Authoring height is 1.45 with floor zero. The verified local pack gives
height digits 607 (6 feet 7 inches), or 2.0066 metres, classification `GENETIC`.

The foot and toe pads are anchored to the identity root. Smooth weights
connect the hocks and haunches to knees, hips and torso. Shoulders, elbows
and wrists articulate separately; fingers and their contact pads share each
hand. The neck and spinal arch blend through torso, neck and head carriers.
The complete gaze, muzzle and horns follow the head. Three tail carriers
blend by nearest-centerline arc length, with a coherent rounded terminal.
Idle, attack and recoil use rotation-only source-cue clips and return
literally to neutral. No render lift or physical-size adjustment is added.

Eight stored-model checks pass for canonical schema/limits, closed outward
solids, actual normals, front triangle and edge clearance, occupied roots
and a detached-fingertip negative control, planted pads, neutral endpoints,
independent motion, winding, strain, nonintersecting anatomy and source cues.
Two focused Rust checks pass for exact registry routing, planted pads,
coherent head ownership, independent motion, conservative culling bounds
and GPU activation at verified pack size. Shiny verification is skipped.

The two wider exact-species grounding/volume and elapsed-idle/source-attack
regressions pass. Neutral and idle/attack/recoil front and rear software views
were inspected. The rebuilt production Route 29 encounter against Pidgey
passes terrain admission (`accepted=true`). The inspected 3D/2D pair preserves
state at core frame 1332, checksum 1546947256, LCD animation frame 1943 and
virtual time 34.104813792. The complete body fits at original pack scale;
feet rest on source ground, the narrow gaze and three-finger hands read clearly,
and the HUD identifies both species. No shader errors were reported. The
original grass and forest remain in place. This verifies the held native view;
browser visual acceptance remains open. Final ignored images, state receipt
and QA logs use prefix `target/pokemon-native-review/mewtwo-normal`.
