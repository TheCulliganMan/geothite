# Chansey sculpture

The current canonical asset is `chansey.glb`, authored by
`tools/chansey_sculpt.py` and `tools/egg_nurse_glb.py`. Its 24 logical
closed solids pack losslessly into 16 runtime primitives: 10,390 vertices,
18,704 triangles, 17 joints, 124 accessors and 499,164 bytes. This replaces
the static mesh's 28,420 vertices and 51,296 triangles. The original 1.10
normalized height, physical-size path, soft pink palette and egg-shaped
silhouette remain. Existing geometry, part, joint, accessor and byte limits
are unchanged.

The rig keeps the contained egg and genuinely open pink cup on the body
carrier. The distal palms settle onto that carrier to preserve their grip;
the arms flex between moving shoulders/elbows and the supported palm ends.
A continuous body/head field keeps the fitted gaze, smile and chin coherent.
Six independently weighted rounded ribbons and a two-carrier pointed tail
retain their real roots. Complete feet remain fixed in battle, with supported
short ankle articulation. Idle is 3.2 seconds; attack and hit are one-second
rotation-only clips driven by source cues.

Focused round sampling, adaptive fitted face layers and fewer samples on the
original closed outer/inner cup retain its physical rim, empty spaces beside
the egg and supported bottom. No filled-pouch substitute is used. Two separate
Hand solids are retired into continuous forelimbs and palms; the remaining
24 logical solids are individually recoverable from stored part ranges.

Logical geometry pin:
`a5b1ee98864b13d123d3f59ff5c7f2b81b530d746157a0fc8bd397a14f387a27`.
Packed geometry pin:
`9b0d87c52e08fad76e7ee66f2983e075382f8a3da37b989e4cc025be0061021e`.
Background Blender dispatch reproduces the exact canonical bytes.
Six neutral/idle/attack/hit software views were inspected offscreen.

Native close review exposed a dark closing seam at the earlier separate
pink hand/arm joins. Each forelimb is now one closed surface through the
elbow, wrist and rounded palm, following the established continuous-palm
approach. The two separate Hand solids are retired. Eased wrist centerlines
prevent inside-bend crossings without reducing the palm dimensions. A new
regression rejects large coplanar closing fans outside the buried shoulder
roots. The final wrist adjustment preserves all other source solids exactly:
22 for Chansey and 36 for Blissey.

All 33 final Python pair regressions pass in 39.749 seconds. They
independently decode stored geometry, source-part ranges, skin weights,
hierarchy, inverse binds and animation keys; test every non-root carrier in
isolation; retain actual appendage roots and uncrossed surfaces through all
clips; check exposed fitted vertices, edges and face centers against posed
parents; keep every egg vertex below the rim within the actual pouch
surfaces; and retain empty pouch/mouth probes in their moving carrier
frames. Closed outward surfaces, normals, palette, floor, winding, edge
strain, neutral endpoints, budgets and strict-reader mutation rejection
also pass. The posed-root test keeps the shared occupied-interval grid and
tolerance while the independent triangle index culls ray candidates. All
four final Rust pair GPU, support, bounds, independent-carrier and two-foot
walking regressions pass in 1.60 seconds.

Both shared registry and source-cue timing regressions pass in 90.83 and
105.89 seconds respectively. The final native render build completes in
1m 03s.

Final native review uses three selected 1000×700 production 3D/2D pairs:
Route36 at (20, 8), hour 12 in first-person and elevated views, and a Route29
walking encounter. Every capture is entirely offscreen and muted, with
explicit headless/image-target controls. Preflight verifies the current
executable guards and exact embedded compressed GLBs; both executable
identities and both canonical assets remain stable through capture.
The exact current asset SHA-256 is
`ace72cce107a10111009b248a4f7687e35900e0d8ca5aa9fe93063060d6e2f50`.

The close view confirms the earlier dark wrist seams are gone, with the
rounded continuous palms supported against the egg. The face, open cup,
rounded ribbons and grounded feet remain readable; nearby foliage partly
obscures the left ribbon ends. The elevated side view retains the original
scale beside the trainer. The battle shows the complete body, supported
palms/egg, broad feet and pointed tail with named CHANSEY/PIDGEY HUDs and
no model fallback.

Terrain admission succeeds at the original 1.0922 m pack height with
zero support offset, after 171 candidate checks (73 unsupported). Existing
whole-body support, frustum, omission and admission rules remain unchanged.
The three selected captures complete in 46.19, 30.65 and 70.22 seconds.
Every pair records `state_preserved=true`:

| Capture | State frame/hash | LCD frame | Virtual seconds |
| --- | --- | --- | --- |
| first | 2297 / 2083970636 | 2297 | 39.119332959 |
| elevated | 1497 / 154914726 | 1497 | 25.490588709 |
| battle | 2665 / 777105225 | 3285 | 56.650221999 |

Browser and moving-world visual review remain open. Walking-intro frames
are not proof of GPU animation playback; stored-pose and Rust GPU checks
supply their narrower evidence. Shiny checks are skipped. This batch leaves
grass height, AA and filtering to their existing owners.

## Original static sculpture history

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
