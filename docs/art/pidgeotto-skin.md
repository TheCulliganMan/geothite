# Pidgeotto full-body production skin

The canonical production asset is `models/battle_species/pidgeotto.glb`,
a standard GPU skin authored by `tools/pidgeotto_skin_glb.py` and exposed through
`tools/pidgeotto_glb.py`. Both production renderers use the shared species-rig
path. The dedicated wing-node parser and three-group renderer have been retired.
The full-roster model-improvement goal remains active.

The skin retains all 34 original anatomy names and every original
position, normal, color and triangle bit. The 32 non-wing parts are unchanged;
the two layered-wing primitives gain closed flexible shoulder covers after
the original buffers. The original source geometry is 5,941 vertices and
5,208 triangles. The skin has 6,293 vertices, 5,776 triangles, 34 parts,
14 joints, 161 accessors and 298,664 bytes, within the existing limits.
Every neutral bound and the 1.0 authoring height remain exactly unchanged.
No palette, physical-size, camera, scenery, grass, audio, pack or gameplay
changes are part of this migration. No dedicated shiny checks or captures were run.

The 14 controls are fixed floor, body, coherent head, two wing/wrist chains,
two supporting shins, tail base, three separate tail layers and swept crest.
The head, beak, eyes and two cheek-mask components share one head transform;
the lower breast remains on the body. The existing exact shoulder pivots and
all 65 quaternion keys of the 0.8-second wing loop are preserved. New wrists
remain neutral during that original loop and flex during attack/recoil.
Claws remain fixed; supporting shins blend body/knee/floor. Tail and crest
fields retain their roots while their tips articulate. Idle, attack and hit
use rotation-only, source-relative clips with neutral endpoints.

An occupied-volume check found the original wing detached from the body at
an upstroke sample. Each new shoulder cover stays seated inside the body,
blends across the hinge, and overlaps the original wing throughout checked
poses. The first narrow blend exceeded the existing 20 percent strain limit;
a wider field fixes the deformation without weakening the check or changing
the original wing motion. The first knee motion was also too small to be
meaningful, so the knee control amplitude was increased.

Eleven independent decoded-skin checks pass in 22.156 seconds. They cover
original f32 prefixes and unchanged other parts/bounds; closed outward stored
components and shading; hierarchy/inverse binds and used controls; exact
original wing pivots, key bytes and posed vertices; coherent face/cheek masks;
independent head/wrist/tail controls; grounded claws, positive winding and
bounded strain; neutral endpoints/cue metadata; actual head, wing, shoulder
cover, tail, crest, breast and foot roots over all three clips; detached
negative controls; deterministic export and malformed-byte rejection.
`original_model()` recovers and verifies the original anatomy from the
augmented skin without a geometry sidecar, allowing future canonical-only
rebuilds and migration comparisons.

Original geometry SHA256:
`68043320a306517c7d21bd8135a91bfbfec38a9426ad1beebe5df73d32f7bf56`.
Augmented geometry/packed SHA256:
`a9a41d37c7ee99cc44e3be2513069b47edd87bbb73c47abc5e5bf02890f10650`.
Canonical production GLB SHA256:
`bb26658d8996af0c6e0b63beb2cb47df650a71a1defc73852297a7d166916234`.
The retired wing-node GLB had SHA256
`c730bdd90bb284dc8e5e45106fdb65d3bb1f4e358bb85d8436d02b74791c9eea`.

Twenty-one Python and 32 distinct scoped Rust checks pass. They cover strict
schema rejection, original geometry and size, exact authored flight, live GPU
cue/rewind/frame-rate behavior, actor isolation and retirement, whole-mesh normal
palette flashes, row-cache ownership and full-envelope camera admission. The
finite camera fixture explicitly rejects its former undersized room and accepts
a room with more foreground coverage; production camera gates are unchanged.
The Blender background rebuild and both public export paths reproduce the exact
canonical GLB. Source decimals are quantized to f32 before contour refinement so
Blender and serialized inputs follow the same geometry path.

Three same-state production 2D/3D pairs (front, world and battle), plus a moving
world frame, were inspected inline. Every capture was headless, offscreen and
muted. Binary identities and canonical source hashes remained unchanged. The
battle HUD identifies PIDGEOTTO and SENTRET; the complete bird fits at the
original 1.0922-metre pack size with planted claws. World motion retains its
intentional flying clearance. Wide side views overlap feathers and show less
facial detail. These captures establish model review, not full gameplay or
browser acceptance.

Export and test without opening windows:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 tools/pidgeotto_skin_glb.py \
  --output target/pidgeotto-skin-review
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s tools \
  -p test_pidgeotto_skin_glb.py -v
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s tools \
  -p test_pidgeotto_glb.py -v
```

Tests normally build the skin in memory from canonical geometry. Set
`CRYSTAL_PIDGEOTTO_SKIN_TEST_MODELS` to an ignored directory to test an explicit
export. Native capture requires `CRYSTAL_RENDER_TEST_HEADLESS=1`,
`CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image` and `CRYSTAL_PREVIEW_MUTE=1`.
