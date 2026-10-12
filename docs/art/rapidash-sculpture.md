# Rapidash sculpture

The canonical Rapidash mesh uses original geometry from `tools/rapidash_sculpt.py`.
Its 54 named closed parts contain 86,516 triangles, with a grounded floor and
normalized height 1.30. SHA-256:
`cd7d461ad54d109e64377e22e7a1516b35fc2e8d21559b7518b4f7f7c0395fd8`.

The adult body has an athletic horizontal torso, long arched neck, proportioned
horse head, pointed ears and a long ivory forehead horn. Layered narrow eyes
carry fitted red irises and dark slits. The separate upper snout, lower jaw,
recessed dark oral chamber and pink tongue retain an actual empty mouth opening.
Rounded shoulders connect to four curved limbs: three dark blue hooves support
the model while one bent foreleg holds its hoof above the floor.

Seven closed orange fire banks connect to the crown, neck/back, rooted tail stem
and four lower limbs. Fourteen yellow core layers follow both actual stored
sides. The larger tail required a distinct seating offset to keep core face
centers above the curved orange parent; this was corrected without relaxing the
clearance regression. The flame helper's optional offset preserves Ponyta's
canonical geometry byte-for-byte. Both horses now use dedicated pure builder
dispatch; the obsolete generic horse implementation has been removed while the
existing exact Rust species routes are preserved.

All fourteen candidate checks passed in 52.872 seconds. These cover recipe
parity, finite closed outward solids, face-aligned normals, height/floor/budget,
named anatomy, actual roots for body/neck/head/ears/horn/jaws/tongue/shoulders/
legs/hooves/tail/fire, three grounded hooves and one raised hoof, fitted vertices
and face/edge clearance, uncrossed curved parts, twelve empty aperture probes,
a filled-mouth negative and detached appendage negatives. Final front, oblique
and rear software renders each exited 0 at 800×824 and were inspected in full.
The installed mesh bytes match the reviewed candidate.

All fourteen installed canonical checks passed in 41.216 seconds. The fresh native build exited 0. The queued Rust integration test passed once
in 1.50 seconds. A direct repeat confirmed both complete final horse SHAs in
the executable and passed once in 3.81 seconds, with executable identity
`(217688309, 223956624, 1791810097391192065)` and both canonical model SHAs
stable across the test. This verifies bounded reusable models, not all-251
art quality.

An actual muted native image-target GPU battle capture exited 0 at 2000×1400
and was inspected in full. Rapidash's tall body, raised foreleg, red eye,
horn, mouth and mane/tail silhouette were visible with opposing Pidgeotto,
arena floor and battle HUD. Both final horse SHAs and the mute switch were
present in the executable; `CRYSTAL_PREVIEW_MUTE=1` was set for launch.
Executable identity `(217278815, 204526496, 1791808141760262307)` and both
model SHAs stayed stable across capture. These are static image-target checks;
live-window, browser, animation and gameplay behavior are separate scopes.

The [source review](fire-horses-source-review.md) records official references and
specific CC BY candidate listings. No external archive or rig was retrieved or
imported. This is original static geometry, with no animation claim. Task-owned ignored fire-horse verification scratch was removed after all
verification jobs completed.
