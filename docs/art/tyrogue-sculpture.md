# Original Tyrogue sculpture and skin

`tools/tyrogue_sculpt.py` authors the original small fighter;
`tools/tyrogue_glb.py` exports and strictly reads its canonical standard GLB in
`crates/crystal-voxel-view/models/battle_species/tyrogue.glb`. The duplicate
static JSON is retired. No external model, rig or animation is imported.

A continuous three-pronged crown carries fitted angular eyes, dark pupils and
a determined mouth. Side ears have fitted contours. The compact torso meets a
short neck, white waist band and joined brown shorts. Bent arms end in white
wrist wraps, open palms and three curved fingers. Slim legs support grounded
brown shoes with fitted toe contours. The local pack normal palette supplies
pink (189,123,148), brown (140,57,24), white and black. Pack height `204` sets a
0.7112 m production scale; authoring height remains 0.86.

Nineteen parts preserve exact previous neutral f32 geometry: body, belt, shorts,
shorts legs, arms, wrist wraps, all fingers, legs and shoes. Crown, iris/glint,
ear contour and shoe contour sampling is reduced; neck, ears and palms use fewer
radial samples. Facial layers regenerate against the actual crown triangles.
The toe contours sit far enough forward to clear the shoe at triangle centers
and edge midpoints. The resulting asset has 40 parts, 19,924 vertices, 33,424
triangles, 195 accessors and 764,016 bytes. Neutral digest:
`6f08d0c81d0c633e32fd55a565109a56759eeb7250212e0e4dfef93812dc71a2`.

Twelve joints control torso, neck, head, ears, two stages of each bent arm and
leg middles. The root fixes shoes and their toe contours. Body/neck weights
retain contact; crown and fitted gaze share the head owner. Each wrist wrap,
palm and three fingers share the corresponding forearm owner. The shorts and
belt stay with the torso. Idle, attack and hit clips rotate joints with neutral
endpoints; production battle cues control attack/hit timing.

```sh
blender -b -t 2 --python tools/build-battle-species.py -- /tmp/geothite-tyrogue --only TYROGUE
python3 -m unittest discover -s tools -p test_tyrogue_glb.py
python3 -m unittest discover -s tools -p test_tyrogue_sculpt.py
```

All ten final sculpture checks and seven skin checks pass. They cover closed
outward geometry, normals, canonical bytes, budgets, parity, anatomy, actual
neutral/posed root volumes, fitted vertices/face centers/edge clearance, fixed
shoes, neutral endpoints, winding, strain and detached negative controls.
Blender regenerates byte-identical data; final front/rear software views were
inspected. Both focused Rust checks, the native build and normal paired review
pass. Shiny checks are skipped at the user's request. These checks do not establish TUI
or MCP gameplay.

The matched normal native 3D/2D review against Sentret passes. Tyrogue's
continuous crown, fitted gaze, wrist wraps and three-finger hands, joined shorts
and planted shoes remain readable at the verified scale. Terrain is accepted;
the paired receipt preserves core frame 2543, hash 3028213223, LCD frame 3253,
time 59.500983002 and `state_preserved=true`. Front/rear software views cover
surfaces concealed by the production profile. Shiny checks remain skipped.

The wider catalog grounding and actual elapsed battle pose regressions pass.
