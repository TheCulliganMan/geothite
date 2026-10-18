# Pinsir stag beetle and spiked-pincer skin

`tools/pinsir_sculpt.py` replaces the old generic upright beetle with an
original dedicated sculpture guided by
[the official Pinsir artwork](https://sg.portal-pokemon.com/pokedex/0127/).
One continuous oval brown carapace carries a stern fitted ivory gaze, tiny
black pupils and a dark upright mouth with four paired ivory tooth bars.
Three closed fitted seams wrap the lower shell. Huge pale curved pincers
each carry five rooted thorns. Thin bent arms rise to three-claw palms;
short rooted shanks end in planted brown tarsi with three broad ivory nails.
No reference pixels or third-party geometry are imported.

`tools/pinsir_glb.py` exports the canonical standard glTF skin at
`crates/crystal-voxel-view/models/battle_species/pinsir.glb`. The obsolete
static JSON and generic Pinsir builder are removed. The model has 53 closed
parts, 14,082 vertices, 23,756 triangles, 13 joints and 199 accessors in
565,000 bytes. Existing renderer limits are unchanged. Neutral/source SHA-256:
`347418889561cb58a6f9b3f05d441ca8f6bbe2a7b8fb6c4df81ec1a24ea01d84`.
Authoring height is 1.16 with floor zero. The verified local pack gives
height digits 411 (4 feet 11 inches), or 1.4986 metres, classification
`STAGBEETLE`.

Tarsi and foot claws remain on the identity root. Smooth weights connect
the shanks to independent leg and abdomen carriers. The one-piece shell
blends from abdomen to upper carapace, with all fitted eyes, aperture and
teeth fully owned by the upper carrier. Each pincer and all five thorns share
one carrier. Shoulders, elbows and wrists articulate separately; hand claws
share their palm carrier. Attack closes the pincers. Idle, attack and recoil
use rotation-only source-cue clips and return literally to neutral. No hover
or size adjustment is introduced.

The first thumb curve folded back through itself; its centerline was corrected.
The shell seams are closed toroidal strips fitted to stored front and rear
sections, with no coincident end caps. Nine stored-model checks pass for
canonical schema and limits, closed outward solids, actual normals, fitted
front triangle/edge clearance, occupied roots and a detached-spike negative
control, coherent owners, planted tarsi, posed floor, winding/strain,
self-intersections, independent motion, neutral endpoints and source cues.
Front and rear neutral, idle, attack and recoil views were inspected against
the old static model. Shiny verification is skipped.

Two focused Rust checks pass for exact registry routing, real production
geometry, coherent face/pincer/claw ownership, planted feet, independent
pincer/elbow/wrist motion, conservative culling bounds and GPU activation
at verified pack size. The common Blender builder regenerates identical bytes.

The two wider exact-species grounding/volume and elapsed-idle/source-attack
regressions pass. The rebuilt production Route 29 encounter against Sentret
passes terrain admission (`accepted=true`). The inspected 3D/2D pair preserves
state at core frame 2454, checksum 1836009259, LCD animation frame 3109 and
virtual time 54.840088499. The complete pincers, thorns, raised hands and feet
fit at original pack scale; the mouth bars and stern gaze remain readable,
feet rest on source ground and the HUD identifies both species. No shader
errors were reported. Original grass and forest remain in place. This verifies
the held native view; browser visual acceptance remains open. Final ignored
screenshots, state receipt and QA logs use prefix
`target/pokemon-native-review/pinsir-normal`.
