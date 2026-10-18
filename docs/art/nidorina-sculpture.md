# Nidorina poison-pin sculpture

The current optimized GLB geometry, standard skin and production verification
are recorded in [Nidorina and Nidorino skins](nido-middle-skins.md). The section
below is the earlier static-sculpture record, including its former budgets and
source fingerprints.

## Prior static sculpture record

`tools/nidorina_sculpt.py` replaces the generic family primitive with an
original hornless Nidorina. The species builder dispatches this pure recipe
and rejects its old Nidorina branch. Nidorino and both small Nidoran now have separate original sculptures.
Nidoqueen and Nidoking still need their own replacements. The existing
dedicated Rust Nidorina registry entry requires no routing change.

The [source review](nidoran-source-review.md) includes the individually viewed
official artwork, the rendered baseline and specific external model checks.
No accessible validated open rig was obtained. No external geometry or
reference pixels were imported; this sculpture is original static geometry.

A broad hornless blue head and low muzzle carry fitted pointed ivory eyes,
orange irises, dark pupils, tiny glints, nostrils, a shallow smile and two
rooted ivory fangs. Two thick notched ears have fitted navy interiors and blue
lower folds. A squat continuous pear body has a rounded fitted pale breast.
Bent forelimbs, substantial spotted hindquarters, short ankles, grounded paws
and twelve ivory claws complete the stance. Three swept dorsal barbs and a
short rear point have real volume roots. Navy flank and haunch markings fit
the actual side-facing stored triangles.

The first geometry check caught the ankle reaching below the foot. Reducing
its terminal depth grounded the paws without changing the test threshold.
The final views also prompted a rounded breast outline and fuller palms,
which cover the original visible wrist junctions.

Nine final candidate tests passed in 21.308 seconds. All nine installed-model
tests passed again in 19.784 seconds. They inspect closed connected outward
solids, actual face shading, recipe parity, floor zero, height 1.12, unique
parts, feature counts and a 100,000-triangle budget. Stored-ray checks require
volume roots for the head, muzzle, ears, fangs, tail, dorsal barbs, arms,
hands, legs, feet and every claw. Fitted layers must follow their actual parent
at vertices, front face centers and edge midpoints. Side markings are tested
in rotated charts derived from the stored model. Nonadjacent face checks cover
ears, insets, folds, limbs, tail and dorsal barbs. Deliberately detached ears,
tail and spine are rejected.

The installed source is byte-identical to the reviewed candidate: 59 parts,
93,956 triangles. SHA-256:
`4c29ee7b88e823f186e1acaea51d0edae94c1d4d4459ef04a852c709325f0bfd`.
Final front, oblique and rear software renders exited zero and were individually
inspected after the final changes. The recipe, tests and builder parsed.
Canonical layout validation and storage validation of 575 raw JSON models
passed. The relevant whitespace check passed.

The older native executable and completed turtle Rust test binary lack this
model's full fingerprint and cannot verify it. Its first fresh native build
exited 101 on two unrelated undeclared `Model` references in
`mesh/modeled_dungeons.rs`. The shared source was subsequently inspected and
already uses the proper dungeon model instance methods; this model task made
no edits to that module. The build retry uses that corrected shared state.

The fresh native build retry exited zero after 12 minutes 53 seconds including
shared-cache waits. The full source fingerprints for Nidorina, Nidorino and both
small Nidoran, plus the mute switch, were verified in the actual rebuilt
executable before capture. All four serial captures used one unchanged binary:
inode 216262677, size 185981536, modification time 1791775675758159751. Each
source hash was checked before and after its capture. Music and effects were
muted by setting `CRYSTAL_PREVIEW_MUTE=1` before process creation. The inspected
native backend sets sink volume to zero before audio append and keeps music
refresh at zero.

Nidorina's capture exited zero in 6.80 seconds and produced a 2000×1400 image
with 37,411 colors. The full frame was individually inspected: the actual
hornless blue creature, notched navy-lined ears, orange eyes, pale breast,
spotted haunches, dorsal barbs and grounded clawed feet are visible with
Pidgeotto, arena scenery and the HUD. Complementary software views cover the
complete rear and tail outline. The all-251 Rust model regression passed in 1.31 seconds with zero failures (1,280 other voxel tests filtered out). All four current Nidoran-family source fingerprints were verified in its actual test executable.
The full roster goal and separate live-window capture issue remain open.
Task-generated review images and logs were removed after verification.
