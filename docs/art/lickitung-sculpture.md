# Lickitung sculpture

The canonical `lickitung.mesh.json` uses original geometry from
`tools/lickitung_sculpt.py`: 35 named closed parts, 108,716 triangles,
grounded floor and normalized height 1.18. SHA-256:
`1e9f8516822bee309daa3163b94c32c312500ace0ebbb3070e17fe378c07d469`.

A stocky pink torso has four separate fitted cream belly bands and rooted
heavy haunches, each with a fitted cream ring and pink center. Broad feet
carry white toe tips. Rounded shoulders, broad short forelimbs, blunt palms
and small white hand tips retain actual volume roots. A broad pink tail
curves upward behind the body with a tapered end. Its initial pinched turn
was rejected visually and replaced by a smoother ascending curve.

A rounded pink head has small fitted dark eyes, white glints, subtle nostril
marks and an actual localized recess beneath a broad fitted dark oral lining.
The long pink tongue is separately modeled with a flattened section and a
shallow fitted center marking. Closed recessed geometry is used; a hollow
head or through-hole is not claimed. A reused surface helper rebuilds its
part, so the head palette is explicitly restored after that rebuild. This
corrected the first candidate's mismatched head color without changing
Cubone, Marowak or their helpers.

The first tongue looked supported in the software view but failed an actual
volume-root check against the head. Its root now extends into the head and
passes the check. A face/edge clearance check also rejected the tongue's
center marking; increasing its surface offset corrected the failure without
relaxing the clearance threshold. All ten final candidate checks passed in
35.632 seconds. They cover closed outward surfaces, actual face-aligned
normals, recipe parity, palette, height/floor/budget and anatomy counts,
head/neck/tongue/tail roots, all limb/palm/nail/haunch/foot/toe roots, fitted
vertices and face/edge clearance, uncrossed arms/tongue/tail, broad tongue
and heavy tail dimensions, and detached tongue/appendage negatives.

Final front, oblique and rear software renders each exited 0 at 800×824 and
were individually inspected after the final tongue-root and marking changes.
Canonical bytes match the reviewed candidate. The pure builder now uses the
original dedicated recipe, and its previous generic implementation rejects
Lickitung. The existing exact Rust species route is preserved. All ten installed canonical checks passed in 23.016 seconds. The fresh
native build exited 0. A battle capture with explicit
`CRYSTAL_PREVIEW_MUTE=1` exited 0 at 2000×1400 and was inspected in full:
the broad tongue, cream bands/rings, white tips and upward tail read on the
actual arena floor. Frozen renderer `(217899252, 209925056,
1791815178450414724)` (inode, bytes, modification nanoseconds) contained
the full final canonical hash and mute switch; executable identity and
source hash stayed stable through the capture.

The union-feature Rust integration exited 0, with its exact all-species
model check passing in 3.65 seconds. A direct exact rerun passed in 3.76
seconds against frozen `crystal_voxel_view-dc3731eb4a201fc3` executable
`(217922395, 234722816, 1791815375146727944)`, containing the final canonical
hash. Both executable identity and source hash stayed stable through the
rerun. This proves model loading and bounded geometry, not every species'
art. An image-target capture does not establish ordinary live-window,
browser, animation or gameplay behavior.

The [source review](lickitung-source-review.md) records official anatomy and
specific external model candidates. No external archive, mesh, texture, rig
or clip was retrieved or imported. This is original static geometry.
Software renders do not establish ordinary live-window, browser, animation
or gameplay behavior. All Lickitung verification processes are terminal. Task-owned ignored
scratch was removed after the final proofs.
