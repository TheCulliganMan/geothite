# Original armored Nidoking sculpture

The canonical Nidoking model now comes from `tools/nidoking_sculpt.py`.
It replaces the generic family biped with a forward-leaning purple body, a long
lowered wedge skull, a large swept nasal lance, thick ragged teal-lined ears,
four cheek barbs, fitted angular eyes and an actually open toothed jaw. A broad
cream breast shield and four separate abdominal plates follow the torso.
Shoulder and knee armor, strong bent forelimbs, three-clawed fists, broad single
ivory toe claws and a massive upturned tail distinguish it from Nidoqueen.
Three tail collars and six back/tail barbs have actual volume roots. The
existing dedicated Rust registry entry is unchanged. The builder dispatches
to the pure recipe; all six obsolete generic family branches are rejected.

This is original static geometry. The [family source review](nidoran-source-review.md)
records the specific external candidates and their access/license limitations.
No accessible validated open rig was retrieved or imported.

The model has 70 closed parts and 74,756 triangles, with floor zero and height
1.40. Its raw canonical SHA-256 is
`bdbb4818caf7c58e767e4d1c1715fb4cabb83314d7677f091d964788402124f6`.

All twelve candidate stored-mesh tests passed in 10.749 seconds. They check
connected closed solids, outward face directions and normals, recipe parity,
anatomy counts, budget and grounding. Head/horn/ears/cheek barbs, jaws, all six
teeth, tongue, shoulder/arm/hand and haunch/knee/ankle/foot chains, all claws,
tail/collars and barbs are checked for actual solid intersections. Fitted eyes,
nostrils, ear fields and chest/belly armor are checked against actual parent
triangles at vertices, face centers and edge midpoints. Selected curved solids
are checked for nonadjacent face crossings.

The mouth check samples actual stored occupied intervals through both jaws,
the tongue and every other part, requiring empty space inside the open mouth.
A deliberately filled mouth must fail; detached ear/tail/claw negative
controls must fail attachment. The first candidate exposed a detached lower
jaw hinge. Its rear geometry was extended into the head without closing the
tested mouth gap. The final front, oblique and rear 800×824 software images
all rendered successfully and were individually inspected after that fix.
The rear view also shows the substantial tail, collars and rooted dorsal barbs.

All twelve canonical tests passed in 11.042 seconds. Model layout, AST and
575-file raw JSON storage validation also passed. The native build completed successfully after the earlier shared footwear
syntax failure was corrected. The final raw mesh fingerprint and the mute
switch were present in the actual rebuilt executable. One unchanged executable
(inode 216441911, size 189531952, mtime_ns 1791779637085927116) produced
all four adult/fighter production GPU captures with exit zero. This model's
2000×1400 capture contained 49,422 colors and was individually inspected in
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
