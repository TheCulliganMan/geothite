# Clefable and Wigglytuff canonical skins

The optimized sculptures are now canonical production standard skins. Normal
builder/readers publish/read the stored GLBs and retire the old static JSONs.
Rust registers their exact geometry, hierarchy, source-size GPU articulation
and complete-foot walking. The registry contains 223 skins and 52 static
species files remain. The full-roster model and rig goal remains active.

Wigglytuff's eye-border, eye-white and mouth subdivision levels fall from three
to two. The original outlines, tall body, ears, belly, arms, feet, colors and
forelock remain. The broad white belly retains its four subdivision levels and
offset; its fitted mouth remains visible. Vertices fall from 28,880 to 18,648
and triangles from 48,060 to 31,548, bringing the sculpture below the existing
20k-vertex/40k-triangle skin limits.

Clefable's original body, ears, wings, hands, feet, face, forelock and curled tail
remain. A stronger edge/centre/vertex check exposed ear-tip cap edges sinking
0.000711 authoring units into their parent at rest. The four brown ear caps now
clip the actual front/rear parent triangles and use their shading field, rather
than subdivided chords. This fixes clearance and reduces vertices from 18,549
to 15,603 and triangles from 32,136 to 27,448. No silhouette shrink is used.
Both source recipes expose raw coordinates for skeleton pivots, and normalize
numerical zero signs for identical float32 geometry across Python runtimes.

`tools/fairy_pair_glb.py` supplies standard canonical exporters and bounded strict
readers. The floor root is identity, the round body and fitted facial/belly
layers share one carrier, and the complete feet/toenails stay planted in battle
clips. Both species have independent ears, arms and forelock. Clefable adds
independent wings and two tail carriers. Its existing rear-wing authoring
rotation reverses the signed source suffix: skin ownership uses the actual
stored X side, verified against the physical joint pivot. No empty head or leg
controls are invented for these headless pear/balloon sculptures.

Idle is 3.2 seconds with 81 keys, attack/hit are one second with 65 keys, and
all endpoints are neutral. Rotation-only clips retain source-cue timing. Both
arms swing substantially during attack; every control moves stored geometry
independently. Logical source part order and all per-part buffers are preserved
without combining feet or surface layers. Authoring reference height is 1.0
for both; original physical pack dimensions are verified by the final native stage traces below.

| Species | Parts | Vertices | Triangles | Controls | Accessors | Bytes |
| --- | --- | --- | --- | --- | --- | --- |
| Clefable | 32 | 15,603 | 27,448 | 11 | 150 | 608,640 |
| Wigglytuff | 24 | 18,648 | 31,548 | 7 | 116 | 740,968 |

All existing limits remain unchanged: 20k vertices, 40k triangles, 64 runtime
parts, 64 joints, 256 accessors and 1 MiB. Background Blender's normal static
exports match current source exactly. Both Python runtimes generate identical
canonical GLB hashes. No duplicated geometry catalog or intermediate binary is
added; tests and software review decode generated skin bytes directly.

All eighteen sculpture checks and eight independently decoded skin checks
pass together in 125.120 seconds against the final stored canonical GLBs. They cover actual stored closed
outward topology/shading, geometry pins/budgets, hierarchy/inverse binds,
complete joint use, physical-sided controls, coherent fitted carriers,
independent controls and both-arm reach, fixed feet, floor/winding/strain,
neutral endpoints, occupied-volume roots through clips, fitted cap vertices,
edge midpoints and triangle centres through poses, and strict byte/source/species
mutation rejection. The fitted-surface clearance thresholds stay 0.0001/0.017.

Posed fitting casts in the actual parent's transformed authored front direction.
A rear cap can project beyond its ear's world-XY silhouette during rotation
while remaining supported; a world-Z ray alone is then not a surface-distance
oracle. The test first requires the actual decoded parent and child to share
the expected rigid carrier and unit weight, then transforms both posed meshes
back through that decoded carrier and tests their actual triangles. This
retains the strict samples and clearance thresholds while making the check
independent of camera direction. It still catches the original rest cap defect.

The ignored `target/pokemon-native-review/fairy-pair-skin-review.png` was
inspected inline. Rows compare the old static sculpture with the skin
quarter/front/rear views and decoded attack at progress 0.34. The existing
silhouettes, layered eyes, mouth and belly remain readable; the fairy wings,
rolled forelocks, long Wigglytuff ears and Clefable's curled tail stay distinct.
This software review is authoring evidence, not production visual acceptance.
Final original-size native review is recorded below; broader browser/camera/world
acceptance remains open. All rendering was offscreen, with no window, browser tab or focus change.
No shiny checks, grass, AA/filtering, audio, pack or gameplay work is included.

Reproduction commands:

```sh
/Applications/Blender.app/Contents/MacOS/Blender --background --threads 2 \
  --python tools/build-battle-species.py -- \
  crates/crystal-voxel-view/models/battle_species --only clefable,wigglytuff
PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=tools \
  python3 -m unittest -v test_clefable_sculpt test_wigglytuff_sculpt
PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=tools \
  python3 -m unittest -v test_fairy_pair_glb
```

The initial source checks used ignored candidate JSONs equal to the normal
builder outputs. All twenty-six checks now read the actual canonical GLBs,
retaining exact float32 geometry parity with the recipes. Existing anatomy
checks now enforce the actual 20k/40k skin budgets. Temporary candidate JSONs
were removed after parity verification. Final logs, metrics/hashes and the
authoring sheet remain ignored under `target/pokemon-native-review/fairy-pair-*`.

| Species | Float32 geometry SHA-256 | Canonical GLB SHA-256 |
| --- | --- | --- |
| Clefable | `2dcfdca000ef1ccc8fa751fe5e95753104aa8097685d5200ff18cb6dbcf1c3e8` | `e50aca9edd66f426173f6d155c8c9b3ba3f3c34bcde58a438632aad535d7371b` |
| Wigglytuff | `ba8c7ae0e3fadd9d0a8f91ae55d2ab67ae4e2aed3432ea20df0254265ccf7614` | `b562180a39a8f224d1adccd7ffd4d80fc043ad89b3d152e4a200d06672dd2370` |


## Production runtime checks

Four focused Rust checks pass in 1.78 seconds: exact registered geometry,
fixed battle feet and animated bounds, independent anatomical carriers, two
original-size GPU instances, and two complete foot support trees over 64 gait
phases. World walking preserves every non-foot canonical carrier, including
the fitted eyes/ears, forelocks, wings, tails and hand details. Clefable's
complete toenail stays attached to its own foot. No locomotion algorithm or
shared role heuristic changed for this integration.

The all-species schema check passes in 95.44 seconds and the shared exact
battle-cue timing check passes in 108.47 seconds against the current assets.
Thirty-two scoped checks pass in total (26 Python, four focused Rust, two
shared Rust). This does not claim a complete gameplay-suite pass.
The native examples rebuilt after the asset/source freeze in 1m 02s.

```sh
CARGO_TARGET_DIR=target/pokemon-models \
CRYSTAL_RENDER_TEST_PACK="$PWD/content-packs/text-tui.crystalpack" \
  cargo test --locked -p crystal-voxel-view fairy_pair --lib -j1 \
  --features bevy/webgl2,bevy/bevy_winit,bevy/x11,bevy/bevy_gilrs,bevy/zstd_rust,crystal-assets/test-fixtures \
  --config profile.dev.package.crystal-voxel-view.opt-level=0 \
  --config profile.dev.package.crystal-bevy.opt-level=0 \
  --config profile.dev.package.crystal-assets.opt-level=2 \
  --config profile.dev.package.crystal-core.opt-level=0
CARGO_TARGET_DIR=target/pokemon-models cargo build --locked -p crystal-bevy \
  --features location-tester,bevy/zstd_rust \
  --example immersive_battle_3d --example render_at_location -j1 \
  --config profile.dev.package.crystal-voxel-view.opt-level=0 \
  --config profile.dev.package.crystal-bevy.opt-level=0 \
  --config profile.dev.package.crystal-assets.opt-level=2 \
  --config profile.dev.package.crystal-core.opt-level=0
```


## Final native presentation review

Six production 1000×700 3D/2D pairs and two actual moving-world frames were
inspected inline. All renders use `CRYSTAL_RENDER_TEST_HEADLESS=1`,
`CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image` and `CRYSTAL_PREVIEW_MUTE=1`.
No application window, browser tab, activation or focus change is needed.
No shiny verification was run.

World captures use the completed external
`content-packs/core-modular.browser.crystalpack`, Route36 (20,8), hour 12,
`--view both`, synthetic walking D, ordinary front camera `0 0 20 1` or world
camera `3 2 35 0`. Four movement traces each contain 32 samples and five actual
authoritative coordinates, (20,8) through (20,12). Extra moving readbacks
supplement the frozen final pairs; they are not additional time-matched 2D
comparisons. Every final pair has `state_preserved=true`.

Battle captures use the actual Route29 walking encounter, held production
battle, `--walk RALARALARALARALARALARALARALARALARALARALARALARALARALARALARALARALARALARALARALARALA`,
`--size 1000x700`, `--screenshot` for 3D and `--second` for 2D. Both final
encounters face SENTRET. The actual HUD names are CLEFABLE/SENTRET and
WIGGLYTUFF/SENTRET. Last terrain admissions accept complete original-sized
bodies with `reason=None`, no fallback and zero support offset. Stage
scale/reference calculations retain Clefable 1.2954 m and Wigglytuff 0.9906 m.
Clefable's search initially rejected border/scenery-divided camera shots before
finding its accepted composition. No model shrinking, false support, or relaxed
terrain/frustum/omission rule was used to bypass those rejections.

| Capture | Authoritative frame / hash | LCD frame | Virtual seconds |
| --- | --- | --- | --- |
| clefable front | 2430 / 3368153747 | 2438 | 41.589225625 |
| clefable world | 2155 / 2095978686 | 2164 | 36.877480375 |
| clefable battle | 3985 / 1921311945 | 4759 | 82.234049499 |
| wigglytuff front | 2170 / 1823134826 | 2181 | 37.270673959 |
| wigglytuff world | 2940 / 729127789 | 2953 | 51.351389459 |
| wigglytuff battle | 2801 / 3812347337 | 3429 | 59.264603208 |

Before launch, both executables were checked for headless/mute support and
exact compressed current canonical GLB payloads, decoded equal to source.
Streamed hashes, byte sizes and modification timestamps remained stable
through the six final captures:

| Executable | SHA-256 | Bytes | mtime ns |
| --- | --- | --- | --- |
| render_at_location | `38ceea071175b5435152a328c445d1f9a06d9ef2fb786c5fe1a2839e120900b0` | 408507376 | 1792303151526085583 |
| immersive_battle_3d | `ae05f26246c8b29715cf63131c47ad77d2534811926ff097c68f9e48866b3a8a` | 408272840 | 1792303152880288801 |

Clefable retains its wide brown-tipped ears, rolled forelock, quiet eyes/smile,
raised hands, pointed wing webs, curled tail and complete feet. Front foliage
overlaps one hand/part of a wing; the full silhouette fits the reviewed ordinary
camera at original size. World profile naturally makes the thin wing webs look
edge-on. The battle view retains the ear caps, face, wing/tail silhouette and
feet, without cropping. Wigglytuff retains its tall ears, charcoal linings,
large blue eyes, visible smile, fitted white belly, forelock and complete feet
in the front, world, moving and battle views. Its side view naturally hides
one eye. Some faint belly lighting bands remain presentation polish, alongside
broader foliage/camera/aliasing issues. Shapes, rigging and original-size
placement are verified in these scoped native scenes; full browser and broader
world/camera/material acceptance remain open.

The existing Surf fixture failure and unrelated project-wide issues are not
modified or reported as passing. No complete playable-game or TUI/MCP claim
follows from these model checks. No grass, AA/filtering, audio, pack or gameplay
changes are included.

Final ignored evidence lives under `target/pokemon-native-review/fairy-pair-*`:
full-resolution pairs/raw images, paired-state receipts, world movement traces,
selected moving frames, current test/build logs, asset hashes and executable
identity. The neutral/rear/front/decoded-attack authoring sheet remains
`fairy-pair-skin-review.png`. Temporary capture producers, candidate metrics,
duplicate CLI exports and battle-introduction movement scratch were removed.
Build products and external packs remain ignored and unstaged.


Direct sculpture CLI exports now also write canonical GLBs. Both an explicit
file output (`clefable_sculpt.py --output .../clefable.glb`) and a directory
output (`wigglytuff_sculpt.py --output ...`) were executed into ignored scratch
and matched the stored canonical bytes exactly, with no duplicate JSON emitted.
Those scratch copies were removed after verification.
