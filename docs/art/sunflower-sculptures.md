# Sunkern and Sunflora sculptures

`tools/sunflower_sculpt.py` authors original canonical Sunkern and Sunflora
meshes. Canonical runtime assets now use the weighted GLBs described in the
rig migration below. The battle species builder dispatches these recipes before Blender
batching and rejects their old shared paths. Bellossom retains its existing
recipe. Both species keep their 0.95 presentation height and floor origin.

Sunkern has a rounded gold seed shell with eight serrated crown teeth, an
actual open rim and a recessed interior floor. A thin sprout grows from below
that floor into two broad green leaves with fitted midribs. Four broad green
coat panels, two front and two rear, follow the seed surface and end in jagged
upper edges. Large black button eyes have ivory catchlights and a fitted white smile inset follows the composite shell/coat surface. The asset has 18 closed parts
and 16,524 triangles.

Sunflora has a continuous plump green bulb and rising stem, a broad white
flower disk and a cheerful closed-eye expression with a white smile inset. Eight
large curved rounded petals overlap eight smaller rear golden bracts. The
asymmetrical leaf arms include a raised curled tip and fitted midribs. Curved
root legs join two grounded broad green foot pads. The asset has 30 closed
parts and 13,696 triangles.

The shared `fitted_contour` helper now accepts an optional subdivision count.
Existing recipes retain its two-level default. Sunkern uses three levels for
its highly curved coat panels and tongue; stored-triangle clearance checks
caught and eliminated their initial shell/parent intersections. Existing Aipom
regressions verify unchanged default regeneration and geometry.

Silhouette and palette were checked against the
[official Sunkern artwork](https://assets.pokemon.com/assets/cms2/img/pokedex/full/191.png)
and [official Sunflora artwork](https://assets.pokemon.com/assets/cms2/img/pokedex/full/192.png).
External search found [Sunkern](https://sketchfab.com/3d-models/sunkern-be39b7cd8eaf4879a0a732ae24250f8d),
[Clay Sunkern](https://sketchfab.com/3d-models/clay-sunkern-b71cd4da1f5f437591529321ad427367)
and [Sunflora](https://sketchfab.com/3d-models/sunflora-3c78b210bcd749f28b89e859586146bd)
listings with CC Attribution licensing. Their search metadata did not establish
an original usable rig. No external mesh or textures were imported. The initial stage used original static meshes on the native base-color path.
The standard skins below supersede that static runtime path.

`python3 -m unittest discover -s tools -p test_sunflower_sculpt.py` checks
closed topology, winding, connectivity, unit normals, deterministic regeneration,
height, budget, actual crown/cavity geometry, sprout and leaf roots, fitted coat,
eye, mouth and tongue vertices and front triangle clearance, fitted midribs,
every petal/bract root and both supporting leg/foot junctions. Software views
and native battle captures check model presentation rather than gameplay behavior.

Final verification passed all ten sunflower geometry regressions and all seven
Aipom shared-helper regressions. Front and rear software views and both final
native battle captures were visually inspected. The Rust catalog regression
passed for the 220-species battle catalog; it checks grounding and volume, not visual quality.
Model layout and storage validation passed for all 571 JSON assets. Temporary
review images and build logs were removed after inspection.


## Standard rig migration (2026-10-14)

`tools/sunflower_glb.py` now exports canonical `sunkern.glb` and `sunflora.glb`.
An audit first matched both stored static models to their recipes exactly;
a second audit preserved every triangle position, normal and index while
applying the intentional pack material corrections. Duplicate static JSON
files were retired. Exact position zeros are normalized across authoring
runtimes. Existing loader limits and shared mesh/bind ownership are unchanged.

The supplied pack's front/back/normal palettes agree. Sunkern uses RGB555
yellow (29,28,1), green (13,18,8), black and white, expanded to RGB8 yellow
(239,231,8) and green (107,148,66). The paired back sprite establishes green coat panels; black is used for
eyes and outlines. The initial black-coat revision was rejected visually.
Sunflora uses gold (26,22,3), green (10,21,0), black and white: RGB8 (214,181,24)
and (82,173,0). The source front sprite establishes the white face disk and
smile inset. Fitted midrib shade remains authored detail. Height digits 100
and 207 establish 0.3048 m for Sunkern and 0.7874 m for Sunflora; both retain
the 0.95-unit authored neutral span.

Sunkern's seven joints articulate the seed body, sprout and two-stage leaves.
The shell's lowest vertices stay root-owned. Coat panels share the shell's
height blend; leaves and fitted midribs share their bend fields. Sunflora's
twenty-five joints articulate bulb, two-stage stem, flower disk, two-stage
leaf arms, eight front petals and eight rear bracts. Supporting feet stay
root-owned, fitted face parts share the disk, and each ray has a rigid owner
rooted in the disk. Three rotation-only clips have neutral endpoints and no
root motion; flower response follows the body brace. These are authored
anatomical responses; detailed source-frame motion and gaits remain open.

| Species | Parts | Vertices | Triangles | Joints | Accessors | Bytes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Sunkern | 18 | 11,694 | 16,524 | 7 | 96 | 475,748 |
| Sunflora | 30 | 9,135 | 13,696 | 25 | 195 | 477,324 |

Neutral digests:

- Sunkern: `2c11fc4e36f23be374beb989f959372f218b2cb292dcc91286cb8804df72ea8b`.
- Sunflora: `3342555c766e836e7ea17ed6fc8a931fb727e29fbb06f97eee739104ea670f70`.

After the green-coat correction, five independent stored checks and all ten
anatomy checks pass. Both fresh Rust family regressions pass: they exercise
registry deformation and generic GPU skinning at the verified pack heights.
Blender's production builder reproduces both corrected GLBs byte-for-byte,
and the production native example builds successfully. Before this material-only
correction, all 80 standard variant schemas and the art validator passed
(61 editable sources, 609 runtime models in 609 files, 757 embedded references).

The corrected normal Sunkern retained-map pair preserves core frame 1726,
checksum 2531288909, LCD frame 2327 and time 40.765335458. Both images were
inspected: the green panels agree with the back sprite, both leaves project
visibly, the seed stays grounded and the production HUD names SUNKERN.
The model remains small at its source height; leaf width and silhouette fidelity
still merit refinement. The shiny pair also preserves state: core frame 2132, checksum 2538937951,
LCD frame 2844, time 51.328301835. Both inspected images show the source's
brighter yellow-green replacement across shell, panels, sprout and leaves;
the same mesh and rig serve both appearances. Normal Sunflora's inspected pair preserves core frame 1759, checksum
566196954, LCD frame 2374 and time 41.498861499. The white face, golden rays,
leaf arms and supporting feet stay attached and readable. Petals project more
sharply than the source sprite; shape fidelity remains open. The retained map
also shows a broad gray terrain slab in the distance, outside this rig migration.
The shiny Sunflora pair preserves core frame 1302, checksum 1498643714,
LCD frame 2008 and time 36.292988541. Both images were inspected: bright yellow
rays and olive body/leaf surfaces match the source palette change while the
white disk and attached features remain stable. These four paired reviews
verify native rig/palette integration, not complete shape or motion fidelity.
The queued shiny capture was deferred once for shared GPU measurement before
running successfully; no active capture was interrupted. Temporary pack readers, reference PNGs and focused Blender output were
removed; ignored QA logs and paired captures remain.


Reproduce the field review with the built native example (set the species to
`SUNKERN` or `SUNFLORA` and shiny to `0` or `1`; output files are disposable):

```sh
CRYSTAL_PREVIEW_MUTE=1 \
CRYSTAL_RENDER_TEST_CAPTURE_TIMEOUT_SECS=300 \
CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image \
CRYSTAL_RENDER_TEST_PARTY_SPECIES=SUNFLORA \
CRYSTAL_RENDER_TEST_PARTY_SHINY=1 \
CRYSTAL_RENDER_TEST_WALK_INPUT=synthetic \
CRYSTAL_RENDER_TEST_HOLD_BATTLE=1 \
python3 tools/with-graphics-lock.py -- \
  ./target/pokemon-models/debug/examples/immersive_battle_3d \
  content-packs/core-modular.browser.crystalpack \
  --walking-encounter --walk "$(python3 -c 'print("RALA" * 20)')" \
  --size 1000x700 \
  --screenshot target/pokemon-native-review/sunflora-review-3d.png \
  --second target/pokemon-native-review/sunflora-review-2d.png
```

This selects only a disposable party lead. Controller movement, encounter rolls,
battle scripts and camera readability guards remain authoritative. Each paired
receipt must preserve state; inspect both images before accepting a visual change.


## Rounded petal refinement (2026-10-14)

The first rigged native review exposed narrow pointed tips. The eight front
petals now have broad rounded caps, closer to the source sprite's silhouette.
Outline sampling, maximum reach, physical height, triangles, materials, joints
and animation definitions remain unchanged. An executed binary audit confirms
all 22 non-petal parts retain exact positions, normals, indices and skin weights,
and the node and clip schema remains exact. Sunkern is unchanged. Smoother edge
normal grouping reduces Sunflora by 48 vertices to 9,135 and by 1,368 bytes to
477,324 bytes. The top fifth of each petal spans over 70% of its full width;
the rejected old first petal spanned only 16.7% there. A stored geometry
regression checks all eight caps independently of the authoring recipe.

Blender reproduces both canonical files byte-for-byte. Six stored checks,
ten anatomy checks and both focused Rust deformation/GPU tests pass. The native
rebuild passes. The inspected normal pair preserves core frame 1369, checksum
2891771878, LCD frame 1958 and time 34.028171958. The rounded rays read clearly,
with face and leaves attached; tall grass partly hides the feet. The shiny pair also preserves core frame 1681, checksum 3389832729, LCD frame
2361 and time 41.560019001. Both images were inspected: broad rounded rays,
attached face/leaf arms and visible supporting feet retain the expected
bright-yellow/olive shiny palette. The two shots use different encounters and
camera compositions; only the matched 3D/2D pair within each receipt shares a
state. This verifies the rounded rest silhouette and retained rig integration;
source-frame motion, gaits and browser fidelity remain open. The preceding
paired receipts describe the earlier pointed-petal model.
