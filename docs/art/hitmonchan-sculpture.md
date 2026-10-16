# Original Hitmonchan sculpture and boxing skin

`tools/hitmonchan_sculpt.py` authors the original boxer;
`tools/hitmonchan_glb.py` exports and strictly reads the canonical standard GLB in
`crates/crystal-voxel-view/models/battle_species/hitmonchan.glb`. The duplicate
static JSON is retired. No external model, rig or animation is imported.

A continuous five-point crown carries fitted angular eyes and a firm jaw crease.
Pointed shoulder guards meet a sleeveless tunic, black belt and eight-pleat skirt.
Long legs support grounded boots and ankle collars. One bent arm reaches forward
while the other guards the face. Each padded glove has a rooted thumb, cuff and
fitted lower contour. The local pack normal palette supplies tan (173,123,99),
pink (222,24,148), white and black, including pink gloves and kit. Pack height
`407` sets a 1.397 m production scale; authoring height remains 1.30.

Twenty parts preserve exact previous neutral f32 geometry: tunic, neck, skirt,
belt, legs, boots, ankle collars, shoulders, arms, glove cuffs, mitts and thumbs.
The crown and selected fitted markings reduce sampling; facial layers regenerate
on the crown. The shoe/glove contours clear their supporting surfaces. The asset
has 36 parts, 18,362 vertices, 30,096 triangles, 12 joints, 177 accessors and
716,544 bytes. Neutral digest:
`59c5d015861346b850e7aca45fa3056246a8d8bc950ce5d9793a0035d7856daf`.

The root fixes boots, ankle cuffs and fitted piping. Body/neck weights retain
contact; crown and fitted gaze share the head owner. Shoulder guards, upper arms
and forearms articulate separately. Each glove, cuff, thumb and seam shares its
wrist owner. Both mitts advance during the attack; idle uses a smaller guard
motion. Supporting leg middles move above fixed ankles. Idle, attack and hit
clips rotate joints with neutral endpoints; production battle cues control
attack/hit timing. Exact move-specific choreography remains open.

```sh
blender -b -t 2 --python tools/build-battle-species.py -- /tmp/geothite-hitmonchan --only HITMONCHAN
python3 -m unittest discover -s tools -p test_hitmonchan_glb.py
python3 -m unittest discover -s tools -p test_hitmonchan_sculpt.py
```

All eleven sculpture and eight skin checks pass. They cover closed outward
geometry, normals and crown-cap continuity, parity, anatomy, actual neutral/posed
attachment volumes, face/contour fitting and clearance, fixed boots/cuffs,
neutral endpoints, winding, strain, forward glove movement and detached negative
controls. Blender regeneration is byte-identical; front/rear and attack/hit
software views were inspected. Both focused Rust integration tests, the native build and both wider catalog
grounding / elapsed battle pose
regressions pass. The normal paired review passes. Shiny checks are
skipped at the user's request. These model checks do not establish TUI or MCP
gameplay.

The matched normal native 3D/2D review against Sentret passes. Hitmonchan's
crown, asymmetrical guard, pink mitts and kit, pleated hem and planted boots
remain readable at the verified scale. The held glove obscures part of the gaze;
the inspected attack/hit phase views cover the changing arm positions. Terrain
is accepted; the receipt preserves core frame 2807, hash 2119763869, LCD frame
3550, time 64.156540085 and `state_preserved=true`. Shiny checks remain skipped.
