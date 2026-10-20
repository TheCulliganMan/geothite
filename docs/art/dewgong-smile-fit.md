# Dewgong smile fitting and reproducible nose centroid

The old smile placed sparse controls above the muzzle and then interpolated
their depth. Its narrow tube did not retain substantial occupied roots across
the muzzle. The revised recipe preserves the XY centerline, radius and sample
count, fits every smoothed sample to the actual stored muzzle triangles, and
embeds the center by using a `.0006` offset with the existing `.0020` radius.
Normals follow the corrected surface. The flipper folds are unchanged.

A separate exporter difference came from float summation: Python 3.9 and 3.12
produced different signed zeros at two nose coordinates. An explicit `fsum`
centroid makes both installed versions reproduce the original canonical GLB
byte for byte before the smile change. Both also export identical bytes for
the refitted asset. The actual stored nose remains bit-identical.

Only smile positions and normals changed. All other geometry, triangle indices,
materials, skin weights, hierarchy, inverse binds and decoded animation keys
remain identical. Complete neutral bounds and the 1.03 reference height are
unchanged. Dewgong retains 22 parts, nine joints, 15,206 vertices, 23,740
triangles and 649,548 bytes, within the existing limits. Seel is byte-identical.

Ten anatomy checks and eight seal skin checks pass. The new decoded-skin check
requires occupied muzzle contact and a visible front at fifteen sections across
the smile through idle, attack and hit, at three times each. It rejects the old
asset. The Rust seal support/articulation regression and original-size Dewgong
skin/GPU-route regression also pass: twenty focused checks.

```sh
python3 tools/seal_glb.py --species dewgong --output target/dewgong-review/dewgong.glb
python3 -m unittest discover -s tools -p 'test_dewgong_sculpt.py'
python3 -m unittest discover -s tools -p 'test_seal_glb.py'
```

Two muted headless native pairs at Ecruteak `(18,21)`, noon, were inspected with
their same-state 2D references. The front camera `0 0 20 1` shows the muzzle,
fitted smile, eyes and canines; the rear flukes overlap above the head at that
angle. The world camera `3 2 35 0` and an actual moving frame show the complete
curved tail, flippers and low body support. Fine mouth detail is small in that
profile view. The unchanged local pack supplies height digits 507, or 1.7018 m.

The world trace has 32 synthetic-input samples covering `(18,21)` through
`(18,25)`. Both receipts preserve game state and confirm the published Dewgong
follower. The current compressed GLB was checked inside the capture executable;
both executable and asset stayed unchanged through review. No windows were
opened or shiny variants tested. These views accept this surface refinement,
not every-angle model quality or completion of the full roster.
