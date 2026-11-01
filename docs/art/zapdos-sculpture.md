# Zapdos thunder fans and planted talons

`tools/zapdos_sculpt.py` authors the original angular thunder bird.
`tools/zapdos_glb.py` exports its canonical skinned GLB, and the battle-species
builder uses the same export path. No third-party geometry is included.

Two broad gold wing fans have sharp outer points and deep zigzag edges.
Each carries three black lightning underfeathers and a fitted gold shoulder
covert. A lean breast has rooted jagged side feathers; grey feather spats
connect it to narrow shins. Each foot has three forward digits, a rear digit
and ivory talon tips. The compact head carries three long crown points, two
jagged cheek rays, fitted narrow ivory eyes and a long ochre bill. The tail
has a five-point gold fan over a black underfan.

The canonical GLB has 54 parts, 18,495 vertices, 28,584 triangles, nine joints,
216 accessors and 729,848 bytes. Removing excess interior pupil subdivisions
brings the former 20,739-vertex static model within the existing skin limits.
All 52 non-pupil parts retain their original float32 positions, normals,
colors and triangle order. Their geometry SHA-256 is
`f127297ad8e2e71188ad235f4d43678895b7eeca66157130d14a1f82f6e1b00d`.
The canonical geometry pin is
`75512d822cf34c6097ea7c8421a159f7bad0291a985a4ae20ab5ae2642ba041f`.

The identity root fixes all feet, digits and talons. Blended shin and thigh
weights bridge floor contacts to the yawing torso. Neck and head motion keep
the fitted throat, eyes, cheek rays, crown and closed bill coherent. Each
complete gold-and-black wing assembly has its own rigid carrier, preventing
separation of its interlocking zigzag blades. Both tail fans share a yawing
joint so they preserve their floor clearance. Rotation-only idle, attack and
recoil clips return exactly to neutral.

Authoring height remains 1.28 with its lowest point at zero. The local verified
pack supplies a 5 ft 3 in production height of 1.6002 metres.

Eleven anatomy checks and eight decoded-skin checks passed. They cover actual
occupied anatomical joins, supported and unobscured fitted surfaces, outward
winding, physical edge strain, fixed floor contacts, coherent owners, visible
wing strokes, independent tail motion, canonical round trips and unchanged
source geometry. Blender regeneration is byte-identical. Fixed-camera front
and rear neutral, idle, attack and recoil software views were inspected.
Shiny checks are skipped at the user's request.

Two focused Rust rig/GPU checks and the wider exact-species grounding/volume
and elapsed-animation timing regressions passed. The production native client
built successfully with:

```sh
CARGO_TARGET_DIR=target/pokemon-models cargo build --locked -p crystal-bevy \
  --features location-tester,bevy/zstd_rust --example immersive_battle_3d -j1 \
  --config profile.dev.package.crystal-voxel-view.opt-level=0 \
  --config profile.dev.package.crystal-bevy.opt-level=0
```

The normal paired 3D/2D encounter against Pidgey was inspected. The HUD names
Zapdos explicitly. Its narrow bill, crown rays, black underfeathers, fitted
gold wing fans and lightning tail remain readable; grey feet and ivory talons
meet visibly rendered terrain. The camera accepted the shot and state was
preserved: core frame 3263/hash 2569537076, LCD animation frame 3994, virtual
time 73.532027458. Final ignored QA prefix is
`target/pokemon-native-review/zapdos-normal`. This accepts the current Zapdos
pass; the full model catalog and remaining animation work stay open.
