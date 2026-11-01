# Mr. Mime upper smile refinement

The upper mouth recess previously crossed too few head triangles. In close
front and angled views, their interpolated surface produced visible steps above
the dark lining. `refine_upper_smile` in `tools/mr_mime_sculpt.py` adds shared
ellipsoid midpoints only around that upper recess, before applying the existing
dent. Incident triangles split conformingly, and the lining is regenerated
against the actual stored face. The two cheek patches follow the refined
surface at their inner edges.

The canonical GLB has 19,916 vertices and 33,656 triangles across 67 logical
solids packed into 34 primitives. It retains all 26 joints and stays within
the existing 20,000-vertex, 40,000-triangle, and 1 MiB limits. Its reference
height is still 1.20; the pack continues to determine its physical game size.
The non-facial solids, hierarchy, inverse binds, animation keys, and non-oral
skin weights remain unchanged. No global material or lighting change is used.

The regression samples 1,495 points on the upper recess against its analytic
surface. Worst error fell from 0.006591 to 0.001782 authoring units; mean error
fell from 0.001418 to 0.000334. The stored-model test requires maximum error
below 0.002 and mean error below 0.0004, which the earlier mesh fails. Existing
checks still require closed outward surfaces, fitted layers, rooted anatomy,
an empty mouth aperture, coherent posed attachments, and supported slippers.

Rebuild and check the normal asset using the existing recipe:

```sh
python3 tools/mr_mime_glb.py --output target/pokemon-native-review/mime-mouth-candidates/mr__mime.glb
python3 -m unittest discover -s tools -p 'test_mr_mime_*.py'
```

The 24 Python anatomy and skin checks passed against the candidate before
integration. The canonical bytes are reproduced by `export_mr_mime()`; source
and packed geometry digests are pinned in the exporter and Rust registry.

Both Rust checks passed: the focused recessed-smile/slipper/digit regression
and the schema regression covering all 276 registered variants. The CLI export
also reproduced the integrated canonical GLB byte for byte.

Production review used muted headless image capture at Ecruteak `(18,21)`,
noon, with the normal Mr. Mime party preview. The front and walking world views
were inspected alongside their same-state 2D map references. An actual moving
field frame was also inspected. The upper rim is smoother, the tongue and
lining remain readable, and the slippers retain ground contact. Movement
recording contains 32 samples across the expected five positions. The current
compressed GLB was verified inside the native binary, which remained unchanged
through both pairs. This establishes this refinement's review scope, not
every-angle acceptance of Mr. Mime or completion of the whole model roster.
