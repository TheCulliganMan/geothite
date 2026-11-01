# Blissey sculpture

The current canonical asset is `blissey.glb`, authored by
`tools/blissey_sculpt.py` and `tools/egg_nurse_glb.py`. Its 38 logical
closed solids pack losslessly into 19 runtime primitives: 19,379 vertices,
34,472 triangles, 25 joints, 163 accessors and 918,928 bytes. This replaces
the static mesh's 39,460 vertices and 70,872 triangles. The original 1.20
normalized height, physical-size path, pink/white palette, complete crown
locks, six white shoulder frills and ten pointed skirt petals remain.
All existing geometry, part, joint, accessor and byte limits are unchanged.

Body/head, independent shoulder/elbow chains and supported slim legs carry
separate crown/whole-curl pairs, two complete shoulder-frill assemblies and
ten individual skirt-petal carriers. Every appendage retains its actual
occupied root. The contained egg and actual open pink cup share the body
carrier; complete feet remain fixed in battle. Fitted eyes/glints, dark mouth
and pink tongue share the upper facial carrier. Idle is 3.2 seconds; attack
and hit are one-second rotation-only clips driven by source cues.

Focused sampling reduces the limbs, curls, petals and closed outer/inner
cup while keeping denser upper-body sampling around the small happy mouth.
Software review caught a faceted depression extending above the visible
opening. Tightening the physical recess to the opening removes that patch
while retaining the existing empty-mouth probes. The closed dark lining
clips the actual front body triangles at 1 mm source-space standoff; it
cannot bridge the recess with raised chords. Intersections weld at the
exporter's coordinate precision so tiny slivers do not collapse after
rounding. The shared helper's default precision and Mr. Mime's canonical
bytes remain unchanged.

Logical geometry pin:
`d48c3eeb58816ec7caa4200536f9e065544a3306a8f48f69c04e29f114ef8a80`.
Packed geometry pin:
`316186bb6e6a3a5c2eac8d4040d19e52536c3d642bdb23d6dba0288056f62a0e`.
Background Blender dispatch reproduces the exact canonical bytes.
Six final neutral/idle/attack/hit software views were inspected offscreen.

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
`ec047b0913132137bafbc4d13e8f4df9677bc0cf92f849c1f19d026851bd5450`.

The first close view at ordinary camera controls `0 0 -10 1` exposed clean
mouth/palm detail but cropped the lower body. A final repeat using the
existing `0 0 20 1` controls keeps the complete silhouette in frame; camera
implementation and model size are unchanged. The frontal cup occludes the
feet from this angle, while the elevated side and battle views retain visible
grounded feet. Rounded crown locks, the refined happy mouth, continuous
palms and white shoulder/skirt frills remain readable; foliage partially
obscures the left shoulder frills. The BLISSEY/SENTRET battle uses both full
models and named HUDs without fallback.

Terrain admission succeeds at the original 1.4986 m pack height with
zero support offset, after 124 candidate checks (41 unsupported). Existing
whole-body support, frustum, omission and admission rules remain unchanged.
The three selected captures complete in 45.65, 23.92 and 59.14 seconds.
Every pair records `state_preserved=true`:

| Capture | State frame/hash | LCD frame | Virtual seconds |
| --- | --- | --- | --- |
| first | 2381 / 1514237320 | 2381 | 40.790657209 |
| elevated | 1179 / 897147789 | 1179 | 20.034412917 |
| battle | 2367 / 1651539020 | 3012 | 51.745380583 |

Browser and moving-world visual review remain open. Walking-intro frames
are not proof of GPU animation playback; stored-pose and Rust GPU checks
supply their narrower evidence. Shiny checks are skipped. This batch leaves
grass height, AA and filtering to their existing owners.

## Original static sculpture history

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
