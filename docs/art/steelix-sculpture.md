# Original Steelix sculpture

`tools/steelix_sculpt.py` authors ten irregular steel boulders with nine buried
volume-rooted joints, eight alternating body spikes and a pointed tail. The
broad angular skull has fitted narrow ivory eyes, red irises and dark pupils.
A separate low jaw connects through two actual metal hinges. Four upper and
three lower squared teeth leave a real front aperture, backed by a recessed
dark chamber. The incorrect old head prongs and exposed segment rods are gone.

The canonical `battle_species/steelix.mesh.json` has 48 closed parts and 9,244
triangles, floor zero and height 1.45. Its SHA-256 is
`d0f1a84536b48bc57a1781676d3871ba7a2f041af2be908aa3ccf663709fdf03`.
All thirteen candidate checks passed in 3.075 seconds; all thirteen canonical
checks passed in 3.045 seconds. Tests cover closed connected outward surfaces,
unit normals following actual faces, recipe parity and anatomy counts, actual
adjacent-rock/joint/spike/tail/head/hinge/chamber/tooth volume roots, grounded
tail supports, fitted eye vertices and face/edge clearances, uncrossed spikes
and joints, twelve actual empty aperture samples and filled/detached negatives.
The body-reference test compares Rust against the authored normalized centerline.

The first checks caught socket and ivory-eye faces crossing their faceted
parents. Increasing the fitted layers' depth corrected the geometry without
relaxing the checks. Final front, oblique and rear 800×824 software captures
exited zero and were individually inspected after the correction.

The dedicated builder dispatch preserves the existing Steelix registry path;
the old generic serpent function now rejects Steelix. Its normalized
tail-to-head centerline is 2.867623161993143, used in battle layout as
`2.867_623_2 / 1.45`.

The [source review](rock-serpents-source-review.md) records inspected official
art and specific external candidates. Their archive endpoints returned 401;
no external mesh, rig, skin or animation was available for inspection or import.
This sculpture is original static geometry.

The all-251 Rust job finished successfully (one test, 2.26 seconds). A direct
test executable containing the canonical SHA also passed that exact test in
2.56 seconds, but the following layout suite exposed a remaining old Steelix
fixture: its model bounds and length assertion still described the previous
sculpture. Seventeen tests passed and that fixture failed. It now uses the
actual canonical bounds and the new measured reference span. Fresh layout
execution remains required; the failed suite is not approval.

The original queued layout job also executed the stale fixture and exited 101.
Only after that job was confirmed terminal was a fresh layout retry started.
The native build finished in 15m00s. The executable was checked for the full
canonical hash and mute switch, then captured with `CRYSTAL_PREVIEW_MUTE=1`
and the known-good local pack. The image-target capture exited zero. Its full
2000×1400 GPU frame was individually inspected: the broad toothed head, red
gaze, connected steel chain, alternating spikes and pointed tail render with
Pidgeotto, the arena and production battle HUD. Tail segments meet the arena
floor and the complete silhouette fits the frame. Executable identity remained
`(217030958, 197903744, 1791801856225148706)` (inode, bytes, mtime nanoseconds),
and the canonical hash remained unchanged.

The corrected-layout retry finished successfully. Independent final-hash
verification used test executable identity
`(217082565, 212678000, 1791802643816871505)`. It contained the full Steelix
and Rhydon canonical hashes. The exact all-251 loaded-species test passed in
2.59 seconds and all 18 battle-layout tests passed, including the corrected
serpent fixture. Executable identity and source hashes remained stable.
That binary lacked the newer Graveler hash, so this result does not verify
that replacement. The native image target proves this capture path, not a
live window, browser, animation or interactive gameplay. Completed Steelix
review scratch is removed after inspection.
