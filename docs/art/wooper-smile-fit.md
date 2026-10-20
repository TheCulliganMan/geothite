# Wooper smile fitting

Wooper's smile tube previously sat entirely above its actual head triangles.
The `.003` authoring offset exceeded its `.0027` cross-section radius; the
closest measured stored vertex still had a positive surface gap. The juvenile
recipe now uses `.0008`, embedding the back while keeping the front visible.
The smile's XY outline, cross-section, normals, triangle order, colors and skin
weights are unchanged. Quagsire retains its existing offset and identical GLB.

An independent decoded-skin regression requires occupied body/mouth volume and
checks that all fifteen tube cross-sections straddle the actual posed skin,
including both ends. It samples idle, attack and hit at three times each. The
old GLB fails the contact condition. Seven amphibian anatomy checks and six
Wooper skin checks pass, as do the Rust Wooper family and full 276-variant
anatomy/skin/size schema checks: fifteen focused checks in total.

Only the mouth's vertex positions changed. All other stored geometry, all
normals, joint assignments, weights, inverse binds, hierarchy, materials and
decoded clip keys are identical. Complete neutral bounds and the `.70`
reference height remain unchanged. The asset still has 22 parts, nine joints,
3,991 vertices, 5,532 triangles and 218,156 bytes.

```sh
python3 tools/wooper_glb.py --output target/wooper-review/wooper.glb
python3 -m unittest discover -s tools -p 'test_axolotl_sculpt.py'
python3 -m unittest discover -s tools -p 'test_wooper_glb.py'
```

Muted headless native pairs at Ecruteak `(18,21)`, noon, were inspected beside
their same-state 2D references. The close camera `2 0 45 1` shows the fitted
smile and unchanged face, gills and planted pads. The world camera `3 2 35 0`
retains the original 0.4064-metre pack size. Its final pair and actual moving
frame were inspected; facial detail is small in that wide profile view. The
32 synthetic movement samples cover `(18,21)` through `(18,25)`. Both receipts
confirm the published Wooper follower and preserved paired state. The current
compressed GLB was verified in the unchanged capture binary. No windows were
opened or shiny variants tested. This is scoped acceptance of the smile refit,
not completion of the whole roster.
