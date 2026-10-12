# Original Magmar spitfire sculpture

`tools/magmar_sculpt.py` generates the canonical original static mesh. A broad
orange pear body carries a yellow belly field with upward flame-shaped edges
fitted to stored body triangles. Yellow round shoulders and haunches support
orange arms and broad clawed feet. Four closed jagged orange cuffs are rooted
in their forearms; hands have three ivory claws each and feet have two.
Dark blue collar and lower shins retain the reference's contrasting bands.

The broad orange head has two heavy forehead lobes, narrow fitted ivory eyes
and tiny dark pupils. Separated cream duck-bill jaws expose a dark oral chamber
and rooted pink tongue. Two closed flowing yellow crown flames have fitted
orange base glows on both faces. Three rooted orange dorsal points and a long
curved yellow tail lead to a closed orange flame with fitted yellow heart
layers on both faces. No fire pyramids or unsupported planar spikes remain.

The first belly layer test found a face center dipping into its body. Raising
the shallow layer's clearance corrected that geometry. Visual review also
exposed ear-clipped surface shading streaks. Cap normals now follow the body
curvature and are constrained to agree with their actual stored face normals;
side walls retain their own normals. The unchanged shading regression caught
one thin cap triangle before that constraint was added. Raising the head and
widening the collar then exposed the blue neck band that had been hidden by
head/body overlap. Tests were not weakened.

The mesh has 55 closed parts and 77,108 triangles, floor zero and height 1.30.
Its raw canonical SHA-256 is
`956ada452f260111200c47656c20ab2e2b4ee655c5f84cdaa5d750b0cc038265`.

All twelve final candidate checks passed in 21.617 seconds. They cover closed
connected outward solids, normals, recipe parity, dimensions, budgets, distinct
anatomy, occupied-volume roots throughout head/lobes/collar/jaws/tongue,
shoulders/arms/cuffs/hands/claws, thighs/shins/feet/toes, spines/tail/flames,
fitted layers at vertices, face centers and edge midpoints, nonadjacent face
crossings, clear bill cavity and deliberately filled/detached negatives.
The final front, oblique and rear 800×824 software renders exited zero and
were individually inspected after the final collar and shading changes.
AST, canonical layout and 575-file raw JSON storage validation passed.
The builder uses the pure recipe and rejects the old generic branch.
Existing dedicated Rust registry entries remain unchanged.

The [source review](fire-pair-source-review.md) records inspected official art
and specific CC BY 4.0 model candidates whose archive endpoints returned 401.
No external mesh, skin, rig or animation was imported. This is original static
geometry.

All twelve canonical checks passed in 21.310 seconds.

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
