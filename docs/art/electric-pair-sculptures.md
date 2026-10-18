# Original Electabuzz and Elekid sculptures

`tools/electric_pair_sculpt.py` authors both species. Electabuzz now uses
`crates/crystal-voxel-view/models/battle_species/electabuzz.glb`, authored and
strictly read by `tools/electabuzz_glb.py`; its static JSON is removed.
Elekid's canonical asset is
`crates/crystal-voxel-view/models/battle_species/elekid.glb`, exported and strictly
read by `tools/elekid_glb.py`. Its duplicate static JSON is removed.

Electabuzz has a broad low head, sharp fitted white eyes, black chevron, pointed
twin scalp tufts, ball-tipped antennae, separated fang-bearing jaws and a rooted
tongue. Its body carries a chest bolt, flank sweeps and a curved rear stripe
fitted to stored triangles. Bent arms end in knuckles, thumbs and wrist fur.
Striped shins support broad clawed feet; a curved striped tail has a round tip.
All 70 named parts remain. Adult-only adaptive face fitting, focused round
sampling and lower-density stripe interiors reduce 64,252 vertices/113,628
triangles to 19,502 vertices/33,748 triangles. Height 1.28, floor zero and
colors remain unchanged. Elekid's recipe and canonical bytes remain exact.
The [earlier source review](electric-pair-source-review.md) records references;
no external model, rig or animation is imported.

Elekid has a continuous yellow egg body, two rounded contact blades with real
through holes, fitted angular eyes, pointed grin, lower bolt and a closed black
waist wrap. Slim arm roots lead to broad striped forearms and three-clawed fists.
Short legs end in grounded feet. Normal colors come from the local pack:
(255,255,41) yellow, white and black. Pack height `200` gives a 0.6096 m
production scale; authoring height remains 0.91.

Twenty-eight Elekid parts preserve exact neutral f32 geometry. Only the four
arm bands reduce their longitudinal sampling to fit the existing skin limit.
The resulting model has 32 parts, 19,466 vertices, 33,128 triangles, 139 accessors
and 715,700 bytes. Neutral digest:
`5087eaed56db849c823c7e069ef7879375ebb39c6d3f68c84de469b858ad076a`.

Nine joints control the torso, upper body, two plug blades, arms and leg middles.
The root fixes the feet. Body, waist band and bolt use one spatial weight field;
the fitted gaze and mouth stay with the upper body. Each arm connector, forearm,
stripe, fist and claw shares one rigid owner, preserving wrap fit through motion.
Plug blades rotate from rooted pivots while retaining their open holes. Idle,
attack and hit clips rotate joints with neutral endpoints; production cues
control attack/hit timing.

```sh
blender -b -t 2 --python tools/build-battle-species.py -- /tmp/geothite-elekid --only ELEKID
python3 -m unittest discover -s tools -p test_elekid_glb.py
python3 -m unittest discover -s tools -p test_electric_pair_sculpt.py
```

The eight skin checks pass, including canonical bytes, budgets, neutral bind,
normal palette, ownership, fixed feet, neutral endpoints, winding, edge strain,
actual posed attachment volumes, and five rays through each posed plug hole
against every posed part. The pair sculpture suite passes all thirteen checks
for closed solids, normals, parity, anatomy, neutral joins, face fitting and
clearance, self crossings, open holes/jaws and filled/detached negative controls.
Blender exports byte-identical Elekid data. Front/rear software views were
inspected. Both focused Rust tests, the native build and wider catalog grounding /
elapsed battle pose regressions pass. The normal paired review passes.
Shiny checks are skipped at the user's request. These model checks do not
establish TUI or MCP gameplay.

The matched normal native 3D/2D review against Hoppip passes. Elekid's yellow
egg body, fitted gaze and bolt, open contact holes, striped forearms, clawed
fists and planted feet read clearly at the verified scale. The terrain is
accepted; the paired receipt preserves core frame 2194, hash 498538258, LCD
frame 2811, time 49.87255725 and `state_preserved=true`. Inspected front/rear
software views cover surfaces concealed by the battle profile. Shiny checks
remain skipped.


The Electabuzz geometry preparation passes all thirteen pair-anatomy checks
against the current canonical assets (14.413 s). Elekid still exports exactly
the same canonical bytes (SHA-256
`51c37811045e412c342acca14259d538fdf5166936b45a1bc3aed29c9d2ec9af`).
Adult-specific helper functions preserve the baby recipe. This records the
geometry preparation before the rig migration below. All further automated
native review must be headless and must not create windows or change focus.


A native GPU paired capture now completes entirely offscreen with Winit and
native window retention disabled. It uses explicit
`CRYSTAL_RENDER_TEST_HEADLESS=1 CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image`,
muted audio and normal-only party settings. Route36 (20,8), noon produces
1000×700 images with preserved state: frame/LCD 2178, checksum 3615010567,
virtual seconds 37.959714334. This verifies the non-disruptive capture path;
This earlier capture predates the canonical skin migration below.


## Electabuzz canonical skin

| Vertices | Triangles | Logical / runtime parts | Joints | Accessors | Bytes |
|---:|---:|---:|---:|---:|---:|
| 19,502 | 33,748 | 70 / 41 | 19 | 234 | 836,212 |

The nineteen joints are root, body, neck, head, two shoulder/elbow/wrist
chains, two supported leg middles, two antennae, two scalp tufts and three
tail carriers. Arm pivots follow the existing distinct raised-left and
forward-right poses. Closest points on the actual smooth arm and tail
centerlines define continuous spatial weight fields. Every stripe uses its
parent's field. Knuckles and thumb folds follow the matching wrist, while
jaw, fangs, tongue, nostrils and gaze share a coherent head. Antenna stalks
and terminal balls move as complete assemblies. Battle feet and toe claws
remain fixed; world walking uses two complete support chains.

All 70 named source solids and their order remain in losslessly grouped
material assemblies, with exact f32 geometry, normals, colors and per-vertex
weights. The existing 20,000 vertex, 40,000 triangle, 64 part, 256 accessor
and 1 MiB limits remain unchanged. No physical-size, terrain support,
omission or whole-body/frustum admission workaround is used.

Logical geometry pin:
`b8587f1ad8a7c2bfcf108f28c065448958c95600f35fc98eeda1ec4b3c1da4da`.
Packed geometry pin:
`b043a49e3f3885b12adb6401eee1999cb9dcaa956952ca39e300cabc97bc0c34`.

All 21 Python checks pass (66.550 s), combining eight independently decoded
stored-skin checks with the thirteen pair anatomy checks. They cover exact
canonical geometry/packing/binds/budgets, rigid owners, all eighteen independent
carriers, winding and edge strain through clips, fixed supports, occupied
roots and stripe wraps, uncrossed surfaces, fitted face clearance, and jaw
space against every posed solid. Filled/detached and byte/geometry mutation
controls remain effective. The full Blender builder exports exactly the
canonical Electabuzz bytes. Elekid's canonical bytes remain unchanged.

Idle uses 81 keys over 3.2 seconds; attack and hit use 65 keys over one
source-relative second. Clip endpoints are neutral. Production source cues
own playback timing. Rust and final offscreen native verification are recorded below. Browser and moving-world visual acceptance remain open.
Shiny checks are skipped; grass and renderer AA/filtering are untouched.


Three focused Rust checks pass (1.77 s): production GPU activation at the
external pack's verified 1.0922 m height, exact rigid owners plus independent
features and bounds through battle clips, and 64 walking phases with two
complete support chains and non-leg weights preserved. Original geometry
reference height remains 1.28 m. Front/side/rear comparison and
idle/attack/hit sheets retain the raised-left and forward-right silhouette,
coherent stripe wraps, knuckled fists, open fang-bearing jaws, antennae,
scalp tufts and curved tail. These sheets are software art review, not native
production evidence; the final offscreen production results follow below.


The complete 202-entry registry/schema and production source-cue timing
checks pass (195.14 s). Both rebuilt native examples contain the exact current
canonical compressed Electabuzz asset. All three review runs explicitly use
headless image targets and muted audio; Winit and native window retention
are disabled. No desktop windows or focus changes are needed.

Final native close/elevated Route36 (20,8), noon pairs preserve state against
the matching 2D images. The low skull, fitted chevron/gaze, open jaws/fangs,
coherent stripe wraps, knuckled fists, antennae and complete supported feet
remain readable. Tree foliage partly occludes the raised fist in the close
view; the software sheet and battle view show the complete hand. The elevated
view retains the full silhouette and tail. The paired Route29 battle exposes
correct ELECTABUZZ/PIDGEY HUD names and both complete models, with no fallback.
Authentic terrain admission passes (`accepted=true`) at the original pack
size. Support, omission and whole-body/frustum rules remain unchanged.

| Final headless view | Frame | State checksum | LCD frame | Virtual seconds |
|---|---:|---:|---:|---:|
| Route36 close | 2582 | 3000170094 | 2582 | 46.753952458 |
| Route36 elevated | 1546 | 3035634296 | 1546 | 26.551283625 |
| Route29 battle | 3726 | 3636466923 | 4486 | 88.241272706 |

Final ignored native evidence uses the `electabuzz-rig-` prefix under
`target/pokemon-native-review/`. These views accept the local anatomy,
coherence and support changes. Browser and moving-world visual acceptance
remain open. Elekid is byte-identical, shiny checks are skipped, and grass
plus renderer AA/filtering remain untouched.
