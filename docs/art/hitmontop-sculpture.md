# Original spinning Hitmontop sculpture

The canonical model comes from `tools/hitmontop_sculpt.py`. It replaces the
upright generic brown fighter with a handstand on a grounded tan spin point.
A continuous broad tan helmet supports fitted round eyes and ivory pupils,
side folds and a small mouth. The compact blue body has a fitted tan belly
shield; long curved tan legs end in large blue feet with three rooted ivory
spikes and fitted tan sole pads. Bent arms carry ivory wrist cuffs and round
tan fists. A long curved tail ends in a smaller blue ball and ivory point.
The entire authored anatomy rotates around Z before grounding, preserving
front-facing features and correctly rotating its surface normals.

This is original static geometry. The [source review](hitmon-source-review.md)
records official visual references and the specific downloadable CC BY 4.0
candidate whose download endpoint returned 401. No external rig was imported.

The model contains 35 closed parts and 54,724 triangles, floor zero and height
1.25. Its raw canonical SHA-256 is
`4162267c3f9ed86cbcd0a8727b64c7a12fef95be88080a247033ba6a956b8f1d`.

All eleven candidate mesh checks passed in 12.306 seconds, and all eleven
canonical checks passed in 12.533 seconds. These verify closed connected
outward solids, face normals, recipe parity, budget, anatomy counts and the
actual handstand placement. Occupied-volume intersections attach neck/head,
spin point, both arms/wrists/fists, legs/ankle wraps/feet, all six foot spikes,
and tail/ball/point. Fitted belly, gaze, folds, mouth and sole pads follow
actual parent triangles at vertices, face centers and edge midpoints. Curved
head, limbs and points reject nonadjacent face crossings. Deliberately detached
spin point, tail ball and foot spike negative controls are rejected.

Final front, oblique and rear 800×824 software renders exited zero and were
individually inspected. The rear view exposes the tail curve and three blue
balance ends. The builder dispatches to the pure recipe and rejects its old
generic branch; the existing dedicated Rust registry entry is unchanged.

AST, canonical layout and 575-file raw JSON storage validation passed.

The fresh native build finished with exit zero. Its actual executable contained
all four final Hitmontop, Tyrogue, Jynx and Smoochum SHA-256 fingerprints and
`CRYSTAL_PREVIEW_MUTE`. One unchanged executable (inode 216555973, size
193,759,392, mtime_ns 1791782582913840981) produced four 2000×1400 production
GPU battle captures, each with exit zero. All four full images were individually
inspected: species anatomy, Pidgeotto, arena and battle HUD are visible.
Music and effects were muted before playback with `CRYSTAL_PREVIEW_MUTE=1`.

The exact all-251 Rust model regression passed in 2.57 seconds, one test and
zero failures, with all four final fingerprints verified in the actual test
executable before and after. That executable remained unchanged (inode
216610276, size 201,634,144, mtime_ns 1791782840851990349). The Cargo run also
completed successfully. This proves these installed meshes render in the GPU
image target; the separate live-window issue, browser and animation review
remain outside this evidence. Completed ignored review output was removed.
