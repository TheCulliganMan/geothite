# Blissey sculpture

The original recipe `tools/blissey_sculpt.py` gives Blissey a separate pink
upper body and white lower body, two large side crown lobes and curling
front locks, tiny fitted eyes and glints, a recessed open smile, a pink
tongue, white wing-like shoulder frills, pink arm tips, ten pointed white
skirt feathers, slim pink legs, rounded grounded feet and a pale egg in a
pink cup. Only the reusable cup construction and palette come from the
Chansey recipe; the adult anatomy is authored separately. This is a static
mesh, with no imported rig or animation.

The first curls failed the actual nonadjacent-triangle crossing check and
visually looked like horns. Smaller loops with thinner curled ends passed
that check. Their exposed root caps were then buried into their actual
crown lobes. Narrow edge-on skirt petals were replaced with wider feathers
whose flat planes follow the circumference of the body. The initial oval
mouth read as surprised, so the latest candidate has a broader straight
upper edge and a rounded, narrower bottom with a small tongue.

The first candidate failed one of ten geometry checks. The next two
iterations each passed all ten (75.402 and 54.627 seconds). Checks cover
closed outward topology, face-aligned normals, source parity, palette,
height, floor, triangle budget, physical roots, fitted face vertices and
triangle interiors, uncrossed curls and frills, egg containment, actual
vacant space inside the pouch, and deliberately invalid detached parts.
The final smile revision passed all ten candidate checks in 22.086 seconds
and all ten canonical checks in 24.978 seconds. Its updated front, oblique
and rear software views each exited 0 at 800×824 and were inspected in full.
The canonical replacement is installed and the builder dispatches to its
separate recipe, retiring the generic egg-nurse geometry. It has 40 parts,
70,872 triangles, a floor at zero and height 1.20. Full canonical SHA-256:
`a15c0962691c96627fc6f07183d96f961e3a5d34fc011ff2cf2ac63a688c5d3f`.
Two additional checks cover the actual recessed mouth space and reject
filled mouth/pouch solids. All 12 expanded canonical checks passed in
25.717 seconds. The final union-feature native build exited 0 in 3 minutes
58 seconds. Frozen native executable
`(218227345, 213673024, 1791819620271693010)` contained both full final
Chansey and Blissey hashes and the preview mute switch. Both explicitly
muted image-target GPU captures exited 0 at 2000×1400 and were individually
inspected in full. Executable identity and both source hashes stayed stable
after each capture. A direct exact Rust check passed in 9.75 seconds against
frozen executable `(218278594, 242268720, 1791820052147815341)`, containing
both full final canonical hashes. Executable identity and both source
hashes stayed stable through the run. It executed one test covering loading
and bounded geometry for all 251 species, not all-species art approval. The
union-feature cargo job subsequently exited 0, executing the same actual
one-test check in 5.14 seconds. No ordinary live-window, browser or gameplay
approval is claimed. See the [source review](egg-nurses-source-review.md) for the official
anatomy and specific external listings checked. All egg-nurse verification
jobs are terminal. Task-owned ignored reference,
candidate and capture scratch was removed after the final proofs.
