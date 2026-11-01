# Donphan flexible tire and rigid tread skin

`tools/donphan_sculpt.py` authors the armored elephant. `tools/elephant_glb.py`
exports the canonical `models/battle_species/donphan.glb` through the common
production skin reader/GPU path. The parallel static JSON is retired; the
builder's former static dispatch now invokes the canonical exporter.

All 49 original parts retain exact positions, normals and triangle order:
17,395 vertices, 29,596 triangles and normalized height 1.02. A broad barrel
body and lowered cranium support a continuous tire over the back, crown and
hanging curled trunk. Nine transverse tread plates retain their actual tire
contacts. Broad swept ears have fitted lining; outward tusks, watchful eyes,
stocky legs, ankle cuffs, grounded pads, twelve fitted nails and the rear tail
retain their original support relationships.

The local normal RGB555 palette supplies gray skin (18,20,19), dark green armor
(5,11,8), white and black. Tread plate faces use the light skin tone; tire, ears,
lining, cuffs and tail use dark green. Tusks, nails and eye whites use white.
The 13-joint asset has 186 accessors and occupies 608,840 bytes. Neutral digest:
`7a95df265d655ae2c8f7f273c5e42040da683449abe331b9c667955a295f70ca`.
Pack height 307 means 3 feet 7 inches (1.0922 m).

Grounded pads, nails and ankle cuffs stay on the identity root; leg deformation
begins above the fixed cuffs. The torso and head carry their fitted features,
with separate ear and tail articulation. The continuous tire uses one spatial
blend across torso, head and two trunk stages. Tread plates stay rigid on the
corresponding body/head/trunk joints, preserving their armor shape while the
tire bends. The initial blended tread field compressed a tiny curled-tip
plate edge; rigid ownership fixes that defect without weakening strain checks.
An independent posed-volume test confirms that all nine plates still overlap
the actual tire during idle, attack and hit samples. Source-cue clips use
rotations only, neutral endpoints and a delayed trunk tip.

All nine anatomy checks and seven shared skin checks pass, including tire and
tread joins, dorsal seating, fitted face/ear/nail clearance, foot/leg/cuff
contacts, canonical bind/budgets, source normal colors, rigid features, fixed
supports, animated tread contact, winding and edge strain. Final Blender/native
bytes match; front/rear software views were inspected. Both focused production Rust checks pass, including pack sizes, fixed
supports, coherent owners and animated bounds. Matched normal native review
passes. Shiny checks are skipped. Exact
sprite-frame motion, field gaits and browser review remain open.

```sh
python3 tools/elephant_glb.py --species donphan --output crates/crystal-voxel-view/models/battle_species/donphan.glb
python3 -m unittest discover -s tools -p test_donphan_sculpt.py
python3 -m unittest discover -s tools -p test_elephant_glb.py
cargo test --locked -p crystal-voxel-view elephant_family_ --lib
blender -b -t 2 --python tools/build-battle-species.py -- target/elephant-review --only PHANPY,DONPHAN
```

Builder exports, source-image helpers and capture intermediates remain ignored
verification scratch. They are not additional deliverables.


Donphan's matched normal native 3D/2D review against Sentret passes. Grounded
pads and fixed cuffs, the hanging curled trunk, raised tire plates, broad body
and outward tusks read clearly at the verified pack scale. This side angle
obscures part of the far tusk/ear; front/rear software views cover those details.
The paired receipt preserves core frame 2052, hash 4214981733, LCD frame 2677
and time 48.122411497 (`state_preserved=true`). The native build and wider
catalog grounding/actual elapsed-pose regressions pass. Shiny checks remain
skipped; exact motion, field gaits and browser review remain open.
