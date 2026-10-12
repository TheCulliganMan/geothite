# Original Jynx and Smoochum sculptures

The canonical models come from `tools/ice_dancers_sculpt.py`. The two recipes
retain distinct silhouettes rather than scaling a shared generic humanoid.

Jynx has a continuous flared seven-fold red gown with gold bodice lobes,
fitted red insets and a supported central gold bridge. A broad purple head
carries fitted ivory eyes, tiny dark pupils and a rooted pink oval kiss rim.
Arched center-parted blonde forelocks meet long swept curtains and four
layered tips; five subtle fitted strand contours define the front and back.
Bent white sleeves lead to open purple palms with five rooted digits per hand.

Smoochum has a compact pink pear body, cream chest collar and fitted pointed
bib, short cream arms with small three-finger hands, and supported cream toes.
Its broad pink face carries fitted ivory eyes, broad dark pupils, green lower
irises, eye sparks and a pink kiss rim. A short golden bob has curved side
locks, a continuous notched fringe and four rooted leaf-like tuft points. The
head was lowered toward the collar and the throat shortened after visual
review exposed adult-like neck proportions.

Both kiss rims are closed annular surfaces rooted into their actual head
geometry. Their centers remain open above dark fitted mouth patches, creating
a real recess within the rim rather than a filled pink mouth sphere. The
geometry check samples those stored openings, checks root volume and recess
depth, and detects a deliberately filled negative. This checks the lip recess;
it does not claim a tunnel through the head.

These are original static meshes. The [source review](ice-dancers-source-review.md)
records individually inspected official references and specific CC BY 4.0
candidates whose archive endpoints returned 401. No external rig, skin or
animation was imported.

Jynx contains 45 closed parts and 79,520 triangles, floor zero and height 1.28.
Its raw canonical SHA-256 is
`a2c8b54200217413d29a698576a95e3ba11bff83144c864d2f793f6c3c2985cd`.
Smoochum contains 40 closed parts and 82,896 triangles, floor zero and height
0.78. Its raw canonical SHA-256 is
`a8d3bbb802e2d66d5131c7960fcb4f4a0b5abc871c5037bb9efd39618770993b`.

All eleven final candidate checks passed in 41.445 seconds. They verify closed
connected outward solids, normals, parity, budgets, distinct anatomy and
actual occupied-volume roots throughout clothes, neck/head, hair/crown/locks,
arms/hands/digits and baby legs/toes. Fitted eyes, irises, eye sparks, bodice
insets, hair strands, mouth patches and cream bib follow stored parent
triangles at vertices, face centers and edge midpoints. Curved gown, hair,
limbs, digits and lips reject nonadjacent triangle crossings. Detached crown,
lips and finger negatives fail attachment.

The initial checks found a disconnected Jynx hair tip and a crossed baby
finger. Extending the tip root into its curtain and easing the finger bend
fixed those failures. Additional visual review refined the center part,
shortened the baby's throat and buried its side-lock root within the cap.
Crossing checks then caught overly tight forelock/side-lock curves; wider
paths and tapering widths corrected the geometry without relaxing the checks.
Final paired front, oblique and rear 1600×824 software images all exited zero
and were individually inspected after the final geometry changes.

All eleven canonical checks passed in 42.080 seconds. AST, canonical layout and 575-file
raw JSON storage validation passed. The builder dispatches both species to the
pure recipes and rejects their old generic branch; existing dedicated Rust
registry entries remain unchanged.

The fresh native build finished with exit zero. Its actual executable contained
all four final Hitmontop, Tyrogue, Jynx and Smoochum SHA-256 fingerprints and
`CRYSTAL_PREVIEW_MUTE`. One unchanged executable (inode 216555973, size
193,759,392, mtime_ns 1791782582913840981) produced four 2000×1400 production
GPU battle captures, each with exit zero. All four full images were individually
inspected: species anatomy, Pidgeotto, arena and battle HUD are visible.
Music and effects were muted before playback with `CRYSTAL_PREVIEW_MUTE=1`.

The exact all-251 Rust model regression passed in 2.57 seconds, one test and
zero failures, with all four final fingerprints verified in the actual test
executable before and after. That executable remained unchanged (inode
216610276, size 201,634,144, mtime_ns 1791782840851990349). The Cargo run also
completed successfully. This proves these installed meshes render in the GPU
image target; the separate live-window issue, browser and animation review
remain outside this evidence. Completed ignored review output was removed.
