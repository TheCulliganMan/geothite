# Pidgeotto sculpture and production skin

The canonical model is `models/battle_species/pidgeotto.glb`. Original geometry
now regenerates from `tools/pidgeotto_sculpt.py`; `tools/pidgeotto_skin_glb.py`
authors the standard GPU skin and `tools/pidgeotto_glb.py` exposes the strict
public reader. The normal background builder uses that exporter directly.
Reading the canonical GLB as its own authoring source has been removed.

The [family fidelity review](pidgey-family-fidelity.md) supersedes the earlier
skin migration's visual acceptance. Crystal's complete five-frame front
animation and back picture govern the grounded folded resting pose. A genuine
Battle Revolution orbit supplies depth and pigments. The current original body
has a deep stocky breast, a shorter forward skull with fitted small lateral
eyes, five swept red crest blades, brown folded wing cases with broad cream
profile vanes and a raised red/cream tail. Short shanks stand above separately
projecting pink digits and ivory talons. Foreign geometry is reference-only.

The model retains its 34 public anatomical names and 14 controls. Current
geometry has 9,353 vertices and 14,116 triangles, with 167 accessors and
414,368 bytes. Authoring height remains 1.0 and floor zero; the verified pack
height remains 1.0922 metres. Source and packed geometry SHA256:
`49c12b14c5250c0b106ea983d6d9239166200e27a9f444d3bf7e43cebe2abcf1`.
Canonical GLB SHA256:
`9465383e283f85b42e31d9094f70c03777c7e6703ed8dc20f74e97b05f4bc6f7`.

Controls are the fixed floor, body, coherent head, paired wing/wrist chains,
supporting shins, tail base, three tail layers and swept crest. Head, bill,
eyes and hooked masks share the head transform. Breast tissue follows its
actual body or head carrier. Talons remain fixed. Wing roots have occupied
body coverage throughout the stored clips; their rigid ownership avoids
compressing the thin fitted shoulder. Tail pivots and skin fields follow the
revised raised tail, and wrists follow the folded vanes. Final shading is
computed from the actual stored f32 triangle corners.

The 0.8-second idle uses a small folded-wing settling motion and a flat-ended
periodic wave. Its previous wing seam is checked at the unchanged absolute
10-microsecond production offsets and positional tolerance. Attack/hit retain
source-relative cues and neutral endpoints. Separate world flight still has
a genuine upstroke and downstroke; resting posture is not an airborne fan.

```sh
python3 tools/pidgeotto_skin_glb.py --output target/pidgeotto-skin-review
PYTHONPATH=tools python3 -m unittest test_pidgeotto_glb test_pidgeotto_skin_glb test_pidgey_family_fidelity -v
```

The eight public API and eleven decoded-skin checks pass for pure-source parity,
bounded strict readers, malformed inputs, STL/public routing, closed components,
normals, inverse binds, normalized weights, used anatomical controls, coherent
head, fixed talons, positive winding, unchanged strain limits, actual occupied
roots through all three clips and detached negative controls. Additional family
checks measure torso depth, folded feather plane, crest count and shank direction.
The normal Blender builder reproduces the canonical asset byte for byte.

Production captures, exact executable identity, physical size, world flight,
loop results and remaining coverage limits are recorded in the family review.
No dedicated shiny or browser acceptance follows from this native sample.
