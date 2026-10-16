# Houndour and Houndoom sculptures and standard skins

`tools/hellhound_sculpt.py` authors the bone-armored dogs, and
`tools/hellhound_glb.py` exports canonical standard skins. The species builder
now explicitly dispatches both GLBs and removes duplicate static JSON.
Production mesh lookup and GPU skinning share the same neutral geometry.

Houndour retains 44 parts and 14,248 triangles, with 11,024 vertices instead of
16,338. Its GLB is 464,124 bytes with 13 joints and 181 accessors. Its source
and packed digest is
`f35d0751435b77708a37e8d0c8f5ece071f3de0b65c8fb2586c3775b5d9c1ce6`.
Houndoom retains 67 source parts and 22,736 triangles, with 15,865 vertices
instead of 23,390. Each paw's three claws share a same-material, fixed-root
runtime primitive, yielding 59 primitives. Root-level source ranges recover
all twelve individually named closed claws and original source ordering.
Primitive extras retain the loader's strict `part` schema. The GLB is
634,844 bytes with 12 joints and 218 accessors. Source digest:
`4fe7c0e7bec59d663b355075b40d4ecea6f9ee20620fa2a9da28d1145fb15a80`.
Separately pinned packed-runtime digest:
`2d2d89e4bd3ce410dfbbeee0d49659d1c6b78e4f9d0e143e14fa02b54bfde878`.

Crease-aware smoothing on the curved dorsal plates and chest markings reduces
redundant flat corners and removes rough shading on the puppy's white plates.
Every original indexed triangle position and triangle order is preserved.
The other 41 puppy parts and 63 adult parts also preserve original f32 corner
normals. The existing 20,000-vertex, 64-primitive, 256-accessor and one-MiB
limits remain unchanged. Explicit original left-to-right patch centroids keep
native and Blender signed-zero geometry bytes identical.

Actual local normal front/back sprites and RGB555 palettes were inspected.
Houndour uses coat (49,66,82), orange (255,99,0), white and black, including
orange puppy paws. Houndoom uses dark purple coat (74,33,140), the same orange,
white and black. Horns and bone plates use white; orange replaces invented red
irises and tongue colors. Historical part names retain ivory/red/pink wording.
Pack heights 200 and 407 give 0.6096 m and 1.397 m uniform scales. Both preserve
the normalized 1.20 height and floor origin.

An identity root fixes all four paws, fitted toe separations, ankle cuffs and
adult claws. Leg middles yaw above the cuffs and blend into the yawing torso.
Dorsal armor shares the torso owner. Chest markings share the neck's transition
field; the adult bone collar, skull pendant and its fitted eyes stay together
on the neck owner. Facial layers and adult horns remain rigid to the head.
The adult jaw and tongue articulate together; upper canines stay on the head.
Three tail stages carry the tapered puppy tail or long adult curl and coherent
terminal spade. Puppy ears articulate separately. Idle, attack and hit use
rotations only, neutral endpoints and authoritative source-cue timing.

All seven stored-skin and eight migrated anatomy checks pass. They cover
canonical reconstruction, bind identity, budgets, source colors, exact claw
reconstruction, fixed paw/cuff support, coherent feature ownership, winding and
edge strain, closed geometry, fitted surfaces and physical armor/appendage
contacts. Front/rear software views were inspected; the corrected smooth front
view was inspected again. Final focused Blender exports are byte-identical
with no duplicate static JSON. Both focused production Rust checks pass: GPU skinning at pack sizes and
off-grid fixed paws/cuffs, rigid feature ownership, tail movement and culling
bounds. Matched normal native 3D/2D reviews against Pidgey pass for both dogs.
Houndour's corrected smooth back plates, forehead shield, orange paws and
ankle bands read clearly. Houndoom's horns, skull pendant, dorsal plates,
claws and complete curling tail/spade are visible with grounded paw placement.
Houndour preserves core frame 4301, hash 4159166349, LCD frame 5193 and time
103.78049458. Houndoom preserves core frame 5845, hash 1331536335, LCD frame
6684 and time 145.51494033. Both paired receipts report `state_preserved=true`.
Captures remain ignored local QA artifacts. Exact sprite-frame motion, field gaits and browser
review remain open. Shiny checks/captures remain skipped at the user's request.

The wider `every_exact_species_is_grounded_and_volumetric` and
`body_pose_uses_elapsed_idle_time_and_exact_current_attack_progress` regressions
also pass. They cover catalog geometry and elapsed-pose sampling, without
claiming visual acceptance for unrelated models. The native client build passes.
