# Charmeleon and Charizard canonical skins

Charmeleon and Charizard replace the old Blender primitive assemblies with
original shaped fire-lizard sculptures and canonical articulated GLBs. The
background builder, direct sculpture CLI, strict logical readers and Rust
species dispatch use the same deterministic bytes. Registration now contains
239 skin variants and 36 static species files; the full model improvement goal
remains active.

| Species | Previous vertices / triangles | Current vertices / triangles | Logical / runtime parts | Joints | Accessors | Bytes |
| --- | --- | --- | --- | --- | --- | --- |
| Charmeleon | 1,632 / 2,942 | 10,656 / 17,122 | 52 / 29 | 13 | 152 | 456,904 |
| Charizard | 1,884 / 3,294 | 13,693 / 21,962 | 65 / 38 | 15 | 192 | 574,784 |

Both have a continuous tapered torso/neck, shaped projecting skull, separate
open lower mandible, tongue and upper fangs. The cream bib and nested angular
gaze layers conform to their actual stored parents. Each forelimb has a palm,
three complete coat fingers and three ivory claws; each planted hind pad has
three rooted toes. A continuous curved tail supports five coherent folded
fire layers. Charmeleon has one swept red crest. Charizard has two orange horns,
broad scalloped teal bat wings, continuous leading spars, three branching ribs
per wing and ivory wrist hooks. Local pack 2D back views were inspected before
authoring; these references establish neither front-sprite nor gameplay review.

Thirteen controls comprise the weighted ground root, body, skull, mandible,
two shoulders, two wrists, two hind limbs, two tail controls and fire control.
Charizard adds two wing hinges. Feet/toes remain on the identity ground root in
battle; thigh blends transition from fixed soles to independently moving legs.
Belly and gaze remain coherent with their actual rigid carriers. Tail bindings
use the source's sampled cross-section rings and arc distances. The complete
fire assembly follows one control. Each wing's membrane, spar, ribs and hook
share its hinge. Peak wing motion initially separated the membrane from the
body; extending its inner root into the torso fixes occupied attachment through
all sampled poses without reducing motion or weakening the overlap check.

Idle lasts 3.2 seconds with 81 keys; attack/hit last one second with 65 keys.
All tracks are rotation-only with neutral endpoints and source-cue timing.
Packing combines same-material parts on the same controls, preserving source
ranges for all 52/65 logical parts. Left/right planted toe groups remain
separate. Runtime names are unique and bounded. Existing 20k vertex, 40k triangle,
64 runtime-part/joint, 256 accessor and 1 MiB limits remain unchanged.

Twenty Python checks pass against the final stored assets in 23.920 seconds.
They cover source identity, closed outward connected solids, normals, complete
digits, actual occupied anatomy roots, fitted layer vertices/edges/centres,
real empty jaws, detached/filled negative controls, independent controls,
carrier coherence, fixed soles, positive posed triangle winding/strain,
neutral endpoints, hierarchy/inverse binds, packing/names/limits and mutated
source/byte/species rejection. Four focused Rust checks pass in 1.15 seconds:
original-size GPU activation for both species, canonical supports/face/jaw/
controls, and world walking/authored flight with complete bounds.

Charmeleon keeps two world walking supports. Charizard retains its canonical
weights and authored wing hinges, with flight clearance across all sampled
vertices. Pack dimensions remain 1.0922 m and 1.7018 m against standing height;
authoring heights are 1.20 and 1.40. Uniform scale, support offsets, full animated
bounds and admission gates are preserved. The background builder, direct CLI
and candidate GLBs match byte for byte.

The ignored `target/pokemon-native-review/fire-dragon-skin-review.png` was
inspected inline after the wing-root correction: previous model at the same
height reference, decoded neutral, front, rear, side and attack at 0.34. Angular
profiles, a single/twin horn distinction, actual jaw gaps, complete digits,
curved tails and coherent bat wings improve the primitive silhouettes. Thin
claws and folded fire edges remain distance-dependent. This authoring review
alone does not establish production visual acceptance.

Reproduction:

```sh
/Applications/Blender.app/Contents/MacOS/Blender --background --threads 2 \
  --python tools/build-battle-species.py -- \
  crates/crystal-voxel-view/models/battle_species --only charmeleon,charizard
PYTHONDONTWRITEBYTECODE=1 PYTHONPATH=tools python3 -m unittest -v \
  test_fire_dragon_sculpt test_fire_dragon_glb
```

| Species | Logical float32 geometry SHA-256 | Packed float32 geometry SHA-256 | GLB SHA-256 |
| --- | --- | --- | --- |
| Charmeleon | `1a2f4dde815962e0d07e93ba445c7b854b918cdb68bf0233c083aa789bbd0a66` | `5c7062be45d9caa717b32a0dd665c207e4b6e563add9971868dee91edc343007` | `dcb010779349e576ee950c50a0e71ba30ed0b9f1faabbe530e6aee4f79b6fa78` |
| Charizard | `fbcfbf20178a097dd6cd5b9d8329fd7e36bb955574921b780292c57d262b2f8d` | `20ca8700dd7dbe8c6db55ee12f00bae240ee62f1a39255d3aa06dec9d01ef05b` | `69b393e4069bba185f292d757cf427ce76d6d9ce31423d83808307e9b9b44d73` |

Native review uses `CRYSTAL_RENDER_TEST_HEADLESS=1`,
`CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image`, `CRYSTAL_PREVIEW_MUTE=1` and normal
palette only. No visible windows, GUI application launches or focus changes are
permitted. No shiny verification, grass-height, AA/filtering, audio, pack or
gameplay changes belong to this batch.

The shared schema/physical-size check passes in 87.32 seconds and the shared
elapsed-idle/exact-current-attack cue check in 122.04 seconds. Together with the
20 stored Python and four focused Rust checks, 26 scoped checks pass. Both
native examples built in 47.68 seconds with `location-tester,bevy/zstd_rust`.
These checks do not establish full-suite or gameplay acceptance.


## Native paired review

Six 1000×700 production pairs and two moving-world frames were inspected inline,
headless and muted. Every paired authoritative state is preserved. Four
front/world traces each retain 32 actual synthetic-input rows across five
Route36 positions, (20,8) through (20,12). Both named walking battles admit
complete original-size models with uniform scale, zero support offset and no
fallback. Full animated bounds, camera, authentic-ground, omitted-geometry and
shadow gates remain intact. No physical dimension or FOV changed for review.

Charmeleon's single swept crest, angular glare, empty jaw, cream bib, complete
claws and curved burning tail remain readable. Its walking feet follow path
height. Charizard's twin horns, longer skull, open jaw, teal scalloped wings,
ribs, hooks and complete digits remain recognizable. Canonical wing beats
provide intentional world flight, while battle feet retain their support root.
World camera zoom is 3 for Charmeleon and 1 for Charizard to include the wider
body; front views use the same existing first-person camera settings. Scale
still derives from each species' original pack dimension.

Charizard's moving tail/flame is partly obscured by a nearby tree and the sign
crosses its feet in projection. Front wing views obscure the tail, and profile
views obscure far limbs. These remain broader follower/scenery placement and
camera-coverage concerns. Thin claws, folded fire edges and wing ribs alias at
distance, and mouth/belly shadows are strong at some angles. Broader browser,
all-camera/material coverage and gameplay remain unverified. No grass-height or
AA/filtering changes belong to this batch.

Final captures took 45–65 seconds each. Both executables retained exactly the
same SHA-256, size and modification time before/after all captures. Preflight
verified headless/mute tags and exact current compressed GLBs in both binaries.
Completed producer scripts, temporary source/candidate/CLI copies and redundant
intermediate frames were removed. Final raw/paired images, receipts, four
movement CSVs, two moving-world frames, metrics and logs remain ignored under
`target/pokemon-native-review`. The full model and rig improvement goal remains
active.

| Species / view | Frame / checksum | LCD frame | Virtual seconds |
| --- | --- | --- | --- |
| Charmeleon front | 1997 / 1385433688 | 2004 | 34.160367292 |
| Charmeleon world | 2133 / 3952109238 | 2142 | 36.809522293 |
| Charmeleon battle | 2310 / 1758054517 | 2912 | 49.936052875 |
| Charizard front | 2102 / 2129044075 | 2113 | 36.023708875 |
| Charizard world | 2383 / 1010155576 | 2392 | 41.139320083 |
| Charizard battle | 1927 / 4119407173 | 2581 | 44.423305875 |

Stable executable SHA-256:

- `render_at_location`: `fa4c011b564e57eaeefb5c9fbf788267756d9afc55bda16ac664bb18739e1be2`.
- `immersive_battle_3d`: `f0262981500ca202f583ffebea1048c4a57a880a8b3b972ae241c4424bced663`.
