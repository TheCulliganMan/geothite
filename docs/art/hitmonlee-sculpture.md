# Original Hitmonlee sculpture and kicking skin

`tools/hitmonlee_sculpt.py` authors the original headless fighter;
`tools/hitmonlee_glb.py` exports and strictly reads the canonical standard GLB in
`crates/crystal-voxel-view/models/battle_species/hitmonlee.glb`. The duplicate
static JSON is retired. No external model, rig or animation is imported.

A continuous pear body carries fitted sloping eyes, without a separate head or
mouth. Tapered hips meet continuous nine-rib shin coils and broad three-clawed
feet. Bent upper arms meet five-rib wrist coils, palms and three curled fingers.
Round sole markings follow the stored foot surfaces. Local pack normal colors
are ochre (181,115,41), gray (123,132,123), white and black. Pack height `411`
sets a 1.4986 m production scale; authoring height remains 1.24.

Twenty-seven parts preserve exact previous neutral f32 geometry, including all
four coils, every body/limb/hand/foot/toe solid and the eye sockets. Only pupils,
pupil cores and sole markings reduce sampling. Each coil retains 1,568 unique
positions and 3,132 triangles. The canonical asset has 33 parts, 19,747 vertices,
35,276 triangles, 10 joints, 151 accessors and 775,640 bytes. Neutral digest:
`ac4facbaf5bc5ea34f73342afaa5bc98e8ffa962e8704c7c7d34da37f8ffd0ac`.

The right foot, toes and sole stay fixed. During attack, separate left hip and
knee joints lift and advance the other foot, then return it to neutral. Idle
and hit keep both feet planted. The continuous leg coil blends between hip and
knee; the kicking foot, claws and sole share the knee owner. A supporting leg
field joins the fixed right foot to the torso. Upper-body gaze stays fitted.
Arm/wrist fields bend the wrist coils coherently; each palm and all its fingers
share the wrist owner. Clips rotate joints with neutral endpoints and production
source-cue timing. Exact move-specific choreography remains open.

```sh
blender -b -t 2 --python tools/build-battle-species.py -- /tmp/geothite-hitmonlee --only HITMONLEE
python3 -m unittest discover -s tools -p test_hitmonlee_glb.py
python3 -m unittest discover -s tools -p test_hitmonlee_sculpt.py
```

All nine sculpture and eight skin checks pass, covering closed outward geometry,
normals, canonical bytes, parity, anatomy, coil density, fitted layers and sole
clearance, actual neutral/posed attachment volumes, a fixed support foot, a lifted
and advancing kick, return to neutral, floor exclusion, winding, strain and
detached controls. Blender regenerates byte-identical data; front/rear and
kick/hit software views were inspected. Both focused Rust tests, the native
build and the wider catalog grounding / elapsed battle pose regressions pass.
The normal paired review passes. Shiny checks are skipped at the user's
request. These model checks do not establish TUI or MCP gameplay.

The matched normal native 3D/2D review against Pidgey passes. Hitmonlee's
headless silhouette, fitted gaze, coiled limbs, curled hands and planted feet
remain readable at the verified scale. The idle snapshot complements the
inspected kick/hit phase views. Terrain is accepted; the receipt preserves core
frame 2585, hash 1293280905, LCD frame 3305, time 58.295432041 and
`state_preserved=true`. Shiny checks remain skipped.
