# Koffing and Weezing sculptures

Current assets use the canonical skins described under
[bounded geometry and animation](#canonical-skins-and-bounded-geometry).
The earlier static review below records the original anatomy.

## Original static anatomy review

The canonical meshes use original geometry from `tools/gas_pair_sculpt.py`:

| Species | Closed named parts | Triangles | Grounded floor | Normalized height | SHA-256 |
| --- | ---: | ---: | ---: | ---: | --- |
| Koffing | 23 | 55,900 | 0 | 1.00 | `f702467bf5f2da45383b8dfa1ef68e5d86c57badce8ea3cf6be13c4ca53318c9` |
| Weezing | 35 | 90,924 | 0 | 1.20 | `4a5f33be17a259f56314a688ea40cb7fc8354a60368a5883b3b4287f8adcffdb` |

The lavender bodies have soft radial contours with thirteen actual recessed
craters on each main orb. The closed front surface is also recessed around
the expression. Half-open white eyes and small dark pupils fit the stored
surface. Cream skull-shaped rings, purple centers and rounded crossbones
replace protruding emblem primitives. These are closed recesses and fitted
markings; hollow bodies, vent tunnels or through-hole skull marks are not
claimed.

Koffing has a broad cheerful grin, fitted pink lower mouth and two small
solid downward canines. Weezing has a large lower orb, smaller upper orb,
rear node and two actual connecting stalks. Both faces have a strained
frown; the large face has prominent upward canines and the smaller face has
short downward teeth. The first painted adult fangs bent over a steep
recess and looked like tear trails. Solid tapered teeth now root in the
actual lip surface. The first mouth lining failed a face/edge clearance
check; raising its surface offset corrected the failure without changing
the clearance threshold.

Two pale gas billows float asymmetrically around each model. Each has a
closed central volume and four actually rooted rounded lobes. The billows
are static presentation geometry, not anatomical appendages. Earlier rigid
attached trails looked like antennae; a ridged single-volume experiment
looked like a pinecone. Both were rejected visually. The final floating
connected clouds preserve the reference silhouette without those rods.
No particle simulation, rig or animated gas is claimed.

All twelve final candidate checks passed in 47.143 seconds. They cover
closed outward surfaces and actual face-aligned normals, recipe parity,
height/floor/budget/anatomy counts, Weezing's two joining stalks and three
orbs, each solid canine's actual root, fitted vertices and face/edge
clearances, uncrossed teeth/stalks, all cloud-lobe roots, thirteen actual
empty crater probes per main orb, empty front-expression recess probes,
filled-crater negatives and detached teeth/stalk negatives. The cloud
volumes float intentionally; their lobes remain physically connected.

Final oblique, front and rear software views for each species exited 0 at
800×824 and were individually inspected after the last cloud change.
Canonical bytes match those reviewed candidates. The pure builder now
uses the dedicated original recipes, and the old shared gas-ball function
rejects both replaced identities. Existing exact Rust species routes are
preserved. All twelve installed canonical checks passed in 57.767 seconds. The two
queued shared-cache jobs were explicitly canceled after confirming that
they were waiting without compiler children. Fresh native/Rust jobs used an
isolated ignored verification cache seeded by filesystem clones of dependency
build products. The native build exited 0 in 4 minutes 45 seconds.

Both final-hash GPU battle captures with explicit `CRYSTAL_PREVIEW_MUTE=1`
exited 0 at 2000×1400 and were inspected in full. Frozen renderer
`(218060122, 212054864, 1791817291985011789)` (inode, bytes, modification
nanoseconds) contained both full canonical hashes above and the mute switch.
Its identity and both source hashes stayed stable through each capture.
The cratered bodies, different expressions, solid canines and pale billows
read in the arena. The lower curved emblems are shaded and foreshortened
from this oblique camera; the final software front views also cover them.
Koffing is small at its actual battle presentation size. No artificial
species scaling or shared camera change was introduced.

The union-feature Rust integration exited 0, with the exact all-species
model check passing in 3.97 seconds. A direct exact rerun passed in 3.17
seconds against isolated frozen `crystal_voxel_view-dc3731eb4a201fc3`
executable `(218084485, 238801488, 1791817618734045465)`, containing both full
final canonical hashes. Executable identity and both source hashes stayed
stable through that rerun. This checks model loading and bounded geometry,
not every species' art. Image-target captures do not establish ordinary
live-window, browser, animation or gameplay behavior.

The [source review](gas-pair-source-review.md) records official anatomy and
five specific external listings, including one advertised animation clip.
No archive, mesh, texture, rig or animation was retrieved or imported.
Software views do not establish ordinary live-window, browser, animation
or gameplay behavior. All gas-pair verification jobs are terminal. Task-owned ignored reference
and capture scratch was removed after the final proofs. The isolated ignored
dependency cache remains in use for the ongoing full species upgrade; build
products are temporary verification output, not deliverables.


## Canonical skins and bounded geometry

`tools/gas_pair_glb.py` now writes the canonical Koffing and Weezing skins.
Both static JSON duplicates are removed. Runtime skin routing, strict model
readers and the main Blender builder use the GLBs. The pure sculpture recipe
still authors all 23/35 original named parts.

Redundant orb, cloud and patch sampling was reduced locally. Both orbs use
64-by-44 sampling while retaining all thirteen actual radial crater recesses.
The expression recesses, Koffing grin/tongue, strained Weezing frowns, rooted
canines, skull rings, crossbones and adult stalks remain. Adaptive fitted
patches refine only where needed to clear their actual stored parents, with
matching skin shading. Cloud cores and lobes retain their original centers,
radii and rounded silhouettes with lower surface redundancy. Heights and
species registration are unchanged.

| Species | Vertices | Triangles | Parts | Joints | Accessors | Bytes | Authoring height | Verified pack height |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Koffing | 10,563 | 18,264 | 23 | 12 | 127 | 442,308 | 1.00 | 0.6096 m |
| Weezing | 16,974 | 28,756 | 35 | 14 | 188 | 687,324 | 1.20 | 1.1938 m |

Both fit the unchanged 20,000-vertex, 40,000-triangle, 64-primitive,
256-accessor and 1 MiB limits. Rest geometry SHA-256 pins:

- Koffing: `ba5c5ad08e422850b832e4cb9412e18bcecbc53b62530186b11fa4f879b957de`
- Weezing: `b655fa7a75abd024c672927149600b45231e833200036b2c7bc97ed9ddba0c70`

The main orb yaws as a coherent solid with all fitted features and fangs.
Weezing's small upper orb and rear node move on independent carriers, with
both stalks blending continuously between their actual parent orbs. Two gas
clouds per species are parented to the root and move independently, each with
four additional lobe carriers. Lobes blend from occupied cloud-core roots to
free rounded ends. Distinct idle phases give the eight lobes separate motion.
Tiny polar rings share one compact weight field so quantized weight steps do
not pinch the cap edges. The existing intentional battle/world hover is
preserved; these are floating species, and the gas remains detached vapor.
No particle simulation or anatomical stalks attaching the clouds are added.

Idle uses 81 keys over 3.2 seconds; attack/hit use 65 keys over one second.
Source battle cues own timing, with neutral endpoints. All 20 combined Python
checks pass in 21.851 seconds, including thirteen empty crater probes per orb
at rest and under motion, fitted face/edge clearance, original negative
controls, occupied canine/stalk/lobe roots through all clips, coherent feature
owners, hierarchy/weights/binds, floor, winding and bounded strain, isolated
carriers, distinct gas phases and strict canonical readers.

Final front, side and rear old/idle/attack/hit sheets were inspected. Both
Blender exports match the final canonical files byte for byte. Rust/native
proof follows below. Shiny checks
remain skipped, grass height is untouched, and browser visual acceptance
remains open.


Four focused Rust checks pass (1.78 seconds): GPU activation for each species
at its verified pack height, all independent carriers with coherent features,
animated floor/culling bounds and preserved intentional battle hover, and 64
world floating phases with every authored influence preserved. Both world rigs
have no walk legs or additional limb motion.


The rebuilt native Koffing/Sentret Route 29 3D/2D pair was inspected at preserved
state: frame 2506, checksum 2573841080, LCD frame 3143 and virtual time
55.148248124. The cheerful grin, tongue/canines, cratered silhouette, lower
skull/crossbones and separate billows remain readable, with the correct KOFFING
HUD and intentional hovering clearance. Normal terrain admission passed at the
original 0.6096 m pack height and existing -110-degree review yaw, with normal
support/omission/whole-body/frustum constraints active, no fallback and no
shader errors. Ignored evidence: `target/pokemon-native-review/koffing-normal`.


The native Weezing/Pidgey Route 29 pair was also inspected at preserved state:
frame 2347, checksum 3242319440, LCD frame 3031 and virtual time 54.178510084.
The linked main/upper orbs, rear node, strained expressions, solid canines,
craters, cream emblems and independent billows remain readable, with the
correct WEEZING HUD and intentional hovering clearance. Normal admission
passed at the original 1.1938 m pack height and existing -110-degree review
yaw with normal support/omission/whole-body/frustum checks active, no fallback
and no shader errors. Ignored evidence: `target/pokemon-native-review/weezing-normal`.
Browser visual acceptance remains open. Shiny checks and grass-height edits
remain skipped.
