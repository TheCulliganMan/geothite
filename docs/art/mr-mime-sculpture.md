# Mr. Mime sculpture

The canonical `mr__mime.mesh.json` uses original geometry from
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
