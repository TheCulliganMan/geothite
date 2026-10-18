# Goldeen, Seaking and Magikarp skins

The original flowing fish sculptures now fit the existing standard skin
budgets. Canonical GLBs replace their static JSON exports; the normal builder,
direct sculpture CLI, strict logical readers and Rust dispatch use these assets.
Registration contains 229 skins and 46 static species files. Full-roster model
and rig improvement remains active. Native paired evidence is recorded below; stored tests alone do not establish
production appearance.

Actual-parent clipped layers replace dense eye, paint and flank subdivisions.
Their cap normals follow the stored parents. Magikarp's forked tail retains its
outline with two subdivisions instead of three. Its concave ivory inset uses
bounded adaptive fitting (maximum surface error 0.001) because direct clipped
triangulation produced degenerate slivers. This keeps both inset sides seated
without changing shared mesh generation or fitting tolerances. All original
named parts, mouth cavities, horns, fins, whiskers and markings remain.

| Species | Previous vertices / triangles | Current vertices / triangles | Logical / runtime parts | Joints | Accessors | Bytes |
| --- | --- | --- | --- | --- | --- | --- |
| Goldeen | 43,891 / 66,788 | 16,748 / 20,668 | 36 / 28 | 6 | 155 | 710,424 |
| Seaking | 35,909 / 51,716 | 19,923 / 24,272 | 70 / 29 | 8 | 167 | 847,216 |
| Magikarp | 36,302 / 51,680 | 15,922 / 19,240 | 30 / 25 | 9 | 140 | 665,840 |

Body, lips, oral floor, eyes, horn, gills, forehead/flank markings and carp
scale seams share one body control. A coherent caudal fin, paired pectoral
paddles, dorsal sails, Seaking pelvic streamers and Magikarp pelvic crown/barbels have separate
controls. Fin paint/spots/rays, carp tail insets and paddle rims follow their
actual carrier. Every non-root joint moves geometry independently.
Goldeen and Seaking have six/eight joints; Magikarp has nine. Both source
caudal halves share one anatomical tail hinge. An initial split-hinge candidate
required world weight welding at their identical roots; blending those narrow
roots produced a posed winding failure. The final shared hinge retains the
full rigid tail, shared roots and paint without any tolerance relaxation or
change to the production world weight welder. The identity root is structural and has no directly weighted vertices for these swimmers.
Yaw strokes preserve the authored waterline exactly; no floor clamp is used.
Idle is 3.2 seconds/81 keys and attack/hit are one second/65 keys, rotation-only
with source-cue timing and neutral endpoints.

Packing combines only same-material coherent assemblies: per-carrier painted
fin spots/washes/rays, supported body markings and each side's carp scale seams.
Exact logical ranges restore every named original part and its geometry order.
No 20k-vertex, 40k-triangle, 64-part, 64-joint, 256-accessor or 1 MiB limit is
increased. Authoring reference heights remain 0.8, 1.0 and 0.9 respectively.

Eleven anatomy and eight independently decoded skin checks pass against the
final canonical stored GLBs (19 checks in 42.133 seconds). Checks cover exact
source/packing, hierarchy/inverse binds, all controls, original budgets, closed
outward solids, actual corner normals, waterline, rigid strain/winding and
endpoints, occupied fin/lip/whisker roots through poses, fitted vertices/edges/
centres against actual parent triangles, real empty mouth space through all
posed parts, a filled-mouth negative control, and byte/source/species mutation
rejection. An initial posed-fit helper omitted normals needed for reflecting
rear fin layers; it was corrected without changing geometry or tolerances.

Five focused Rust checks pass in 2.47 seconds: original-size GPU activation for
all three fish, canonical waterline/coherent mouth/independent controls, and
complete world locomotion bounds with exactly unchanged canonical bindings.
Original pack dimensions are Goldeen 0.6096 m, Seaking 1.2954 m and Magikarp
0.889 m. The existing `battle_layout::reference_span` maps these fish dimensions
to longitudinal nose-to-tail Z extent, not vertical height. The native audit
uses that same reference with uniform scale and unchanged zero support offset.
The normal background builder and direct sculpture CLI match Python canonical
export bytes exactly. Both current native capture tools built in 58.22 seconds.

The ignored `target/pokemon-native-review/goldfish-skin-review.png` was inspected
inline: previous static, optimized decoded GLB, front/rear/side and decoded
attack at 0.34 for all three species. Flowing horned Goldeen, speckled Seaking
and Magikarp's vacant gaze, long barbels and forked sail remain distinct.
Open lips and layered fins/markings remain readable. Native original-size
paired review is a separate gate below. All rendering stays offscreen and
muted. No windows, focus changes, shiny checks, grass, AA/filtering, pack,
audio or gameplay changes belong to this batch.

Reproduction:

```sh
/Applications/Blender.app/Contents/MacOS/Blender --background --threads 2 \
  --python tools/build-battle-species.py -- \
  crates/crystal-voxel-view/models/battle_species --only goldeen,seaking,magikarp
PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=tools \
  python3 -m unittest -v test_goldfish_sculpt test_goldfish_glb
```

| Species | Logical float32 geometry SHA-256 | Packed float32 geometry SHA-256 | GLB SHA-256 |
| --- | --- | --- | --- |
| Goldeen | `2404715bff4e90536eae741ef0234b920327cbde595b664707417ed7dfedcb33` | `248b6d97497e6c5d260216233c6692c8d2636c2f99573ee5302025acc47bba31` | `33dd186f4d99e79cdef73c9b65e16c32a2455302b8f416d824c1adf8b369764e` |
| Seaking | `c18e842366cc30e35c38be85016d512d9f56f8b0051ed9a727effb9f94928343` | `598ef7f5ae399ebef8d5bed97e398152862673a02de191b4cade5bb6c2ea9b57` | `7fbaf016e489103f83df753c56e58e76337cd2f538477e19c0e160efc73d0137` |
| Magikarp | `260a49a0907f7cba1ffe561fd845f113196cf24b663b5e65842d51f97ce77c50` | `75e6d1ec0cd9b364d8c982351adf6acd23f477048cbea91a461d53daf622aabc` | `686605e61e6d20bf7460660d779f71ff04f3fb92437e649cbaf9ec087eeff956` |

The production capture examples use `location-tester,bevy/zstd_rust`. Native
launches set `CRYSTAL_RENDER_TEST_HEADLESS=1`,
`CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image` and `CRYSTAL_PREVIEW_MUTE=1`.
The path disables Winit and macOS primary-window retention and uses the schedule
runner. Binary preflight checks headless/mute tags and the exact compressed
current GLB payloads. Captures use normal coloration; no shiny verification is
performed. Current test/build and receipt evidence stays ignored under target.

Example offscreen world reproduction from this checkout:

```sh
CRYSTAL_RENDER_TEST_HEADLESS=1 CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image \
CRYSTAL_PREVIEW_MUTE=1 CRYSTAL_RENDER_TEST_PARTY_SHINY=0 \
CRYSTAL_RENDER_TEST_PARTY_SPECIES=SEAKING \
CRYSTAL_RENDER_TEST_WINDOW_SIZE=1000x700 \
CRYSTAL_RENDER_TEST_CAMERA='3 2 35 0' \
CRYSTAL_RENDER_TEST_WALK_INPUT=synthetic CRYSTAL_RENDER_TEST_WALK_STEPS=0 \
./target/pokemon-models/debug/examples/render_at_location \
  --pack content-packs/core-modular.browser.crystalpack \
  --map Route36 --x 20 --y 8 --view both --hour 12 --walk D \
  --screenshot target/pokemon-native-review/goldfish-seaking-world.png
```

The shared schema/physical-size check passes in 87.11 seconds and the shared
elapsed-idle/exact-current-attack cue check in 110.33 seconds. Together with
19 stored Python checks and five focused Rust checks, 26 scoped checks pass.
No full-suite or complete gameplay claim is made from this model review.

## Repeated source-height preparation

An initial offscreen battle route reached terrain setup but exceeded the
external 360-second review allowance. Sampling its live retry found repeated
`render_playfield -> terrain_ledges::datum -> source_terrace_datum` work,
including terrace-level/domain rebuilding. This was presentation preparation,
not a failing fish geometry or stage admission check.

`mesh/terrace_datum.rs` now keeps at most four complete results, keyed by exact
map name, ordered ledges and ordered native source cells (including each tile's
identity and authored land permission). Equal inputs reuse the immutable result;
changed evidence rebuilds through the original calculation. Negative results
are cached too. Inputs over 65,536 source cells or ledges use the same uncached
calculation. Camera crops are never a key or a substitute for native evidence.
No terrain height, permission, support, stage rule or scene geometry changes.

Eight datum checks pass in 0.06 seconds: coastal/island and waterfall ownership,
legacy indoor/forest/inland handling, complete native fields across camera
crops, exact terrain/actor-support agreement with unchanged water geometry,
changed map/edge/waterfall invalidation, source overlap priority and bounded
retention. Together with the 26 model/schema/pose checks, 34 scoped checks pass.
Both capture tools rebuilt successfully in 44.36 seconds after this cache.
The final native review uses that same rebuilt pair of executables throughout.

## Native paired review

Nine current production 3D/2D pairs (front, world and battle for each fish)
and three moving-world frames were inspected inline. All images are 1000×700,
headless and muted. Every paired authoritative state is preserved. Six front/
world captures retain 32 actual synthetic-input movement rows over five Route36
positions, (20,8) through (20,12). Three named walking battles preserve state,
report no fallback and admit complete bodies at original 0.6096/1.2954/0.889 m
nose-to-tail dimensions, with uniform scale and zero support offset. Camera,
authentic-ground, omitted-geometry/shadow and complete-body gates are unchanged.
The first local audit incorrectly used height for these longitudinal fish;
the corrected audit matches the existing shared production reference exactly.
No renderer measurement or physical dimension changed to satisfy the audit.

The original horns, empty oval mouths, sleepy/vacant/glossy eyes, flowing or
forked tails, fin paint, body markings and barbels remain distinct and readable.
Both tail halves retain one carrier without cracks. World profile views hide
the far eye, some fin layers and recessed oral floors. The far white caudal
surface on Goldeen/Seaking can receive a very dark shadow; thin folds and scale
seams show edge aliasing at distance. These remain broader lighting/AA work.
Existing intentional Float locomotion is retained. Browser, all camera/world/
material combinations and full gameplay acceptance are not claimed.

The final cache-enabled captures completed in 35–75 seconds each. Goldeen's
front/world loops recorded 44.16/42.82 seconds versus 65.17/73.59 before the
cache; its completed battle saved images in about 57 seconds. These are observed
review timings, not an isolated benchmark (encounter witnesses and scene clocks
can vary). All nine final pairs use the same stable rebuilt executable IDs and
exact embedded current GLBs. Temporary source/candidate copies, completed
producer scripts and redundant intermediate frames were removed. Final raw/
paired images, receipts, movement CSVs, selected moving frames, metrics, logs
and diagnostic sampling remain ignored under `target/pokemon-native-review`.
No shiny, grass-height, AA/filtering, pack, audio or gameplay changes are included.
The full-roster model and rig goal remains active.

| Species / view | Frame / checksum | LCD frame | Virtual seconds |
| --- | --- | --- | --- |
| Goldeen front | 2001 / 3137978681 | 2010 | 34.512865166 |
| Goldeen world | 1820 / 1609086 | 1831 | 31.274679626 |
| Goldeen battle | 2004 / 2179121656 | 2618 | 44.81669425 |
| Seaking front | 1596 / 2486667308 | 1602 | 27.44606925 |
| Seaking world | 1530 / 567033575 | 1536 | 26.239072167 |
| Seaking battle | 2428 / 3267013274 | 3065 | 53.594821791 |
| Magikarp front | 1977 / 1937229432 | 1985 | 33.978803834 |
| Magikarp world | 2082 / 913571799 | 2089 | 36.287966834 |
| Magikarp battle | 2573 / 47884718 | 3279 | 56.893622667 |

Stable executable SHA-256:
- `render_at_location`: `f5192d2fbed943dbc09abc799c68e001c48abf4381920e6e0673b8ed3cfd1fa9`.
- `immersive_battle_3d`: `a9fd821ce4a8aec9f367bbe622528d514c4a2f96d2527e9b7f873a65c47b0d7f`.
