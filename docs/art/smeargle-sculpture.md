# Smeargle painter-dog sculpture and skin

`tools/smeargle_sculpt.py` authors the anatomy and `tools/smeargle_glb.py` exports
canonical `smeargle.glb`. The duplicate static JSON is removed. The battle
builder dispatches this standard skin before generic Blender batching.

The torso, brown collar, canine head and muzzle, fitted eyes and nostrils,
floppy ears, slanted cream beret, tongue, three-toed paws, ankle markings,
fingers, looping tail and scalloped green brush retain their original geometry.
Those 39 non-arm parts preserve every original neutral f32 position, normal,
color and triangle. The 26 colored arm bands now use the current shared
continuous two-span arm builder, retaining shared ring frames and normals.
This adds 382 vertices and 728 triangles rather than retaining the older
single-span band implementation.

Thirteen joints provide a supported torso and neck, coherent fitted face,
separate floppy ears and beret, articulated forelimbs, and a painting grip.
Paws, toe grooves and brown ankle cuffs stay on the identity floor root. The
right palm, its three fingers, the held distal tail and both brush layers share
one owner. The continuous tail blends back toward the torso; its root stays
attached while the grip and paint tip move through a forward painting stroke.
The left open hand gestures independently. Standard rotation-only idle and
source-relative attack/hit clips return to literal neutral endpoints.

The four material groups for the two banded arms follow the existing packed
model pattern. All 65 source parts round-trip in their original order through
`source_part_ranges`; the runtime uses 43 primitives. No geometry is dropped.
The skin has 16,875 vertices, 21,004 triangles and 13 joints, using 669,624 bytes
and 204 accessors. Existing loader budgets remain unchanged. Authoring height
is 1.14; verified pack height is 1.1938 m.

- Source neutral f32 digest: `41339016b1387248e3407fbba1d1412e7fee742f47744f29bf3f6dff2211a753`
- Packed runtime digest: `e87a6aa122e9560a34741eb17c164e9ae34f2da00990129878ef18676460b595`

The original sculpture pass used [official artwork](https://assets.pokemon.com/assets/cms2/img/pokedex/full/235.png)
as a temporary reference. No external mesh, texture, rig or extracted game
content is shipped. Ordered fitted-patch centroids preserve deterministic
geometry across Python and Blender; the full GLB regenerates byte-identically.

```sh
python3 -m unittest discover -s tools -p 'test_smeargle*.py'
```

Eight anatomy checks cover topology, normals, source regeneration, height,
colors, physical appendage support and fitted details. Eight skin checks decode
the actual keys and weights to verify packed source ranges, fixed paws, coherent
owners, forward paint-tip motion, actual posed hand/tail/brush contact, neutral
endpoints, winding and strain. All 14 points of each adjacent arm-band ring and
all 36 points of the scalloped paint boundary remain exactly shared during
idle, painting and recoil. Shiny checks are intentionally skipped.

All 16 Python anatomy/skin checks, two focused Rust rig/GPU regressions and two wider catalog/timing regressions passed. Blender regeneration is byte-identical to the canonical GLB. The production native client built successfully. Front/rear and fixed-camera neutral, idle, painting and recovery views were inspected, followed by a paired normal 3D/2D battle against Hoppip with preserved game state and accepted, visibly rendered terrain. Shiny checks were skipped as requested.

Final normal pair: core 1260/hash 1815607576, LCD 1868, virtual time 32.666663291; `state_preserved=true`.

Final ignored QA prefix: `target/pokemon-native-review/smeargle-normal`. Temporary
helpers, software previews, movement frames and builder parity outputs were
removed after inspection.
