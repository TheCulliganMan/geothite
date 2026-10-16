# Marill and Azumarill welded-belly skins

`tools/aqua_rabbit_sculpt.py` retains the original water-rabbit anatomy.
`tools/water_rabbit_glb.py` exports canonical standard skins for the common
production reader and GPU path. Static JSON is retired, and the battle builder
now reaches the canonical exporter directly.

Every original position, normal and triangle is preserved. Marill retains its
round body and ears; Azumarill retains its longer body, asymmetric bent/upright
ears and six fitted belly spots. Both have flattened arm paddles, planted feet,
a small fitted smile and gaze, and a continuous zigzag wire ending at a blue
buoy. The lower belly is part of the same anatomical body, with forty exact
shared seam vertices across its material boundary.

Both skins use twelve joints: identity floor root, torso, upper body/head,
separate ear bases/tips, arm paddles and a three-stage tail. The two body colors
and adult belly spots share one spatial torso/head field. Fitted facial details
share the head transform. Ear silhouettes and red linings share the same field
through their bases and tips; the bent adult tip retains its original fold.
Feet remain fixed while arms and tail stages articulate. The buoy and wire end
share the terminal tail joint. Idle, attack and hit use rotation-only clips,
neutral endpoints and the existing production source-cue timing boundary.

Normal colors come from the local pack. Marill uses RGB555 blue (9,13,29), warm
lining (25,8,5), white and black. Azumarill uses blue (10,15,24), warm lining
(22,7,4), white and black. Tongues share the warm lining color. Shiny checks and
captures remain skipped at the user's request.

| Asset | Parts | Vertices | Triangles | Accessors | Bytes | Pack height |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| Marill | 19 | 3,437 | 5,648 | 115 | 191,256 | 104 = 1 foot 4 inches = 0.4064 m |
| Azumarill | 25 | 4,873 | 7,448 | 140 | 250,292 | 207 = 2 feet 7 inches = 0.7874 m |

Both retain normalized height 1.02. Neutral digests:

- Marill: `39c4a5f6e20db9f7c4aeddef3353131a3dda11ddd1befd021dfcb5aa1e88fbe7`.
- Azumarill: `19aba9a5fabbfd5f187d24b28f9c10fc291c0312046b6142dfa73d3e11219333`.

All six anatomy and eight decoded-skin checks pass. They cover joined closed
body topology, original normals and regeneration, actual support contacts,
fitted features, the body seam through all clips, canonical bind/budgets,
source normal colors, fixed feet, articulation, winding/edge strain, posed
belly/ear fitting and actual cord-to-buoy attachment. Final Blender/native
exports are byte-identical. Front/rear software views were inspected.
Both focused production Rust checks pass, including pack sizes, coherent
feature owners, fixed feet, welded belly seams and animated bounds. Matched
normal native reviews pass. Exact
sprite-frame motion, field gaits and browser visual review remain open.

```sh
python3 tools/water_rabbit_glb.py --species marill --output crates/crystal-voxel-view/models/battle_species/marill.glb
python3 tools/water_rabbit_glb.py --species azumarill --output crates/crystal-voxel-view/models/battle_species/azumarill.glb
python3 -m unittest discover -s tools -p test_aqua_rabbit_sculpt.py
python3 -m unittest discover -s tools -p test_water_rabbit_glb.py
cargo test --locked -p crystal-voxel-view water_rabbit_family_ --lib
blender -b -t 2 --python tools/build-battle-species.py -- target/water-rabbit-review --only MARILL,AZUMARILL
```

Builder exports, source-image helpers and capture intermediates remain ignored
verification scratch, with canonical GLBs as the model deliverables.


Both matched normal native 3D/2D reviews pass. Marill against Rattata shows
planted feet, its round ears, clean belly seam and attached zigzag buoy tail;
the paired receipt preserves core frame 1933, hash 2779120648, LCD frame 2547
and time 45.426498708 (`state_preserved=true`). Azumarill against Sentret shows
the bent/upright ears, fitted spots, wavy belly boundary, planted paddles and
attached buoy at the verified pack size; its receipt preserves core frame
2080, hash 2842356162, LCD frame 2671 and time 47.184705956
(`state_preserved=true`). Rear details were also inspected in software views.
The native build and wider catalog grounding/actual elapsed-pose regressions
pass. Shiny checks remain skipped; exact motion, field gaits, browser review
and the all-species objective remain open.
