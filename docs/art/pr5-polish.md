# PR #5 visual polish

The goal is to polish the whole PR. Model availability, topology checks and
successful compilation do not establish finished art. The catalog currently
contains 571 static JSON models, eight individual animated GLBs and 75 human
scenes in the shared catalog: 654 neutral review entries. This includes
scenery variants and non-Pokémon props, not 654 distinct Pokémon.

## Papercraft direction

The requested finish is papercraft with gentle, smooth contours. Blend lighting
across rounded skin, limbs and facial surfaces; retain crisp normals only at
intentional folds, crests, rims and seams. The previous all-flat panel treatment
made incidental triangles too visible and was rejected. Shape the silhouette
with curved contour profiles and finer outlines before smoothing the normals.
Proportions, connected forms and fitted details matter before shading.

Pokémon XD may supply proportion and pose references through an external local
export. The TeamOrre decompilation contains code, not the game models. Do not
commit imported meshes, textures, disc images or converted copies. References
stay outside Git; reference availability does not mark an authored model as
polished. No XD assets have been downloaded or integrated yet.

## Review method

`tools/render-model-review.py` renders actual canonical geometry, node
transforms, normals and linear materials into ignored review sheets. It needs
NumPy and Pillow for inspection only; neither becomes a game dependency.

```sh
python3 tools/render-model-review.py crates/crystal-voxel-view/models \
  --size 240 --out target/model-polish/all
```

These are orthographic neutral asset inspections. They do not replace animated
pose review, the production camera, actual map composition or GPU gameplay
verification. In-game readability at normal physical size is a final gate.

Each asset needs a recognizable silhouette, correct proportions, coherent
anatomy or construction, fitted surface details, restrained source-appropriate
color and deliberate normals. Shared primitives are useful construction tools;
the final forms must distinguish species and character identities. Original
map footprints, source collision, actor placement, physical species dimensions,
authored scripts and attack effects remain authoritative.

## Current assessment

The baseline complementary-species sheets have been visually inspected. Many families
still need reconstruction, rather than a material adjustment: detached ball
heads and bellies, cylindrical limbs, protruding eyeballs and repeated family
silhouettes dominate the existing first pass. Clear baseline examples include the
Machop family (now reconstructed below), Abra/Alakazam, Drowzee/Hypno, evolution pairs such as
Croconaw/Feraligatr, and the generic quadrupeds. The more recently sculpted
starters, Gengar and birds also require final camera and motion review.

Representative human, furniture and exterior sheets have been inspected. Human
stance, face fitting, garment silhouettes and distinguishing body proportions
remain open. Furniture must be assessed in its room as well as in isolation;
flat repeated façades, interior trim and architecture need map-specific review.
The full 28-sheet neutral catalog is generated, but uninspected sheets are not
counted as reviewed or polished.

| Workstream | Required finish | Status |
| --- | --- | --- |
| All 251 Pokémon | Individual silhouettes/anatomy/materials; appropriate animation and special states; normal-size battle review | Open |
| All 75 humans | Distinct proportions/outfits/faces; planted locomotion and interactions; map-context review | Open |
| Furniture and machinery | Coherent constructions, clear openings, fitted controls, actor clearance and interactions | Open |
| Buildings and exterior props | Distinct source layouts, intentional trim/materials, camera/reveal behavior and approaches | Open |
| Interiors, floors and boundaries | Complete joined forms, source-faithful surfaces, puzzle states, collision and room composition | Open |
| Battle presentation | Art readability, cue-aligned motion, effects placement, switch/faint/capture/return and physical scale | Open |
| Native/browser integration | Actual gameplay and GPU review, portable source regeneration, content-exclusion and full regressions | Open |

## Ampharos reconstruction

Ampharos now has one continuous pear torso, slender neck and rounded muzzle,
instead of stacked body/head spheres and a neck rod. Broad tapered flippers
replace stick arms with ball hands. Its eyes, forehead beacon, subtle smile and
ivory chest fit the surface. Rounded striped horns, two neck bands and a curved
striped tail carry the species silhouette; broad paired feet stay on Y=0.

`tools/ampharos_sculpt.py` is its standard-library authoring recipe. The existing
`ampharos.mesh.json` remains its sole canonical geometry file; the legacy
Ampharos branch is removed from the Blender batch builder. The builder delegates
to the recipe. Explicit seven-decimal source precision removes near-zero libm
noise from normals. The unchanged 1.35-unit height retains the existing runtime
physical scaling contract. There are 26 closed anatomy parts, 4,700 triangles
and 2,402 vertices.

```sh
python3 tools/ampharos_sculpt.py
python3 tools/test_ampharos_sculpt.py
```

Four stored-model checks pass: exact recipe reproduction, one connected oriented
closed solid per part, finite unit normals agreeing with real triangle winding,
outward chest faces, paired anatomy and grounded height. Model inspection found
and corrected an inverted belly-panel edge closure before publication. Neutral
before/after views are reviewed. Final in-game battle review and articulation
remain open; this increment does not finish Ampharos or the broader catalog.

## Machop-family reconstruction

The three previous models shared a sphere head, cylindrical limbs and nearly
the same torso. Their replacements have distinct anatomical profiles:

- Machop has a larger head relative to its narrower young body, a curved tail,
  lower arms, broad palms and planted three-toed feet.
- Machoke has a long-legged, angular wrestler silhouette, shaped deltoids,
  narrower elbows, fitted red arm markings and a waist-following belt/trunks.
- Machamp has a broader chest and shoulders, separate upper and lower arm
  attachments, four independent squared hands, thick thighs and broad lips.

Each torso joins pelvis, waist, chest, neck, jaw and cranium into one closed
surface. Pectoral relief is part of that surface; the initial detached chest
volumes were rejected during inspection. All faces have fitted almond eyes
and shallow pupils, with three swept cranial plates rather than pointed
ornaments. The static source heights stay 0.85/1.15/1.45 units, retaining the
existing pack-derived physical scale. No animation or attack event is added.

`tools/machop_sculpt.py` authors all three canonical JSON models. The batch
builder delegates to it and no longer contains the old Machop-family geometry.
`tools/sculpt_geometry.py` shares closed surface construction with Ampharos;
the refactor reproduces Ampharos's canonical JSON exactly. Species anatomy
and palettes remain in their own recipes.

| Species | Closed parts | Triangles |
| --- | ---: | ---: |
| Machop | 32 | 3,628 |
| Machoke | 41 | 4,144 |
| Machamp | 51 | 5,216 |

The four new family checks verify actual stored topology, oriented closure,
finite unit normals agreeing with triangles, source reproduction, scale,
planted toes, fitted eyes, two/four arm anatomy and distinct proportional
widths. They caught inward-facing smooth normals at two early elbow shapes;
the elbow geometry was revised before acceptance. Front, three-quarter and
rear model renders have been reviewed. The existing species validator passes
on the four remodeled species as an explicitly scoped subset. All 957 voxel
tests pass with three existing ignored checks. The four Ampharos checks also
continue to pass.

```sh
python3 tools/machop_sculpt.py
python3 tools/test_machop_sculpt.py
```

This is a static anatomy increment. In-game camera/readability, animated
clearance and species articulation remain open for all three models, alongside
the full catalog's remaining workstreams.

## Psyduck and Golduck reconstruction

The old pair reused a sphere head, sphere body and rod limbs. Psyduck now has
one continuous egg body and broad head, a flattened two-shell bill, small
pupils in rounded fitted eyes, hands reaching its temples, three bent hairs
and single three-lobed web surfaces under its short ankles. Golduck has a
narrow swimmer waist, shaped arms and legs, splayed webbed hands with claws,
a long tapered tail, sharp fitted eyes, a ruby and four swept pointed fins.
Its pale bill and web membranes follow the official species artwork:
[Psyduck](https://www.pokemon.com/static-assets/content-assets/cms2/img/pokedex/full/054.png)
and [Golduck](https://www.pokemon.com/static-assets/content-assets/cms2/img/pokedex/full/055.png).
No reference images are stored or shipped.

`tools/duck_sculpt.py` authors the existing canonical JSON files; the legacy
Blender duck geometry has been removed and the batch builder delegates to the
recipe. Source height remains 1.03 units for both, preserving runtime physical
scaling. Psyduck has 28 closed parts and 3,828 triangles; Golduck has 45 closed
parts and 4,460 triangles. Shared shallow patches follow the authored head
surface instead of placing detached eye spheres in front of it.

```sh
python3 tools/duck_sculpt.py
python3 tools/test_duck_sculpt.py
```

Five tests verify stored-model connected oriented closure, shading/winding,
exact source reproduction, ground and height, distinct proportions, true web
surfaces and species anatomy. Eye and ruby fitting is checked against the
actual stored head triangles, independently of the recipe's surface function.
The checks caught inverted patch edges and inward normals on thin foot
membranes; both were corrected before publication. The Machop topology/shading
checks now share their independent geometry assertions with this family.

Front, three-quarter and rear neutral renders have been reviewed. All six
reconstructed species pass the scoped species validator (25,976 triangles,
18,626 vertices). Ampharos's four tests, the Machop family's four tests and all
957 voxel tests pass, with three existing ignored voxel checks. The whole-source
GLB regeneration check still has the previously observed unchanged
Chikorita/Bayleef near-zero floating-point differences on macOS; it is not
reported as passing. Static inspection does not establish in-game camera,
animation clearance or GPU verification. Those remain open for both ducks and
the catalog.

## Abra and Alakazam reconstruction

Family inspection found that the previous Alakazam reused Abra-like spherical
proportions and incorrectly carried a tail and Kadabra's forehead star. Abra
and Alakazam now have transverse fox skull sections with long tapered muzzles,
pointed closed ears with inset planes, fitted eyes and narrow necks. Abra has
closed meditation creases, a crouched young body, resting hands, rounded
shoulder shells and a substantial upward tail with a dark saddle band.
Alakazam has focused eyes, broad sweeping serrated moustache blades, a narrow
waist, elongated forearms, fitted knee armor and two spoons held through its
curled hands. Alakazam has neither a tail nor a star. Official artwork used for
identity checks:
[Abra](https://www.pokemon.com/static-assets/content-assets/cms2/img/pokedex/full/063.png),
[Kadabra](https://www.pokemon.com/static-assets/content-assets/cms2/img/pokedex/full/064.png),
[Alakazam](https://www.pokemon.com/static-assets/content-assets/cms2/img/pokedex/full/065.png).
Reference image downloads were removed after inspection; none are shipped.

`tools/abra_sculpt.py` replaces the legacy Abra/Alakazam builder, with each
existing JSON file remaining the sole canonical geometry. The newer Kadabra
sculpt was reviewed alongside both and preserved. Source heights remain 1.12
units. Abra has 43 parts and 4,248 triangles; Alakazam has 53 parts and 6,392
triangles. Concave moustache blades use ear-clipped caps instead of overlapping
triangle fans. Their head details fit the actual stored triangle surfaces.

```sh
python3 tools/abra_sculpt.py
python3 tools/test_abra_sculpt.py
```

Four checks pass for stored connected oriented solids, triangle/shading
agreement, exact recipe reproduction, source height and paired ground
contacts, species-specific anatomy and spoons starting inside their actual
palms. An over-bent haunch produced inward shading in the initial shape; its
profile was corrected rather than relaxing the check. Front, three-quarter
and rear neutral views have been reviewed. All eight rebuilt species pass the
scoped validator (36,616 triangles and 25,020 vertices), all 17 sculpture
checks pass, all 571 static JSON documents validate and all 957 voxel tests
pass, with three existing ignored checks. The scope of these checks remains
static asset integrity. Native/browser GPU, physical-size battle cameras and
species articulation remain open, including Kadabra's final motion review.

## Drowzee and Hypno reconstruction

The previous pair shared sphere heads, thin trunks and a brown waist. Hypno's
rounded ears and brown body were incorrect. Drowzee now has a broad joined
belly/neck/head and brown lower anatomy. Its yellow/brown boundary is a wavy,
welded material seam with identical normals on both sides; it is not a second
belly volume. Rounded inset ears, fitted half-lidded eyes, a short curved nose,
shaped hands and small yellow toes distinguish the young tapir. Hypno has a
narrow entirely yellow body, pointed inset ears, a tapered longer nose, bent
arms with raised hands and broad feet. A closed serrated ivory annulus provides
its ruff, with an actual neck opening. The thin cord starts inside the raised
hand and ends at an open metal pendulum ring.

Official identity references:
[Drowzee](https://www.pokemon.com/static-assets/content-assets/cms2/img/pokedex/full/096.png),
[Hypno](https://www.pokemon.com/static-assets/content-assets/cms2/img/pokedex/full/097.png).
Reference downloads were removed after review and are not shipped.
`tools/tapir_sculpt.py` replaces both legacy geometry builders; the existing
JSON files remain their only canonical geometry. Original source height stays
1.12 units. Drowzee has 33 material/anatomy parts and 5,504 triangles; Hypno has
38 parts and 6,832 triangles.

```sh
python3 tools/tapir_sculpt.py
python3 tools/test_tapir_sculpt.py
```

Five checks verify stored closed connected anatomy (joining the two Drowzee
material regions for its body topology), triangle/normal agreement, exact
recipe reproduction, dimensions/grounding, specific anatomy, the continuous
wavy seam, the ruff neck hole and a cord anchored inside the actual palm. The
first cord anchor was slightly outside the hand and was corrected. Front,
three-quarter and rear neutral model renders are reviewed. All ten rebuilt
species pass the scoped validator (48,952 triangles, 33,668 vertices), all 22
sculpture checks pass and all 957 voxel tests pass, with three existing ignored
checks. Native/browser GPU, normal-size battle review and species articulation
remain open; these checks do not sign off the whole catalog.

## Production native camera checkpoint

`immersive_battle_3d` now accepts `--species NAME` alongside `--enemy-gust`.
It validates the name against authored model coverage, then seeds the existing
fresh, disposable trainer-battle fixture with the pack-created level-25
Pokémon and its natural moves. It retains the existing backup Totodile and
trainer, normal controller entry/commands and bounded DIV stimuli. The older
`--starter` options and their Meganium evolution fixture remain available.
The fixture requires no save destination; neither normal play nor loaded saves
use it. It does not force a battle move or replace the trainer's AI.

```sh
cargo build --locked -p crystal-bevy --features location-tester \
  --example immersive_battle_3d
./target/debug/examples/immersive_battle_3d \
  content-packs/core-modular.browser.crystalpack --enemy-gust \
  --species MACHAMP --size 640x480 \
  --screenshot target/model-polish/native-battle/machamp-640.png
```

The ten reconstructed species were captured and visually inspected through
Bevy's real battle renderer with the existing external browser core pack.
Native adapter logs identify Apple M5 integrated GPU / Metal (explicitly
selected for nine captures). The tenth, Ampharos, used the same native renderer
without an explicit backend log. Requested 1024×768 windows produced 2048×1536
Retina readbacks. Captures show actual species names, HP, production command
menus and modeled opponents. This establishes native idle model integration,
source scale and lighting at that camera; it does not establish attack motion,
browser integration, every menu state or catalog-wide performance.

Additional Psyduck and Machamp captures use requested 640×480 windows
(1280×960 Retina readbacks). Psyduck is readable, but Machamp's crest overlaps
the HP backing. This exposes a renderer gap: the camera fits fixed normalized
HUD margins while glyph-sized production panels occupy a larger screen
fraction in small windows. **Issue identified at this checkpoint (addressed below):** fit the shared camera to actual
UI exclusion bounds, preserve both participants' physical scales, verify tall
and small models at desktop/mobile sizes, and review the new GPU output.
These captures must not be presented as final camera sign-off.

The new external-pack regression exercises all ten species, checks the real
lead and backup party, levels, moves with positive PP, active Pidgeotto trainer
battle, command readiness and absent save destination. It passes. The original
five-starter real attack/damage-cue regression also passes. Generated PNGs and logs remain
ignored under `target/model-polish`.

## Camera fit to measured production HUD

The production shell now publishes `BattleUiBounds` from the actual HP sprite
bounds and backing dimensions, plus the full command/narration window height
and an 8-unit gap. The renderer includes those normalized reservations in its
layout cache key. The shared perspective fit handles off-center vertical
strips analytically; it changes one camera while retaining both actors' scales,
feet, facing, origins and hit anchors. Encounter-terrain cameras retain the
same strip and still undergo their existing support/frustum acceptance.

Reservations do not shrink when HP panels are temporarily erased or a smaller
menu appears. Window resize resets the reservation. This keeps cosmetic
visibility and input from reframing a held attack; original animation-envelope
bounds continue to determine the complete fitted geometry.

All 959 voxel tests pass, with three existing ignored checks. Two new tests
cover whole-body corners inside asymmetric HUD-free strips at three aspect
ratios, unchanged geometry/anchors, alternate camera reservations, cosmetic
erasure stability, invalid bounds and resize. The native client builds.

Five real Metal captures were reviewed using the external core browser pack:
Machamp at requested 640×480, 1024×768, 320×568 and 568×320 window sizes, plus
Psyduck at 640×480. Every adapter log identifies Apple M5 / Metal. The small
Machamp crest now clears the HP backing, its feet clear the command window,
and portrait/landscape retain complete bodies. The small Psyduck retains its
physical size relative to Pidgeotto. Generated PNGs/logs remain ignored under
`target/model-polish/ui-fit-native`. These are native GPU readbacks; they do
not prove browser rendering, touch input or every animated/menu state. Those
remain open alongside full-catalog model polish.

The wider 50-test `immersive_` controller/source subset passes 43 and fails
seven source animation/audio timing assertions against this external pack.
The identical subset was rerun with every source file restored to previous
PR head `68c0a42`: it reproduces the same seven failures and exact left/right
values (43 pass, 7 fail). Examples include Psychic 176 versus 165 frames,
Shadow Ball 60 versus 57 and Surf 191 versus 185. The camera change is restored
and these existing discrepancies remain open; no audio programs, PCM, attack
source or authored timings were changed to hide them. The subset is not
reported as passing. Its ten-species preview and five-starter real attack
checks continue to pass within those 43 checks.

## Crocodile family and papercraft finish

Croconaw and Feraligatr replace the shared sphere/rod water-starter recipe with
joined body/head profiles, broad shaped jaws, fitted eyes and nostrils, bent
limbs, splayed toes and tapered tails. Croconaw has a round pale belly with blue
islands and a three-point sail; Feraligatr has a wider chest, narrower waist,
open mouth, upper/lower teeth, pale claws, V belly and fitted chamfered armor.
Source heights remain 1.30 units with planted feet and pack-derived runtime size.

The papercraft pass uses bounded polygon subdivision on the jaw cages to
soften their boxy contours. The initial all-flat panel treatment was rejected by
the user. Body profiles now use monotone cubic contours and 40-point outlines;
connected faces blend their area-weighted corner normals below a 40-degree
crease angle. Sharp crests retain authored normals and armor keeps its fitted
shape. No downloaded model or texture has been integrated.

`tools/crocodile_sculpt.py` is the reproducible authoring recipe; the generic
water-evolution recipe is removed and the batch builder delegates here. There
are 49/80 closed parts and 8,540/9,444 triangles respectively. Six stored-model
checks cover reproduction, oriented connected closure, triangle-consistent
normals, smooth skin with sharp crest folds, anatomy/proportions, height/ground and independent
triangle intersections for fitted face/armor details. All 28 sculpt checks pass.

The rebuilt native Metal battle preview is reviewed for both species. The
12-species production controller preview test passes against the external pack;
all 959 voxel tests pass (three existing ignored), and 571 canonical JSON files
validate. Screenshots and logs stay under ignored target/model-polish. This is a
first family pass toward papercraft; full catalog restyling, browser review and
articulation remain open. XD references await an external local export.

## Marill and Azumarill papercraft reconstruction

The old pair shared one body, a projecting white belly, stick arms and nearly
identical faces. Marill now has a compact round silhouette, rounded mouse ears,
small curved paddles and a left-side buoy tail. Azumarill has a taller tapered
body, broad paddle arms, one folded rabbit ear, a right-side zigzag buoy tail,
six fitted white spots and a wavy pale belly boundary. Both have shallow fitted
oval eyes, white glints, open smiles and pink tongues. The lower body is rounded
through additional contour rings and meets the planted feet.

The pale and blue body regions are one welded anatomical surface; there is no
separate inflated belly. Ear linings follow actual stored skin triangles,
including the bent tip. Their side faces were corrected after inspection found
inverted lining surfaces. Rounded faces blend their corner normals while deliberate creases remain crisp.
Monotone cubic body profiles and 40-point outlines smooth the actual silhouette
as well as its lighting. Original 1.02-unit source heights and pack-derived physical scaling stay
intact. The geometry is original, authored against the official
[Marill reference](https://www.pokemon.com/static-assets/content-assets/cms2/img/pokedex/full/183.png)
and [Azumarill reference](https://www.pokemon.com/static-assets/content-assets/cms2/img/pokedex/full/184.png).
Downloaded reference images were removed after review.

`tools/aqua_rabbit_sculpt.py` reproduces both canonical meshes; the old generic
water-rabbit recipe is removed and the batch builder delegates to it. Marill has
19 parts and 5,648 triangles; Azumarill has 25 parts and 7,448 triangles. Six new
checks verify oriented closed geometry across welded body material regions,
actual smooth normals, reproduction/scale/grounding, distinct proportions,
the shared belly seam and independent stored-triangle intersections for face,
ear lining and foot attachments. They also check that the tail cord meets its
buoy. All 34 family sculpt checks pass, plus three shared contour/crease checks and
one review-rasterizer regression. The exact-species validator passes on the
explicit four-species rounded subset: 31,080 triangles and 22,252 vertices. Its
authoring queue count is a scoped-registry artifact, not evidence of missing
production models.

Neutral front, rear and three-quarter renders are reviewed. Fresh native Metal
battle captures of all four rounded species are reviewed at 1024×768 using the
rebuilt executable and the actual external pack. The expanded 14-species
production controller preview test passes. All 959 voxel tests pass, with three
existing ignored checks, and all 571 canonical JSON files validate. Capture/log
output stays ignored under target/model-polish. The wider catalog,
articulation, attack motion and browser GPU review remain open. The same native
captures show coarse Pidgeotto body facets and tree canopies that still need
work; the rounded four are not evidence that the whole scene is polished.

## Smooth contour and review contracts

`curved_profiles` in `tools/sculpt_geometry.py` keeps authored profile knots
and bounds while adding monotone cubic contour samples. `crease_normals` changes
only corner normals/index sharing, preserving the exact triangle positions. It
smooths through connected edges below 40 degrees and keeps sharper edges split;
material regions are split afterward to avoid artificial shading seams.
Three checks verify curved profiles/bounds, exact triangle preservation, crisp
cube edges and smooth tube sides with separate cap normals.

The neutral review rasterizer now interpolates authored corner normals per
pixel. Its former triangle-average lighting could show facets even on smooth
source geometry. A raster check requires varying lighting within a triangle
with differing corner normals and one constant shade on a genuinely flat face.
This improves model inspection; it still does not establish native/browser GPU
rendering or finished art quality for unreviewed assets.

```sh
python3 tools/test_contour_sculpt.py
python3 tools/test_aqua_rabbit_sculpt.py
python3 tools/test_crocodile_sculpt.py
# With the review-only NumPy and Pillow dependencies:
python3 tools/test_render_model_review.py
```

## Pidgeotto contours and tree lighting

Pidgeotto retains its 34-part rig, wing hinges, ten animated feather parts and
closed production idle curve. The rounded body, head and breast now use cubic
contour profiles and finer outlines with smooth corner normals. The other 31
anatomy parts retain their exact geometry, normals and colors. Its canonical
GLB contains 5,208 triangles; the original batch authoring source calls the same
refinement helper before export. Canonical regeneration and wing-motion checks
pass, including a fingerprint of the untouched parts.

Both the full tree and the battle's tree LOD blend crown lighting across leaf
material boundaries. Their triangle positions, colors, bounds and wood parts
remain exact; counts remain 724 and 248 triangles. This softens lighting only:
the coarse LOD outline and polygonal color patches still need art review.
Three stored-asset checks cover exact preservation, cross-material normals,
idempotence and the LOD budget. CI runs these checks.

The focused Pidgeotto suite passes nine checks with one existing migration
skip. Its separate whole-catalog check still fails on the unchanged Bayleef
canonical-export discrepancy on this macOS host. The broad check remains in CI;
no unrelated model was regenerated to hide this failure.

Free external options checked include Quaternius's CC0 Ultimate Stylized Nature
pack, Kenney's CC0 Furniture Kit, and downloadable Pokémon XD assets listed by
The Models Resource. The latter site returns HTTP 403 here, so no XD model
has been downloaded. An accessible Pokémon-3D-api catalog contains files for
all 251 required species; Pidgeotto and Azumarill were downloaded into ignored
storage and inspected with their textures. Both have skeletons but no animation
clips, and Draco/WebP compression requires conversion or new loader support.
No external asset has been integrated or committed. These are candidates, not
evidence that the catalog is polished. The user selected Quaternius and Kenney
CC0 assets for scenery/interiors; their source downloads remain external and
ignored. Pokémon replacement integration remains unselected.

A fresh 1024×768 native battle capture on Apple M5 / Metal was inspected through
the rebuilt production example. The bird's rounded breast/head are smoother,
while its authored feather folds remain visible. The trees' triangular color
patches and angular silhouette remain conspicuous: this lighting increment does
not sign them off. The scene uses the external pack and a disposable read-only
battle, with no user save destination.

All 959 voxel tests pass with three existing ignored checks after updating the
reviewed Pidgeotto neutral-surface count and digest. The rig's motion, topology,
part grouping and hinge checks remain intact. All 571 static JSON files validate.

## External open furniture integration

The user selected Quaternius and Kenney CC0 assets for scenery and interiors.
Downloaded sources, licenses, arrangement files and converted meshes stay in
the ignored local `content-packs/open-models/` directory. None is staged or
embedded in the checked-in catalog. The Rust `import_open_model` tool converts
static opaque GLBs, applying the complete node hierarchy and inverse-transpose
normals. Reflections reverse triangle winding. It rejects skins, animations,
morphs, vertex-painted models and textured/translucent materials rather than
silently dropping those features. A JSON arrangement can compose up to 64
source GLBs with explicit translation/scale before grounding the complete
fixture. Quaternius leaf cutouts still need a textured scenery path.

Native interior model caches optionally read the matching JSON filenames below
`CRYSTAL_OPEN_MODEL_ROOT`. This changes art only; existing source drawing
identity, bounds, collision, actor positions and controller behavior remain in
the production placement/controller paths. Imported fixtures use uniform scaling
and center inside the existing footprint, preserving their proportions. Authored
assets retain their existing fitting behavior. The root is fixed on first load;
restart to change it. Missing/invalid/oversized files retain the checked-in art.
Diagnostics require `CRYSTAL_OPEN_MODEL_TRACE=1`; ordinary fullscreen clients
receive no loader stdout/stderr traffic. Browser loading remains open.

The initial native bedroom inspection exposed an elongated imported TV caused
by full-bound nonuniform fitting. Contained fitting fixes its proportions. The
generic decorated-bed override was removed pending preservation of its pattern.
A source-faithful 2D/3D pair was captured from the same external pack on Apple
M5/Metal. These are map-render fixture images, not proof of gameplay completion.
All 962 model library tests pass, plus the importer hierarchy/reflection test.
Native model tests are now included in CI without downloading external assets.

A concurrent checkout build overwrote the shared unversioned preview executable
and produced a blank first capture. That output was rejected. Subsequent checks
use this branch's rebuilt executable copied immediately into its own ignored
preview directory. Keep this isolation when using the shared Cargo cache.

Local reproduction for the downloaded Kenney chair:

```sh
cargo run --locked -p crystal-voxel-view --bin import_open_model -- \
  "/Users/ryanculligan/GitHub/geothite/content-packs/open-models/kenney-furniture-kit/Models/GLTF format/chairRounded.glb" \
  /Users/ryanculligan/GitHub/geothite/content-packs/open-models/converted/interiors/chair.mesh.json
CRYSTAL_OPEN_MODEL_ROOT=/Users/ryanculligan/GitHub/geothite/content-packs/open-models/converted \
  cargo run --locked -p crystal-bevy --features location-tester --example render_at_location -- \
  --pack /Users/ryanculligan/GitHub/geothite/content-packs/core-modular.browser.crystalpack \
  --map PlayersHouse2F --view both --x 3 --y 4 \
  --screenshot target/model-polish/open-bedroom.png
```

This is an initial native integration increment, not final approval of the
furniture catalog. Shelf contents, fixture composition, specialized appearances,
all affected rooms, browser loading and Quaternius foliage remain open.

The current local bookcase uses Kenney's taller closed case with three groups of
books fitted on its measured shelf tops. The TV combines the vintage display
and a media cabinet. Fresh paired native bedroom/home captures were inspected;
TV proportions and grounding are corrected and the shelf contents stay inside
the case. The first-floor shot predates the last CRT/stand proportion adjustment.
The rounded chair and red bed are converted candidates but have not been
approved in a matching room. No imported mesh or arrangement is committed.
