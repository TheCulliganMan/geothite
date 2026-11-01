# Gyarados original coiled sculpture

Gyarados now has an exact entry in the battle/world species registry. The
canonical asset is `crates/crystal-voxel-view/models/battle_species/gyarados.glb`;
`tools/gyarados_sculpt.py` authors the geometry; `tools/gyarados_glb.py`
authors its standard glTF skin and rotation clips. No icon
mesh, imported geometry, retired model or duplicate neutral export is used.
The generic `icon_gyarados` source keeps its separate family routing.

Inspected [official Gyarados artwork](https://sg.portal-pokemon.com/pokedex/0130/)
guided the trident forehead, open fanged mouth, curled barbels, paired cheek
fans, cream belly armor and white dorsal/tail fins. The original sculpture has
40 named closed parts, 9,187 vertices and 11,666 triangles. Its neutral floor
is exactly zero and full silhouette height is 1.40 units.

The first front preview showed shredded belly panels and detached dorsal
sails. Belly panels now follow the body's exact smoothed centerline and
parallel-transport frame; both embedded shell thickness and consistent edge
winding are retained. Sharp perimeter normals prevent the tight lower bends
from averaging across opposite shell faces. The sails now embed into the
body. A final silhouette pass adds subtle swollen blue segments rather than
leaving an undifferentiated tube.

The final stored candidate was inspected from front, oblique and rear in
`target/pokemon-gyarados-review/segmented-three-angles.png`. The open mouth,
four fangs and blue trident remain readable; the belly continues around the
coil without the earlier missing triangles. Fins are attached. Small blue
seams at the tight lower bend and the simple spherical brow remain refinement
candidates. This software render does not establish native or browser acceptance.

The current authored tail-to-mouth centerline is 2.7092259 units. Runtime
`reference_span` uses that length, excluding horns and sails, instead of the
retired 0.82-height pose's calibration. `reference_length()` derives the value
from the recipe and canonical body transform. The pack's physical dimension
remains authoritative; it is not copied into an art catalog or clamped for
readability.

```sh
python3 tools/gyarados_glb.py
python3 tools/test_gyarados_glb.py
python3 tools/test_gyarados_sculpt.py
cargo test --locked -p crystal-voxel-view gyarados --lib
```

Seven sculpture tests pass, covering canonical regeneration, closed connected
outward parts, normalized normals, shading winding, grounded volumetric bounds,
open-mouth anatomy, fitted belly topology and the centerline calibration. Both
Rust Gyarados tests pass: exact battle/world source sharing and physical scale.
Both whole-registry loaded-model checks pass across 226 species, proving finite
bounds, grounding, reuse and unit normals. They do not prove all-species appearance.

The rebuilt native reviewer captured and individually inspected:

- `target/pokemon-native-review/gyarados-wild.png`: the real blue model,
  continuous cream coil armor, open fanged mouth, trident, barbels and white
  sails appear in the production arena with the real HUD. No fin floats as in
  the first candidate. The strong warm lighting makes ivory sails beige and
  the blue body dark; the tiny fallback Sudowoodo exaggerates scene size contrast.
- `target/pokemon-native-review/gyarados-shiny.png`: the real body changes to
  red while its belly remains cream/gold. Geometry, camera and grounding remain
  readable, but both HUD panels and command panel have lost their text. This
  is a visible presentation defect, not a passing complete shiny review.
- `target/pokemon-native-review/gyarados-classic.png`: the same fresh pack,
  species, level, full HP and battle-menu state show the original 2D back sprite
  and Sudowoodo. This separate control has not certified identical simulation
  timestamps or facing; it is not a frozen frame-for-frame fidelity proof.

```sh
CRYSTAL_PREVIEW_MUTE=1 CRYSTAL_RENDER_TEST_CAPTURE_TARGET=image \
  ./target/debug/examples/immersive_battle_3d \
  content-packs/core-modular.browser.crystalpack --wild-species GYARADOS \
  --size 1000x700 --screenshot target/pokemon-native-review/gyarados-wild.png
```

Add `--shiny` or `--classic` with a distinct screenshot path for the controls.
The fresh reviewer's `--catalog` verifies 251 packed species, 226 exact bodies
and 25 missing bodies. Capture images/logs remain ignored verification output.
Lake of Rage presentation, dedicated articulation, corrected shiny HUD and
browser review remain pending. The full model and rigging goal remains active.

## Dedicated skin increment

The single canonical GLB replaces the static JSON. Its neutral geometry,
normal bits, palette, 40 part identities, counts, bounds and centerline datum
remain exact; the neutral digest is
`555f2c6aace148f0fa0e3412cdb5912581b5f6d6f60341c6eeaecdcb7d4ac045`.
Nine joints provide an anchored coil, two neck stages, a rigid head, an opening
lower jaw, two barbel hinges and a two-stage tail. The fixed floor root owns
coil contacts; tail motion uses vertical axes to preserve floor clearance.
Eyes, crest and cheek fans follow the head rigidly. The lip blends across the
jaw boundary while lower fangs and tongue follow the mandible.

Idle has a slow neck survey and delayed barbel/tail response. Attack braces
at 18% of the existing source cue, opens the jaw at 12%, then settles its
appendages at 28%. Hit gives a short recoil. No root displacement, scale keys,
move effects, audio, HP timing or gameplay state are authored by these clips.
The original source programs remain authoritative.

Five independent stored-skin checks, seven decoded sculpture checks and the
single-source routing check pass. They cover exact regeneration and mutation
rejection, normalized byte weights, hierarchy/inverse binds, rigid details,
loop endpoints and idle seam, grounded coil through clips/blends, floor
clearance, winding/triangle strain, early bracing and actual jaw motion.
Decoded neutral/idle/attack/hit poses were individually inspected in
`target/pokemon-gyarados-skin-review/posed-angles.png` from front, oblique and
rear. The jaw opens independently, the neck drives forward, the coil remains
fixed and no eye, horn or fin separates. The existing small lower-belly seam
remains visible. These are stored-animation software views.

The earlier production images above show the static predecessor. The fresh
skin review has these individually inspected production images:

- `target/pokemon-native-review/gyarados-skin-3d.png` and
  `gyarados-skin-2d.png`: the built-in paired capture reports
  `state_preserved=true`, checksum hash 2629288151 and LCD frame 39. It holds
  simulation state across both readbacks; both show level 25, full 90 HP and
  the same battle menu. The 3D body retains a grounded coil, open jaw, attached
  fins/crest and thin barbels. The 2D source remains more saturated blue;
  dark teal shading, the broad simple brow and the strong lower muzzle shadow
  are still visual refinement targets. The fallback Sudowoodo remains tiny in
  3D, reinforcing the need for its missing exact body.
- `target/pokemon-native-review/gyarados-skin-shiny-first.png` and
  `gyarados-skin-shiny-later.png`: both current captures show the red body,
  cream/gold belly, complete HUD and menu. The previous blank-HUD symptom did
  not recur here. No HUD source was changed by this skin increment, so this
  observation is not evidence of a permanent fix. These same-view temporal
  captures advance time; they are not a frozen comparison with normal form.

The current Rust species-rig suite passes 16 tests, with zero failures and
one existing migration audit ignored. The 12 battle skinning tests also pass,
including new Gyarados/Wooper GPU entity and shared inverse-bind checks.
Gyarados uses the authored centerline for physical scale; the generic GPU test
helper now checks that datum rather than assuming every body measures vertical
height. Source-cue pose sampling and retirement/resource bounds also pass. This includes Gyarados registry sharing,
coil contacts, schema, bind/clip endpoints and dense animation bounds. The
first Rust attempt stopped on three type errors in the terrain regression
helper; its signed casts and tuple-to-quad UV comparison have been corrected.
The fresh skin builds and renders against the production pack without changing
that pack. Dedicated bodies still cover 226 of 251 packed species. Source-cue
attack/hit GPU playback, Lake of Rage presentation, browser review and remaining
appearance refinements are still open.

A four-second ordinary window recording attempted GPU idle proof and returned
118 uniformly black frames, despite successful runtime and timing output. Every
recorded PNG was checked; one failed frame is retained at
`target/pokemon-gyarados-skin-review/failed-window-record.png`, with the attempt
log at `idle-record.log`. The black frame bundle was removed. This supplies no
GPU motion evidence. Explicit image-target capture remains the working review
path; the window-recording failure is still open.
