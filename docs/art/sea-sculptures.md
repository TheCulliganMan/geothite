# Corsola and Mantine sculptures

`tools/sea_sculpt.py` authors both species. Corsola now uses `corsola.glb`,
exported by `tools/corsola_glb.py` and dispatched before Blender batching.
Its retired static JSON is removed. All 37 original parts retain every neutral
f32 position, normal, color and triangle: 7,359 vertices and 14,180 triangles.
Nine joints use 286,056 bytes and 149 accessors within unchanged loader budgets.
Authoring height remains 0.93; verified pack height is 0.6096 m.

Corsola's five complete coral stems articulate independently; each stem carries
its rounded primary tip, lateral fork and fork tip as one coherent unit.
Each arm carries its rounded tip through a visible forward/upward gesture.
The pink crown, white bowl, eyes, glints, fitted smile and diamond flecks share
a body owner. The body turns above four pads fixed on the identity floor root.
Idle gives a gentle branch sway; attack gestures with both arms; recoil settles
back. Literal neutral endpoints and standard rotation-only source-cue timing
remain. Neutral f32 digest:
`70a819e196fadc5fade144ed07e3483aa3a404f9c1a373f572975adb9d9f65d2`.

Mantine now uses `mantine.glb`, authored by `tools/mantine_glb.py` and dispatched
by the battle builder before Blender batching. Its retired static JSON is
removed. Broad swept two-tone wings retain their curved cross sections and
shared blue/pale boundaries, seated zigzag underside borders, fitted blue
mantle, rounded cephalic paddles, gills and tapered tail with two seated terminal
fin layers. The attached sea-green Remora retains its muzzle, fitted mouth,
eyes and glints, dorsal attachment fin, curved stalk and forked tail.

Every original triangle position, triangle order and color is preserved.
The dorsal mantle, two wing borders and five Remora markings gain shared
vertices and crease-aware smooth normals. The other 24 parts preserve their
exact original neutral f32 positions, normals, colors and triangles. This
reduces the old 40,933 vertices to 19,485 within the existing rig limit.
The model has 32 parts, 27,416 triangles, 13 joints, 824,704 bytes and 168
accessors. Loader budgets remain unchanged. Authoring height stays 1.02;
verified pack dimension is 2.1082 m, applied to nose-to-tail length using the
existing fish measurement rule. Uniform proportions are preserved; cephalic
fin elevation is not used as a standing-body size reference.

The identity floor root supports a body that turns without changing contact
height. Coherent marked wing spans lift in a swimming stroke; flex is confined
to their unmarked distal tips so the narrow zigzags stay seated and undistorted.
Both color layers use the same spatial weight field and retain every shared
seam exactly. Cephalic paddles articulate independently. The long tail blends
into a rigid owner shared by both seated terminal layers. Remora follows the
proximal right wing, keeps its complete face together, and flicks its linked
tail. The resting pose is the lowest wing pose, keeping the attached fish above
the floor throughout idle, attack and recoil. Standard rotation-only
source-cue timing and literal neutral endpoints remain.

Neutral f32 digest:
`712a8adc15f3cc0fde8aab46912528e87185c6ea8c953ec4283a9f00db1dd7b7`.
Original triangle-position/color digest:
`5e376dff70947ca933869c3ad0a542b0ca33a0d379d048719cf81f29ace4910e`.
The 24 unchanged parts retain digest
`541ff047c2b76639172432964a8ddaa140c2321232185f563fc6842834240053`.

No external mesh, texture, rig or extracted content is shipped.

```sh
PYTHONPATH=tools python3 -m unittest test_sea_sculpt test_mantine_glb test_corsola_glb
```

Ten anatomy checks retain closed topology, shading, regeneration, Corsola's
original shape and grounding, shared wing boundaries, fitted facial/back/wing
and Remora details, and physical occupied joins. Nine skin checks decode the
stored hierarchy, keys and weights to verify original triangles and unaffected
parts, coherent owners, bind and neutral endpoints, floor clearance, visible
motion, every shared wing seam, winding and physical edge strain, actual posed
joins, seated markings, strong wing lift and independent distal-tip flex.
Shiny checks are intentionally skipped.

All ten sea anatomy checks and nine Mantine skin checks passed, including the
final unchanged-part/longitudinal-span pin and independent distal-tip check.
Three focused Rust size/rig/GPU regressions and two wider catalog/timing
regressions passed. Blender regeneration is byte-identical to the canonical
GLB. The production native client built successfully. Front, rear and
underside neutral/idle/attack/recoil views were inspected, followed by a normal
paired 3D/2D battle against Sentret. The corrected fish scale produces an
accepted shot on visibly rendered terrain; belly contact, the broad silhouette,
seated zigzags, face and attached Remora are readable. Game state is preserved.
Shiny checks were skipped as requested.

Final normal pair: core 1145/hash 1722243278, LCD 1816, virtual time
32.474173165; `state_preserved=true`. Final QA prefix: `mantine-normal` under
ignored `target/pokemon-native-review`.

Eight Corsola skin checks decode the stored hierarchy, keys and weights to
verify exact original geometry, bind identity, coherent whole-branch and arm
owners, four fixed pads, neutral endpoints, floor clearance, independent tip
motion, winding and unchanged edge lengths, actual posed crown/bowl/foot,
branch/fork/tip and eye/arm joins, and fitted smile/fleck support.

All 18 Python sea-anatomy/Corsola-skin checks, two focused Rust rig/GPU
regressions and two wider catalog/timing regressions passed. Blender
regeneration is byte-identical to the canonical GLB. The production native
client built successfully. Front/rear and fixed-camera neutral, idle, attack
and recoil views were inspected, followed by a normal paired 3D/2D battle
against Sentret. The pads meet visibly rendered terrain, the crown and complete
forks remain readable, and game state is preserved. Shiny checks were skipped
as requested.

Final normal pair: core 1669/hash 2191196249, LCD 2317, virtual time
41.757402709; `state_preserved=true`. Final QA prefix: `corsola-normal` under
ignored `target/pokemon-native-review`.
