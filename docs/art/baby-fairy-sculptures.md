# Pichu, Cleffa and Igglybuff

The canonical assets are the three standard skinned GLBs in
`crates/crystal-voxel-view/models/battle_species/`. `tools/baby_glb.py` exports
and strictly reads them. Pichu is an original sculpture in
`tools/pichu_sculpt.py`; Cleffa and Igglybuff retain every neutral position,
normal and triangle from `tools/baby_fairy_sculpt.py`. The old static JSON
copies are removed. No downloaded model is used.

Pichu replaces the generic mouse prefab with broad black ears and fitted yellow
insets, an oversized head, electric cheek pouches, a continuous jagged black
collar, short arms, supporting legs, planted toes and a rooted black lightning
tail. Cleffa keeps its broad brown star ears, rolled forehead tuft and fitted
curl, inward hands, tiny face and rear puff. Igglybuff keeps its balloon body,
three joined crown lobes, fitted forehead spiral, red irises and petal feet.
Normal colors come from the local pack's RGB555 palettes. All three source
Pokédex heights are `100` (0.3048 m). Authoring heights remain 0.62, 0.60 and
0.58 respectively; production scales them from the pack.

The fixed root owns the feet. A shared spatial field joins torso and head;
faces follow the head, and appendages have their own joints. Pichu's supporting
legs blend into the body above the planted toes. Cleffa's tuft and fitted curl
share one joint. Igglybuff's three crown lobes share one joint. Idle, attack and
hit clips rotate these joints with neutral endpoints, and production battle
cues control attack/hit timing.

| Species | Parts | Vertices | Triangles | Joints | GLB bytes |
| --- | ---: | ---: | ---: | ---: | ---: |
| Pichu | 24 | 9,462 | 9,104 | 10 | 381,376 |
| Cleffa | 17 | 14,716 | 24,376 | 9 | 587,892 |
| Igglybuff | 19 | 16,964 | 28,684 | 6 | 630,808 |

Regenerate through the existing builder:

```sh
blender -b -t 2 --python tools/build-battle-species.py -- /tmp/geothite-babies --only PICHU,CLEFFA,IGGLYBUFF
python3 -m unittest discover -s tools -p test_baby_glb.py
python3 -m unittest discover -s tools -p test_baby_fairy_sculpt.py
```

The skin tests cover canonical bytes, neutral bind matrices, budgets, original
Pichu geometry and fitting, ownership, planted feet, neutral clip endpoints,
visible articulation, triangle winding, edge strain and actual posed volume
contacts. The nine fairy sculpture tests retain their original face clearance,
closed geometry, physical joins and spiral support checks. Rust regressions
cover registry/catalog identity, immutable feet, fitted faces, floor and animated
bounds, and GPU selection at the verified pack heights. Blender regenerated all
three assets byte for byte. The native reviewer builds successfully; the two
wider regressions for catalog grounding and battle cue timing pass.

All three matched normal native 3D/2D reviews pass. Pichu's broad tipped ears,
fitted gaze/cheeks, jagged collar, planted toes and lightning tail read clearly
against Pidgey. Cleffa's star ears, rolled tuft, tiny face and planted feet read
clearly against Sentret. Igglybuff's crown, forehead spiral, red eyes and grounded
petal feet read clearly against Pidgey. Their small production scales follow the
pack. Each paired receipt preserves state, and encounter terrain is accepted.
Pichu preserves core frame 1654, hash 892235761, LCD frame 2259 and time
39.108565458. Cleffa preserves core frame 1639, hash 2539579671, LCD frame 2253
and time 39.007266249. Igglybuff preserves core frame 1921, hash 3830763378,
LCD frame 2624 and time 46.141430541. Profile views conceal some rear detail;
the inspected front/rear software views cover it. Shiny checks remain skipped.

These model checks do not establish TUI or MCP gameplay.
