# Distinct small Nidoran sculptures

`tools/nidoran_sculpt.py` replaces the shared rounded family variants with two
original small poison-pin creatures. The pure builder dispatches each identity
and rejects the obsolete primitive branches. Existing dedicated Rust registry
entries preserve their individual battle routing. The [source review](nidoran-source-review.md)
records individually inspected official artwork, the baseline rendering and
specific unsuccessful rig access. No external geometry or reference pixels
were imported. Both new models are original static geometry.

The female has pale periwinkle skin, a compact tall pear body and rounded
cranium, a low muzzle, a small upright horn and six rooted cheek barbs. Broad
outward ears contain fitted navy fields and teal lower lobes. Dark navy spots
follow the actual flank and haunch surfaces. The male has a low crouching
purple barrel, a lowered wedge head, a forward nasal horn, much larger upright
ragged teal-lined ears and pronounced rounded hindquarters. Dark purple spots
fit its flanks and rear quarters. Both have pointed fitted ivory eyes with
orange irises, dark pupils and tiny glints, nostrils, shallow smiles, small
ivory teeth, four planted paw chains, eight ivory toes, three rooted dorsal
barbs and short rear points. Their body and ear proportions are independently
checked so they cannot regress to palette variants of one silhouette.

The initial attachment check caught one female lower cheek barb outside the
skull. Moving its root inward into the real head volume corrected that defect
without relaxing the overlap threshold. Final front, oblique and rear software
sheets exited zero and both models were inspected in each complete sheet
after the last geometry change.

Ten final candidate tests passed in 19.514 seconds. They check closed connected
outward surfaces, face normals, recipe parity, floor zero, individual height,
unique part names, anatomical counts and a 100,000-triangle budget per model.
Stored-ray checks require physical roots for heads, muzzles, horns, ears,
teeth, cheek barbs, tails, spines, all four leg chains and every toe. Fitted
facial layers, ear fields/folds and side markings must follow actual parent
triangles at vertices, front face centers and edge midpoints. Nonadjacent face
checks cover ears, folds, horns, ankles, cheek barbs, tails and spines.
Deliberately detached horn, ear and tail controls are rejected. A separate
proportion check requires the female's tall compact body and the male's low
barrel and larger upright ears.

Both installed sources are byte-identical to the reviewed candidates:

| Species | Parts | Triangles | Height | SHA-256 |
| --- | ---: | ---: | ---: | --- |
| Nidoran female | 65 | 53,856 | 0.78 | `09e9590d2ba33e10e5549a837eeac75364add63c15baf8fbaaa586c508af70ec` |
| Nidoran male | 53 | 47,152 | 0.80 | `0351ecf0b36bb133daff27bb6db1e4a41ec936db6c57c18d0fb3ae6b1b276d0e` |

The recipe, tests and builder parsed. Canonical model layout and storage
validation of 575 raw JSON documents passed; the relevant whitespace check
passed. All ten installed-model tests passed again in 14.246 seconds.

The fresh native build retry exited zero after 12 minutes 53 seconds including
shared-cache waits. The full source fingerprints for Nidorina, Nidorino and both
small Nidoran, plus the mute switch, were verified in the actual rebuilt
executable before capture. All four serial captures used one unchanged binary:
inode 216262677, size 185981536, modification time 1791775675758159751. Each
source hash was checked before and after its capture. Music and effects were
muted by setting `CRYSTAL_PREVIEW_MUTE=1` before process creation. The inspected
native backend sets sink volume to zero before audio append and keeps music
refresh at zero.

Although the build was queued before the small pair was installed, both final
small-source fingerprints are present in its actual executable. The female
capture exited zero in 3.13 seconds and produced a 2000×1400 image with 36,515
colors. The male capture exited zero in 3.48 seconds and produced a 2000×1400
image with 36,304 colors. Both full frames were individually inspected: the
actual small creatures are visible at their small species scale with Pidgeotto,
arena scenery and HUD. The female's broad ears, pale upright body, cheek barbs
and spots differ visibly from the male's purple crouch, large upright teal-lined
ears, horn and rear quarters. Detailed facial and complete rear geometry are
covered by the complementary full software views. The all-251 Rust model regression passed in 1.31 seconds with zero failures (1,280 other voxel tests filtered out). All four current Nidoran-family source fingerprints were verified in its actual test executable.
The full roster goal and separate live-window capture issue remain open.
Task-generated review images and logs were removed after verification.
