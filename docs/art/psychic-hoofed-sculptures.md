# Girafarig and Stantler sculptures

The [October 21 Sneasel/Stantler fidelity revision](claw-deer-fidelity.md)
supersedes the historical shape, palette, metrics and coverage below for these
two species. Girafarig remains unchanged.

`tools/hoofed_sculpt.py` authors these distinct quadrupeds. Both species now use canonical standard skins from `tools/girafarig_glb.py`
and `tools/stantler_glb.py`, sharing the production reader and GPU path. Their
parallel static JSON models are retired.

Girafarig has long supporting legs, a sculpted neck and tapered face, a pink
muzzle and ear interiors, ivory ossicones, an open smile and seven triangular
mane plates. Golden forequarters and black hindquarters share the same seam
ring. Pink shoulder/neck spots and gold rear spots follow actual side surfaces. Its slender black tail rises
into a second head with eyes and a grin facing rearward. Its thirteen-joint skin is described below.

Stantler retains all 51 original parts, exact positions, normals and triangle
order: 15,993 vertices, 20,548 triangles and normalized height 1.28. The normal
materials come from the local pack: brown RGB555 (23,15,7), navy (4,5,16), white
and black. The antlers and pale markings are white; the nose, hooves, ear
interiors, brows and antler orbs are navy. The canonical GLB is 614,848 bytes;
neutral digest `7508b413c731950b19f25b4f91fd096dedd8d449e8158fa2439e06ae8c98d843`.
The pack height 407 means 4 feet 7 inches (1.397 m).

The twelve-joint Stantler skin keeps cloven hooves and fitted clefts on the
identity root, blends leg middles into the torso, and articulates neck/head,
ears and a two-stage curled tail. Antler stems, hollow loops, branches and
orbs share the head's rigid transform, as do the fitted gaze, nose and nostrils.
Flank markings share the torso; the cream chest patch uses the same spatial
field as the neck beneath it. Idle, attack and hit use rotations only, neutral
endpoints and the existing source battle cues. Exact sprite-frame motion,
field gaits and browser visual review remain open. Shiny checks are skipped
at the user's request.

The original nine anatomy regressions cover closed oriented parts, normals,
deterministic geometry, bounds, volume contacts, Girafarig's seam and second
head, Stantler's hollow antlers and attached tail, and fitted markings/faces.
Six decoded-skin checks cover canonical reconstruction, bind identity, normal
palette, coherent owners, planted hooves, moving features, winding, strain and
delayed tail motion. All nine anatomy and six skin checks pass. Front/rear software views were
inspected. A species-local patch wrapper preserves original ordered centroid
sums across Python versions; geometry and palette in native Python remain
unchanged. Final Blender/native exports are byte-identical. Both focused production Rust checks pass, including
GPU skinning at the verified pack height, coherent feature owners, planted
hooves and animated bounds. The wider catalog grounding and actual elapsed-pose regressions also pass.
Normal native review passes.

```sh
python3 tools/stantler_glb.py
python3 -m unittest discover -s tools -p test_stantler_glb.py
python3 -m unittest discover -s tools -p test_hoofed_sculpt.py
cargo test --locked -p crystal-voxel-view stantler_skin_ --lib
blender -b -t 2 --python tools/build-battle-species.py -- target/stantler-review --only STANTLER
```

Builder exports and capture intermediates remain ignored verification scratch.
No content pack or extracted game geometry is delivered. The full all-species
model and rigging objective remains open.


The native build and matched normal 3D/2D review against Pidgey pass. Stantler's
hooves meet the retained terrain, its neck and flank markings remain fitted,
and the curled tail stays attached at the verified pack scale. This capture
shows a side profile, so the antler planes are nearly edge-on; the front/rear
software views establish the hollow loops, branches and separate orbs. The
paired receipt preserves core frame 2817, hash 4132824355, LCD frame 3718 and
time 80.771744292 (`state_preserved=true`). Shiny checks remain skipped. Exact
motion, field gaits, browser review and broader species work remain open.


## Girafarig fitted neck and independent second head

The source retains all original indexed triangle positions and order, with
eight fitted dark spot centers added to the 64 original anatomy parts. Curved fitted eye and spot panels use crease-aware shading; the
other 40 parts retain their original indexed normals. This removes panel
faceting and reduces stored vertices from 24,775 to 14,025 including the new centers. There are 22,432 triangles, with normalized
height 1.28 unchanged. Twelve outer spots and eight centers are packed into
five surfaces by material and supporting deformation field. Source part
ranges restore all 72 parts and ordering through the strict reader.
The runtime asset has 57 parts, 245 accessors and occupies 594,400 bytes.

The local pack's normal palette is RGB555 gold (31,22,4), pink (28,10,16),
white and black. Forequarters/neck/head use gold; rear body and tail use black;
fitted front spot borders, mane, muzzle, ear interiors and hooves use pink.
The front spots have dark centers fitted to their actual outer panels. The source
height 411 means 4 feet 11 inches (1.4986 m). Source neutral digest:
`9b38583cbb8ceee957133a2774eebd309860117c5f3014922201877716781810`.
Packed neutral digest, also pinned by the Rust reader:
`6c3c3a187cdce79b3adbe6a17a77a5214cfc55b7c5b0dfed356b2dc46d3bcb4c`.

Hooves and fitted clefts remain on the identity root. Both body colors and
their back crests use the same torso transform, preserving the shared seam.
The long neck, fitted neck spots and crest roots use one spatial blend field.
Main face, open mouth, tongue and ossicones share the head. Ears and leg
middles articulate separately. The second neck has base/bend/head stages;
the rear-facing head owns its fitted eyes, grin and tooth dividers. Delayed
second-head motion differs from the main head. The three source-cue clips
use rotations only, neutral endpoints and planted floor contact.

All nine shared anatomy and eight decoded-skin checks pass. They cover real
support contacts, fitted surfaces, canonical part reconstruction, original
body seam in every clip, coherent heads, normal palette, neutral bind, floor
contact, moving features, winding, edge strain and delayed second-head motion.
Blender/native bytes are identical. Front/rear software views were inspected;
both faces, smooth curved spots, black hindquarters and pink mane/hooves read
clearly. Stantler's geometry, palette and neutral digest remain unchanged.
Both focused production Rust checks passed before the added dark spot centers.
Final Rust and matched normal native review pass. Exact motion,
field gaits and browser review remain open. Shiny checks are skipped.

```sh
python3 tools/girafarig_glb.py
python3 -m unittest discover -s tools -p test_girafarig_glb.py
python3 -m unittest discover -s tools -p test_hoofed_sculpt.py
cargo test --locked -p crystal-voxel-view girafarig_skin_ --lib
blender -b -t 2 --python tools/build-battle-species.py -- target/girafarig-review --only GIRAFARIG
```


The first normal native comparison against Hoppip exposed solid pink front
spots. The source sprite has dark centers inside pink borders; eight fitted
centers now correct that detail. All nine anatomy and eight skin checks pass
again, including fitting each center against its actual outer panel. Final
Blender/native bytes are identical. The initial capture is not final visual
acceptance for the corrected material layering. Shiny checks remain skipped.


The corrected matched normal native 3D/2D review against Pidgey passes.
Dark spot centers sit inside pink borders, the long neck and two-color trunk
remain coherent, and all hooves meet the retained terrain at the pack-based
scale. The second head faces away from this camera; its fitted gaze/grin were
inspected in the rear software view. The paired receipt preserves core frame
2747, hash 2531382474, LCD frame 3381 and time 60.614596083
(`state_preserved=true`). Both final focused Rust checks, the final native
build and wider catalog grounding/elapsed-pose regressions pass after the
added centers. Shiny checks remain skipped; exact motion, field gaits, browser
review and the all-species objective remain open.
