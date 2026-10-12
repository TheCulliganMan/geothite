# Original Onix sculpture

`tools/onix_sculpt.py` authors a long arch of thirteen irregular grey boulders,
a wedge-shaped head with fitted narrow ivory eyes, pointed nose keel, dark
angular lower jaw and oral chamber, and one narrow forehead horn. Twelve
buried connectors root into adjacent boulders. Every adjacent boulder also
has actual occupied-volume overlap; early visible tail rods were eliminated
by bringing the four small tail rocks together. Three tail boulders provide
floor support within 0.003 authored units of zero.

The canonical model has 36 closed parts and 9,848 triangles, floor zero and height
1.72. SHA-256 is
`06c601ffc43ee41a64babcdff501c7cefec859fbdb5458448dfe13ff6fbded9f`.
All eleven canonical checks passed in 1.469 seconds and the eleven candidate
checks passed in 2.289 seconds. They cover closed connected
outward surfaces, normals, recipe parity, bounds/budget/anatomy counts,
adjacent-rock and joint occupied-volume roots, head/nose/jaw/chamber/horn
roots, three ground supports, fitted eye vertices/face centers/edge midpoints,
nonadjacent horn/joint crossings, detached negatives and the Rust body-reference
measurement against the authored normalized centerline. Front, oblique and
rear 800×824 software images exited zero and were individually inspected
following the final tail changes. AST checks passed.

The [source review](rock-serpents-source-review.md) records inspected official
art and specific model candidates. One Onix listing advertises an animation
with a noncommercial license, but its archive endpoint returned HTTP 401.
No mesh, skin, bones or clip was available for inspection or import. This
sculpture is original static geometry.

The sculpture is installed in `actor_props/big_onix.mesh.json`, preserving the
existing shared Onix registry route. The actor-prop builder dispatches to the
dedicated recipe and excludes Onix from its generic Blender loop. The normalized
tail-to-head centerline is 2.8677406192405615, used by battle layout as
`2.867_740_6 / 1.72`. Actual mesh bounds replace the old pose's test fixtures.

The native build completed in 8m23s. Before capture, the executable was checked
for the full canonical SHA and mute switch. With `CRYSTAL_PREVIEW_MUTE=1` and
the known-good local pack, the Onix image-target capture exited zero. The full
2000×1400 GPU image was inspected: Onix's complete arched silhouette, fitted
gaze, horn, tapering rock chain and grounded tail are visible alongside
Pidgeotto, the arena and production battle HUD. The tail supports meet the
arena floor; the earlier apparent clearance is absent in this capture.
Executable identity remained `(216873933, 197821552, 1791785620957093241)`
(inode, bytes, mtime nanoseconds), and the canonical hash remained unchanged.

The test executable was independently checked for the canonical SHA. The exact
all-251 visible-species test passed in 1.35 seconds, and all 18 battle-layout
tests passed. Its identity remained
`(216943556, 210003344, 1791787580926601040)` and the mesh hash stayed unchanged.
These prove this native image-target and layout path, not a live window,
browser, animation or interactive gameplay. Temporary review output is removed
after inspection.
