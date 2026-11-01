# Scizor steel mantis and open-pincer skin

The current walking support packing and its revised runtime counts/pins are
recorded in [Bilateral walking support packing](world-support-packing.md).
The original logical sculpture, battle weights and clips are unchanged; the
earlier runtime packing and capture record below remains historical evidence.

`tools/scizor_sculpt.py` replaces the generic round-headed insect and its
painted claw splits with a dedicated original steel mantis guided by
[the official Scizor artwork](https://sg.portal-pokemon.com/pokedex/0212/).
An angular red thorax narrows to a slender waist above a long hanging abdomen.
Dark fitted chest armor and three abdominal chevrons follow those stored
surfaces. A short neck supports a pointed mask, narrow golden gaze and three
swept red cranial points. Armored shoulders connect through thin upper arms
and forearms to large red pincers. Separate upper and lower solids leave real
empty jaw apertures, edged by fitted dark crushing lips. Yellow false-eye
markings follow each upper claw. Four pale wing leaves carry fitted gray ribs.
Slender femurs lead to long red greaves, narrow grounded pads and pale pointed
toe caps. No reference pixels or third-party geometry are imported.

`tools/scizor_glb.py` exports the canonical standard glTF skin at
`crates/crystal-voxel-view/models/battle_species/scizor.glb`. The old static
JSON and generic Scizor builder are removed. The sculpture has 55 closed
source parts, recoverable from 43 runtime parts through explicit vertex/index
ranges. Only compatible colors and carriers are packed together: fixed leg
armor, crests, abdominal plates and paired wing leaves/ribs. No geometry is
removed by packing. The model has 18,558 vertices, 27,348 triangles, 17 joints
and 239 accessors in 813,656 bytes. Existing renderer limits are unchanged.
Source SHA-256:
`9b03e41cd8685a3ec5521fbc993abcdc872bd931a17cfdfb4fb1fdb1982bf7f3`.
Packed neutral SHA-256:
`b460947f4351d6f0f28a3d92168aab50ab81ca53215d603ad2f41aa1893d64a8`.
Authoring height is 1.28 with floor zero. The verified local pack gives height
digits 511 (5 feet 11 inches), or 1.8034 metres, classification `SCISSORS`.

Greaves, foot pads and toe caps stay on the identity root. Smooth weights
support femurs and hip armor while thorax, abdomen, neck and head articulate
separately. The fitted golden gaze and three cranial points share the head.
Shoulders, elbows, wrist pincers and lower jaws have their own carriers.
Crushing lips follow the corresponding jaw, false-eye layers follow the upper
claw, and fitted wing ribs follow their paired leaf carriers. Attack closes
the jaws; idle flutters the wings. Rotation-only clips follow source cues and
return literally to neutral. No hover or physical-size change is introduced.

Unnecessary pincer-cap and false-eye subdivisions were reduced while retaining
the complete silhouette. Wing ribs were fitted to actual stored leaves rather
than suspended at a constant offset. Eleven stored-model checks pass for source
recovery and schema limits, closed outward solids, actual normals, fitted
front triangle/edge clearance, real empty pincer apertures and a filled-gap
negative control, occupied roots and a detached-jaw control, coherent owners,
planted armor, posed floor/winding/strain, self-intersections, independent
motion, real jaw-tip closure with retained clearance, literal neutral endpoints
and source cues. Shiny verification is skipped.

Two focused Rust checks pass for exact registry routing and production geometry,
coherent face/jaw/wing/armor ownership, independent head/pincer/jaw/wing motion,
planted armor, conservative culling bounds and GPU activation at verified pack
size. The common Blender builder regenerates identical bytes. Front and rear
neutral, idle, attack and recoil views were inspected against the old generic
static model.

The two wider exact-species grounding/volume and elapsed-idle/source-attack
regressions pass with the new skin registered.

The rebuilt production Route 29 encounter against Pidgey passes terrain
admission (`accepted=true`, 867 candidate stages checked). The inspected
3D/2D pair preserves state at core frame 6686, checksum 135714849, LCD animation
frame 7633 and virtual time 169.180527293. The full pincers and three-pointed
crown fit at original pack scale; open jaw gaps and false eyes remain readable,
foot pads rest on source ground, and the HUD names both species. No shader
errors were reported. Original grass and forest remain in place. This verifies
the held native view; browser visual acceptance remains open. Final ignored
screenshots, state receipt and QA logs use prefix
`target/pokemon-native-review/scizor-normal`.

The successful local native build used `CARGO_TARGET_DIR=target/pokemon-models`,
`cargo build --locked -p crystal-bevy --features location-tester,bevy/zstd_rust
--example immersive_battle_3d -j1`, with `profile.dev.package.<crate>.opt-level=0`
for `crystal-core`, `crystal-assets`, `crystal-voxel-view` and `crystal-bevy`.
The successful build log is `scizor-native-build-space-retry.log`.
