# Mareep, Flaaffy and Ampharos standard skins

`tools/sheep_sculpt.py` and `tools/ampharos_sculpt.py` author the electric line.
`tools/electric_sheep_glb.py` exports their canonical standard skins through the
common production reader and GPU path. Static JSON is retired. The builder
exports all three from the family hook instead of older static branches.

Mareep keeps its scalloped fleece, tapered face, curled forelock and four blue
supports. Flaaffy keeps its pear body, shoulder cape, paired wool curls, arm
flippers, supporting haunches and fitted toe caps. Sixteen Mareep parts and
23 Flaaffy parts retain exact original positions, normals and indices. Their
ten ear/tail bands now use the existing shared helper's refined connected
ring frames, replacing the older straight band geometry in the static assets.
Each adjacent band retains fourteen identical seam vertices through animation.
No parallel copy of the old band helper was introduced.

Ampharos keeps its continuous torso/neck/head, fitted belly, flippers, striped
horns, collar rings and beacons. Twenty-five parts preserve exact geometry.
The bottom body cage is slightly lower and wider to retain actual foot/body
volume contact during movement. Flipper weighting runs along each paddle's
length instead of across its width, avoiding shoulder-edge stretch. Independent
posed-volume tests cover both foot/body joins and neck/tail ring contacts.

Mareep and Flaaffy have thirteen joints each, with different quadruped/biped
layouts; Ampharos has nine. Ground supports stay fixed while leg middles,
neck/head, ears, arms and three-stage tails articulate. Fleece/crown curls and
their fitted grooves remain coherent. Flaaffy's cape and Ampharos's belly and
neck rings share their supporting body field. Horns and head details stay
rigid to the head. Each tail beacon and its highlight share the terminal joint.
Idle, attack and hit are rotation-only clips with neutral endpoints, delayed
beacon motion and the production source-cue timing boundary.

Normal colors come from the local pack's RGB555 palettes, plus white/black:
Mareep wool (24,20,7) and skin (10,5,19); Flaaffy skin (31,13,19) and shade
(10,3,14); Ampharos yellow (31,31,2) and lamp (21,10,1). Shiny checks and captures
remain skipped at the user's request.

| Asset | Parts | Joints | Vertices | Triangles | Accessors | Bytes | Pack height |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| Mareep | 26 | 13 | 6,382 | 11,840 | 140 | 314,456 | 200 = 0.6096 m |
| Flaaffy | 33 | 13 | 10,302 | 16,636 | 170 | 427,480 | 207 = 0.7874 m |
| Ampharos | 26 | 9 | 2,402 | 4,700 | 121 | 136,560 | 407 = 1.397 m |

Normalized heights remain 0.85, 1.0 and 1.35. Neutral digests:

- Mareep: `058c61bce99e43bc24f2bf17a0cabe8bb62a0a6475fcefcd081d34cf9f580c8d`.
- Flaaffy: `eaabe519c0cb6edfa85b9b9ddc5a9b1ca5f5a4fa3bbdedfe948dc57bb68bb8d6`.
- Ampharos: `19d3138402ccf1b0ffed193b47dea3a25ec684a5f40a1593079e3c5fac04527e`.

All six sheep anatomy, four Ampharos anatomy and eight decoded-skin checks
pass. They cover closed outward surfaces, normals, source regeneration,
actual supports, fitted curls/nails, canonical bind/budgets, normal palettes,
coherent owners, fixed supports, shared band seams, articulation, winding/strain,
posed Ampharos contacts and delayed beacons. The old serialized-JSON size check
now measures the actual canonical GLB. Final Blender/native exports are
byte-identical, and front/rear software views were inspected. The native build
passes. Both focused production Rust checks pass at the verified pack sizes,
including fixed supports, coherent features and animated bounds. Matched
normal native reviews pass. Exact
sprite-frame motion, field gaits and browser visual review remain open.

```sh
python3 tools/electric_sheep_glb.py --species mareep --output crates/crystal-voxel-view/models/battle_species/mareep.glb
python3 tools/electric_sheep_glb.py --species flaaffy --output crates/crystal-voxel-view/models/battle_species/flaaffy.glb
python3 tools/electric_sheep_glb.py --species ampharos --output crates/crystal-voxel-view/models/battle_species/ampharos.glb
python3 -m unittest discover -s tools -p test_sheep_sculpt.py
python3 -m unittest discover -s tools -p test_ampharos_sculpt.py
python3 -m unittest discover -s tools -p test_electric_sheep_glb.py
cargo test --locked -p crystal-voxel-view electric_sheep_family_ --lib
blender -b -t 2 --python tools/build-battle-species.py -- target/electric-sheep-review --only MAREEP,FLAAFFY,AMPHAROS
```

Builder exports, source-image helpers and capture intermediates remain ignored
verification scratch. Canonical GLBs and authoring code are model deliverables.


All three matched normal native 3D/2D reviews against Pidgey pass. Mareep's
planted supports, scalloped fleece, fitted curl and connected bands read clearly;
its receipt preserves core frame 1229, hash 2673380250, LCD frame 1794 and time
31.150453125. Flaaffy's planted pads, fitted toe caps, cape/crown curls and tail
remain coherent; its receipt preserves core frame 1411, hash 51933262, LCD frame
1970 and time 35.068303209. Ampharos's strengthened body/foot joins, long neck,
collar rings, flippers and banded lamp tail read clearly at the verified scale;
its receipt preserves core frame 2080, hash 1828807289, LCD frame 2798 and time
50.957366501. Every paired receipt has `state_preserved=true`. Ampharos's profile
obscures some facial/belly detail; front/rear software views cover those areas.
The native build, focused Rust checks and wider catalog grounding/actual
elapsed-pose regressions pass. Shiny checks remain skipped; exact motion,
field gaits, browser review and the full all-species objective remain open.
