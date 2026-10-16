# Porygon2 sculpture and rig

`tools/porygon2_sculpt.py` authors the rounded virtual duck;
`tools/porygon2_glb.py` exports its canonical standard glTF skin. The builder
writes the GLB before Blender batching. The retired static JSON is removed.

The continuous rose torso, slender curved neck and rounded head retain their
original geometry. The seated cyan belly follows the actual torso contour
levels on both sides. The rooted cyan leaf tail, downward-sloping spatulate
bill, both splayed paddles and fitted body/paddle reflections also retain their
exact original neutral f32 positions, normals, colors and triangles: nine parts
in total. Eye borders and whites retain their outlines with two interior
subdivisions instead of three; pupils and glints are refitted to those surfaces.
This reduces the old 23,691 vertices to 14,189 within the existing 20,000-vertex
rig limit, without relaxing any loader budget.

The model has 17 closed parts, 14,189 vertices, 23,912 triangles, six joints,
95 accessors and 609,300 bytes. Authoring height remains 1.05; the verified local
pack height is 0.6096 m. No external mesh, texture, rig or extracted game content
is shipped.

An identity floor root holds both paddles and their highlights fixed. One
spatial field supports the lower torso and its seated cyan coat, then bends
the rising neck into a rigid head. The head carries the complete fitted eyes,
pupils, glints, crown reflection and bill. Idle turns a curious gaze; attack
advances the bill with a forward nod; recoil reverses the gesture. Two tail
joints give the rooted leaf an independent delayed sway. All clips use the
standard rotation-only source-cue timing and literal neutral endpoints.

Neutral f32 digest:
`449d2c9aa5b3d054b2a96818696b9e677a72369d926bfa80e39093976bcb08ac`.
The nine unchanged non-eye parts retain digest
`4d894b9d33428b82d3b02db38aae4014af151a38302fc0e740d558250bdfcd5c`.

```sh
python3 -m unittest discover -s tools -p 'test_porygon2*.py'
```

Eight anatomy checks cover closed outward topology, shading, recipe parity,
fit and triangle clearance of eyes/reflections, actual occupied joins, both
paddles' support by the belly and front/rear blue coat coverage. Eight skin
checks decode the actual stored hierarchy, keys and weights to verify coherent
owners, fixed paddles, floor clearance, neutral endpoints, visible articulation,
winding and physical edge strain, posed joins, fitted features, forward bill
motion and independent tail sway. The canonical test pins the unchanged
non-eye geometry. Shiny checks are intentionally skipped.

All 16 Python anatomy/skin checks passed, along with the final canonical
non-eye geometry pin. Two focused Rust rig/GPU regressions and two wider
catalog/timing regressions passed. Blender regeneration is byte-identical to
the canonical GLB. The production native client built successfully.
Front/rear and fixed-camera neutral, idle, attack and recoil views were
inspected, followed by a normal paired 3D/2D battle against Pidgey. The paddles
meet visibly rendered terrain, the bill and gaze remain readable, and game
state is preserved. Shiny checks were skipped as requested.

Final normal pair: core 3596/hash 3811849301, LCD 4422, virtual time
80.779143917; `state_preserved=true`. Final QA prefix: `porygon2-normal` under
ignored `target/pokemon-native-review`.
