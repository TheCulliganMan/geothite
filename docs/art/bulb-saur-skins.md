# Ivysaur and Venusaur articulated skins

`tools/bulb_saur_sculpt.py` authors the original growing and mature seed saurs.
`tools/bulb_saur_glb.py` exports their standard skins and reads their exact
canonical bytes. The production loader, build dispatcher and roster checker
now use `ivysaur.glb` and `venusaur.glb`; their generic static JSON copies are
retired. Both installed skins have completed offscreen native review. See the
[sculpture guide](bulb-saur-sculptures.md) for anatomy work.

| Species | Vertices | Triangles | Logical / packed parts | Joints | Accessors | GLB bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Ivysaur | 18,609 | 29,796 | 58 / 26 | 21 | 142 | 784,004 |
| Venusaur | 19,789 | 31,500 | 100 / 29 | 32 | 178 | 880,280 |

Both fit the existing 20,000-vertex, 40,000-triangle, 64-part/joint,
256-accessor and 1 MiB limits. Material packing retains all logical anatomy,
source order, f32 positions/normals/colors and original triangles. Authoring
height remains 1.20. The actual local pack records heightDigits 303 / 607:
0.9906 / 2.0066 metres, retained through uniform scaling.

The torso and head brace early in the source attack cue; the bud or woody
crown follows later. Four limb controls flex above all four complete planted
feet and twelve claws. Facial features and ear interiors remain coherent on
their own head. Six independent leaf base/tip chains share radial weights
with their midribs. Ivysaur has a two-stage closed bud. Venusaur adds a woody
stalk/crown, five independent petal base/tip chains, fitted cream markings
with the same petal field, and a rooted flower heart/stamen control. Idle,
attack and hit are rotation-only with neutral endpoints. Existing game
programs retain event and audio ownership.

All nine anatomy checks passed against the installed GLBs. All twelve decoded
skin checks passed, including actual posed roots, floor/winding/strain,
independent controls, coherent expressions, complete fixed supports, posed
fitted features, neutral bounds, packing, limits and strict reader rejection.
A cap diagnostic distinguished 33 deliberately embedded side walls from the
actual front cap triangles: zero real cap intersections were found. The posed
clearance test follows those original cap triangles and retains its .0001
threshold. The background Blender dispatcher reproduced both GLBs byte for
byte. The initial skin revision passed GPU, full animation-envelope and shared
276-variant regressions. The refined Venusaur passed both focused Rust checks,
and its background Blender build reproduces the installed bytes exactly. Its
shared registry schema and animation-cue checks also passed across all 276
variants. Offscreen native review passed on the refined revision.
Ivysaur remains byte-identical and has passed three
same-state native pairs plus actual walking-frame inspection. Dedicated shiny checks remain skipped.

```sh
PYTHONDONTWRITEBYTECODE=1 python3 tools/bulb_saur_glb.py \
  --output target/pokemon-native-review/bulb-saur-candidates
BULB_SAUR_SKIN_MODELS="$PWD/target/pokemon-native-review/bulb-saur-candidates" \
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s tools \
  -p test_bulb_saur_glb.py -v
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s tools \
  -p test_bulb_saur_sculpt.py -v
```

The shared registry lists 276 variants, covering all 251 species plus the
additional Unown letters. Every canonical species path resolves uniquely to a
GLB. This is asset coverage; full-roster visual quality remains an active goal.
No pack, grass/scenery, camera, gameplay or audio changes accompany these skins.

Pinned SHA256 values:

ivysaur:

- source_sha256: `4195e2b91009093f48f1ca35847d42e9058b0147cc4b13209b9da41fca21bd7f`
- packed_sha256: `05ae76219415afb128f34a59f243f170231d578d3e99a2959d55419aaff5bfda`
- glb_sha256: `444f37e6a101c1922df813babb99d99023659961b730d8066536e8a63115ecc9`

venusaur:

- source_sha256: `ba3e09a51c8df2b1d3e4a30ea99c955679b8b1d7fdb630f349bd7645b471230b`
- packed_sha256: `68b82bacf260770f0ef9db47eb72c1547c3d962acf4dcd15d365cd7a87185673`
- glb_sha256: `2503d38edead0dfac1267a854c4bfe92b85e453cafe662baf1aa1cb862ff4658`

A direct comparison with the user-supplied pack's normal front sprites exposed
Venusaur's overly narrow lower body and first-stage angular forehead markings.
The refined lower body is 15% broader, with inverse-transpose normal correction
and corresponding hip pivots. Three rounded head bumps and eight rooted flank
bumps replace the angular forehead pattern. The flower, six full neutral bounds,
authoring height and pack scale remain unchanged. Slightly coarser tiny stamen
tips keep the model within its original vertex limit. All nine anatomy and
twelve decoded pose checks pass on this revision; native visual acceptance
passed in front, world and battle views, plus an actual walking frame. Initial
Venusaur renders remain diagnostic before views.

Native captures use `CRYSTAL_RENDER_TEST_HEADLESS=1`,
`CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image` and `CRYSTAL_PREVIEW_MUTE=1`.
They do not open windows or change application focus. Review images identify
the species and pair the 2D and 3D views at the same authoritative game state.

The refined Venusaur captures completed in 91.979, 116.053 and 520.481 seconds.
The National Park battle stage admits the full animation bounds at the original
2.0066 metre size with uniform scale, zero support offset and no fallback.
The HUD identifies VENUSAUR and NIDORAN♀. Feet meet the ground in the close,
world, walking and battle views; the broad cheeks, rounded skin bumps, fitted
eyes, leaves and complete flower remain readable. Strong shadows remain under
the canopy. Ivysaur's three accepted pairs use the earlier reviewed binary;
its GLB remained byte-identical throughout the Venusaur refinement. Independent
asset, state, physical-size, movement and capture-binary audits passed for both
species. These views complete this pair's migration, while broader roster
fidelity, camera and browser coverage remain active work.
