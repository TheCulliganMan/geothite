# Scyther green mantis and curved-scythe skin

The current walking support packing and its revised runtime counts/pins are
recorded in [Bilateral walking support packing](world-support-packing.md).
The original logical sculpture, battle weights and clips are unchanged; the
earlier runtime packing and capture record below remains historical evidence.

`tools/scyther_sculpt.py` replaces the generic round-headed insect with an
original dedicated green mantis guided by
[the official Scyther artwork](https://sg.portal-pokemon.com/pokedex/0123/).
A rounded green thorax and pale breast shield narrow to a cream waist above
a tapered segmented abdomen. A short pale throat supports the angular mask,
white fitted eyes, tiny pupils, muzzle pores and restrained mouth seam.
Swept cranial points and a slim backward antenna complete the head silhouette.
Large rounded shoulders connect through thin pale elbow stalks to curved ivory
scythes with fitted green spines. Four green wing cases carry pale fitted, double-sided
membranes and gray central ribs. Pointed hip armor, pale shanks, flared green
greaves and broad three-claw feet replace the old short boot-shaped legs.
No reference pixels or third-party geometry are imported.

`tools/scyther_glb.py` exports the canonical standard glTF skin at
`crates/crystal-voxel-view/models/battle_species/scyther.glb`. The obsolete
static JSON and generic Scyther builder are removed. All 57 closed source
parts are exactly recoverable from 36 runtime parts through explicit vertex
and index ranges. Packing combines only compatible colors and carriers:
fixed greaves/pads/nails, cranial points and dark face marks, abdomen seams,
and paired wing cases/membranes/ribs. Geometry is preserved. The model has
19,533 vertices, 30,360 triangles, 13 joints and 191 accessors in 811,488 bytes.
Existing renderer limits are unchanged. Source SHA-256:
`ea4538d82e02fd1c896ad210cfe655355ddd53b56953e5ecdff59f86131452ea`.
Packed neutral SHA-256:
`d27719b98ff97185509c5375d650cfb8bfa4a25db49e9fe13115972b00903d3d`.
Authoring height is 1.18 with floor zero. The verified local pack gives height
digits 411 (4 feet 11 inches), or 1.4986 metres, classification `MANTIS`.

Greaves, foot pads and all six ivory nails stay on the identity root. Pale
shanks blend into independent armored coxae beneath the abdomen. Thorax,
abdomen, neck and head articulate separately; the complete gaze, muzzle and
cranial points share the head. Each scythe and its green spine share a blade
carrier below the upper arm. Four wing membranes, their cases and fitted ribs
move coherently on paired wing carriers. Attack sweeps the actual blade tips
independently of the upper arms; idle flutters the wings. Rotation-only clips
follow source cues and return literally to neutral. No hover or physical-size
adjustment is introduced.

The first elbow stalk stopped just short of its scythe; it was extended into
the actual blade root, and the pivot moved to that connection. Redundant blade
cap subdivisions were reduced while retaining the outline. Thirteen stored-model
checks pass for source recovery/schema limits, closed outward solids, actual
normals, fitted triangle/edge clearance, unfilled blade cutouts, occupied roots
and detached-blade/vein negative controls, coherent owners, planted armor,
posed floor/winding/strain, self-intersections, independent motion, real blade
tip sweep, neutral endpoints and source cues. Thin diagonal ribs are checked
against actual stored membrane volumes; 26–28 back vertices per rib are
embedded, while a displaced rib has none. Common ownership preserves that
attachment during motion. Blender regeneration is byte-identical. Shiny
verification is skipped.

Two focused Rust checks pass for exact registry routing and production geometry,
coherent gaze/blade-spine/wing/armor ownership, independently visible head,
blade, wing and coxa motion, planted armor, conservative culling bounds and
GPU activation at verified pack size. Front and rear neutral, idle, attack
and recoil views were inspected against the old generic static model. The
rear inspection led to pale membrane geometry on both wing faces, preserving
the green rims; actual back-face triangle/edge clearance is now checked too.

The two wider exact-species grounding/volume and elapsed-idle/source-attack
regressions pass with the final model and skin. The locked native production
example builds successfully using `target/pokemon-models` with local
`opt-level=0` overrides for crystal-core/assets/voxel-view/bevy.

The production Route 29 encounter against Rattata passes terrain admission
(`accepted=true`, 854 candidate stages checked). The inspected 3D/2D pair
preserves state at core frame 4531, checksum 1806325052, LCD animation frame
5374 and virtual time 113.673177877. The full curved blades, crest and wing
silhouette fit at original pack scale; the pale elbow stalks meet the blades,
clawed feet rest on source ground, and the HUD names both species. No shader
errors were reported. Original grass and forest remain in place. This verifies
the held native view; browser visual acceptance remains open. Final ignored
screenshots, state receipt and QA logs use prefix
`target/pokemon-native-review/scyther-normal`.
