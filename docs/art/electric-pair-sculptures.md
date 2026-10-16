# Original Electabuzz and Elekid sculptures

`tools/electric_pair_sculpt.py` authors both species. Electabuzz retains its
static JSON asset and existing recipe. Elekid's canonical asset is now
`crates/crystal-voxel-view/models/battle_species/elekid.glb`, exported and strictly
read by `tools/elekid_glb.py`. Its duplicate static JSON is removed.

Electabuzz has a broad low head, sharp fitted white eyes, black chevron, pointed
twin scalp tufts, ball-tipped antennae, separated fang-bearing jaws and a rooted
tongue. Its body carries a chest bolt, flank sweeps and a curved rear stripe
fitted to stored triangles. Bent arms end in knuckles, thumbs and wrist fur.
Striped shins support broad clawed feet; a curved striped tail has a round tip.
All 70 parts' geometry and colors regenerate exactly as before this Elekid
migration. Its 113,628 triangles, height 1.28 and floor zero are unchanged.
The [earlier source review](electric-pair-source-review.md) records references;
no external model, rig or animation is imported.

Elekid has a continuous yellow egg body, two rounded contact blades with real
through holes, fitted angular eyes, pointed grin, lower bolt and a closed black
waist wrap. Slim arm roots lead to broad striped forearms and three-clawed fists.
Short legs end in grounded feet. Normal colors come from the local pack:
(255,255,41) yellow, white and black. Pack height `200` gives a 0.6096 m
production scale; authoring height remains 0.91.

Twenty-eight Elekid parts preserve exact neutral f32 geometry. Only the four
arm bands reduce their longitudinal sampling to fit the existing skin limit.
The resulting model has 32 parts, 19,466 vertices, 33,128 triangles, 139 accessors
and 715,700 bytes. Neutral digest:
`5087eaed56db849c823c7e069ef7879375ebb39c6d3f68c84de469b858ad076a`.

Nine joints control the torso, upper body, two plug blades, arms and leg middles.
The root fixes the feet. Body, waist band and bolt use one spatial weight field;
the fitted gaze and mouth stay with the upper body. Each arm connector, forearm,
stripe, fist and claw shares one rigid owner, preserving wrap fit through motion.
Plug blades rotate from rooted pivots while retaining their open holes. Idle,
attack and hit clips rotate joints with neutral endpoints; production cues
control attack/hit timing.

```sh
blender -b -t 2 --python tools/build-battle-species.py -- /tmp/geothite-elekid --only ELEKID
python3 -m unittest discover -s tools -p test_elekid_glb.py
python3 -m unittest discover -s tools -p test_electric_pair_sculpt.py
```

The eight skin checks pass, including canonical bytes, budgets, neutral bind,
normal palette, ownership, fixed feet, neutral endpoints, winding, edge strain,
actual posed attachment volumes, and five rays through each posed plug hole
against every posed part. The pair sculpture suite passes all thirteen checks
for closed solids, normals, parity, anatomy, neutral joins, face fitting and
clearance, self crossings, open holes/jaws and filled/detached negative controls.
Blender exports byte-identical Elekid data. Front/rear software views were
inspected. Both focused Rust tests, the native build and wider catalog grounding /
elapsed battle pose regressions pass. The normal paired review passes.
Shiny checks are skipped at the user's request. These model checks do not
establish TUI or MCP gameplay.

The matched normal native 3D/2D review against Hoppip passes. Elekid's yellow
egg body, fitted gaze and bolt, open contact holes, striped forearms, clawed
fists and planted feet read clearly at the verified scale. The terrain is
accepted; the paired receipt preserves core frame 2194, hash 498538258, LCD
frame 2811, time 49.87255725 and `state_preserved=true`. Inspected front/rear
software views cover surfaces concealed by the battle profile. Shiny checks
remain skipped.
