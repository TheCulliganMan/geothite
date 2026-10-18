# Heracross single-horn beetle and armored combat skin

`tools/heracross_sculpt.py` replaces the generic round-eyed, mitten-handed
static beetle with an original dedicated sculpture guided by
[the official Heracross artwork](https://sg.portal-pokemon.com/pokedex/0214/).
No reference pixels or third-party geometry are imported. A broad blue
armored abdomen supports a fitted central belly plate with two longitudinal
seams, a thick thorax collar and rounded head mask. Golden fitted eyes and
a shallow scalloped mouth seam replace the projecting white eye buttons.
The long flattened horn has a continuous closed forked silhouette, rather
than three intersecting rods. Slim curved antennae end in round beads.
Armored shoulder sockets connect through narrow upper arms to broad
serrated gauntlets and paired hooked ivory claws. Large coxae carry pointed
upper armor; short shins end in planted tarsi with broad forward claws.
A fitted rear seam divides the closed wing cases.

`tools/heracross_glb.py` exports the canonical standard glTF skin at
`crates/crystal-voxel-view/models/battle_species/heracross.glb`.
The obsolete static JSON is removed; the common Blender builder regenerates
identical bytes. The model has 47 closed parts, 19,588 vertices, 32,148
triangles, 12 joints and 223 accessors in 814,644 bytes. Existing renderer
limits are unchanged. Neutral/source SHA-256:
`a33ddd1481ba4cb3435132daf36f3c5770e6b8155e24afc627142a32a82d923d`.
Authoring height is 1.16 with floor zero. The verified local pack gives
height digits 411 (4 feet 11 inches), or 1.4986 metres, classification
`SINGLEHORN`.

Foot pads and their claws remain on the identity root. Smooth weights join
shins and coxae through leg and body carriers. Body, thorax and head articulate
separately. The complete gaze, mask seam and forked horn share the head;
antenna stems and beads share their own carriers. Each gauntlet, its serrations,
knuckle and two claws share a forearm carrier, articulated below the upper
arm. Attack lowers the horn; idle and recoil follow source-cue timing.
Rotation-only clips return literally to neutral. No hover or size change is
introduced.

Nine stored-model checks pass for canonical schema and limits, closed outward
solids, actual normals, fitted front triangle and edge clearance, occupied
roots and a detached-bead negative control, coherent owners, fixed tarsi,
posed floor, winding/strain, self-intersections, independent motion, literal
neutral endpoints and source cues. Two focused Rust checks pass for registry
routing, real production geometry, coherent owners, fixed claws, independent
head/antenna/forearm motion, culling bounds and GPU activation at verified
pack size. Blender regeneration is byte-identical. Front and rear neutral,
idle, attack and recoil software views were inspected against the previous
static model. Shiny verification is skipped.

The two wider exact-species grounding/volume and elapsed-idle/source-attack
regressions pass. The rebuilt production Route 29 encounter against Pidgey
passes terrain admission (`accepted=true`). The inspected 3D/2D pair preserves
state at core frame 3014, checksum 2825358273, LCD animation frame 3737 and
virtual time 72.298003834. The whole horn, feeler beads and claws fit in view;
the armor reads clearly, feet rest on source ground and the HUD identifies
both species. No shader errors were reported. Original grass and forest
remain in place. This verifies the held native view; browser visual acceptance
remains open. Final ignored screenshots, state receipt and QA logs use prefix
`target/pokemon-native-review/heracross-normal`.


Canonical count correction: independently read GLB POSITION accessors total
19,588 vertices; the current source exporter reproduces all 814,644 bytes
exactly. The registry fixture and count above now match that unchanged asset.
