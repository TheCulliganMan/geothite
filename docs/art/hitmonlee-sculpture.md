# Original headless Hitmonlee sculpture

The canonical Hitmonlee model now comes from `tools/hitmonlee_sculpt.py`.
It replaces the primitive fighter with one continuous headless tan pear, fitted
sloping black eyes, small ivory pupils, cream nine-rib shins and five-rib wrists,
three curled tan fingers per hand, grounded tan feet, six ivory toe claws and
fitted gold sole marks. Each spring limb is one closed continuous ribbed solid,
not separate floating rings. The existing dedicated Rust battle registry entry
is unchanged. The builder dispatches to the pure recipe and rejects the old
Hitmonlee branch while retaining the other three fighters for later work.

This is original static geometry. The [source review](hitmon-source-review.md)
records the specific external models checked, including an advertised animated
pack containing Hitmonchan whose download requires access that was unavailable.
No third-party mesh, skin, bones or animation was retrieved or imported.

The model has 33 closed parts and 43,116 triangles, with floor zero and height
1.24. Its raw canonical SHA-256 is
`7345f5b394cd12eb496187abf9432c7d4b4315f4f6213e8ebd5d50a58a6109f8`.

All nine candidate stored-mesh tests passed in 6.403 seconds. They check
connected closed solids, outward faces and normals, parity, anatomy counts,
budget and floor contact. Actual occupied-volume intersections attach both
hips, shins and feet, all six claws, both arms, wrists and palms, and all six
fingers. Fitted black eyes, pupils and gold side markings follow actual stored
parent triangles at vertices, face centers and edge midpoints. Curved torso,
hips, limbs, springs, fingers and claws are checked for nonadjacent face
crossings. Detached springs, fingers and feet must fail attachment checks.

The first candidates exposed a shin/foot gap, a folded upper arm and curled
finger intersections. The shin was extended into the foot, the upper-arm path
was broadened and the finger roots/curls were corrected. The geometry checks
were retained. Final front, oblique and rear 800×824 software images all
rendered successfully and were individually inspected after the corrections.
They show the continuous headless silhouette, grounded spring limbs and
three-finger hands; the rear view also exposes the supported wrist roots.

All nine canonical tests passed in 5.562 seconds. AST, model layout and
575-file raw JSON storage checks passed. The native build completed successfully after the earlier shared footwear
syntax failure was corrected. The final raw mesh fingerprint and the mute
switch were present in the actual rebuilt executable. One unchanged executable
(inode 216441911, size 189531952, mtime_ns 1791779637085927116) produced
all four adult/fighter production GPU captures with exit zero. This model's
2000×1400 capture contained 41,868 colors and was individually inspected in
full frame: its authored anatomy appeared with Pidgeotto, the arena and battle
HUD. Executable identity and all four source fingerprints were verified again
after capture. All previews launched with `CRYSTAL_PREVIEW_MUTE=1`; system
volume was unchanged.

The exact all-251 Rust model regression passed directly in 2.19 seconds
(one actual test, zero failures, 1,293 filtered), with all four final mesh
fingerprints verified in the frozen test executable before and after the run.
This proves the model load path and this GPU image preview; the separate
live-window issue remains unresolved. Completed ignored review scratch was
removed after recording these results.
