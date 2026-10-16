# Miltank and Tauros sculptures and standard skins

`tools/bovine_sculpt.py` authors the cow and three-tailed bull, and
`tools/bovine_glb.py` exports canonical standard skins. The species builder
explicitly emits these GLBs and removes duplicate static JSON. Production
neutral mesh lookup and GPU animation share the same geometry.

Miltank retains all 45 parts, 17,281 vertices and 17,976 triangles. Its GLB is
651,592 bytes with 14 joints and 210 accessors. Digest:
`cab41187fb7632ada815e8d5ef241e6ed3e56af5ba964d6195e44cf5a09e851c`.
Tauros retains all 42 parts, 14,025 vertices and 16,504 triangles. Its GLB is
567,740 bytes with 17 joints and 194 accessors. Digest:
`9c7a80af7788de4b71e74f6217e1abc0eced54d06e9a6280dbd6fabd57da2edc`.
Every original f32 position, normal and triangle order remains exact. Both fit
the existing skin loader budgets without simplification or relaxed validation.

Actual local normal front/back sprites and RGB555 palettes were inspected.
Miltank uses pink (222,123,148), navy (49,49,107), white and black. Its tail
shaft is pink and its udder white; eyes, hood, cloven hooves and rear markings
use the source palette. Tauros uses gold (206,123,41), brown (82,57,33), white
and black, replacing invented gray mane/horns/hooves. Whips are gold and tufts
brown. Historical part labels retain cream/blue/gray wording.

Pack heights 311 and 407 give uniform production references of 1.1938 m and
1.397 m. Normalized full silhouette heights remain 1.12 and 1.28. Tauros's
body reference is 1.1175279, measured from ground to its highest horn point;
raised tail tufts remain in full fit/culling bounds. The shared layout applies
the pack height to this horn/body reference instead of the raised tail, keeping
the bull's body from shrinking. Miltank uses its full standing height.

An identity root fixes all hooves and fitted cloven grooves. Leg middles yaw
above those supports and blend into the torso. Miltank's body, udder, four
teats and rear spots share one owner; the fitted face and horns share the head.
Its arms blend through wrists to coherent forehooves/grooves, paired ears carry
their fitted inner panels, and a three-stage tail carries the terminal ball. Tauros's
mane and fitted strands share a mane owner; facial layers, horns and all six
forehead rivet layers remain rigid to the head. Each of three tails has its own
base, bend and tuft joint, with shared positional weights near the shaft/tuft
join. Idle, attack and hit use rotations only, neutral endpoints and
source-battle-cue timing. Delayed tail stages have distinct branch motion.

Original left-to-right patch centroids keep native and Blender geometry bytes
identical across Python versions. Seven stored-skin and eight migrated anatomy
checks pass, including canonical reconstruction, bind identity, budgets, source
colors, coherent feature ownership, independent tail chains, planted hooves,
winding and edge strain, real contacts and fitted markings. The additional
Tauros body-reference regression and final canonical roundtrip also pass after
the metadata-only reference adjustment. Front/rear software views were
inspected. Focused Blender outputs are byte-identical with no static JSON.
Both focused production Rust checks pass: GPU skinning at pack sizes, Tauros
body-reference scaling, coherent feature ownership, fixed hooves, tail motion
and off-grid culling bounds. Matched normal native 3D/2D reviews pass.
Exact sprite-frame motion, field gaits and browser review remain open. Shiny
checks/captures remain skipped at the user's request.

The wider `every_exact_species_is_grounded_and_volumetric` and
`body_pose_uses_elapsed_idle_time_and_exact_current_attack_progress` regressions
also pass. These cover catalog geometry and elapsed-pose sampling, without
claiming visual acceptance for unrelated models. The native client build passes.

Tauros's matched normal 3D/2D review against Pidgey passes: grounded cloven
hooves, the mane and all three tail shafts/tufts are visible at the corrected
body scale. The far horn and some forehead details are obscured at this angle;
front/rear software views cover those details. The paired receipt preserves
core frame 2102, hash 1577348228, LCD frame 2719 and time 48.875408417
(`state_preserved=true`). Miltank's first view exposed incorrect white inner
ears. A magnified actual source frame shows dark interiors; both inner panels
now use black. The binary geometry/weights, nodes, skins, clips, accessors and
materials are identical; only two primitive material assignments and the source
hash changed. Final canonical/palette checks pass.

Miltank's final matched normal 3D/2D review against Pidgey passes. The dark
inner ears now match the source; planted hooves, raised forehooves, udder and
tail remain coherent. The paired receipt preserves core frame 2044, hash
873632486, LCD frame 2639 and time 46.967535708 (`state_preserved=true`).
Both focused Rust checks and the native rebuild pass after the material-only
correction; the final Blender export is byte-identical. Shiny checks remain
skipped at the user's request.
