# Chansey sculpture

Chansey now uses an original soft pink egg-shaped body, six long rounded
side ribbons, tiny fitted eyes and glints, a curved closed smile, short arms
and hands gathered around the pale egg, a pink cup-shaped pouch, broad
grounded feet and a short pointed rear tail. The reusable recipe is
`tools/chansey_sculpt.py`; the builder routes Chansey to that recipe before
the generic batch. Blissey also uses its separate original recipe. The
generic egg-nurse geometry is retired. Neither species' runtime identity
was changed.

The first candidate's egg overlapped its pouch but visibly protruded through
the bottom. Moving the egg upward corrected the render. A new regression
checks every stored egg vertex below the rim against the pouch's actual
outer front/back surfaces, and rejects a deliberately lowered egg. Eight
points beside the egg establish real vacant space inside the open rim; an
inserted solid deliberately fills that space and is rejected. Overlap alone
is not accepted as evidence of egg containment.

All 12 candidate checks passed in 13.812 seconds and all 12 canonical checks
passed in 13.330 seconds. They cover closed outward topology, normals,
recipe parity, palette, floor, height, triangle budget, every physical root
(including both hands to the egg), fitted vertices and triangle interiors,
uncrossed appendages, containment, vacant pouch space and invalid detached
parts. The updated oblique, front and rear software renders each exited 0
at 800×824 and were inspected in full. The bottom protrusion is gone.

The canonical static mesh has 26 parts and 51,296 triangles, a floor at zero
and height 1.10. Its full canonical SHA-256 is
`ff57a4e9d819d3e89486cdd280d701de4349fafd624a6de8e6e59f6f4ffed34c`.
The union-feature native build exited 0 in 7 minutes 30 seconds. Its frozen
executable `(218210401, 212715344, 1791819080189702935)` contained that full
hash and the preview mute switch. The explicitly muted image-target GPU
capture exited 0; the full battle image was inspected. Both executable
identity and the canonical source hash stayed stable through capture.

The first union-feature test
build exited 101 on unrelated interior compilation errors: a moved surface
value and 11 test calls missing a newly added map parameter. The moved-value
issue and missing arguments were corrected by concurrent work when
inspected again; no
unrelated repair was applied by this model task. The successful retry and
direct exact check are recorded below. No rig or
animation was imported. The [source review](egg-nurses-source-review.md)
records the official anatomy and inaccessible external download endpoints.
All egg-nurse verification jobs are terminal. Task-owned ignored reference,
candidate and capture scratch was removed after the final proofs.

The subsequent union-feature native build exited 0 in 3 minutes 58 seconds.
Its frozen executable `(218227345, 213673024, 1791819620271693010)` contained
both full final Chansey and Blissey hashes. Both explicitly muted
image-target captures exited 0 at 2000×1400 and were individually inspected
in full, with executable identity and both canonical source hashes stable
after each. A direct exact Rust check passed in 9.75 seconds against frozen
executable `(218278594, 242268720, 1791820052147815341)`, containing both full
final canonical hashes. Executable identity and both hashes stayed stable
through the run. It executed one test covering loading and bounded geometry
for all 251 species; this is not all-species art approval. The union-feature
cargo job subsequently exited 0, executing that same actual one-test
all-species check in 5.14 seconds. These captures do not establish ordinary live-window, browser, animation or gameplay behavior.
