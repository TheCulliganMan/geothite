# Blastoise armored turtle sculpture with hollow cannons

`tools/blastoise_sculpt.py` replaces Blastoise's shared primitive turtle recipe
with an original individual sculpture. The battle-species builder dispatches
this pure recipe. Both obsolete Wartortle and Blastoise primitive branches
reject execution. The [source review](turtle-source-review.md) did not yield an
accessible validated open rig; no third-party geometry or reference pixels
were imported. This model is original and static.

A substantial brown domed shell, continuous ivory rim and fitted scute seams
frame a pale plated belly. The belly clears the blue mantle beneath it.
Blastoise has a broad angular blue cranium, compact pointed ears with fitted
dark insets, narrow fitted ivory eyes, restrained brown irises and small dark
pupils. A broad low blue muzzle has tiny nostrils and a fitted dark smile over
a cream chin. Muscular forelimbs and fists carry six ivory hand claws. Strong
thighs, short ankles, broad grounded feet and six ivory foot hooks complete
the stance. A short tapered blue tail emerges from the rear shell.

Each cannon has a blue shoulder mount and continuous closed silver geometry
that turns inward at the muzzle. The inner wall stops well behind the rim,
leaving actual recessed empty space. A dark floor seats in the rear of the
bore. A separate closed dark annular lining roots in the steel wall and
preserves the central opening. It makes the interior readable in oblique
views without a solid black plug at the muzzle.

The first candidate's bore checks passed, but topology checks caught tiny
polar triangles in the thin dark floor. Increasing its real depth and reducing
its polar sampling corrected the geometry without changing the test threshold.
The initial software views also showed overly light inner walls. The rooted
annular lining corrected that visual defect; its cavity remains tested.

Thirteen final candidate tests passed in 12.509 seconds. They inspect stored
triangles for closed connected outward solids, face normals, recipe parity,
height, floor, feature counts and a 110,000-triangle budget. Stored-ray volume
checks cover body/shell/belly/rim/head/muzzle/chin/tail roots, shoulder mounts,
barrels, linings, ears, complete arm/hand/claw chains and thigh/ankle/foot/toe/
claw chains. Fitted eyes, ear insets, mouth, nostrils and shell/belly seams must
follow their actual parent at vertices, front triangle centers and edges.
Independent samples require the belly to clear the blue mantle. Nonadjacent
triangle checks cover barrels, linings, ears, tail, arms and ankles.

The cannon oracle derives each mouth center from the actual stored muzzle
ring, rotates stored triangles into the expected bore direction and pairs
ray crossings into occupied intervals. Both central cavities must stay empty
through the tested depth range; their floors must remain recessed and have a
real overlap with the rear barrel bed. A filled-barrel negative control and
a detached-cannon negative control must fail. This checks real geometry,
rather than treating a dark surface or a barrel-shaped bounding box as hollow.

The installed source is byte-identical to the reviewed candidate: 73 parts,
73,188 triangles, height 1.18 and floor zero. Its SHA-256 is
`174cf2ea9ce6e0e0cd263ad59acfc8a8906f9ab456935b988527102baf73e3ef`.
Final front, oblique and rear software images exited zero and were individually
inspected after the final geometry change. Canonical layout validation passed;
storage validation passed for the current shared tree of 575 raw JSON model
documents. The new Python source and tests parsed, and the builder whitespace
check passed.

All thirteen canonical tests passed again in 15.040 seconds. The fresh native
build exited zero after 29 minutes 10 seconds including shared-cache waits.
Both current turtle source fingerprints and the mute switch were found in
the rebuilt executable before capture. Blastoise's muted capture exited zero
in 2.82 seconds and produced a 2000×1400 image with 49,974 colors. The full
frame was individually inspected: the actual armored turtle, both dark cannon
openings and silver barrels, shell rim, plated belly, strong limbs, claws and
grounded feet are visible with Pidgeotto, arena scenery and the HUD. The rear
shell and short tail are covered by complementary software views. The source
hash and executable identity (inode 216188333, size 182643952, modification
time 1791774173723679430) stayed unchanged across both serial turtle captures.
The all-251 Rust regression exited zero: its one matching voxel-view test
passed in 1.45 seconds, with 1,274 filtered tests. The other three packages
had no matching tests. Both current turtle full source fingerprints were
verified in the actual test executable. The run took 33 minutes 37 seconds
including shared-cache waits. Its binary lacks the new Nidorina fingerprint
and does not verify that later replacement.

All trials set `CRYSTAL_PREVIEW_MUTE=1` before process creation for both music
and effects. The native backend sets zero sink volume before appending audio
and preserves zero volume on music refresh. The full roster appearance goal
and separate live-window capture issue remain open. Images and logs are
ignored verification scratch and are removed after the model's checks finish.
