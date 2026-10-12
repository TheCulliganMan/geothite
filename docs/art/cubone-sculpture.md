# Cubone sculpture

The canonical `cubone.mesh.json` uses original geometry from
`tools/cubone_sculpt.py`. Its 45 named closed parts contain 94,992 triangles,
with a grounded floor and normalized height 1.00. SHA-256:
`d89f8384a715d2b5d4688dfaa71151b639ffaf324615244cbed72a53b9a05685`.

A compact tan body, broad cream belly, heavy thighs and grounded paws sit
beneath an oversized white skull helmet. The helmet has two physically
recessed orbits with fitted tan linings, angular white eyes, dark pupils and
small glints. Its broad snout carries recessed triangular nostrils; curved
outward horns and pointed cheek projections complete the skull silhouette.
These are closed recessed surfaces. A completely hollow removable helmet
or through-holes are not claimed.

Rounded shoulders join bent arms and shaped palms. Three curved tan digits
on each hand and a separate gripping thumb retain actual roots. The bone
shaft intersects the holding palm/thumb and has four rooted bifurcated end
lobes. The paws have rounded white toe claws, and a short pointed tail joins
the torso. The juvenile torso was shortened after visual review, with inverse
scaling of its normals, preserving the larger helmet proportions.

The first candidate failed an uncrossed-finger check and the existing
100,000-triangle budget. A wider tapered finger bend and less unnecessary
socket contour sampling fixed those failures. Fitted belly/socket shading
still showed radial seams; increasing the crease angle had no visible effect.
Local cap normals now follow the actual parent mesh's interpolated normals,
with face-supported sidewalls and outward cap shading at steep recess edges.
The first interpolation failed an outward-normal check, which was corrected
without relaxing the regression. Renderer code and unrelated models are
unchanged.

All thirteen final candidate checks passed in 36.772 seconds. They cover
recipe parity, finite closed outward surfaces, face-aligned normals, budget/
height/floor and named anatomy, actual roots for neck/head/skull/snout/horns/
cheeks/shoulders/arms/hands/digits/thumb/haunches/paws/claws/tail, the actual
club grip and four rooted lobes, fitted vertices and face/edge clearances,
uncrossed curved appendages, sixteen empty orbital/nostril recess probes,
filled-recess negatives and detached appendage/club negatives.

Final oblique, front and rear software renders each exited 0 at 800×824 and
were individually inspected after the final normal correction. Installed bytes
match that candidate. The pure builder has a dedicated Cubone dispatch through
the existing exact Rust species route. The legacy function rejects both identities now that Marowak also has a
separate original adult recipe.

All thirteen installed canonical checks passed in 33.094 seconds. The fresh
native build exited 0, and a final-hash GPU battle capture with explicit
`CRYSTAL_PREVIEW_MUTE=1` exited 0 at 2000×1400 and was inspected in full.
The frozen renderer executable was `(217792530, 207350416,
1791812562137305949)` (inode, bytes, modification nanoseconds), contained the
full canonical SHA above and the mute switch, and stayed stable through the
capture. Cubone is small at its actual battle presentation size, with the
skull, held bone, tan body and grounded paws visible in the arena.

The union-feature Rust integration exited 0 with the exact all-species
model check passing in 3.10 seconds. A direct exact rerun passed in 3.41
seconds against frozen executable `(217798650, 229537600,
1791812807399036813)`, which contained the final Cubone hash. Both executable
identity and canonical source hash stayed stable through the rerun. This
checks model loading and bounded geometry; it is not approval of every
species' art. Software views and the image-target battle capture do not
establish ordinary live-window, browser, animation or gameplay behavior.
The later Marowak integration also passed with both final hashes in its
frozen executable. All bone-pair verification jobs are terminal, and
task-owned ignored scratch was removed after the final proofs.

The [source review](bone-pair-source-review.md) records official anatomy and
specific external model/rig candidates. No external archive, mesh, texture,
rig or clip was retrieved or imported. This is original static geometry.
