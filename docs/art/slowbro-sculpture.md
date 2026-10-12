# Slowbro sculpture

The canonical `slowbro.mesh.json` now uses original geometry from
`tools/slowbro_sculpt.py`, replacing the generic oval head, round protruding
eyes, mitten arms and ball-shaped shell. Its 74 named closed parts contain
76,932 triangles, with a grounded floor and normalized height 1.20.
The final SHA-256 is
`d41f80791ac48c331106e0ae521dcbe112b784dd916e26e79d41e806cc950d8a`.

A continuous pink head, neck and pear-shaped torso carries fitted tall white
eyes with tiny dark pupils, rounded ears with fitted inner curls, a broad
rounded cream upper lip, a separate rounded lower jaw, recessed dark oral
chamber, visible pink tongue and two small upper canines. Four fitted cream
bands follow the actual lower torso. Rounded shoulders and hanging forelimbs
end in short palms with three white claws each. Heavy thighs connect through
short ankles to two grounded feet with two white claws each.

The thick pink tail roots into a separate gray-blue Shellder creature. Its
progressively tapering shell has three slanted fitted overlapping turns,
seven rooted spines, an angular fitted white gaze, a separate lower biting
jaw, eight rooted teeth and a solid rear wall around its recessed oral
chamber. The front aperture remains open around the tail. This is a modeled
shell creature rather than a sphere with a decorative spiral.

The candidate passed fourteen checks in 47.579 seconds after the final
shoulder and rear-wall changes. Checks cover stored recipe parity, finite
closed outward surfaces, face-aligned normals, triangle budget, named anatomy,
actual occupied-volume roots for limbs/claws/ears/teeth/tail/shell, grounded
feet, fitted vertices and face/edge clearances, actual shell coil outer/inner
face occupancy, uncrossed curved limbs, twenty actual unoccupied sample
points across the hero and shell apertures, filled-mouth negatives and
detached attachment negatives. Earlier failures identified a weak canine
root and a tail obstructing a shell-mouth sample; geometry was corrected
without relaxing the checks. The tongue was raised from inside the lower lip
and the rear oral chamber covered after visual review.

All three final software views (front, oblique and rear, 800×824) rendered
successfully and were individually inspected. Installed canonical bytes
match the reviewed candidate. The dedicated pure builder dispatch preserves
the existing exact Rust Slowbro route; the legacy Blender function explicitly
rejects both Slowbro and Slowking. The
[source review](slow-pair-source-review.md) records official artwork and
specific externally licensed candidates whose archive endpoints returned
HTTP 401. No external geometry, rig, skin or animation was retrieved or
imported. This is original static geometry.

All fourteen installed canonical regressions passed in 44.908 seconds.
The matching native build finished with exit zero in 4m44s. Its full final
Slowbro hash and mute switch were verified before launching the explicit
image-target battle capture with `CRYSTAL_PREVIEW_MUTE=1`. The capture exited
zero; its full 2000×1400 GPU frame was inspected for the visible anatomy,
floor contact, arena and HUD. Executable identity
`(217163579, 200875920, 1791804654471009916)` and canonical hash stayed stable
through the launch.

The all-251 Cargo job finished with exit zero in 6m27s and ran one roster
test, passing in 1.92 seconds. A direct exact check then passed in 2.52
seconds with the full final Slowbro hash verified and stable executable
identity `(217174951, 216558656, 1791804819123248078)` and unchanged canonical
bytes. Both completed executables lacked the newer Slowking hash, so these
checks do not verify that replacement. The native image target and roster
loading pass for Slowbro; live-window, browser, animation and gameplay paths
remain unapproved. The later exact all-251 check passed in 1.39 seconds
with both final species hashes verified in stable executable identity
`(217198810, 218441024, 1791805361793194901)` and unchanged canonical bytes.
Slowking also passed its own final-hash muted GPU capture. Completed shared
temporary review images and logs have been removed.
