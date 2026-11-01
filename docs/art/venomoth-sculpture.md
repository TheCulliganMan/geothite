# Venomoth sculpture and rig

The generic stacked insect and angular fans are replaced by an original lilac
moth with long swept forewings and smaller scalloped hindwings. Fine dark veins
follow both faces of the actual wing surfaces. A round head carries pale blue
lenses, fitted dark round pupils and ivory reflections, paired curved ivory
mandibles and three pointed lilac antennae. Six short tucked legs flank a long
gray lavender abdomen with three seated zigzag seams. The visual reference is
the [official Venomoth artwork](https://www.pokemon.com/us/pokedex/venomoth).
Reference pixels are not shipped.

`tools/venomoth_sculpt.py` authors 53 closed parts, 19,412 vertices and
30,580 triangles. `tools/venomoth_glb.py` exports a canonical 784,824-byte GLB
with 17 joints and 223 accessors at authoring height 1.16. The verified local
pack supplies height digits 411 (1.4986 metres), POISONMOTH. Original pack size
is retained. The neutral geometry digest is
`7670a960d16e7c38c3e79a8ffe12ba570ca202b6e77bb25f1d7f21d17dcb5323`.
Existing renderer and schema limits remain intact.

Four independent wing joints carry each complete membrane and its matching
veins. Three antenna carriers articulate separately from the head, keeping the
pupils, glints and jaws coherent with the face. The striped abdomen turns on
its own carrier. Six leg carriers keep each tucked toe coherent with its leg;
leg roots blend into the thorax. The floor root remains neutral, and yaw-only
thorax/abdomen motion preserves the lower oval's floor contact. Idle, attack
and hit retain neutral endpoints and source-cue timing. Existing Fly locomotion
is unchanged; no new battle hover is introduced.

Nine Python checks pass: canonical bytes and schema limits, closed outward
solids, actual face normals, fitted pupils and glints, anatomical owners,
occupied roots through motion with a detached negative control, seated veins
and zigzag seams on stored surfaces, floor/endpoints/winding/strain/motion,
and source-cue metadata. Front, side and rear poses of all three clips were
inspected. Blender generation is byte-identical. Shiny verification is skipped
at the user's request.

Rebuild with `python3 tools/venomoth_glb.py`. Run asset checks with
`python3 -m unittest discover -s tools -p test_venomoth_glb.py`.

Both focused Rust checks pass, covering runtime routing, exact anatomy owners,
floor/culling bounds, independent wings/antennae/legs/abdomen and GPU activation
at verified pack height. Shared grounded/volumetric and source-cue pose timing
regressions also pass.

The rebuilt native Route 29 walking encounter against Sentret passes normal
terrain admission at original pack size. The inspected 3D/2D pair preserves
core frame 1801, checksum 3957057059, LCD animation frame 2417 and virtual time
42.825647333. `CRYSTAL_BATTLE_REVIEW_YAW=-110` selects an existing supported
angle with normal size, support, omission and whole-body/frustum checks.
The four swept wings and veins, pointed antennae, pale round gaze, ivory jaws,
tucked legs and striped abdomen remain readable. The lower oval meets the
ground, and the matching VENOMOTH HUD identifies the modeled Pokémon.
No shader errors were reported. Final ignored evidence uses
`target/pokemon-native-review/venomoth-normal`. Browser visual acceptance
remains open.
