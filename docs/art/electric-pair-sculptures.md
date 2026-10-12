# Original Electabuzz and Elekid sculptures

`tools/electric_pair_sculpt.py` generates both canonical meshes. Electabuzz
has a broad low head, sharp fitted ivory eyes, black chevron, pointed twin
scalp tufts, ball-tipped antennae, separated fang-bearing jaws and a pink
rooted tongue. A substantial yellow body carries a large chest bolt, flank
sweeps and a thick curved rear stripe fitted to actual parent triangles.
Asymmetric bent arms end in clenched knuckles and thumbs with wrist fur.
Striped shins support broad feet with two front claws and one heel claw each.
A long curved, striped tail ends in a rounded tip.

Elekid has one continuous yellow egg body, two rounded yellow contact blades
with actual through holes, fitted angular eyes, pointed grin, lower bolt and
closed black waist wrap. Slim arm roots lead to broad striped forearms and
three-clawed fists. Short legs end in grounded yellow feet.

Black arm, leg and tail wraps are closed solids intersecting their actual
yellow parent volume. Elekid's blade holes are annular extrusions, without
black caps or filled decals. Stored-mesh checks sample five rays through each
opening against every part, and detect a deliberately filled negative.
Electabuzz's mouth check samples nine cavity positions between the actual
stored jaws and detects a deliberately filled negative.

These are original static meshes. The [source review](electric-pair-source-review.md)
records inspected official artworks and a back sprite plus specific primary
model metadata. Neither checked listing exposed an accessible openly licensed
archive. No external mesh, rig, skin or animation was imported.

Electabuzz has 70 closed parts, 113,628 triangles, height 1.28 and floor zero.
Its raw canonical SHA-256 is
`b327eede6f8cbcc5e70c77b89f758cd749b5dbf638164d7a15efd1c943dfd338`.
Elekid has 32 closed parts, 39,784 triangles, height 0.91 and floor zero.
Its raw canonical SHA-256 is
`45ef6c72ba9584f97a64c316d426767ea1ccf9b7fe444a0481f4b53ed0fe3710`.

All thirteen final candidate checks passed in 28.564 seconds and all thirteen
canonical checks passed in 22.834 seconds. They verify closed connected outward
solids, normals, recipe parity, distinct anatomy, bounds, budgets, actual
occupied-volume roots throughout anatomy and wraps, open holes and jaw cavity,
fitted-layer vertices, face centers and edge midpoints, nonadjacent triangle
crossings and detached negative controls. Rear fitting is tested after rotating
both the stored patch and parent into the same front-oriented frame.

Final front, oblique and rear paired 1600×824 software images each exited zero
and were fully inspected after the rear stripe and heel claws were added.
AST, canonical layout and 575-file raw JSON storage validation passed. The
builder dispatches both species to their pure recipes and rejects their old
generic branch. Existing dedicated Rust registry entries remain unchanged.

The first native build and all-251 Rust attempts exited 101 on an unresolved
`park_flowers` call in shared flowerbed code. Exposing the existing module and
its helper at the enclosing mesh boundary, and importing it in the caller,
fixed that compile blocker without duplicating geometry. The native retry
completed successfully in 2 minutes 51 seconds.

The actual native executable contained both final fingerprints and
`CRYSTAL_PREVIEW_MUTE`. Its identity remained unchanged before and after both
captures: inode 216612378, size 195,955,712, mtime_ns 1791783174232157031.
Both production GPU battle images exited zero, measured 2000×1400 and were
individually inspected in full. The adult's face, jaws, stripe wraps and claws,
and the baby's silhouette, blades and arm claws render with the arena,
Pidgeotto and battle HUD. Sources also remained unchanged. Both launches used
`CRYSTAL_PREVIEW_MUTE=1` before playback.

The all-251 Cargo retry completed successfully with one model test passing in
2.56 seconds. The exact regression was then repeated against its actual test
executable after both final fingerprints were verified. One test passed in
3.16 seconds, zero failures and 1,309 filtered tests. Test executable identity
remained unchanged before and after: inode 216629101, size 206,274,032,
mtime_ns 1791783448397555977. Both canonical sources also remained unchanged.
An earlier direct attempt correctly refused coverage while the old binary
still lacked the electric-pair fingerprint.

This evidence covers the native GPU image target and stored-model Rust
regression; the separate live-window issue, browser and animation review
remain open. Completed ignored review output was removed.
