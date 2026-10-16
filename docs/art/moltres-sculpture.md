# Moltres flame sculpture and rig

`tools/moltres_sculpt.py` authors the original gold bird and closed flame
banks. `tools/moltres_glb.py` exports the canonical skinned
`models/battle_species/moltres.glb`; the battle-species builder regenerates it
through the same recipe. No third-party geometry is included.

The slender breast, narrow throat, compact head, fitted eyes, pointed grey
bill, gold leading wing arms, red-orange fire banks, crown and swept tail
retain the established design. Grey feet have three forward digits, a rear
digit and ivory talon tips. Warm orange and hot gold layers fit the actual
closed flame surfaces on both sides.

The optimized asset has 62 recoverable source parts packed into 46 runtime
primitives, 19,671 vertices, 26,024 triangles and 12 joints. It occupies
800,464 bytes and 206 accessors, within the existing renderer limits. The
previous static source had 58,645 vertices and 75,968 triangles. Redundant
interior subdivisions were removed and crease normals shared; flame outlines
were retained. All 26 body, neck, head, bill, leg, foot, digit and talon parts
retain their original float32 geometry exactly. The other 36 parts retain
their projected outline boundaries within 0.00000015 authoring units.

Authoring height remains 1.28 with the floor at zero. The local pack's 6 ft
7 in dimension supplies the production height of 2.0066 metres. The source
geometry SHA-256 is
`dc2e868b13c6cc3334e76cd38c4089930ff2e2fec9c4935b394793eb6f0bdf4b`;
the packed runtime geometry pin is
`bc2c5af3c85515bb49c4773acaf16f96232104ac36d08b59681ba4eba064bc33`.
Strict decoding restores the source order and rejects noncanonical data.

The fixed root owns every foot, digit and talon. Shin weights bridge the
planted feet and yawing torso. Separate neck and head joints keep eyes and
bill coherent. Each gold wing arm carries a separate flame-bank joint;
orange cores and gold pockets follow their bank together. The crown and tail
have independent motion, with tail yaw preserving its floor clearance.
Rotation-only idle, attack and recoil clips use literal neutral endpoints.

The 21 Python anatomy and skin checks passed. They cover actual posed occupied
joins, surface support and clearance, winding, physical edge strain, floor
contact, coherent color layers, independent motion, strict round trips and
budgets. Two focused Rust checks passed for planted talons, articulation,
culling bounds and production GPU activation at pack height. Blender
regeneration is byte-identical. Fixed-camera front and rear views of neutral,
idle, attack and recoil were inspected. Shiny checks are skipped at the user's
request.

The production native client built successfully with:

```sh
CARGO_TARGET_DIR=target/pokemon-models cargo build --locked -p crystal-bevy \
  --features location-tester,bevy/zstd_rust --example immersive_battle_3d -j1 \
  --config profile.dev.package.crystal-voxel-view.opt-level=0 \
  --config profile.dev.package.crystal-bevy.opt-level=0
```

The zstd backend flag supplies the backend required by the shared image
features. Normal paired 3D/2D inspection against Sentret shows the Moltres HUD
name, fitted eyes and bill, gold wing arms, red-orange flame silhouette, crown,
swept tail and planted grey feet on visibly rendered terrain. The accepted
camera preserves game state: core frame 2629/hash 275580255, LCD animation frame
3377, virtual time 60.621271291. Final ignored QA prefix is
`target/pokemon-native-review/moltres-normal`. This accepts the current Moltres
pass; the full catalog, move-specific choreography, field gaits and browser
review remain open.
