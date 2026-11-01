# Haunter sculpture and rig

The generic sphere, radial spike leaves and simple red-eyed grin are replaced
by an original continuous ghost head with tall ear points, a serrated shroud,
and a broad rounded central face. Two detached palms carry three tapered,
curved purple claws each. Fitted white slanted eyes, fine vertical pupils,
a pink open grin and eight purple pointed teeth follow the
[official Haunter artwork](https://sg.portal-pokemon.com/pokedex/0093/).
No reference pixels are included in the model.

`tools/haunter_sculpt.py` authors 23 closed parts with 9,833 vertices and
13,444 triangles. `tools/haunter_glb.py` exports a 432,788-byte canonical GLB
at authoring height 1.10. The neutral digest is
`de932449c4d842c05d74073ea292ea5b1a81cc4e565110568e11bef78b047f73`.
The local content pack supplies height digits 503 (1.6002 metres), GAS.
Existing Float presentation supplies the intended hovering clearance.

The 16-joint skin retains one coherent face carrier, six smoothly blended
ear/shroud/hem hinges, two detached hand carriers and six independent finger
carriers. Idle, attack and hit clips curl the claws independently of their
palms. The body hinges use yaw to preserve the authoring floor. The original
authoring root stays neutral and the existing source cues own clip timing.

The body uses a rounded central face volume and a shared peripheral loop for
its continuous spiked silhouette. Adaptive face fitting uses the existing
indexed triangle-surface reader through an optional sampler in the shared
helper. This preserves the complete canonical GLB byte for byte while avoiding
full-mesh ray scans during authoring; existing callers retain the same default.

Nine Python asset checks cover canonical schema and limits, closed outward
solids, actual face normals, well-conditioned convex eye caps, fitted face clearance, ownership, detached hands
and occupied finger roots with a detached negative control, floor/endpoints/
winding/strain/independent motion, and source-cue metadata. Front, rear and
side idle/attack/hit views were inspected. Rebuild with
`python3 tools/haunter_glb.py`; run asset checks with
`python3 -m unittest discover -s tools -p test_haunter_glb.py`.
Shiny verification is skipped at the user's request.

The first native view exposed small dark interference marks on the white eye
lenses. Additional clearance alone did not resolve them. The final lenses use
a smooth convex surface and five-point angular outlines with their backs
seated on the actual body. This removes ill-conditioned cap faces and reduces
the original draft by 4,332 vertices. A regression requires every front eye
triangle to align with the convex surface normal, in addition to clearing
the actual stored parent.

The final asset passes all nine Python checks, both focused Rust checks, the
shared volumetric/model and source-cue timing regressions, and byte-identical
Blender generation. The native example builds with the existing
`target/pokemon-models` cache and `opt-level=0` overrides for
crystal-core/assets/voxel-view/bevy.

The production Route 29 walking encounter against Pidgey passes terrain
admission. The inspected final 3D/2D pair preserves state at core frame 1172,
checksum 2351786109, LCD animation frame 1840 and virtual time 34.827198458.
The white eyes render cleanly; the full head, shroud and detached three-clawed
hands remain readable at original pack size. The intended hover has a ground
shadow and the HUD names both species. No shader errors were reported.
Browser visual acceptance remains open. Final ignored evidence uses prefix
`target/pokemon-native-review/haunter-normal`.
