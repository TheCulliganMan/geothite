# Porygon sculpture and rig

`tools/porygon_sculpt.py` authors the angular virtual duck;
`tools/porygon_glb.py` exports its canonical standard glTF skin. The builder
writes the GLB before Blender batching. The retired static JSON is removed.
Every original neutral f32 position, normal, color and triangle is preserved.

Authored polygon cages form the rose hull, short angular neck, long sloping
head and tapered cyan bill. The cyan breast seats in the forward lower hull
and rises toward the neck. Two block-shaped paddles splay outward; a narrow
rooted cyan blade forms the upright tail. Flat face normals retain crisp
panels. Hexagonal ivory eye panels with thin dark borders follow the actual
head side panels. Circular dark pupils and ivory glints fit those surfaces
in each side panel's rotated coordinate frame.

Seven joints articulate the hull, neck connector, scanning head, blade tail
and two paddles above an identity root. Every part has one rigid owner, so
all original facets stay planar. The cyan breast shares the hull owner; the
bill and every eye layer share the head owner. Idle scans gently, attack
advances the bill with a forward nod and recoil reverses that gesture. The
tail swings independently. Each paddle swivels only around its vertical
axis, retaining the exact contact height of every vertex. Actual occupied
joins remain attached through the clips. Standard rotation-only source-cue
timing and literal neutral endpoints are retained.

The model retains 16 closed parts, 4,789 vertices and 2,008 triangles.
Authoring height remains 1.05; the verified local pack height is 0.7874 m.
The GLB uses seven joints, 94 accessors and 206,136 bytes within unchanged loader budgets.
No external mesh, texture, rig or extracted game content is shipped.

Neutral f32 digest:
`d2c2a841701ee9ed8d38fdc4c27b87b5e1f5544ec665ebba0372b4f1fba4abb7`.

```sh
python3 -m unittest discover -s tools -p 'test_porygon_*.py'
```

Eight anatomy checks retain closed outward topology, shading, recipe parity,
side-eye fit and triangle clearance, physical joins, grounded paddles, actual
cyan chest coverage and flat cage normals. Eight skin checks decode stored
hierarchy, keys and weights to verify exact original geometry, bind identity,
coherent rigid owners, neutral endpoints, contact height, visible articulation,
unchanged facet edge lengths and winding, posed occupied joins, fitted moving
eyes, forward bill motion and independent tail/paddle movement. Shiny checks
are intentionally skipped.

All 16 Python anatomy/skin checks, two focused Rust rig/GPU regressions and two
wider catalog/timing regressions passed. Blender regeneration is byte-identical
to the canonical GLB. The production native client built successfully.
Front/rear and fixed-camera neutral, idle, attack and recoil views were
inspected, followed by a normal paired 3D/2D battle against Sentret. Porygon's
paddles meet visibly rendered terrain, its crisp silhouette and side eye remain
readable, and game state is preserved. Shiny checks were skipped as requested.

Final normal pair: core 3052/hash 813288004, LCD 3789, virtual time
69.23359; `state_preserved=true`. Final QA prefix: `porygon-normal` under
ignored `target/pokemon-native-review`.
