# Farfetch’d production skin

`tools/leek_duck_glb.py` authors the standard skin from the original duck recipe
with `rig_ready=True`. The canonical asset is
`models/battle_species/farfetch_d.glb`; the shared registry, renderer, readers,
STL exporter and background builder select it. Native review is complete.
The full-roster model and rig improvement goal remains active.

The original sculpture has 29,151 vertices and 43,988 triangles, exceeding the
shared skin limits. Adaptive parent-clearance refinement replaces uniform
subdivision on six eye-border, eye-white, brow and bill-seam surfaces. Four
pupil/glint layers refit to those new parent triangles, preserving their exact
XY coordinates and triangle order. All 35 other parts retain every original
f32 position, normal, color and triangle bit. All 45 names and colors remain.

The skin has 17,863 vertices, 26,972 triangles, 45 logical anatomy parts and
21 packed primitives. All six original neutral bounds remain exactly unchanged,
including zero footing and authoring height 1.10. Runtime physical size comes
from the local pack: `heightDigits=207`, or 0.7874 metres. Material packing
retains logical source ranges and individual weights without duplicating geometry.

Twenty-one controls cover the identity floor, yaw body, coherent head and bill,
raised wing and wrist, holding shoulder/elbow/grip, leek base and tip, three
leaf blades, three tail feathers, three crown prongs and two supporting shins.
The complete webs and all six surface-fitted web folds remain fixed in battle.
Both grip curls share the hand transform; the stalk inherits that grip before
its own small sway and tip bend. Leaf roots follow the sheath. The continuous
holding arm blends across its chain, while its three folded feather fingers
move rigidly with the hand. An initial independently blended finger field folded
a thin triangle; the rigid hand fixes it without relaxing winding or strain
checks. Wing wrist/elbow names explicitly retain their wing role so walking
does not apply the generic arm swing to the carried leek.

Idle, attack and hit are rotation-only, neutral-endpoint clips. Attack and hit
are relative to source battle cue progress. No pack, audio, gameplay, grass,
physical-size, camera or palette change is part of this migration. No dedicated
shiny tests or captures were run.

The nine original stored-mesh checks and nine revised anatomy checks pass.
Thirteen decoded-skin/API checks pass, covering canonical bytes, all original names,
35 unchanged parts, four refitted layers’ XY/index preservation, exact bounds,
hierarchy/inverse binds, every used control, coherent face, fixed webs/folds,
independent carry controls, actual solid roots and grip contact over all three
clips, fitted marking clearance, floor/winding/strain limits, neutral cue metadata
and malformed input rejection. Deliberately detached stalk and leaf controls
are rejected by the contact oracle. The background Blender builder reproduces
the canonical GLB byte for byte. Public selection rejects duplicate JSON/GLB
sources, and the canonical neutral STL matches the decoded anatomy.

The old/new oblique, front, rear, side and decoded attack software sheet was
inspected inline. The stern brow, upright hood, rounded bill, raised feather
fan, cross-body fingers, gripping curls, closed leaf blades, tail and complete
webs remain. Three native same-state pairs (front, clear world and battle),
along with moving world frames, were subsequently inspected inline. Everything
was headless, offscreen and muted. The close view retains the stern face and
solid grasp; the battle HUD identifies FARFETCH’D and PIDGEY, with complete webs
resting on the stage and the leek held by both curls. The moving world view
shows two walking supports and the attached stalk. Fine facial detail remains
small at the wide world distance.

The first world angle put the duck behind a foreground tree. A repeat at the
same floor-only Ecruteak position `(18, 21)`, using review orbit 2 instead of 6,
provides a clear view. Production scenery, camera code, FOV and model scale are
unchanged. Route 29 rejected early battle compositions before its existing
search found a valid stage; no admission guard was relaxed.

Nine canonical anatomy, thirteen decoded-skin/API and five scoped Rust checks
pass: 27 checks. Rust verifies shared mesh parity, fixed complete battle webs,
full bounds, two world walking supports, preserved carry bindings and GPU
activation at the pack’s original size. Shared schema and exact current-cue
checks pass across all standard variants. An independent audit confirms all
four captured state pairs (including the occluded diagnostic), the canonical
GLB and both unchanged native binary SHA256/size/mtime identities. This is
native model review, not browser or complete gameplay acceptance.

Export into ignored review space without opening windows:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 tools/leek_duck_glb.py \
  --output target/pokemon-native-review/leek-duck-review
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s tools \
  -p test_leek_duck_glb.py -v
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s tools \
  -p test_leek_duck_sculpt.py -v
```

`LEEK_DUCK_SKIN_MODELS` and `LEEK_DUCK_TEST_MODELS` select an explicit ignored
export directory. `tools/leek_duck_sculpt.py --rig-ready --output ...` can emit
a disposable neutral JSON for authoring comparison; the game needs no sidecar.

Pinned source / packed / GLB SHA256 values:

- source_sha256: `01413a43190c406119b52c86f09f56ea2bfcb32f2cba08e110aac3cd489523a1`
- packed_sha256: `37528f1600a372ff12c162bd213c8e2fe9099e224aa8028582fa00a94fa76a3c`
- glb_sha256: `f51ed4855cd845953496ad9182170eadc714550379a4b10c635f90bf4a7bf271`
