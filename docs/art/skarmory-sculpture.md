# Skarmory sculpture and skin

`tools/skarmory_sculpt.py` authors the anatomy; `tools/skarmory_glb.py` exports
canonical `skarmory.glb`. The duplicate static JSON is removed. The battle
builder dispatches this standard skin before generic Blender batching. Every
original neutral f32 position, normal, color and triangle is retained at the
1.21 authoring height. Raw recipe geometry is cached for normalization and pivots.

The breast, shoulder and rising neck retain their continuous steel shell. The
angular skull and sharp upper bill, lower mandible, fitted yellow eyes, four
neck armor seams, upright crest, four nape plumes, swept wing leading edges,
eight red wing blades, silver haunches, bent hocks, eight hooked talons and long
hooked tail blade are unchanged. The existing diffuse mesh-color path retains
the sharp steel planes and red undersides.

Ten joints articulate the torso, neck, head, both wings, tail, supported leg
middles and lower mandible. All four talons on each foot stay on the identity
floor root. A wing's steel leading edge and four red blades share one rigid
owner, preserving their plate shapes and overlap through the stroke. The head
carries its fitted eyes, crest and nape armor. Neck seams follow the rigid
section of their actual support. The lower mandible and mouth seam share their
hinge; the bill opens during attack and recoil without closing through its
neutral upper edge. Standard rotation-only idle/source-cue timing and literal
neutral endpoints are retained.

The skin has 42 parts, 3,635 vertices, 4,288 triangles and ten joints, using
181,568 bytes and 167 accessors. Existing loader budgets are unchanged.
Verified pack height is 1.7018 m. Neutral f32 digest:
`4a314b8b28c7023ad5044af9003e534b194def610a87be4865b89bde70c1e997`.

Ordered patch and eye-inset centroids preserve the original Python 3.9 geometry
under Blender's newer Python. Both authoring paths regenerate the full GLB
byte-identically.

The original sculpture pass used [official artwork](https://assets.pokemon.com/assets/cms2/img/pokedex/full/227.png)
as a temporary reference. No external mesh, texture, rig or extracted game
content is shipped.

```sh
python3 -m unittest discover -s tools -p 'test_skarmory*.py'
```

Seven anatomy checks retain topology, shading, bounds, blade counts, actual
armor and limb contacts and fitted-surface checks. Eight skin checks decode the
actual keys and weights to verify fixed talons, coherent armor, neutral
endpoints, visible articulation, two-wing attack strokes, an opening bill,
physical posed joins, fitted features, winding and strain. Shiny checks are
intentionally skipped.

All 15 Python anatomy/skin checks, two focused Rust rig/GPU regressions and two wider catalog/timing regressions passed. Blender regeneration is byte-identical to the canonical GLB. The production native client built successfully. Front/rear and fixed-camera neutral, idle, wing-stroke and recovery views were inspected, followed by a paired normal 3D/2D battle against Hoppip with preserved game state and accepted, visibly rendered terrain. Shiny checks were skipped as requested.

Final normal pair: core 1332/hash 453190722, LCD 1965, virtual time 35.581795208; `state_preserved=true`.

Final ignored QA prefix: `target/pokemon-native-review/skarmory-normal`. Temporary
helpers, software previews, movement frames and builder parity outputs were
removed after inspection.
