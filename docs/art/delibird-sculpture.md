# Delibird feather and held-sack skin

`tools/delibird_sculpt.py` authors the original feather fans, fitted collar,
asymmetric flippers and folded sack. `tools/delibird_glb.py` exports the canonical
`crates/crystal-voxel-view/models/battle_species/delibird.glb` through the common
production skin reader and GPU path. The parallel static JSON is retired.

All 28 original parts retain their exact positions, normals and triangle order.
The normal colors come from the user-supplied local pack: RGB555 lavender
(21,16,31), orange (30,8,2), white and black. The red body, flippers, feet and
bill use the source orange; the feather face, collar, fans and sack use white,
with lavender sack folds and black gaze/mouth details. No shiny checks or
captures are performed at the user's request.

The eight joints keep the flat webbed feet on the identity root, blend the
short legs into a yaw-only torso, and give the fitted collar the same
height-based torso/head field as its support. The head owns the fitted gaze
and upper bill; the lower bill and mouth seam move together. Each crest fan
moves independently. The raised flipper articulates separately, while the
lowered flipper, held sack neck, bag and fitted rear folds share one rigid
owner. This preserves the handhold and fitted folds throughout playback.
Idle, attack and hit use rotations only, neutral endpoints and existing
source-cue timing. Exact sprite-frame motion, field gaits and browser visual
review remain open.

The canonical asset contains 9,916 vertices and 17,808 triangles, occupies
402,984 bytes, and retains normalized height 1.10. Its neutral digest is
`a1665f55e8b724b62c20970d1f7b663dd06a46707a6c9764531d71e7c6bd1d4d`.
The pack's height digits 211 mean 2 feet 11 inches (0.889 m).

All 12 Python regressions pass, covering original closed anatomy, support
contacts, fitted collar/folds, normal palette, canonical reconstruction,
bind identity, planted webs, coherent feature ownership, rotation-only clips,
neutral endpoints, delayed held-sack motion, winding and edge strain.
Focused Blender/native exports are byte-identical. Both focused Rust checks
pass, including GPU activation at the verified pack height, coherent feature
ownership, fixed feet and animated bounds. The wider catalog grounding and
actual elapsed-pose regressions also pass. Front/rear software views were
inspected; the pointed collar retains its layered border and the rear folds
remain fitted. Matched normal native review passes; no broader visual
completion is claimed.

Rebuild and focused checks:

```sh
python3 tools/delibird_glb.py
python3 -m unittest discover -s tools -p 'test_delibird*.py'
cargo test --locked -p crystal-voxel-view delibird_skin_ --lib
blender -b -t 2 --python tools/build-battle-species.py -- target/delibird-review --only DELIBIRD
```

The Blender output is verification scratch, not a second deliverable. The
canonical strict reader is also registered with model discovery and the
repository model checker.


The matched normal native 3D/2D review against Sentret passes. Delibird's
webbed feet meet the retained terrain, the held wing and sack remain joined,
and its collar, bill and crest fans read clearly at the pack-based size. The
paired receipt preserves core frame 2584, hash 2663410303, LCD frame 3230 and
time 58.524416959 (`state_preserved=true`). The native build passes. Rear sack
folds were inspected in the software view; exact motion, field gaits and
browser review remain open. Shiny-specific checks remain skipped.
