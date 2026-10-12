# Kangaskhan sculpture candidate

`tools/kangaskhan_sculpt.py` authors an original heavy gray-brown parent,
cream belly and actual open pouch, separate upper muzzle and lower jaw,
pink tongue, two white upper fangs, dark forehead plate, cream ears,
raised heavy arms and clawed fists, armored thighs, broad clawed feet,
four dorsal plates and a balancing tail. A separate pale-violet baby has
a seated body, short neck, rounded head, cream ears, fitted dark eyes,
white glints and arms reaching onto the pouch rim.

The installed canonical model has 70 parts and 113,464 triangles at
height 1.40. Its pure Rust-consumed authoring recipe replaces the former
batch recipe. Canonical SHA-256:
`facbe7601c3c80aa51397c1bac1354dc8d6ca41f999746128fcac98860fd9d5d`.
No external model, skeleton, texture or clip was retrieved or imported;
the [source review](vine-parent-source-review.md) records the individual
third-party claims and actual inaccessible download endpoints.

The first generation failed because a red pupil extended beyond its
angular white eye. The pupil and black slit were reduced to stay on their
actual parent surfaces. The generated candidate's three 800×824 views
each exited 0 and were individually inspected in full. It improves the
prior toy torso and projecting baby, but the forehead/eye treatment,
shoulder plates and wrist transitions still need art refinement.

Its initial ten-check suite failed two checks in 28.384 seconds. One
failure correctly exposed cream-belly triangles penetrating the torso;
the fitted layer offset increased from .004 to .007. The other exposed a
test selector bug: suffix `1 0` also matched left-side suffix `-1 0`, pairing
a left claw with a right fist. The selector now requires the preceding
space and tests the correct side. The physical overlap guard is unchanged.

The expanded eleven-check suite passed against the corrected candidate
in 40.930 seconds.
It verifies closed outward surfaces, face-aligned normals, recipe parity,
bounds and budget, actual attachments for every parent/baby appendage,
fitted face geometry, uncrossed curved limbs and claws, actual empty mouth
space, an open pouch cavity, and the stored baby's containment within the
actual pouch envelope. Detached, displaced and filled-pouch examples
exercise the negative guards. Fresh three-angle renders also exited 0.
The next art revision narrows and slopes the parent's white eye shapes,
places the red pupils toward their outer ends, and replaces the rounded
shoulder ridges with angular four-sided bent armor. It is undergoing
generation and requires fresh checks and individually inspected renders.
That eye/armor revision subsequently passed all eleven checks in 33.290
seconds. Its oblique view was inspected in full and showed the sharper
gaze and angular armor. The fists are now enlarged and lowered slightly
to make the parent heavier and increase their overlap with the wrists;
this latest adjustment requires another full check and three-angle review.
The heavier-fist candidate passed all eleven checks in 39.877 seconds.
All three final 800×824 views exited 0 and were individually inspected in
full. The wrists now overlap more deeply, the parent has a sharper gaze
and angular shoulder armor, and the supported baby reads clearly above
the actual pouch rim. Final candidate SHA-256:
`facbe7601c3c80aa51397c1bac1354dc8d6ca41f999746128fcac98860fd9d5d`.
It retains 70 parts and now has 113,464 triangles. Canonical installation,
canonical checks, native GPU rendering and Rust integration are still
pending. The native lighting review may justify further art refinement.
The final reviewed bytes are now installed as canonical Kangaskhan.
The builder dispatches to this pure recipe before Blender batching and
the prior generic implementation is retired with an explicit guard.
Canonical checks and the native union build are running. This replacement
has no native GPU or Rust integration proof yet. Scratch remains ignored
while the family work continues.

All eleven checks against the installed canonical bytes passed in 48.966
seconds. The exact Rust Kangaskhan route is preserved. The final-hash native
build is still live, and a muted real-pack GPU capture is required next.

The native union build exited 0 in 8 minutes 40 seconds. Before launch,
its binary contained the full final Kangaskhan and Tangela SHAs and the
preview-mute switch; it was frozen at
`(219482854, 217110320, 1791826014813728356)` (inode, bytes, modification
time in nanoseconds). With explicit `CRYSTAL_PREVIEW_MUTE=1` and
`CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image`, the real external
`content-packs/realtime-clock.browser.crystalpack` produced a 2000×1400
native GPU battle capture and exited 0. The full image was individually
inspected: the parent, baby, open pouch, separated jaws, claws, armor,
tail, opponent, arena and HUD rendered under native lighting. The binary
identity and both canonical source hashes remained stable across capture.
This is image-target GPU evidence, not ordinary live-window, browser,
animation or gameplay approval. Final-hash Rust integration is running.

The final-hash Rust union finished successfully: the actual 251-species
geometry regression ran once and passed in 5.06 seconds. A direct exact
rerun passed in 4.77 seconds; the unit contained both final Kangaskhan and
Tangela hashes and remained frozen at
`(219548236, 249172736, 1791826767942085063)` across that run.
These results precede the subsequent shiny and shoreline changes.
