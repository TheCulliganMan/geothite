# Sneasel sculpture and skin

`tools/sneasel_sculpt.py` authors the anatomy and `tools/sneasel_glb.py` exports
the canonical `sneasel.glb` standard skin. The duplicate static JSON is removed.
The battle builder dispatches this rig before generic Blender batching. Every
original neutral f32 position, normal, color and triangle is preserved at the
1.11 authoring height. Raw recipe geometry is cached for normalization and pivots.

The sculpture retains its slim teal torso, tapered muzzle and feline crown,
angular fitted eyes, forehead and chest gems, mischievous grin, tongue and small
fangs. The asymmetric pointed ear and rose plume, three rose tail feathers,
reaching forelimbs, broad palms, hooked hand talons, outward hocks, grounded feet
and curved toe claws are unchanged.

Thirteen joints provide a supported torso and neck, coherent fitted face,
separate ear and plume, one coherent tail fan, articulated forelimbs and wrists,
and supported hock middles. Feet and toe talons remain on the identity floor
root. Each palm and both hooked claws share its wrist owner. The attack advances
the raised left palm and both talons through the same forward strike; idle uses
a smaller guard motion. The fitted face and forehead gem follow the head; the
chest gem follows the torso. The tail fan uses yaw to preserve its floor height.
Idle and source-relative attack/hit clips use the existing timing, rotation-only
keys and literal neutral endpoints.

The skin has 38 parts, 8,779 vertices, 9,936 triangles and 13 joints, using
377,268 bytes and 176 accessors. Existing loader budgets are unchanged. The
verified pack height is 0.889 m. Neutral f32 digest:
`0509cb264ed9206131887afeee6bb84b852aecfbf0d0743e4c46f08652d0f19a`.

Ordered patch and eye-inset centroids preserve the original Python 3.9 sculpture
under Blender's newer Python. The complete GLB regenerates byte-identically in
both environments.

The original sculpture pass used [official artwork](https://assets.pokemon.com/assets/cms2/img/pokedex/full/215.png)
as a temporary reference. No external mesh, texture, rig or extracted game
content is shipped.

```sh
python3 -m unittest discover -s tools -p 'test_sneasel*.py'
```

Seven anatomy checks retain the closed topology, shading, bounds, anatomy
counts, physical root contacts and fitted-surface checks. Eight skin checks
decode the actual keys and weights to verify ownership, fixed feet, neutral
endpoints, visible articulation, forward attack motion, physical posed joins,
fitted features, winding and strain. Shiny checks are intentionally skipped.

All 15 Python anatomy/skin checks, two focused Rust rig/GPU regressions and two wider catalog/timing regressions passed. Blender regeneration is byte-identical to the canonical GLB. The production native client built successfully. Front/rear and fixed-camera neutral, idle, strike and recovery views were inspected, followed by a paired normal 3D/2D battle against Pidgey with preserved game state and accepted, visibly rendered terrain. Shiny checks were skipped as requested.

Final normal pair: core 1202/hash 3785036497, LCD 1780, virtual time 30.835087251; `state_preserved=true`.

Final ignored QA prefix: `target/pokemon-native-review/sneasel-normal`. Temporary
helpers, software previews, movement frames and builder parity outputs were
removed after inspection.
