# Seel and Dewgong surface strokes

Seel's two brows, four flipper folds and tongue crease, plus Dewgong's four
flipper folds, previously interpolated depths from sparse surface samples.
Sections of these narrow tubes floated above their supporting triangles. The
earlier loose proximity checks accepted those gaps.

`surface_stroke` in `tools/surface_strokes.py` preserves the smoothed XY centerline,
sample count and round radii, then fits each sample to the actual stored parent
surface. Its center sits 35% of the radius above that surface, leaving a visible
rounded front and an occupied root. Both seals share the helper. Tube frames and
normals follow the fitted depths.

Only the eleven strokes' positions and normals change. Other geometry,
complete bounds, materials, skin bindings, hierarchy, inverse binds and decoded
animation keys remain identical. Seel retains 13,304 vertices, 21,680 triangles,
27 parts and ten joints; Dewgong retains 15,206 vertices, 23,740 triangles,
22 parts and nine joints. Pack measurements still determine game size.

Seel's pillow helper now supplies an explicit `fsum` centroid to the existing
rounded perimeter helper. This preserves four reviewed positive-zero normals
under Python 3.9 as well as newer compensated summation. It changes no candidate
bytes. Other users of `pillow_ear` retain its previous default centroid behavior.

The decoded-skin regression samples fifteen actual occupied cross-sections of
each stroke, requiring both parent overlap and an exposed front through idle,
attack and hit at three times each. Both previous canonical assets fail the new
test. Deliberately floating and buried copies are also rejected. Existing closed
topology, shading, physical joins, support, winding, strain, culling and motion
checks remain in force. Candidate anatomy and skin verification passed 29 tests.
The installed models also pass all 29 checks after extraction of the shared
stroke helper.

```sh
python3 tools/seal_glb.py --species seel --output target/seal-review/seel.glb
python3 tools/seal_glb.py --species dewgong --output target/seal-review/dewgong.glb
PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=tools \
  python3 -m unittest -v test_seel_sculpt test_dewgong_sculpt test_seal_glb
```

Before/after front, oblique and rear software views were inspected for both
installed candidates. These authoring views establish neither production
appearance nor completion of the whole Pokémon model goal.

## Production world review

Four muted, windowless native 2D/3D pairs were captured and individually
inspected in `PlayersHouse2F`: a close front view and an oblique view after
walking for each species. The current Seel and Dewgong compressed GLB payloads
were verified inside an immutable snapshot of the production reviewer before
capture; its executable hash remained unchanged through the batch.

| Species / view | Core frame | Checksum | LCD frame | Virtual seconds |
| --- | ---: | ---: | ---: | ---: |
| Seel front | 193 | 1632818630 | 193 | 4.087889375 |
| Dewgong front | 141 | 3613744030 | 141 | 3.348392083 |
| Seel after walking | 592 | 787167691 | 595 | 12.737680918 |
| Dewgong after walking | 1615 | 2909443749 | 1616 | 28.480917792 |

Each pair reports `state_preserved=true` and the expected authoritative party
follower identity. Both walking runs exercise 64 input/render samples. The
front views show connected brows, tongue and fan folds; oblique views retain
the curved full bodies, raised paired tail fans and low flippers at pack size.
No body rescaling or altered floor support was needed. The classic room view
does not draw the added 3D companion; it establishes the matching game state,
map position and facing, rather than a new source-sprite likeness assessment.
