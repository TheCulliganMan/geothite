# Teddiursa and Ursaring sculptures and standard skins

`tools/bear_sculpt.py` authors the honey cub and ring bear; `tools/bear_glb.py`
exports their canonical standard skins. The species builder emits GLBs and
removes duplicate static JSON. Both share the production species registry,
neutral mesh lookup and generic GPU animation path.

Teddiursa retains 37 parts and 23,880 triangles, with 19,943 vertices instead of
20,152 after exact stored position/normal duplicate removal. Its GLB is
734,596 bytes, with 12 joints and 167 accessors. Digest:
`38a7cdab920c315dac0608ae661e9f9408e9a3b2709000393b88439a3dafbe3b`.
Ursaring retains 48 parts and 27,116 triangles, with 19,016 vertices instead of
27,630. Smooth crease-aware normals on its curved ring and eight fitted ruff
parts replace redundant flat face corners; sharp borders remain. Its GLB is
710,068 bytes, with 12 joints and 207 accessors. Digest:
`3edc0d94b47fe39e7234b958db6c16a2a8254ae548ad21262b4d5994473260a0`.
Both meet the existing 20,000-vertex, 256-accessor and one-MiB limits without
relaxing the loader. Every indexed triangle corner and triangle order preserves
exact original f32 positions. All Teddiursa corner normals and Ursaring's other
39 parts also preserve their exact original f32 normals. No feature is removed.

Actual local normal front/back sprites and RGB555 palettes were inspected.
Teddiursa uses coat (181,115,49), darker ears/tongue (115,57,33), white lunar
mark/muzzle/claws and black facial details. Ursaring uses brown (165,90,0),
yellow (255,255,49) on its ring, muzzle and ear centers, plus white and black.
These replace the earlier invented cream colors. Historical part labels retain
cream/ivory wording. Pack height 200 gives Teddiursa 0.6096 m; 511 gives
Ursaring 1.8034 m. Both retain the normalized 1.14 height and floor origin;
production scaling is uniform.

Each rig has a torso, coherent head, two arms and wrists, tail, paired ears and
two leg middles above an identity root. Feet and all six foot claws remain
fixed. Shins blend between those supports, middle-leg yaw and torso yaw.
Teddiursa's left arm/wrist inherit the head transform, preserving the honey
paw's exact muzzle contact while its shoulder transitions into the body.
Ursaring's chest, neck and fitted shoulder fur use a common head transition;
its annular ring stays rigid to the torso, preserving its real central opening.
The cub's forehead crescent stays on the head, with its original open gap.
Fitted eye, ear and mouth layers share their supporting owners; hand claws
share their palms. Idle, attack and hit clips use rotations only, neutral
endpoints and authoritative battle-cue timing. Wrists settle after shoulders.

All seven stored-skin and nine migrated anatomy checks pass, covering canonical
reconstruction, bind identity, budgets, source colors, honey contact, planted
paws, feature ownership, winding and edge strain, physical joins, fitted marks,
real lunar openings and adult shoulder-fur support. Both focused Rust checks pass: production GPU skinning at pack sizes and
off-grid fixed paws, honey contact, rigid feature ownership and culling bounds.
Final focused Blender outputs are byte-identical and emit no static JSON.
Front/rear neutral software views were inspected. Matched normal native 3D/2D reviews against Pidgey pass for both bears.
The cub's crescent, held paw and planted feet read clearly; the adult's scale,
annular ring, claws and grounded paws remain clear against the retained grass.
The layered ruff retains its sculpted sharp borders. Rear details were also
inspected in neutral software views. Teddiursa preserves core frame 2596,
hash 1244057618, LCD frame 3287 and time 59.617404872. Ursaring preserves
core frame 2446, hash 1622029825, LCD frame 3230 and time 58.524571168.
Both paired receipts report `state_preserved=true`. Captures remain ignored
local QA artifacts.
Exact sprite-frame motion, field gaits and browser review remain open. Shiny
checks/captures remain skipped at the user's request.

The wider `every_exact_species_is_grounded_and_volumetric` and
`body_pose_uses_elapsed_idle_time_and_exact_current_attack_progress` regressions
also pass. These cover catalog geometry and elapsed-pose sampling, without
claiming visual acceptance for unrelated models. The native build passes.
