# Original Rhydon sculpture

`tools/rhydon_sculpt.py` authors a heavy grey biped with a continuous torso,
four fitted cream belly bands, pointed neck armor, broad armored skull and
muzzle, open fanged lower jaw and supported pink tongue. Tall pointed ears,
fitted narrow red gaze, six rooted cheek points, rounded shoulders, short
powerful arms, three hand claws each, stocky haunches and two claws per foot
replace the old generic rock body. A cream nasal drill carries an actual
three-turn raised spiral. Three back points and three fitted segmented collars
finish a thick tapering upturned tail.

The canonical model has 64 closed parts and 69,896 triangles, floor zero and height
1.40. Its SHA-256 is
`f5404c803679e4c76d6471e6b2742e6be378855047cba540823e96d92cd9ef4d`.
All fourteen candidate tests passed in 12.757 seconds and all fourteen
canonical tests passed in 11.917 seconds. They cover closed
connected outward surfaces, face-aligned unit normals, recipe parity,
bounds/budget/anatomy counts, actual skull/muzzle/jaw/chamber/tongue roots,
ear/horn/spiral/cheek/limb/claw/fang/tail/armor volume roots, fitted marking
vertices and face/edge clearances, uncrossed curved surfaces, twelve actual
empty jaw samples and filled-mouth/detached attachment negatives.

The first collar outline extended outside the torso and generation refused
it; the corrected outline fits its parent. Rear visual inspection also caught
speckling where independently curved tail collars crossed the actual tail.
The collars now share the tail's complete curve, transport frames and radial
sections. They are closed annular shells, with outer faces clearing and inner
faces rooting into the stored tail. A dedicated test checks both surfaces
against occupied volume. The drill spiral similarly follows the horn's actual
smoothed curve and taper instead of an approximate cone. No test was relaxed.

Final front, oblique and rear 800×824 software renders exited zero and were
individually inspected after the horn/tail corrections. The drill spiral reads
as three turns, the tail bands have clean boundaries, and the open mouth,
fitted gaze, belly bands and claws are visible. AST checks passed.

The [source review](rhydon-source-review.md) records the inspected primary art
and accessible rig candidate. The latter was excluded for humanoid avatar
anatomy; no external geometry or rig is imported. This candidate is original
static geometry. It is installed through a dedicated pure builder dispatch,
preserving the existing exact Rhydon registry route. The generic rock builder
rejects both rhinoceroses. The canonical bytes match the final three reviewed
views' candidate hash. Model layout and all 575 raw canonical JSON files pass
validation. A final-hash test executable containing Rhydon and Steelix passed
the exact all-251 loaded-species check in 2.59 seconds and all 18 battle-layout
tests. Its identity remained `(217082565, 212678000, 1791802643816871505)` and
the canonical hashes stayed unchanged. That earlier check predates Graveler and Golem and excludes their replacements.

The matching native executable contained the full final Rhydon, Graveler and
Golem hashes before each launch. All three explicit image-target battle
captures exited zero with `CRYSTAL_PREVIEW_MUTE=1`; each full 2000×1400 GPU
frame was individually inspected for anatomy, floor contact, arena and HUD.
Native executable identity `(217084807, 199802640, 1791803109751071562)` and
all three canonical hashes remained stable through the complete capture batch.
The exact all-251 loaded-species test then passed in 2.96 seconds in an
executable containing all three final hashes, with stable identity
`(217106909, 214395568, 1791803306590544522)` and unchanged canonical bytes.
This verifies the native image target and roster loading; live-window,
browser, animation and gameplay paths remain unapproved. The queued Cargo integration job also finished with exit zero; its roster
check ran one test and passed in 2.70 seconds. Completed temporary review
images and logs have been removed.
