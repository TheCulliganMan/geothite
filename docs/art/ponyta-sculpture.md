# Ponyta sculpture

The canonical `ponyta.mesh.json` now uses original geometry from
`tools/ponyta_sculpt.py`, replacing the toy quadruped and triangular fire
clusters. Its 47 named closed parts contain 81,452 triangles, with a grounded
floor and normalized height 1.10. The final SHA-256 is
`9bac3137357cc65e147cef96ad421fc4142a87a5570b31ec250d7ec6b6a88628`.

A compact horizontal cream torso connects to an arched chest/neck, youthful
horse head and modest long muzzle. The head carries fitted dark oval eyes
with supported white glints, subtle nostrils, a closed smile marking and
pointed ears with fitted dark pink interiors. Rounded front shoulders join
four slender shaped limbs to four grounded dark blue hooves. Ponyta has no
horn. The muzzle is closed; no actual open oral cavity is claimed.

Seven closed orange fire banks grow from the forehead, rear neck/back,
short rooted tail stem and four lower legs. Their curved lobed outlines
replace separate triangular spikes. Volumes taper at the outline and billow
across the two cap faces. Fourteen supported yellow core layers follow both
stored sides, preserving color from the rear. Fire is static authored
geometry, with no borrowed rig or animation.

The final candidate passed twelve checks in 23.119 seconds. Checks cover
stored recipe parity, finite closed outward solids, face-aligned normals,
height/floor and triangle budget, named anatomy, the hornless silhouette,
actual volume roots for head/muzzle/ears/neck/tail/shoulders/legs/hooves/fire,
four grounded blue hoof supports, fitted vertices and face/edge clearances,
uncrossed limbs/ears/tail/fire banks, and detached fire/hoof/tail negatives.
An automatic core contour crossed a concave fire edge and was replaced with
explicit authored contours. Rounding the fire then exposed fitted-cap
clearance failures; denser core caps and a properly seated mane core resolved
them without relaxing the checks. A protruding trunk end was moved inside
the neck, and rounded shoulders cover the foreleg roots.

Final front, oblique and rear software views all rendered at 800×824 and
were individually inspected after those corrections. Installed bytes match
the reviewed candidate. The dedicated pure builder dispatch preserves the
existing exact Rust species route; the legacy function rejects both horses, now using dedicated recipes. The [source review](fire-horses-source-review.md)
records official art and four specific CC BY candidates whose archive
endpoints returned HTTP 401. No external mesh, rig, texture or clip was
retrieved or imported.

All twelve installed canonical checks passed in 34.217 seconds. The fresh native build exited successfully. An actual image-target GPU
battle capture also exited 0 and was inspected in full: Ponyta, its blue
hooves and flame silhouette, the opposing Pidgeotto, arena floor and battle
HUD were visible. The executable contained the complete canonical SHA and
`CRYSTAL_PREVIEW_MUTE`; audio was explicitly muted for launch. Executable
identity `(217246203, 203171296, 1791806984531775190)` and model SHA stayed
stable across capture. The queued all-251 Rust integration job exited 0, running the actual
model test once in 2.60 seconds. A direct final-hash repeat passed once in
3.14 seconds with executable identity `(217253184, 221099168, 1791807236830199311)`
and canonical model SHA stable before/after. This verifies bounded reusable
models; it does not approve the artwork of all 251 species. Software views alone do not approve native image targets,
live windows, browser, animation or gameplay paths. Task-owned ignored fire-horse review scratch was removed after all
integration jobs completed.


A later shared test executable contained both final horse SHAs and passed
once in 3.81 seconds, with both models and executable identity
`(217688309, 223956624, 1791810097391192065)` stable across the run.
