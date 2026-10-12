# Slowking sculpture

The canonical `slowking.mesh.json` now uses original geometry from
`tools/slowking_sculpt.py`. Its 62 named closed parts contain 65,164 triangles,
with a grounded floor and normalized height 1.25. The final SHA-256 is
`912f4d45bcfecffb281acb335c0b9375b9f44962f7e011ad0baf371e0019984d`.

The original royal biped has a slender pink torso and neck, rounded rooted
head, fitted tall white eyes with tiny black pupils, broad cream muzzle,
subtle fitted nostrils and curved smile seam, and two small rooted canines.
Three cream lower-abdomen bands follow the actual torso surface. Short bent
arms with rounded shoulders and palms carry three white claws per hand.
Round thighs connect through short ankles to two grounded feet with two
white claws each. A thick pink tail tapers into an upturned pointed tip.
The muzzle is closed with a fitted smile marking; no open oral aperture is
claimed for this species.

Ten alternating red/ivory closed collar panels flare from buried neck roots.
Their curved sectors replace flat triangular leaves. A separate gray shell
crown roots into the head and carries a broad pale brim, two curved lateral
horns, two slanted fitted overlapping turns and five irregular upper crest
points. Its round red jewel sits within a fitted pale setting and carries a
small supported ivory glint. This replaces the baseline oval crown and two
straight cones.

The candidate passed twelve checks in 10.460 seconds; the installed canonical
model passed the same twelve in 8.979 seconds. Checks cover stored recipe
parity, closed outward finite solids, face-aligned normals, height/floor and
triangle budget, named anatomy, actual occupied-volume attachment roots for
collar/limbs/claws/teeth/tail/crown/horns/crest/turns, both grounded feet,
fitted vertices and face/edge clearances, actual shell-turn outer/inner face
occupancy, uncrossed curved anatomy and detached attachment negatives.
The first collar failed the normal check on its thin edges; separating edge
normals with the existing crease helper fixed the shading without relaxing
the check. Final front, oblique and rear software views all rendered at
800×824 and were individually inspected after this correction.

Canonical bytes match the reviewed candidate. The dedicated pure builder
preserves the existing exact Rust species route, and the retired generic
function rejects both Slowbro and Slowking. The
[source review](slow-pair-source-review.md) records official artwork and
specific licensed external candidates whose archive endpoints returned HTTP
401. No external mesh, rig, texture or animation was retrieved or imported.
This is original static geometry.

The matching native build finished with exit zero in 3m46s. The executable
contained the full final Slowking hash and the mute switch before launch.
The explicit image-target battle capture used `CRYSTAL_PREVIEW_MUTE=1`,
exited zero, and its full 2000×1400 GPU frame was individually inspected for
crown/collar/face/limb appearance, floor contact, arena and HUD. Executable
identity `(217192507, 201817104, 1791805153510236568)` and canonical hash
remained stable through the capture.

The earlier Slowbro test executable lacked the newer Slowking hash and was
excluded. The fresh Cargo all-251 job finished with exit zero in 7m20s,
running one roster test and passing in 1.85 seconds. A direct exact check
then passed in 1.39 seconds in an executable containing both final Slowbro
and Slowking hashes, with stable identity
`(217198810, 218441024, 1791805361793194901)` and unchanged canonical bytes.
Native image-target rendering and roster loading now pass for both species;
live-window, browser, animation and gameplay paths remain unapproved.
Completed shared temporary review images and logs have been removed.
