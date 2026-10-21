# Pidgeot and Fearow production skins

The [2026-10-21 family fidelity revision](pidgey-family-fidelity.md) supersedes
Pidgeot's geometry and resting wing motion below. Its current skin has 18,266
vertices, 24,648 triangles, 105 source parts, 25 packed parts, 20 joints and
834,244 bytes. Height remains 1.35 with zero footing and the original
1.4986-metre pack size. The revised crest, folded-wing frame, wrist/tail
fields and pivots follow the actual new anatomy. Fearow's bytes and motion
are unchanged in that batch. The subsequent
[Spearow/Fearow revision](spearow-fearow-fidelity.md) gives Fearow its airborne
pose, revised pivots, body-owned tucked leg controls, broad wing planes and
strictly closed idle seam. Its current skin has 13,219 vertices, 16,748 triangles,
101 source parts, 21 packed parts, 19 joints and 602,768 bytes. The remaining
migration record describes the earlier skins.

The production models use canonical standard GLB skins authored by
`tools/raptor_bird_glb.py` from the original recipe with `rig_ready=True`.
The shared species registry, GPU renderer, asset readers, STL exporter and
background builder select those skins. The registry contains 271 standard
variants and five static species. Same-state native production review is
complete; the full-roster improvement goal remains active.

The original Pidgeot has 47,647 vertices and 70,352 triangles; Fearow has
42,861 vertices and 57,532 triangles. Both exceed the shared skin loader limits.
The rig topology keeps every named anatomical part and the complete sampled
feather outlines. It removes redundant interior feather subdivisions and fits
face, breast and pale-tip layers with adaptive parent-clearance refinement.
Shared geometry helpers retain their original default behavior for other species.

The replacements have 18,224 vertices / 23,952 triangles / 105 anatomical parts
for Pidgeot and 18,030 / 22,512 / 161 for Fearow. Runtime packing groups matching
materials and related anatomy into 25 and 21 primitives while retaining source
ranges, names, order and individual skin weights. No anatomical part is deleted.
All six neutral bounds exactly match the original stored values. Authoring
heights remain 1.35 and 1.23, with zero footing; physical pack dimensions and
uniform production scaling remain authoritative.

Pidgeot has 20 controls; Fearow has 19. Both have an identity floor root, yaw
body, two-stage neck, coherent head, true lower-bill hinge, paired wings/wrists,
five separately rooted tail feathers and two supporting shin controls. Pidgeot
adds three swept ribbon controls; Fearow adds its pointed crown and rear mane.
Head markings follow one rigid head transform. Battle toes and talons stay fixed.
The idle wing stroke reaches 0.20 radians so the existing world renderer
recognizes authored flight and preserves it. Idle, attack and hit are
rotation-only clips with neutral endpoints; the attack
and hit clips are relative to the original source cue timing.

An initial body-to-wing blend folded a thin spar triangle. The authored spar
already penetrates the body around its hinge, so the complete wing root now
rotates rigidly there and blends only toward the outer wrist. A similar local
fold in a long crest ribbon was corrected by articulating the intact feather
at its base. Occupied-volume root checks and the existing positive winding and
20-percent strain bounds remain required. Fearow's open-jaw oracle undoes the
decoded head transform before casting along the authored bill axis, preserving
the actual moving mandible and every possible mouth obstruction. The throat
marking interpolates the actual quantized stored neck weights,
following the existing fitted-tongue weight-surface pattern. Posed clearance
checks select the original front cap triangles: neck bending can tilt a buried
root wall toward +Z, which must not misclassify that wall as a front marking.

Export and check without opening windows:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 tools/raptor_bird_sculpt.py --rig-ready \
  --output target/pokemon-native-review/raptor-rig-candidates
PYTHONDONTWRITEBYTECODE=1 python3 tools/raptor_bird_glb.py \
  --output target/pokemon-native-review/raptor-rig-candidates
CRYSTAL_RAPTOR_BIRD_TEST_MODELS="$PWD/target/pokemon-native-review/raptor-rig-candidates" \
CRYSTAL_RAPTOR_BIRD_TEST_RIG_READY=1 PYTHONDONTWRITEBYTECODE=1 \
  python3 -m unittest discover -s tools -p test_raptor_bird_sculpt.py -v
CRYSTAL_RAPTOR_BIRD_SKIN_MODELS="$PWD/target/pokemon-native-review/raptor-rig-candidates" \
PYTHONDONTWRITEBYTECODE=1 \
  python3 -m unittest discover -s tools -p test_raptor_bird_glb.py -v
```

The ignored software sheet compares original and replacement fronts, replacement
rears and decoded attack poses. It preserves the swept red/gold ribbons, hooked
masks, complete feather banks and clawed feet of Pidgeot, and Fearow's long open
bill, narrow neck, crown, pale mane and two-sided pale feather terminals. This
is an offscreen geometry review, not production acceptance. No dedicated shiny
checks, grass changes or new camera rules are part of this work.

Fourteen canonical anatomy checks and twelve decoded-skin/API checks pass, including
actual roots over all three clips, exact original neutral bounds, independent
wrists and crests, whole-head coherence, every used control, fixed claws,
positive winding, bounded strain, open mouth clearance and posed fitted front
caps. The original static sculpture fixtures also passed their fourteen checks before
the migration. Three focused Rust checks verify canonical mesh parity, grounded
claws, full animation bounds, authored world flight and GPU activation at the
pack’s original 1.4986/1.1938-metre heights. Shared schema and exact current-cue
checks pass across the expanded registry: 31 scoped checks for this migration.
The background Blender builder reproduces both canonical GLBs byte for byte,
and neutral STL exports match the decoded canonical anatomy. The earlier
front/rear comparison and decoded attack sheets were inspected inline.

Six same-state production pairs (front, world and battle for each bird) and
both moving world frames were inspected inline, entirely headless, offscreen
and muted. Close views retain fitted facial detail, feather layering, complete
claws and individual crests. Battle views show PIDGEOT/PIDGEY and FEAROW/PIDGEY
in the actual HUD, with complete wings and planted claws at original size.
The world retains intentional Fly clearance. Wide side views overlap feather
banks and lose facial detail; the sampled world position also crosses existing
occlusion dithering and NPC overlap. Fearow’s bill is foreshortened head-on
and clear in profile and battle.

Route 29 initially rejected multiple Fearow compositions before the existing
search reached a valid source-supported stage. No geometry, scale, FOV or
terrain-admission guard was relaxed. All six captures preserve authoritative
state, and an independent audit confirms both canonical GLB hashes and both
binary SHA256/size/mtime identities remained unchanged across the capture set.
This establishes native model review, not browser or complete gameplay acceptance.

Pinned source / packed / GLB SHA256 values:

pidgeot:

- source_sha256: `650e0fcd14ee08b408240ac61d1a4899e51a8af7ef1c9ddae5488f2ac81e9db8`
- packed_sha256: `4290723e8ee8dd56c9e7aba24fc9e65aafe69384cc58f960f2cffdc2f84b7241`
- glb_sha256: `70f0d080e773dc8d9f3591d25656b4ecc78ee5b31b16ad988048d0a03dbf661c`

fearow:

- source_sha256: `125c538daa39ea587402505d86a03354672e61acdfd6711a75128f25bc7367b1`
- packed_sha256: `9d317f1778d94a75d2c959dd71d7eacfe45293d7171419b9782fdc04ea114f1a`
- glb_sha256: `69863ca0261ce3bac978e206eab0a45fadd7008fa83f08d23926c71d05886c0e`
