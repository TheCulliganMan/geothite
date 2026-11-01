# Nidorino poison-pin quadruped

The current optimized GLB geometry, standard skin and production verification
are recorded in [Nidorina and Nidorino skins](nido-middle-skins.md). The section
below is the earlier static-sculpture record, including its former budgets and
source fingerprints.

## Prior static sculpture record

`tools/nidorino_sculpt.py` replaces Nidorino's generic rounded family body with
an original low quadruped. Its dedicated Rust battle registry entry remains
in place. The builder dispatches this pure source and rejects the obsolete
Nidorino primitive branch. Nidoqueen and Nidoking still need their own replacements; Nidorina and both
small Nidoran have separate original sculptures.

The [source review](nidoran-source-review.md) records the individually inspected
official artwork, rendered installed baseline and ThirdSon's advertised CC BY
animated rig. That model's download endpoint returned 401 and its linked Maya
scene returned 404. No external mesh, skeleton or reference pixels were
retrieved or imported. This replacement is original static geometry.

A long lowered wedge skull, rooted low mandible and forward swept nasal lance
replace the rounded head. Narrow fitted ivory eyes, muted irises, dark pupils,
tiny glints, nostrils, a shallow muzzle line and small ivory fangs give the face
its pointed expression. Tall ragged ears have thick closed geometry and fitted
dark teal interiors. The long low purple body has four rooted shoulders and
hindquarters, tapered planted legs, broad grounded paws and twelve ivory toes.
Irregular dark purple patches follow actual head, flank and leg triangles.
Four upright dorsal barbs and a short rear point have physical volume roots.

The first attachment check caught both ears outside the skull. Their roots
were moved forward into the real head volume, and their depth now slopes back
along their height. The topology and attachment thresholds were retained.
Final head markings were added before the last review and test runs.

Nine final candidate tests passed in 9.963 seconds. All nine installed-model
tests passed again in 9.482 seconds. They check closed connected outward
solids, actual face normals, recipe parity, floor zero, height 1.12, unique
parts, anatomical feature counts, long head extent and a 110,000-triangle
budget. Stored-ray tests cover head, jaw, horn, ears, fangs, tail and spine
roots, all four complete leg chains and all twelve toes. Fitted facial layers,
ear fields and head/flank/leg markings are tested against actual stored parent
triangles at vertices, front face centers and edge midpoints. Side markings
are inspected in rotated charts derived from the actual stored geometry.
Nonadjacent triangle tests cover horn, ears, fields, tail, spines and legs.
Deliberately detached horn, ear and tail negative controls are rejected.

The installed model is byte-identical to the reviewed candidate: 74 parts,
48,108 triangles. SHA-256:
`b2d7b8e47bf98a68f21a18c093a182075fd7872c295fde90c70f4b7112208c7f`.
Final front, oblique and rear software renders exited zero and were individually
inspected after the final geometry changes. The recipe, test and builder
sources parsed. Canonical layout validation and validation of 575 raw JSON
model documents passed. The relevant whitespace check passed.

The fresh native build retry exited zero after 12 minutes 53 seconds including
shared-cache waits. The full source fingerprints for Nidorina, Nidorino and both
small Nidoran, plus the mute switch, were verified in the actual rebuilt
executable before capture. All four serial captures used one unchanged binary:
inode 216262677, size 185981536, modification time 1791775675758159751. Each
source hash was checked before and after its capture. Music and effects were
muted by setting `CRYSTAL_PREVIEW_MUTE=1` before process creation. The inspected
native backend sets sink volume to zero before audio append and keeps music
refresh at zero.

Nidorino's capture exited zero in 3.37 seconds and produced a 2000×1400 image
with 39,924 colors. The full frame was individually inspected: the actual low
purple quadruped, long wedge head, forward horn, teal ear fields, head/flank/leg
markings, rooted dorsal barbs and four clawed grounded paws are visible with
Pidgeotto, arena and HUD. Complementary software views cover the rear outline.
The all-251 Rust model regression passed in 1.31 seconds with zero failures (1,280 other voxel tests filtered out). All four current Nidoran-family source fingerprints were verified in its actual test executable.
The full roster goal and separate live-window capture issue remain open.
Task-generated review images and logs were removed after verification.
