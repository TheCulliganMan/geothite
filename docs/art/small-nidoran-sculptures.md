# Nidoran female and male sculptures and skins

The current assets are `nidoran_f.glb` and `nidoran_m.glb`, exported by
`tools/nidoran_glb.py` from the original `tools/nidoran_sculpt.py` recipes.
This section supersedes the static export status recorded below.

| Species | Logical solids | Runtime primitives | Vertices | Triangles | Joints | Accessors | Bytes | Reference height |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Nidoran♀ | 65 | 40 | 19,256 | 31,768 | 11 | 184 | 711,400 | 0.78 |
| Nidoran♂ | 53 | 36 | 16,782 | 28,160 | 11 | 173 | 633,084 | 0.80 |

The retired static meshes contained 32,890/28,469 vertices and
53,856/47,152 triangles. Removing recursive subdivision inside planar pinnae
retains their complete authored notched outlines. Thin markings are now clipped
against actual stored parent triangles, including the concave mouth outline,
rather than spanning curved faces with raised chords. The resulting meshes
retain the distinct tall pale female pear and low purple male barrel, broad
versus upright ears, separate horn shapes, cheek barbs, spines, eight claws,
flank and haunch spots, layered gazes and short pointed tails. A 0.0003-unit
adjustment to the male mouth contour removes tiny quantized clipping slivers.
The established normalized heights and physical-size path remain unchanged.

Both standard skins have an identity floor root, body, coherent head,
independent left/right pinnae, four supported ankle fields and two monotone
rearward tail carriers. All four complete paws and their eight claws stay
fixed during battle. Haunches, spots and back spines follow the body; horns,
snouts, teeth, cheek barbs and fitted facial layers follow the head. Ear
fields share their respective pinna carrier. Idle lasts 3.2 seconds; attack
and hit last one second, with rotation-only keys and neutral endpoints.
Production source cues continue to own playback timing. Runtime packing
preserves each logical solid's exact stored buffers and original source order.
Claw groups and haunch spots remain separate per limb, so the production gait
never assigns a combined cross-body marking or nail set to one leg. The walking
regression checks every support's joint tree and each complete claw set's foot
carrier in addition to ground contact and bounds.

The 20,000-vertex, 40,000-triangle, 64-runtime-part, 64-joint,
256-accessor and 1 MiB limits remain unchanged. No external mesh, rig, clip
or reference image was imported. Obsolete static JSON assets are removed;
normal Rust registry, GPU, world-walking, builder and reader routes now use
the canonical GLBs.

Stored geometry pins:

- Female logical: `17ab9379b5525c666b9c429dcd96f5ce773845daa2836ed30fa3c5896229232d`.
- Female packed: `8bd3d7aefc6dd2bb59d701de719be620443ce0c1fc4aaaeb4bcdecc0a9c03802`.
- Male logical: `d1a2dbb46f39b1285f5a1aace952caa2c466c8ff40a735e74fb02efd5b043198`.
- Male packed: `5d11703a3c1639cd8682d31e6d0466e48540efb084afadfbc04d29dfde753c81`.

Full asset SHA-256:

- Female: `10479f15017e27dae6204d14625bf223df7dfc75e19386054e0ceb618f9f59cf`.
- Male: `d2b77648a8b8b2b00819dc5b0c4bc460c796fcef098a82be48115605436bf62f`.

All 17 Python source/skin regressions pass in 68.864 seconds. They check
stored closed outward topology and shading, exact geometry and hierarchy,
inverse binds, four real support chains, actual occupied attachment volumes,
front vertices/edges/centers against the posed parent, independent carriers,
uncrossed appendages, winding, strain, floor contact, neutral endpoints,
budgets and strict mutation rejection. Four Rust GPU/original-size,
battle-support, bounds, independent-carrier and four-foot walking checks pass
in 2.93 seconds. Background Blender reproduces both canonical GLBs byte for
byte. Final software front, oblique, rear, idle, attack and hit sheets were
inspected offscreen.

The shared registry regression passes in 132.47 seconds and the production
source-cue/GPU timing regression passes in 132.52 seconds against the final
registry. The
production examples were rebuilt with `crystal-assets` at optimization level 2.
Both executable preflights require the headless parse guard, mute support and
the exact current gzip payload for each canonical GLB. All six selected native
pairs preserve the same authoritative state across their 3D/2D captures; all
were visually inspected inline. Every capture uses explicit headless image
rendering, preview mute and normal coloration. No window, application activation
or browser tab is used. Executable identities remain unchanged after the batch.

World views use Route36 `(20, 8)` at hour 12, frontal camera `0 0 20 1` and
elevated camera `3 2 50 0`. Battle views use an ordinary Route29 walking encounter
with the authentic terrain stage and original pack sizes. The ignored evidence
prefix is `target/pokemon-native-review/nidoran-`, with per-view paired-state
receipts, logs, final PNGs and executable identity data.

| Selected pair | State frame | State hash | LCD frame | Virtual seconds |
| --- | --- | --- | --- | --- |
| nidoran_f frontal | 2941 | 3208578776 | 2941 | 50.724696292 |
| nidoran_f elevated | 1757 | 1218110954 | 1757 | 30.441758041 |
| nidoran_f battle | 3234 | 2419363333 | 3916 | 70.465453541 |
| nidoran_m frontal | 2420 | 822403460 | 2420 | 41.666574917 |
| nidoran_m elevated | 1506 | 3527854845 | 1506 | 25.5786935 |
| nidoran_m battle | 3928 | 2938843744 | 4637 | 81.844804458 |

| Player / opponent | Pack height (m) | Uniform scale | Reference height | Support Y | Stage checked / unsupported |
| --- | --- | --- | --- | --- | --- |
| nidoran_f / PIDGEY | 0.4064 | 1.380718 | 0.78 | 0.0 | 124 / 41 |
| nidoran_m / RATTATA | 0.5080 | 1.6827501 | 0.8 | 0.0 | 60 / 2 |

Both battles retain modeled species with `fallback=None`, zero support offset
and `accepted=true reason=None`. Stage counters describe candidate observations,
not a support percentage. Admission, whole-body support and physical-size gates
remain unchanged.

Executable SHA-256:

- render_at_location: `5190da166072bf2d1e184d750f7b4eec6d8760d1c82b2e44928f7040ee1b2c42`.
- immersive_battle_3d: `928f91c266e57456120566cad092fb966f7f557d6c3d7c4f53f00d07bf5d2c6d`.

The frontal views retain readable layered gazes, distinct pinnae and the
female cheek barbs versus male nasal horn. Elevated views retain the original
small companion scale and clear separation from the trainer. Battle views show
the complete body silhouettes and seated paws against the actual ground, with
readable species/gender labels and matte materials under production lighting.
Fine facial and toe details are naturally limited in the distant elevated
view. Browser and moving-world visual acceptance remain open. Shiny checks are
skipped. Grass, AA and filtering are outside this batch.

Reproduce the pure assets or targeted Python checks with:

```sh
PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=tools python3 tools/nidoran_glb.py --species nidoran_f --output target/nidoran_f.glb
PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=tools python3 tools/nidoran_glb.py --species nidoran_m --output target/nidoran_m.glb
PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=tools python3 -m unittest tools/test_nidoran_sculpt.py tools/test_nidoran_glb.py -v
```

---

## Prior static sculpture record

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
