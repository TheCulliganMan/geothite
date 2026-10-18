# Doduo and Dodrio: feathered runners with independent necks and hinged bills

Both canonical GLBs retain the original continuous scalloped bodies and heads,
individually curved long necks, distinct head heights/yaws/pitches, tapered
upper and lower bills, long shanks, three forward toes and one rear toe per
foot, and rooted claws. Doduo has two rounded heads, dark necks/shanks, black
oval eyes with pale glints and long pale bills. Dodrio has three heads, angular
white eyes with visible pupils, open bills with tongues, a dark lower feather
apron, six dark crown quills and three broad rounded fan feathers.

The normal local-pack front sprites and RGB555 palettes were inspected.
Doduo uses white/black plus (21,13,9)/(12,8,6); Dodrio uses white/black plus
(26,16,8)/(18,8,6). Decoded material checks verify their expanded linear
colors. Doduo keeps its warm brown pelt, dark necks/legs and pale beak/claws;
Dodrio has a darker reddish brown pelt, warm necks/shanks/bills, black apron/
crests and pale fan. Original pack heights remain 407 (1.397 m) and 511
(1.8034 m). Both original authoring heights remain 1.22, grounded at zero.
Uniform physical scaling, full animated bounds and admission rules remain.

Excess tessellation is reduced without removing anatomy. Heads retain their
scalloped feather displacement; the eyes are closed layers clipped to the
actual stored head triangles instead of densely subdivided approximate
surfaces. Dodrio pupils are wider to remain readable. The old 25,330/27,635
vertices and 42,084/46,896 triangles become:

| Asset | Vertices | Triangles | Logical/runtime parts | Joints | Accessors | Bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Doduo | 11,294 | 18,988 | 37/19 | 12 | 114 | 476,232 |
| Dodrio | 19,801 | 33,196 | 61/32 | 19 | 178 | 786,324 |

Coherent material packing combines each grounded foot's toes/claws and each
head's gaze/crest/cavity details. All logical anatomy ranges survive, with
exact source f32 positions, normals, colors and triangles. Existing 20,000-
vertex, 40,000-triangle, 256-accessor, 64-joint/primitive and 1 MiB limits hold.
The shared asset reader, strict checker, builder, bounded registry and GPU
renderers consume canonical GLBs; legacy JSON assets are removed.

Each bird has a fixed floor, body, two supporting shins, and a four-control
chain per head: curved neck base, neck tip, coherent head and lower-bill hinge.
Dodrio adds three independent fan controls. Faces, eyes, upper bills and
crests stay coherent with their own head; lower bills and tongues share their
jaw. Hinges follow the actual yaw-turned head's lateral axis. Neck fields
blend body/base/tip/head along the actual smoothed source path. Shanks blend
body/knee/floor; all toes and claws remain fixed in battle clips. World motion
constructs two complete support chains and retains canonical non-leg bindings.
Idle/attack/hit rotate only, use source-relative cues and have neutral endpoints.

Doduo source/packed/GLB SHA256:
`dda64c1a1a3292d97538afe7e08c98e591dc4b0d8739221ccccbeb3111e22b84`,
`94090facb05decd9d763ba5fc85c0ad67866d7b05745eb98dcae903ab934f190`,
`6126c840f0758d5a87fc6f9ff1a9be19d0e6cc0eecc64f73193739288258b1c9`.

Dodrio source/packed/GLB SHA256:
`8efaedc8f7b805e8f752e03fdb99c95e03755cca21066e316ca8eca9250b27d5`,
`21ca4a7aeec485dac57a7c5f2a54692259c76b131b22661383cd2e84ef7dd811`,
`8dc8aa14c34c0267d816e1f1987eab24a5723ba16724c24712bd8a28344b4387`.

Twenty Python checks pass in 16.278 seconds: ten stored-anatomy and ten
independently decoded skin checks. They cover closed outward solids and
shading, regeneration/grounded dimensions/existing budgets, anatomy counts,
real occupied roots at necks/heads/bills/crests/fan/apron/legs/feet/toes/claws,
fitted local-head eye vertices and actual front clearance, curved-solid
self-intersections, detached negative controls, exact logical packing,
hierarchy/inverse binds/used independent controls, coherent separate heads,
yaw-aligned jaw hinges without dragging other geometry, fixed clawed feet,
posed roots, positive winding and bounded strain, normal palettes, neutral
endpoints/source cues and mutated data rejection.

Three focused Rust checks pass in 2.25 seconds for canonical geometry, fixed
clawed feet, original-pack-size GPU admission and two complete walking
supports with actual floor contact and full animated bounds. CLI and
background exports match both canonical GLBs byte for byte. Farfetch'd's
f32 geometry remains unchanged through the shared feather helper. Shared
schema and exact elapsed-idle/current-attack-cue checks pass in 105.16/105.28
seconds, bringing the batch to 25 scoped checks. Native clients build in
61 seconds. The decoded before/after, front/rear/side and attack sheet was
inspected inline. Production front pairs at EcruteakCity (20,21) complete in
38.371/37.029 seconds for Doduo/Dodrio. Both were inspected inline: complete
original-size models, readable separate heads/gaze/crests and grounded claws.
Doduo's raised bill underside is very dark in the fixed lighting; its material
is the checked normal pale palette and its complete silhouette remains visible.
World/battle pairs complete in 34.919/67.514 seconds for Doduo and
36.476/64.926 seconds for Dodrio. All six pairs and both moving world frames
were inspected inline. World models and walking supports are complete and
grounded, though heads overlap from the side angle and facial detail is small
at the wide camera distance. Battle views separate the heads and show readable
gaze/bills, grounded claws and actual DODUO/PIDGEY and DODRIO/PIDGEY names,
with no fallback. Doduo's raised bill is clearer in battle lighting.
Both groups have complete same-state receipts and identical binary SHA256/
size/mtime identities matching current executables; canonical GLB hashes
remain unchanged. Rendering stays headless, offscreen and muted. No shiny,
grass-height, scenery, AA, audio, content-pack or gameplay edits belong to
this batch. No full-suite
or gameplay completion is claimed. Registry reaches 268 skin variants and
seven static species files. Pidgeotto also retains an older wing-node GLB
schema outside the standard skin registry; its complete rig/cue upgrade is
still required. Full-roster model and rig work remains active.

Rebuild into ignored output:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 tools/dodo_sculpt.py \
  --output target/dodo-rebuild
/Applications/Blender.app/Contents/MacOS/Blender --background --threads 2 \
  --python tools/build-battle-species.py -- \
  target/dodo-rebuild --only DODUO,DODRIO
```
