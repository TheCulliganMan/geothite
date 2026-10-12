# Original Magby live-coal sculpture

`tools/magby_sculpt.py` generates the canonical original static mesh. A compact
red pear body carries a yellow pointed belly patch fitted to stored triangles.
A dark collar bridges the body and broad face. Five deeply embedded rounded
lobes form a low cloud-like crown. Tall ivory eyes, small vertical pupils and
tiny eye sparks follow the actual face without protruding sphere eyes.
Raised curved red arms end in three ivory claws per hand. Red shins support
small grounded feet with two claws each. The pointed curved red tail has no
flame, matching the inspected primary reference.

The yellow bill is a closed annular tapered surface with a real open front
aperture and a recessed dark oral backing. Stored geometry checks sample five
positions against all parts, ensuring the front recess remains unoccupied,
and detect a deliberately filled negative. This verifies the bill recess,
not a tunnel through the head.

The mesh contains 38 closed parts and 49,032 triangles, floor zero and height
0.77. Its raw canonical SHA-256 is
`33e9109bb2ab8062bfbf72345aad43de7d09b1ffac3e8806de7860ccfc3eb4c9`.

All ten candidate checks passed in 7.584 seconds and all ten canonical checks
passed in 7.993 seconds. They cover closed connected outward surfaces,
normals, recipe parity, distinct anatomy, dimensions, budget, actual volume
roots for crown/collar/head, bill/backing, tail, arms/hands/claws and legs/feet,
fitted-layer vertices, face centers and edge midpoints, nonadjacent face
crossings, open bill recess and filled/detached negative controls.
Final front, oblique and rear 800×824 software renders all exited zero and
were individually inspected. AST, canonical model layout and 575-file raw
JSON validation passed. The builder uses the pure recipe and rejects its
old generic branch; the dedicated Rust registry entry is unchanged.

The [source review](fire-pair-source-review.md) records inspected primary art
and specific CC BY 4.0 candidates whose archive endpoints returned HTTP 401.
No external mesh, skin, rig or animation was imported.

The fresh native build and all-251 Cargo regression completed with exit zero.
Both final fire-pair fingerprints and `CRYSTAL_PREVIEW_MUTE` were verified
inside the actual native executable. One unchanged executable (inode 216677011,
size 197,640,064, mtime_ns 1791784677994592803) produced two 2000×1400
production GPU battle captures, each with exit zero. Both full images were
individually inspected: the requested anatomy, Pidgeotto, arena and battle HUD
are visible. Every launch used `CRYSTAL_PREVIEW_MUTE=1` before playback.

The exact all-251 Rust model regression then passed in 2.09 seconds, one test,
zero failures and 1,314 filtered tests, after both final fingerprints were
verified in the actual test executable. Its identity remained unchanged:
inode 216655376, size 209,444,464, mtime_ns 1791784560671332976. Canonical
sources also remained unchanged throughout captures and tests.
This proves the installed pair renders in the native GPU image target; the
separate live-window issue, browser and animation reviews remain outside
this evidence. Completed ignored review output was removed.
