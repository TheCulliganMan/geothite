# Wartortle feather turtle sculpture

`tools/wartortle_sculpt.py` replaces the shared rounded turtle recipe with an
original individual Wartortle battle sculpture. The battle-species builder
uses the pure recipe and rejects the obsolete Wartortle primitive branch;
Blastoise now uses its separate original armored turtle source.
No third-party model or reference image was imported. The
[source review](turtle-source-review.md) did not yield an accessible validated
open rig. This model is original and static.

Blue-violet skin, a compact fitted pointed gaze, small brown irises and dark
pupils replace the protruding rose eye spheres. A broad low muzzle has a fitted
dark smile, two small fangs and tiny nostrils. Two pale blue volumetric feather
ears have rooted lower lobes and fitted ivory folds. A broad, closed two-curl
cloud tail sweeps behind the shell, with fitted spiral folds on both sides.
The substantial brown shell has a continuous ivory oval rim and fitted rear
scute seams. The domed pale belly has two curved horizontal joins and one
vertical join, all supported by actual stored skin. Bent forelimbs, broad paws,
six ivory hand claws, substantial thighs, short ankles and six blunt blue toes
have physical volume roots.

The first views caught the blue mantle breaking through the pale belly. The
mantle now has less forward depth and the belly clears it. An independent
stored-ray regression samples the visible breast so that this cannot silently
return. The first geometry run also caught a cloud outline self-intersection
and ankle depth extending below the feet. The tail now has an outer two-lobe
outline without the crossing hook; thinner ankle ends leave the feet grounded.
The pointed eye border was reduced to a fine fitted outline.

The final candidate passed twelve stored-mesh tests in 16.943 seconds. They
check closed connected outward solids, actual face normals, recipe parity,
height, floor, part counts and a 100,000-triangle budget. Stored-ray volume
checks cover body/shell/belly/rim/head/muzzle/tail roots, ear/lobe/fang roots,
complete arm/hand/claw chains and thigh/ankle/foot/toe roots. All fitted eye
layers, mouth, nostrils, shell and belly seams, ear folds and both tail faces
are checked at vertices, front triangle centers and edge midpoints against
actual stored parent triangles. Nonadjacent triangle checks cover tail,
spiral folds, ears, lobes, arms and ankles. Detached tail and ear negative
controls are rejected. The separate breast-clearance test requires the pale
belly to stay ahead of the blue mantle at more than 100 independent samples.

The installed model is byte-identical to the reviewed candidate: 68 parts,
75,868 triangles, height 1.10 and floor zero. Its SHA-256 is
`ab1853414c2a5e16a88ce4fda107a171fae6e8035a98ad75c2f0b2d6c0afa17b`.
Final front, oblique and rear software renders exited zero and were individually
inspected after the last geometry change. Canonical layout validation passed,
and storage validation passed for 575 current raw JSON model documents. The
three edited Python sources parsed and the relevant whitespace check passed.

All twelve canonical mesh tests passed again in 13.702 seconds. The fresh
native build exited zero after 29 minutes 10 seconds including shared-cache
waits. The full Wartortle and Blastoise source fingerprints and mute switch
were verified in the rebuilt executable before the serial captures. Its inode
216188333, size 182643952 and modification time 1791774173723679430 remained
unchanged across both captures, as did each source hash. Wartortle's capture
exited zero in 5.14 seconds and produced a 2000×1400 image with 40,225 colors.
The full frame was individually inspected: the actual turtle, feather ears,
pointed gaze, shell rim, plated belly, arms, claws and grounded feet are visible
alongside Pidgeotto, arena scenery and the HUD. The cloud tail is occluded in
this battle view; its complete shape is covered by the reviewed software views.
The all-251 Rust regression exited zero: its one matching voxel-view test
passed in 1.45 seconds, with 1,274 filtered tests; the other three packages
had no matching tests. Both current turtle full source fingerprints were
verified in the actual test executable. The run took 33 minutes 37 seconds
including shared-cache waits. Its binary predates the new Nidorina source
and does not verify that replacement.
Music and effects stay muted at the application level with
`CRYSTAL_PREVIEW_MUTE=1` before process creation. The full roster appearance
objective and separate live-window capture issue remain open. Verification
PNGs and logs stay ignored and are removed when the model's checks finish.
