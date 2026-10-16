# Chinchou and Lanturn sculptures and rigs

`tools/angler_sculpt.py` authors both fish; `tools/angler_glb.py` exports their
canonical standard glTF skins. The builder now dispatches both GLBs before
Blender batching. Their retired static JSON files are removed. Both retain
the 0.95 authoring height and floor origin.

Chinchou retains its broad blue oval body, golden eyes with black cross pupils,
tiny mouth, pale side fins, soft pelvic paddles and two sweeping antennae.
Each antenna ends in a hanging golden teardrop with a fitted pale light center.
Every original neutral f32 position, normal, color and triangle is preserved.

Lanturn retains its deeper blue body, fitted golden eye saddle, ivory/red eyes,
open smile, tapered peduncle, blue upper tail lobe and yellow lower lobe.
The main beacon carries a large orb and a short branch carries a smaller orb.
All original triangle positions, order and colors are retained. The mask keeps
its twelve radial rings, with crease-aware smooth normals and shared vertices.
This reduces the model from 24,483 to 17,971 vertices within the existing
20,000-vertex limit. The other 28 parts retain their exact original neutral
f32 positions, normals, colors and triangles.

Ten Chinchou joints and twelve Lanturn joints articulate coherent pectoral fins,
folds, flexible two-stage stalks and rigid bulb/center pairs. Lanturn's branch
attaches to the main stalk; its tail lobes share a linked distal owner. The
body turns around its vertical axis; independently swiveling pelvic paddles
keep every vertex at its original contact height. Idle gives a soft fin and
beacon sway, attack strokes both side fins, and recoil reverses the gesture.
Standard rotation-only source-cue timing and literal neutral endpoints remain.

| Model | Parts | Vertices | Triangles | Joints | Bytes | Accessors |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Chinchou | 21 | 12,754 | 11,116 | 10 | 475,424 | 121 |
| Lanturn | 29 | 17,971 | 15,572 | 12 | 658,740 | 150 |

Existing loader budgets remain unchanged. Verified local pack heights are
0.508 m and 1.1938 m. Neutral f32 digests:

- Chinchou: `0924cdb724dca3b1b63d2893ac161eda23672ff9052be7728c7d84b216c52828`
- Lanturn: `50ecc25097080930e9c2f4d69a37ea5388ae765b0c4289cfd4b91e6395944019`

Both use native base colors. Pale light centers retain their authored shapes;
no emissive shader, external mesh, texture, rig or extracted content is shipped.

```sh
python3 -m unittest discover -s tools -p 'test_angler_*.py'
```

Eight anatomy checks cover closed outward topology, shading, regeneration,
height, actual occupied joins, fitted eyes/lights and mask clearance. All
1,472 front mask triangles are selected by geometric winding and remain above
the cheek; smooth corner normals do not misclassify vertical rim walls.
Eight skin checks decode the actual stored hierarchy, keys and weights to
verify original triangle-position/color digests, coherent owners, bind and
neutral endpoints, contact height, articulation, winding and physical edge
strain, posed joins, fitted bulb centers and fin folds, strong fin strokes
and independent lure sway. Shiny checks are intentionally skipped.

All 16 Python anatomy/skin checks passed, along with the final unchanged-part
canonical pin. Two focused Rust rig/GPU regressions and two wider catalog/timing
regressions passed. Blender regeneration is byte-identical for both canonical
GLBs. The production native client built successfully. Front/rear and
fixed-camera neutral, idle, attack and recoil views were inspected, followed
by paired normal 3D/2D battles against Pidgey. Both pelvic contact layers meet
visibly rendered terrain; eyes, fin folds, attached beacons and lures remain
readable. Game state is preserved. Shiny checks were skipped as requested.

Final normal pairs (`state_preserved=true`):

- Chinchou: core 1351/hash 3306949276, LCD 1932, virtual time 33.478359749;
  QA prefix `chinchou-normal`.
- Lanturn: core 2219/hash 3830333439, LCD 2843, virtual time 49.772844623;
  QA prefix `lanturn-normal`.

Evidence remains under ignored `target/pokemon-native-review`.
