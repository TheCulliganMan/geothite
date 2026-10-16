# Original Hitmontop sculpture and headstand skin

`tools/hitmontop_sculpt.py` authors the original headstand fighter;
`tools/hitmontop_glb.py` exports and strictly reads the canonical standard GLB in
`crates/crystal-voxel-view/models/battle_species/hitmontop.glb`. The duplicate
static JSON is retired. No external model, rig or animation is imported.

The pointed helmet supports the inverted body. Long legs raise blue spiked
feet; bent arms carry wrapped wrists and round fists. A long curved tail ends
in a blue ball and white point. Fitted eyes, helmet folds, mouth, belly shield
and sole pads follow the stored surfaces. Normal colors come from the local
pack: gold (198,156,33), blue (82,82,173), white and black. Pack height `407`
sets a 1.397 m production scale; authoring height remains 1.25.

Twenty-five parts preserve exact neutral f32 geometry, including every primary
body/head/neck/spin-point, limb, wrap, foot, spike and tail solid. Ten fitted
markings reduce sampling to fit the existing skin limit. The canonical asset
has 35 parts, 17,394 vertices, 29,828 triangles, 164 accessors and 660,816 bytes.
Neutral digest:
`554e1edaafe9860ab6aa4b5c306a67549b3f8e7c2773e3adefad4901867a3de3`.

Eleven joints control the helmet, neck, torso, arms, raised legs and three-stage
tail. The root fixes the spin point on the floor. The helmet completes one forward idle turn every 3.2 seconds and retains its
fitted gaze and folds; body, belly and neck share a spatial field. Each raised
leg keeps ankle wrap, foot, three spikes and fitted pad together. Wrists and
fists share the corresponding arm. Tail ball and spike share the terminal joint.
Idle, attack and hit clips rotate joints with neutral endpoints; production
battle cues control attack/hit timing. The idle turn slows near the neutral loop
boundary without reversing; shortest-arc
quaternion interpolation preserves continuity across the signed quaternion seam.
Exact move-specific choreography remains open.

```sh
blender -b -t 2 --python tools/build-battle-species.py -- /tmp/geothite-hitmontop --only HITMONTOP
python3 -m unittest discover -s tools -p test_hitmontop_glb.py
python3 -m unittest discover -s tools -p test_hitmontop_sculpt.py
```

The unchanged neutral digest preserves the eleven passing sculpture checks.
Eight current skin checks pass, including 161 trajectory samples proving one
forward revolution, smooth quaternion wrap, a stationary spin point, raised feet,
winding, strain and actual posed contacts. Blender regeneration is byte-identical.
Four fixed-camera phase renders were inspected. Three focused Rust checks pass,
including the full-turn/wrap regression and production GPU selection. The
all-species production playback regression passes at 9, 30 and 60 Hz. The native
client builds successfully. Shiny checks are skipped at the user's request.
These checks do not establish TUI or MCP gameplay.

The updated spin asset's matched normal native 3D/2D review against Pidgey
passes with visible terrain. The planted spin point, raised blue feet and tail,
wrapped hands and rotating helmet stay coherent. The snapshot catches a side
phase; the fixed-camera phase views cover front and rear details. Terrain is
accepted; the paired receipt preserves core frame 2565, hash 2286343595, LCD
frame 3347, time 61.322099125 and `state_preserved=true`. Earlier blank-terrain
captures were rejected and are not evidence of acceptance.
