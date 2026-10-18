# Magmar sculpture and skin

The canonical asset is `crates/crystal-voxel-view/models/battle_species/magmar.glb`.
`tools/magmar_sculpt.py` retains the original pure geometry recipe and
`tools/magmar_glb.py` authors its standard glTF skin and strict reader.
The retired static JSON asset is removed. No external geometry, rigs or
animation clips are imported.

Magmar retains its orange pear body, fitted yellow flame-shaped belly,
dark blue collar and shins, broad twin forehead lobes, narrow gaze, genuinely
open cream bill, tongue, toothed orange cuffs, ivory hand and toe claws,
three dorsal points and long yellow tail with layered fire. Adaptive fitted
patches replace uniform dense face subdivision. Round sampling is
concentrated in the torso, brows, joints and supported feet; planar flame
banks retain their curved outlines without unnecessary interior subdivision.
Fitted gaze and flame layers inherit their actual parents' smooth corner
normals. The belly follows the body's continuous curvature field, with
intentional cuff teeth and bill edges preserved.

| Vertices | Triangles | Logical / runtime parts | Joints | Accessors | Bytes |
|---:|---:|---:|---:|---:|---:|
| 19,923 | 30,320 | 55 / 39 | 17 | 210 | 811,876 |

The original static asset used 46,777 vertices and 77,108 triangles. All
55 named source solids remain. Lossless material grouping retains their
exact source order, geometry, normals and per-vertex weights. Existing
20,000 vertex, 40,000 triangle, 64 runtime part, 256 accessor and 1 MiB
limits remain unchanged.

The seventeen joints are root, body, neck, head, two shoulder/elbow/wrist
chains, two supported legs, two crown carriers, tail base, tail tip and
terminal fire. Bill, tongue, forehead lobes and fitted gaze share a coherent
head owner, preserving the real mouth opening. Cuffs use the same continuous
weight field as their forelimbs. Hands and their claws follow the matching
wrist. Complete feet and toe claws stay fixed in battle; the world walking
adapter supplies two complete support chains.

Each thin flame bank and its fitted front/back glow layers share one carrier.
An initial differential bend folded a narrow tail-flame rim, caught by actual
stored-triangle winding checks. Coherent carrier rotations preserve each
bank's curved silhouette and thickness while articulating the twin crown and
tail fire independently. The supporting tail bends through its own two
carriers. Idle lasts 3.2 seconds with 81 keys; attack and hit each last one
source-relative second with 65 keys. Endpoints are neutral and production
battle cues own playback time.

Logical geometry pin:
`beed80700a65f3864fa4e3facfffb0c8d6e4aab03e78ec41e1e0228f53b46d50`.
Packed geometry pin:
`3463aa43d0d0b9d5d979c0a55071283a3ef1a1893d053a2ffa97e5361dd0c83d`.

All twenty final Python checks pass together (53.445 s): eight independently
decoded stored-skin checks and twelve stored-anatomy checks. They cover exact
source and packed geometry, hierarchy, inverse binds, budgets, rigid owners,
independent articulation, floor support, edge strain, triangle winding,
occupied anatomical roots through clips, uncrossed curved surfaces, fitted
layer clearance and real mouth space against every posed solid in neutral
head space. Negative controls reject detached flames/cuffs/claws, a filled
bill and changed geometry or bytes.

The front/side/rear comparison and idle/attack/hit sheets preserve the
silhouette, open bill, rooted cuffs and claws. They are software geometry
review only. Production rig activation, walking and native paired-view
results are recorded below. Browser and moving-world
visual acceptance remain open. Shiny verification is skipped by user
instruction. Grass and renderer AA/filtering remain untouched.


Three final production Rust checks pass (1.37 s): GPU activation at the external
pack's verified 1.2954 m height, rigid bill/claw owners and independent flame
carriers across battle clips, plus 64 world walking phases with two complete
support chains and preserved bill/cuff/flame weights. Final Blender export
matches the canonical bytes exactly. The authored rig registry now has 201
entries.


The initial native close view exposed an angular bill and faint diagonal
belly shading. The final local bill loft uses smooth longitudinal profiles
and 24 rounded cross-section samples, preserving the original opening,
roots and tongue. The belly uses a continuous smooth normal field after
crease processing, constrained to the actual outward face hemisphere.
Smaller shoulder/hand samples keep this selective refinement within the
same budgets. Final anatomy and pose checks above cover this revision.
The earlier 201-entry registry/cue checks passed (244.79 s); the final complete registry
check passes on the new geometry (88.30 s).


Both current canonical compressed payloads are verified in the rebuilt
production examples. Final paired Route36 (20,8), noon first-person/elevated
views preserve state against matching 2D. The rounded bill, real opening,
rooted cuffs/claws, narrow gaze, twin brows and long tail remain coherent.
The full supported body is readable in the elevated view. A paired Route29
battle retains correct MAGMAR/PIDGEY HUD names, both exact models and no
fallback. Original-size terrain admission passes (`accepted=true`); support,
omission and whole-body/frustum admission rules are unchanged.

| Final view | Frame | State checksum | LCD frame | Virtual seconds |
|---|---:|---:|---:|---:|
| Route36 first-person | 2912 | 985710280 | 2912 | 56.720699789 |
| Route36 elevated | 1956 | 513400457 | 1956 | 34.173882375 |
| Route29 battle | 4079 | 3920199925 | 4829 | 104.63901304 |

Final ignored evidence uses the `magmar-first-`, `magmar-elevated-` and
`magmar-battle-` prefixes under `target/pokemon-native-review/`. The bill
rounding is accepted in native close view. Faint diagonal belly shadow bands
remain at close range despite the continuous normal field; surface polish
there remains open. Browser and moving-world visual acceptance also remain
open. This is not full visual acceptance of the model. Shiny checks remain
skipped, and grass/AA/filtering are untouched.


The [original fire-pair source review](fire-pair-source-review.md) records
inspected official art and CC BY 4.0 candidates whose archive endpoints
returned 401; no external geometry was imported. The retired static mesh's
raw JSON SHA-256 was
`956ada452f260111200c47656c20ab2e2b4ee655c5f84cdaa5d750b0cc038265`.
Its earlier twelve anatomy checks and all-251 static-model/native image-target
checks are historical evidence for that source revision. They do not replace
the skin, final geometry, production-controller or visual evidence above.
