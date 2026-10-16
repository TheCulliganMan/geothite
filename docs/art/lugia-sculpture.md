# Lugia hand wings, open roar and planted pads

`tools/lugia_sculpt.py` authors the original sea guardian.
`tools/lugia_glb.py` exports the canonical skin; the battle-species builder
regenerates the same GLB. No third-party geometry is included.

A smooth ivory pear-shaped body carries a fitted pale-blue breast shield and
long forward-curving neck. The streamlined head has navy eye shields, narrow
lenses, small pupils, rising eye flags and a swept occipital crest. The hooked
upper beak and open lower mandible surround a recessed dark oral back and
small pink tongue. Both broad bent wing arms end in rounded palms with five
blunt digits. Ten navy dorsal plates follow the back. A swept ivory tail ends
in two navy lances. Powerful haunches and ankles lead to broad white pads and
three blunt toes per foot.

The source has 56 recoverable parts packed into 34 runtime primitives,
19,823 vertices, 34,796 triangles and 13 joints. The GLB occupies 852,748 bytes
and 182 accessors, within the unchanged renderer limits. The former static
source had 34,165 vertices and 61,688 triangles. Paddle and ellipsoid sampling
were reduced selectively; the highest fingers retain their original sampling.
The 25 retained core meshes preserve topology, normals and colors. Their
normalized coordinates differ from the former source by at most 0.000000375
units because resampling the rounded pads changes the tiny raw floor datum.
The body, neck, head, jaws, crest, eye flags, dorsal plates, ankles, longest
fingers, tail and fins retain their original surfaces.

The breast shield keeps its original thickness. Conforming local cap
refinement resolves the stored body's curved shoulder where uniform coarse
triangles cut into it. Midpoint splits propagate to adjacent triangles,
keeping the surface closed. All fitted markings retain actual stored-surface
support and triangle-center/edge clearance.

The canonical source geometry SHA-256 is
`00a71d50c07f0307e3638ddd0bfe10bbc8b839f24c4061020d41f4029cc7afa9`;
the packed runtime pin is
`8bfe309e8e03304e4547f34346df87add36f0706bed92b5cc7b589b57cf4001b`.
Strict decoding recovers all 56 source parts in order. Same-carrier fingers,
toes and dorsal plates pack together without changing their geometry.

The identity root fixes both pads and all six toes. Blended haunches and
ankles bridge to a yawing torso. The belly and dorsal plates follow the body;
neck, head and complete gaze articulate independently. Wing arms blend toward
separate palm joints; each palm carries its complete five-digit fan. The
lower mandible and tongue share an opening joint. Two yawing tail joints
carry the paired terminal fins and preserve vertical clearance. Rotation-only
idle, attack and recoil clips return exactly to neutral.

Authoring height stays 1.45 with the floor at zero. The verified local pack
supplies height digits 1701, or 5.207 metres.

Thirteen anatomy and nine decoded-skin checks passed for closed solids,
shading, fitted surfaces, occupied roots, coherent owners, fixed pads and toes,
physical edge strain, outward winding, independent motion and a real mouth
void through the posed roar. The mouth-volume oracle now merges duplicate
crossings on shared triangle edges before pairing occupied intervals. Its
filled-mouth negative control and the stronger posed-void check were rerun
successfully on decoded float32 geometry. Blender regeneration is
byte-identical. Fixed-camera front and rear neutral, idle, attack and recoil
software views were inspected. Shiny checks are skipped at the user's request.

Three focused Rust checks passed for dedicated-model routing, GPU activation
at verified pack height, fixed pads and toes, coherent carriers, tail
clearance, independent palms/mandible/fins and animation culling bounds.

The wider exact-species grounding/volume and elapsed-animation timing
regressions passed. The production native client built successfully with:

```sh
CARGO_TARGET_DIR=target/pokemon-models cargo build --locked -p crystal-bevy \
  --features location-tester,bevy/zstd_rust --example immersive_battle_3d -j1 \
  --config profile.dev.package.crystal-voxel-view.opt-level=0 \
  --config profile.dev.package.crystal-bevy.opt-level=0
```

The initial Route 29 native review rejected staging because the complete
animated body did not fit supported ground. Scale and full-body terrain
admission remain intact. The alternate-map walking preview now requires
its outdoor starting tile, as well as the destination, to be actual source
grass. The Route 35 regression passed: both approach tiles are source grass,
and setup leaves battle absent until ordinary gameplay triggers it.

Native acceptance remains open. Route 35 and Route 38 also rejected the
complete animated footprint on the source ground (zero supported staging
candidates). A first Route 38 movement-only run stopped at battle intro text;
the subsequent ordinary A-input run reached Lugia's battle and confirmed the
terrain rejection. No size clamp, source-floor change, footprint relaxation
or grass-height change was introduced. The latest native build and the
Route 35 fixture regression passed. No paired native acceptance is claimed.
