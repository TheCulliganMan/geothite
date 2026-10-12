# Original boxing Hitmonchan sculpture

The canonical Hitmonchan model now comes from `tools/hitmonchan_sculpt.py`.
It replaces the rounded primitive fighter with one continuous five-point crown
head, fitted angular blue eyes, a firm jaw crease, pointed tan shoulder guards,
a complete sleeveless lavender tunic, a charcoal waist belt, a continuous
eight-pleat skirt and grounded lavender boots. One arm reaches forward while
the other guards the face. Both red boxing mitts have rooted thumbs and wrist
cuffs, with fitted lower padding contours. The existing dedicated Rust battle
registry entry is unchanged; the builder dispatches to this pure recipe and
rejects the obsolete boxing branch.

This is original static geometry. The [family source review](hitmon-source-review.md)
records the advertised animated Hitmonchan-containing pack and the specific
sculpture candidate checked. Their download endpoints returned 401; no mesh,
skin, skeleton or animation was retrieved or imported.

The model has 36 closed parts and 43,776 triangles, with floor zero and height
1.30. Its raw canonical SHA-256 is
`0a3c75f05e04d4d4eb4d6b685217d5d129ca745ef612552963edae733e6ac903`.

All eleven candidate stored-mesh tests passed in 10.053 seconds. They check
closed connected solids, outward faces and normals, parity, budget, anatomy
counts, skirt flare and grounding. Actual occupied-volume intersections attach
neck/head, tunic/skirt/belt, shoulder guards, bent arms, mitts/thumbs/cuffs and
both legs/boots/ankle collars. Eyes, iris/pupil/glint, mouth, boot piping and mitt
contours follow actual stored parent triangles at vertices, face centers and
edge midpoints. Selected curved costume, crown and limb surfaces are checked
for nonadjacent face crossings. Detached head, mitt, thumb, boot and skirt
negative controls must fail attachment.

Initial checks found a shin/boot gap and a crossed nonplanar skirt cap. The
legs were extended into the shoes and the hem cap was made planar while
retaining radial pleats. A visual review also exposed crown shading streaks.
The taper now uses a smooth transition and a refined cap with continuous
authored front/back surface normals, retaining crisp perimeter normals. A new
stored-mesh regression checks that shared front-cap vertices have matching
normals. A software edge-coverage hypothesis was tested in scratch but pixel
comparison did not support it; the preview renderer was left unchanged.

Final front, oblique and rear 800×824 software images all rendered successfully
and were individually inspected after the final normal change. The crown is
clean, the fitted clothing replaces the old exposed torso and disconnected
purple hem leaves, and the rear view exposes the supported shoulder guards,
arms, waistband and boots. All eleven canonical checks passed in 10.135
seconds. AST, model layout and 575-file raw JSON storage validation passed.

The native build completed successfully after the earlier shared footwear
syntax failure was corrected. The final raw mesh fingerprint and the mute
switch were present in the actual rebuilt executable. One unchanged executable
(inode 216441911, size 189531952, mtime_ns 1791779637085927116) produced
all four adult/fighter production GPU captures with exit zero. This model's
2000×1400 capture contained 40,131 colors and was individually inspected in
full frame: its authored anatomy appeared with Pidgeotto, the arena and battle
HUD. Executable identity and all four source fingerprints were verified again
after capture. All previews launched with `CRYSTAL_PREVIEW_MUTE=1`; system
volume was unchanged.

The exact all-251 Rust model regression passed directly in 2.19 seconds
(one actual test, zero failures, 1,293 filtered), with all four final mesh
fingerprints verified in the frozen test executable before and after the run.
This proves the model load path and this GPU image preview; the separate
live-window issue remains unresolved. Completed ignored review scratch was
removed after recording these results.
