# Cubone sculpture

The canonical `cubone.mesh.json` uses original geometry from
`tools/cubone_sculpt.py`. Its 45 named closed parts contain 94,992 triangles,
with a grounded floor and normalized height 1.00. SHA-256:
`d89f8384a715d2b5d4688dfaa71151b639ffaf324615244cbed72a53b9a05685`.

A compact tan body, broad cream belly, heavy thighs and grounded paws sit
beneath an oversized white skull helmet. The helmet has two physically
recessed orbits with fitted tan linings, angular white eyes, dark pupils and
small glints. Its broad snout carries recessed triangular nostrils; curved
outward horns and pointed cheek projections complete the skull silhouette.
These are closed recessed surfaces. A completely hollow removable helmet
or through-holes are not claimed.

Rounded shoulders join bent arms and shaped palms. Three curved tan digits
on each hand and a separate gripping thumb retain actual roots. The bone
shaft intersects the holding palm/thumb and has four rooted bifurcated end
lobes. The paws have rounded white toe claws, and a short pointed tail joins
the torso. The juvenile torso was shortened after visual review, with inverse
scaling of its normals, preserving the larger helmet proportions.

The first candidate failed an uncrossed-finger check and the existing
100,000-triangle budget. A wider tapered finger bend and less unnecessary
socket contour sampling fixed those failures. Fitted belly/socket shading
still showed radial seams; increasing the crease angle had no visible effect.
Local cap normals now follow the actual parent mesh's interpolated normals,
with face-supported sidewalls and outward cap shading at steep recess edges.
The first interpolation failed an outward-normal check, which was corrected
without relaxing the regression. Renderer code and unrelated models are
unchanged.

All thirteen final candidate checks passed in 36.772 seconds. They cover
recipe parity, finite closed outward surfaces, face-aligned normals, budget/
height/floor and named anatomy, actual roots for neck/head/skull/snout/horns/
cheeks/shoulders/arms/hands/digits/thumb/haunches/paws/claws/tail, the actual
club grip and four rooted lobes, fitted vertices and face/edge clearances,
uncrossed curved appendages, sixteen empty orbital/nostril recess probes,
filled-recess negatives and detached appendage/club negatives.

Final oblique, front and rear software renders each exited 0 at 800×824 and
were individually inspected after the final normal correction. Installed bytes
match that candidate. The pure builder has a dedicated Cubone dispatch through
the existing exact Rust species route. The legacy function rejects both identities now that Marowak also has a
separate original adult recipe.

All thirteen installed canonical checks passed in 33.094 seconds. The fresh
native build exited 0, and a final-hash GPU battle capture with explicit
`CRYSTAL_PREVIEW_MUTE=1` exited 0 at 2000×1400 and was inspected in full.
The frozen renderer executable was `(217792530, 207350416,
1791812562137305949)` (inode, bytes, modification nanoseconds), contained the
full canonical SHA above and the mute switch, and stayed stable through the
capture. Cubone is small at its actual battle presentation size, with the
skull, held bone, tan body and grounded paws visible in the arena.

The union-feature Rust integration exited 0 with the exact all-species
model check passing in 3.10 seconds. A direct exact rerun passed in 3.41
seconds against frozen executable `(217798650, 229537600,
1791812807399036813)`, which contained the final Cubone hash. Both executable
identity and canonical source hash stayed stable through the rerun. This
checks model loading and bounded geometry; it is not approval of every
species' art. Software views and the image-target battle capture do not
establish ordinary live-window, browser, animation or gameplay behavior.
The later Marowak integration also passed with both final hashes in its
frozen executable. All bone-pair verification jobs are terminal, and
task-owned ignored scratch was removed after the final proofs.

The [source review](bone-pair-source-review.md) records official anatomy and
specific external model/rig candidates. No external archive, mesh, texture,
rig or clip was retrieved or imported. This is original static geometry.


## Articulated canonical model

The static JSON is replaced by a single canonical GLB authored by
`tools/bone_keeper_glb.py`. The original recipe remains editable, and both its
CLI and the Blender builder export the same canonical skin. No external mesh,
rig or animation was imported. The strict reader restores the original named
logical parts and rejects altered geometry or binary data.

The 14-joint hierarchy has an identity floor root, body, neck and skull,
independent left/right upper arms, elbows and wrists, supported thighs, and
two tail carriers. The whole skull, horns, cheeks, snout and fitted recessed
gaze stay rigid together. Each palm, three fingers and hand claw follows its
wrist; the gripping thumb, shaft and four club lobes follow the left wrist.
The club stays rigid and held through every clip. Continuous projected arm
weights avoid nearest-segment discontinuities at bent elbows. Body/belly,
neck, haunch and tail transitions blend; battle feet and toe claws stay fixed.
The world renderer supplies the existing two complete walking support chains.

Idle lasts 3.2 seconds with 81 keys; attack and hit use 65 keys over one second.
All endpoints return to neutral and production source cues own action timing.
Adaptive fitted contours replace redundant uniform face subdivisions while
preserving outlines and physically recessed surfaces. Rounded body/limb
sampling fits the existing 20,000-vertex, 40,000-triangle, 64-part, 256-accessor
and 1 MiB limits without changing authoring or pack height.

Front/side/rear old, idle, attack and hit sheets were inspected. The oversized
juvenile helmet and adult narrow muzzle/raised club remain distinct, the fitted
gaze stays coherent, and fingers remain wrapped around their actual clubs.
Blender exports match canonical bytes. Shiny checks are skipped by request;
grass height is untouched. Final runtime verification is recorded below when
complete; browser visual acceptance remains open.

Current asset: 19,943 vertices, 34,972 triangles, all 45 named parts,
14 joints, 209 accessors, 772,068 bytes, authoring height 1.00.
Neutral geometry pin: `5bd078b289b719850ffbd5621c43b25276736ac2bc919b1e60460c9e84814d77`.

The first reduction failed the existing nostril free-space check; the snout
retains its original 64×40 sampling so both triangular recesses stay clear.

All 32 Python checks pass: 13 Cubone anatomy checks (18.519 seconds),
13 Marowak anatomy checks (17.956 seconds), and six stored skin checks
(67.347 seconds). They cover exact canonical geometry and budgets, closed
solids and actual normals, fitted face clearance, empty recessed spaces,
rooted anatomy and gripping fingers, full hierarchy/weights/binds, independent
carriers, actual occupied joins through clips, fixed feet/claws, bounded edge
strain and winding, source-clock endpoints and mutation rejection. The first
independent-carrier check caught imperceptible Cubone thigh articulation;
the supported thigh yaw was increased while keeping all feet fixed.

The Rust pack reader confirms CUBONE = 104 / LONELY = 0.4064 m and
MAROWAK = 303 / BONEKEEPER = 0.9906 m. Four focused Rust checks passed
in 2.20 seconds: both pack-size GPU activations, fixed battle supports with
club/wrist ownership and bounds, and 64 phases of two-foot world walking
with the held club's authored weights preserved. Native paired battle review is recorded below.

Native Cubone/Sentret Route 29 3D/2D pair inspected: frame 3283, checksum
1716862610, LCD frame 3963, virtual time 73.682151458. The original 0.4064 m
size, oversized smooth skull, recessed eye/nostrils, rooted feet, curved fingers
and held club remain readable. Correct CUBONE HUD; no fallback or shader error.
Normal terrain admission passed at the existing -110-degree review yaw with
support, omission, whole-body and frustum gates intact. State was preserved.
Ignored evidence: `target/pokemon-native-review/cubone-normal`.

Both current canonical compressed GLB payloads were found in the native
executable. World first-person/elevated and browser visual acceptance remain
open; shiny checks are skipped and grass height remains untouched.


The shared all-species grounded/volumetric geometry and exact production
source-cue timing checks both pass against the current 198-entry registry
(2 tests, 124.73 seconds). Fresh battle and world review executables build
successfully; all four current canonical compressed payloads are present in
the battle executable. Multi-view native review continues below.


World QA stack diagnosis: LLDB stopped in
`GameDataSet::apply_runtime_mutation_command_with_checksum` at its stack probe.
The unoptimized native QA override reserves 0x1b1000 + 0xf00 = 1,777,408 bytes
(about 1.70 MiB) in that function, before its caller frames. This is the
Compute Task Pool overflow, not an observed mesh/bind failure. Native world
review was rebuilt with the repository's normal optimized `crystal-assets`
profile (opt-level 2), retaining the same geometry, sizes, controller inputs and
cameras. No runtime source or thread-stack limit was changed. World review
remains unaccepted until the actual paired captures pass. Diagnostic evidence:
`target/pokemon-native-review/cubone-world-debug.log` and
`target/pokemon-native-review/world-stack-disassembly.log`.


The normal optimized-assets build resolves the QA stack overflow. The initial
New Bark rerun reached the authored exit dialogue, so its empty world view was
rejected. The clear Route36 (20,8), noon location now provides preserved-state
paired source/native views without walking into scripts. First-person Cubone
(frame 1457, checksum 1163064811, time 24.757580166) shows the full front at
authored size: smooth skull, recessed eyes/nostrils, curved hands, attached club
and supported feet. The elevated side view separates its complete silhouette
from the trainer at gameplay scale. No geometry, runtime support or admission
settings changed for these captures. Evidence: `cubone-world-first` and
`cubone-world-elevated` under the ignored native-review directory. These are
stationary world checks; browser acceptance remains open.
