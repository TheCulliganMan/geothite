# Jynx: grounded folded gown and articulated five-digit hands

Jynx's original detailed recipe exceeded the existing limits at 48,266 vertices
and 79,520 triangles. The adult recipe now uses bounded tessellation while
retaining shaped parted hair, layered curtains and tips, open five-digit hands,
rooted clothing trim and a real recessed kiss rim. The gown has a smoother
seven-fold profile, the gaze is enlarged, and the kiss rim is broader. Narrow
hair grooves are clipped to their actual parent triangles so they do not cut
through the lower-resolution hair surfaces.

The normal local-pack sprite was inspected directly. Its four-color palette
uses plum RGB555 (14,2,15), pink (29,6,19), white and black: ivory hair and trim,
pink gown/lips and plum skin. These expanded, linearized materials are checked
in the decoded asset. Pack `heightDigits=407` preserves the original 1.397 m
height through uniform scaling; the authoring height remains 1.28. Full animated
bounds, camera, collision and admission rules remain unchanged.

The rig has 22 controls: floor, torso, neck, head, two hair curtains, shoulder/
elbow/wrist chains for both arms and ten individual fingers. The gown blends
from a pinned hem into the upper torso; the throat blends into the head.
Complete gaze and the recessed kiss rim remain rigid on the head. Curtain
markings share the curtain field, and all finger roots blend from their own
wrist. Idle has gentle dance and hand motion; attack and hit use source-relative
cues. Clips rotate only and have neutral endpoints.

Coherent hand packing retains all 45 logical anatomy ranges in 35 runtime
parts without changing any source f32 position, normal, color or triangle.
The canonical GLB contains 15,311 vertices, 24,740 triangles, 22 joints and
209 accessors in 688,640 bytes. All existing model limits stay intact.

`tools/ice_dancers_sculpt.py` remains the anatomy source and
`tools/jynx_glb.py` exports the canonical skin. The shared reader, species
builder, bounded Rust registry and production GPU renderers consume that GLB.
The old Jynx JSON is removed. The paired sculpture CLI now emits canonical
GLBs; both the CLI and background builder match Jynx byte for byte.

Smoochum's authoring and canonical asset remain unchanged, including its
palette and all skin/clip bytes. Its GLB SHA256 remains
`e61adfe66552ccfb6b45f75da84c892494e784c9a693ce191c2d76f9aab62ab5`,
with a dedicated regeneration guard. Adult-only reductions and palette work
are isolated from the baby branches of shared helpers.

Jynx source geometry SHA256:
`d1a7fc03473ac5f3eb42d0f738ec5926ab96bcdeddbc5d0d0017e5a9a294ecf0`.
Packed geometry SHA256:
`d2fe39c6dbd02071e663680a8656da8d7e9caf6cc24daf6fb3a2c612b76a0921`.
GLB SHA256:
`64b578cf210d8cc5ea5c33043b48ec118a389bc47f78e1e4a4753c37f11e0ff8`.

Twenty Python checks pass in 10.174 seconds: eleven shared anatomy checks and
nine independent decoded-skin checks. They cover closed outward solids,
normals, exact regeneration, grounded height and strict budgets, distinct
anatomy, actual clothing/hair/hand/digit/lip roots, fitted surface clearance,
empty kiss aperture with filled/detached negative controls, self-intersection,
logical packing, hierarchy/inverse binds, independent controls and ownership,
fixed hem, positive posed winding and bounded strain through every clip,
source cues/endpoints, palette, mutated data rejection and unchanged Smoochum.
Three focused Rust checks pass in 0.96 seconds for canonical geometry,
original-size GPU admission and composed world bindings/contact/full bounds.
Shared schema and exact elapsed-idle/current-attack-cue checks pass in
116.20/116.50 seconds, bringing the batch to 25 scoped checks. Both native
clients build in 66 seconds. These checks do not claim full-suite or gameplay
completion.

The decoded before/after, front/rear/side and attack sheet was inspected inline.
The production front pair at EcruteakCity (20,21) completed in 50.245 seconds
and was inspected inline. It preserves the same authoritative state between
2D and 3D, with readable pupils, a visible kiss aperture, complete shaped hair
and open hands, and grounded hem. World and battle pairs completed in 58.294/113.105 seconds and were inspected
inline with the moving world frame. The city view keeps the original-size
model visible beside the player, with the hem on the path. The Route29 battle
shows complete Jynx and Rattata models without fallback and readable JYNX/
RATTATA HUD names. Each pair preserves authoritative state between 2D and 3D.
Both completed capture groups use exactly the same binaries, with SHA256,
size and modification time unchanged throughout and independently audited
afterward; the exact current compressed GLB is verified in both executables.
Binary SHA256 values are
`400110cfbca3db6569d3699d87eef73f5d52eead55d3b25967c67c9a45ccb7c8`
(render_at_location) and
`c9b8a299a572ac54080f9832ecf185f785678c6f80db0bd430cf8eeaf55e6448`
(immersive_battle_3d). Broader scene and full-roster acceptance remain open.
All rendering remains headless, offscreen and muted. No shiny, grass-height,
scenery, AA, audio, pack or gameplay changes belong to this batch. The registry
reaches 262 skin variants and 13 static species files; full-roster improvement
remains active.

Rebuild into ignored output:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 tools/ice_dancers_sculpt.py \
  --output target/jynx-rebuild
/Applications/Blender.app/Contents/MacOS/Blender --background --threads 2 \
  --python tools/build-battle-species.py -- target/jynx-rebuild --only JYNX
```
