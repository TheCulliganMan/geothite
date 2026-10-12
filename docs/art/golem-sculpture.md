# Original Golem sculpture

`tools/golem_sculpt.py` authors a nearly spherical continuous dark-brown seam
body with 32 fitted polygonal grey-green stone plates. Twelve pentagonal and
twenty hexagonal cells cover the complete sphere; their shared 90 edges form
a closed cell arrangement. Each stone is a curved closed solid whose outer
faces clear the actual seam body and whose inner faces root into it.

A small broad-browed tan-grey reptile head and muzzle emerge from the shell,
with fitted narrow red gaze, open lower jaw, recessed dark mouth chamber,
supported pink tongue, two upper canines and three lower teeth. Short raised
arms and forearms carry three ivory claws per hand. Stocky haunches and short
shins connect to grounded pads with three forward claws and a rear heel point
each. The scattered brown balls and generic head/fists are removed.

The canonical `battle_species/golem.mesh.json` has 77 closed parts and 35,124
triangles, floor zero and height 1.25. Its SHA-256 is
`dc7c4cfdee64d46c197b26b4fa01799afb0c5c143bbb341e186d7fabf2993f85`.
All fourteen candidate checks passed in 6.274 seconds and all fourteen
canonical checks passed in 6.221 seconds. They cover closed connected outward
surfaces, actual face-aligned unit normals, recipe parity/bounds/budget/anatomy,
complete polygon-cell edge coverage, all plate roots, actual outer/inner face
clearance/rooting, head/muzzle/jaw/chamber/tongue/tooth/limb/claw volume roots,
both grounded feet, fitted gaze vertices and face/edge clearance, twelve actual
empty jaw samples, filled/detached negatives and uncrossed curved surfaces.

The initial oral chamber missed the upper muzzle. Moving and deepening it
provided an actual root while preserving the front jaw aperture. No test was
relaxed. Final front, oblique and rear 800×824 software renders exited zero
and were individually inspected after that correction. The complete shell
and seams, open toothed mouth, fitted gaze and clawed limbs are visible.
The canonical bytes match the final reviewed candidate. Recipe/test/builder
AST, raw model JSON parsing and whitespace checks pass.

The dedicated pure builder dispatch preserves the existing exact Golem route;
the retired generic rock builder rejects all four now-dedicated species. The
[source review](rock-bipeds-source-review.md) records inspected official art and
specific external candidates. No external mesh, rig, skin or clip is imported.
This is original static geometry.

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
