# Original armored Nidoqueen sculpture

The canonical Nidoqueen model now comes from `tools/nidoqueen_sculpt.py`.
It replaces the generic family biped with a broad low reptilian head, fitted
angular eyes, a short forehead horn, round brown-lined ears, a cream lower jaw,
paired cream chest shields and four separate fitted abdominal plates. Blue
shoulder and knee armor supports powerful bent arms, broad three-clawed fists,
two grounded two-clawed feet, a substantial curved tail and rooted dorsal barbs.
The existing dedicated Rust battle registry entry is unchanged. The builder
dispatches to this pure recipe and rejects the obsolete queen branch.

This is original static geometry. The specific external candidates checked in
the [family source review](nidoran-source-review.md) have not yielded an
accessible validated open rig; no third-party mesh, skeleton or texture was
retrieved or imported.

The model has 61 closed parts and 85,324 triangles, with floor zero and height
1.40. Its raw canonical SHA-256 is
`993c4bdcf66d31127266a885716cf736cc6489865ec64b68e33e57ecd535bb80`.

All nine candidate stored-mesh tests passed in 16.055 seconds. They check
connected closed solids, outward face directions and normals, recipe parity,
anatomy counts, triangle budget, grounded feet, actual volume roots for head,
muzzle, horn, ears, fangs, shoulder/arm/hand and haunch/knee/ankle/foot chains,
all claws, tail and back/tail barbs. Fitted armor, chin, eyes, nostrils and ear
fields are checked against actual parent triangles at vertices, face centers
and edge midpoints. Selected curved solids are checked for nonadjacent face
crossings. Detached ear/tail/horn/claw negative controls must fail attachment.

Early checks caught toe geometry below the foot pads and an excessive triangle
count. The claws were raised and armor subdivision reduced while preserving
the same geometry checks. Overlapping belly plate outlines were separated and
the final three software views were regenerated. The final front, oblique and
rear 800×824 images all rendered successfully and were individually inspected.
The wrist caps are enclosed in the broad fists; chest and belly plates remain
separate; the complementary rear image shows the long tail and dorsal roots.

All nine canonical tests passed in 15.889 seconds. Model layout, AST and
575-file raw JSON storage validation also passed. The native build completed successfully after the earlier shared footwear
syntax failure was corrected. The final raw mesh fingerprint and the mute
switch were present in the actual rebuilt executable. One unchanged executable
(inode 216441911, size 189531952, mtime_ns 1791779637085927116) produced
all four adult/fighter production GPU captures with exit zero. This model's
2000×1400 capture contained 49,469 colors and was individually inspected in
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
