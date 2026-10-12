# Original small Tyrogue sculpture

The canonical model comes from `tools/tyrogue_sculpt.py`. A continuous lilac
three-pronged crown replaces the floating scalp spikes. Its round lower jaw,
rooted oval side ears, fitted angular ivory eyes, tall gold irises and dark
pupils establish the small scuffle fighter's face. The compact torso carries
a continuous cream waist band and brown shorts with separate rooted leg hems.
Slim bent arms end in cream wrist wraps, open palms and three curved digits
per hand. Lilac shins support grounded brown shoes with fitted toe contours.
The arm roots extend inside the torso rather than leaving exposed cap seams.

The [family source review](hitmon-source-review.md) records the official visual
reference and a specific downloadable candidate with an inaccessible archive.
This is original static geometry; no external skin, skeleton or animation was
retrieved or imported.

The model contains 40 closed parts and 68,568 triangles, with floor zero and
height 0.86. Its raw canonical SHA-256 is
`0174822bef2f4f5a18455e02e382abda29cd0120fb78095c87dc622c0a07176e`.

All ten final candidate checks passed in 19.031 seconds, and all ten canonical
checks passed in 18.848 seconds. They check closed connected outward solids,
surface normals, recipe parity, budget, anatomy counts and grounding. Actual
occupied-volume intersections support head/neck, ears, torso, waist band,
shorts/hem, arms/wrists/palms, all six digits, legs and shoes. Fitted gaze,
eye spark, mouth, ear folds and shoe seams follow stored parent triangles at
vertices, face centers and edge midpoints. Curved crown, clothes, limbs and
digits reject nonadjacent triangle crossings. Deliberately detached head,
shorts, palm, finger and shoe negative controls fail attachment.

The initial ten candidate checks passed, but visual review exposed a coarse
jaw outline and visible shoulder cap seams. The final recipe rounds the lower
jaw, increases face curvature and extends the arm roots inside the torso.
Final front, oblique and rear 800×824 software renders exited zero and were
individually inspected after those changes. The builder dispatches to the pure
recipe and rejects the obsolete generic dispatcher for all four fighters.
The existing dedicated Rust registry entry remains unchanged. AST, canonical
layout and 575-file raw JSON storage validation passed.

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
