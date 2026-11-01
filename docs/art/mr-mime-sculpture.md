# Mr. Mime sculpture

The current walking support packing and its revised runtime counts/pins are
recorded in [Bilateral walking support packing](world-support-packing.md).
The original logical sculpture, battle weights and clips are unchanged; the
earlier runtime packing and capture record below remains historical evidence.

The current canonical asset is `mr__mime.glb`, built from
`tools/mr_mime_sculpt.py` and `tools/mr_mime_glb.py`. Its 67 original logical
solids pack losslessly into 32 runtime primitives: 19,663 vertices, 33,212
triangles, 26 joints, 218 accessors and 917,600 bytes. This replaces the static
mesh's 63,305 vertices and 105,964 triangles while retaining the existing
20,000-vertex, 40,000-triangle, 64-part, 256-accessor and 1 MiB limits.
Normalized height remains 1.20; production physical scaling is unchanged.

Focused round sampling and removal of redundant coplanar fan subdivisions
retain the cream torso, red shoulder/hip orbs, pale pink head and thin bent
limbs, white gloves and knees, scalloped blue fans and curled blue slippers.
The original asymmetry remains: one glove is raised above the head, while
the other reaches forward. Every glove retains five separate rooted digits,
rounded tip bulbs and fitted rosy pads. Palms remain separate closed glove
surfaces at the intentional skin/glove color boundary.

Body, neck and head, separate shoulder/elbow/wrist chains, hips and knees,
two complete fan carriers and ten independent whole-digit carriers animate
the model. Each digit carries its complete finger, fingertip and fitted pad.
The recessed smile, gaze, cheeks, lips and tongue share the head transform;
complete slippers and curled toes stay fixed to the floor in battle. Idle
is 3.2 seconds; attack and hit are one-second rotation-only source-cue clips.

The closed head retains a physical oral recess and empty smile probes.
The concave lining clips the actual stored front head triangles to its
outline, with a 1 mm source-space offset and buried closed back. It follows
the real depression without bridging it with raised chords. Stored rays
limit exposed lining standoff to 0.1% of body height. A hollow head or
through-hole is not claimed.

The logical stored-buffer geometry pin is
`120668b1f7eb02e42729ae3939a5c4e3ff1258f6ec57b837aec6f3fda6f1a8e9`;
the packed geometry pin is
`11ae6ca7c4535a95e597b2a80209a6a48c24daffee011bdd4cb0a09a744fe401`.
Background Blender dispatch reproduces the exact canonical bytes.

All 22 Python anatomy/skin checks pass in 34.325 seconds. They decode stored
positions, normals, indices, skin weights, hierarchy, inverse binds, clips
and source-part ranges; prove every non-root carrier has independent motion;
retain real anatomical roots and uncrossed appendages through all clips;
check fitted vertices, edges and face centers against posed parents; keep
the smile probes empty in the moving head frame; and verify winding,
edge strain, floor contact, budgets and strict reader mutation rejection.
All three production Rust GPU, battle-support and two-slipper walking checks
pass in 1.29 seconds. Six neutral/idle/attack/hit software views were
inspected without opening any application window. Both shared registry and
source-cue regressions pass in 107.81 and 123.35 seconds respectively.

The fresh native build completes in 1m 17s. Three final production captures
run entirely offscreen with sound muted, explicit headless/image-target
controls, and verified executable identity and exact embedded compressed GLB.
Route36 first-person and elevated views and a Route29 walking encounter
were inspected alongside each locked 2D state. The close view keeps the
complete silhouette in frame; nearby foliage obscures part of the raised
glove. The elevated side view seats both slippers beside the trainer. The
battle gives an unobstructed view of both gloves, scalloped fans, face and
curled slippers, with named MR.MIME/PIDGEY HUDs and no model fallback.

Terrain admission succeeds after 181 candidates (83 unsupported), using the
original 1.2954 m pack height and zero support offset. The whole authored
body remains subject to the existing support, frustum and omission rules;
no scale or admission workaround is added. Close mouth-rim shadow aliasing
remains open for shading polish. Browser and moving-world visual review
remain open. Shiny checks are skipped; this batch leaves grass height, AA
and filtering to their existing owners.

| Capture | State frame/hash | LCD frame | Virtual seconds |
| --- | --- | --- | --- |
| Route36 first-person | 2563 / 3292923686 | 2563 | 43.890241209 |
| Route36 elevated | 1632 / 224961682 | 1632 | 27.748842042 |
| Route29 battle | 3358 / 3725971453 | 4006 | 70.165551668 |

Every pair records `state_preserved=true`. The native screenshot processes
complete in 53.06, 32.65 and 89.97 seconds. The exact final asset SHA-256 is
`9c3c971ee7fadc924bd5c5adf8f30a9e54c0bbc7db9c58c294b204531413fa20`.
The ordinary walking-intro frames are not evidence of GPU animation
playback; the stored-pose and Rust GPU checks supply that narrower proof.

## Original static sculpture history

The retired static `mr__mime.mesh.json` used original geometry from
`tools/mr_mime_sculpt.py`, replacing the pink toy torso, protruding eyes,
rod fingers and triangular head fins. Its 67 named closed parts contain
105,964 triangles, with a grounded floor and normalized height 1.20. SHA-256:
`bd7cada2005a546e8be07f91f46591f13a228bc90c244670a0fb4f2a3fae1718`.

A cream torso carries a fitted rosy chest medallion, rounded red shoulders
and red hip joints. A pale pink neck joins the round head, with small fitted
white eyes, narrow pupils and tiny supported glints. Rosy cheeks follow the
stored face, and two rooted dark blue fans have scalloped outlines. The face
is physically recessed at the smile; a fitted curved dark lining, recessed
oral chamber, restrained upper/lower lips and pink tongue frame an actual
empty opening. A puckered intermediate mouth was rejected during visual
review and replaced by the broader curved smile.

Thin pale pink bent arms hold asymmetrically posed rounded white palms.
Each glove has five independently rooted splayed digits, rounded white tip
bulbs and fitted pink pads. Bent thighs and shins pass through white knee
joints into grounded blue slippers with curved upturned toes. This is
original static geometry; no external rig, clip or animation is claimed.

The first candidate had a gap at its raised wrist and a self-crossed curled
toe. Both meaningful checks failed. Extending the wrist into its actual
palm and widening/tapering the toe corrected them. A later smile change
exposed a detached tongue and exceeded the existing triangle budget. Seating
the tongue against the lower lip and reducing unnecessary contour sampling
fixed those failures without relaxing the checks.

All fourteen final candidate checks passed in 28.866 seconds. They cover
recipe parity, finite closed outward surfaces, face-aligned normals,
height/floor and triangle budget, named anatomy and five digits per hand,
actual roots for head/neck/ears/jaws/tongue/shoulders/arms/palms/digits/tips/
hips/legs/knees/shoes/curls, both grounded blue shoes and asymmetric glove
pose, fitted vertices and face/edge clearances, twelve empty mouth probes,
a filled-mouth negative, uncrossed curved appendages and detached negatives.

Final front, oblique and rear software views each exited 0 at 800×824 and
were inspected in full after the final tongue/contour correction. Installed
bytes match that candidate. The pure builder uses a dedicated dispatch and
rejects the old generic humanoid implementation. The existing exact Rust
`MR__MIME` species route remains unchanged.

All fourteen installed canonical checks passed in 27.805 seconds. The fresh native build exited 0. An actual native image-target GPU
battle capture also exited 0 at 2000×1400 and was inspected in full. Mr. Mime,
its cream torso/red joints, asymmetrical gloves, smile and curled shoes
were visible with the opposing Pidgeotto, arena floor and battle HUD. The
complete final SHA and mute switch were present in the executable, and
`CRYSTAL_PREVIEW_MUTE=1` was set before launch. Executable identity
`(217720953, 206045472, 1791811088427539149)` and canonical model SHA remained
stable across capture. The queued Rust integration job exited 0, running the actual model test once
in 3.20 seconds. A direct final-hash repeat passed once in 3.72 seconds, with
executable identity `(217748493, 226796432, 1791811268977044083)` and the
canonical model SHA stable across the run. This verifies bounded reusable
models, not art quality for every species.
Software checks do not establish live-window, browser, animation or gameplay
behavior. Completed task-owned ignored Mr. Mime verification scratch was removed
after all jobs finished.

The [source review](mr-mime-source-review.md) records official anatomy and
specific external candidates. No external archive, mesh, texture, rig or
clip was retrieved or imported.
