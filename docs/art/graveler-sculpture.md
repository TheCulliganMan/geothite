# Original Graveler sculpture

`tools/graveler_sculpt.py` replaces the separate toy head and humanoid rock
torso with one broad irregular boulder bearing its actual face. A fitted
jagged light forehead field, narrow ivory/dark gaze and curved pink grin
follow the stored rock surface. Two larger spread upper arms and two shorter
folded lower arms carry broad palms and three blunt stone digits per hand.
Two short floor-rooted feet support the body; six seated rear stones break
the boulder's silhouette.

The canonical `battle_species/graveler.mesh.json` has 40 closed parts and
23,836 triangles, floor zero and height 1.10. Its SHA-256 is
`d2353239ad0397ec78ebcc70deb9c0bb44a459d7a954ef310d1a0bd1fafae418`.
All ten candidate checks passed in 3.675 seconds and all ten canonical checks
passed in 4.017 seconds. They cover closed connected outward surfaces,
actual face-aligned unit normals, recipe parity/bounds/anatomy/budget,
occupied-volume arm/palm/finger/foot/rubble roots, distinct arm-pair
proportions, both grounded feet, fitted vertices and actual face/edge
clearances, uncrossed arms/forearms/fingers and detached negatives.

Initial lower-arm sweeps crossed themselves at the tight inward bend.
Reshaping that sweep alone did not resolve it. The folded lower arm now has
separate closed upper-arm and forearm segments with an actual volume-rooted
elbow. Both surfaces pass the original uncrossed-surface checks. The first
forehead render also showed skinny-triangle shading streaks; its cap normals
now follow the boulder's broad gradient with correction against each actual
face. Geometry clearance and normal tests remain unchanged.

Final front, oblique and rear 800×824 software views exited zero and were
individually inspected after those corrections. The forehead shading is
clean, both different arm pairs and stone fingers read, and the rear stones
are seated. The canonical bytes match those final reviewed views. Recipe,
test and builder AST checks, raw model JSON parsing and whitespace checks pass.

The dedicated pure builder dispatch preserves the existing exact species
registry path. The generic rock function rejects Graveler. The
[source review](rock-bipeds-source-review.md) records inspected official art,
specific unavailable external archives and the unlicensed game-model repository.
No external geometry, rig, skin or clip was retrieved or imported. This is
original static geometry. Golem now has its own installed
[fitted-shell sculpture](golem-sculpture.md).

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

The later [canonical stone-pair skins](stone-pair-skins.md) preserve this
boulder sculpture and add four independent arm/wrist chains, fixed battle feet
and two-support world walking. Current captures use the headless image path.
