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


### External browser scenery bundle

The optional browser path now fetches `open-models.json` from the same origin
before starting the production shell. A 404 uses the authored catalog silently;
invalid bundles and downloads that exceed ten seconds retain that fallback.
The Rust parser accepts version 1 furniture and whitelisted tree bundles, validates every mesh
before installing any, caps the whole input at 64 MiB / 128 entries and each
mesh at 16 MiB, and freezes source choices at the first model lookup. No game
pack data, collision, controller state, or save content is replaced.

`bundle_open_models` assembles converted external `interiors/*.mesh.json`
into this bundle. `tools/new-bark-3d-web.sh` includes it only when
`CRYSTAL_OPEN_MODEL_ROOT` is explicitly supplied, and removes a stale optional
bundle when building a preview without that root. The generated bundle remains
ignored build output. The feature-gated `preview=bedroom&multiplayer=off` entry
starts at PlayersHouse2F (3,4), uses the production renderer/controller and has
no save destination. It is for verification, not a production spawn change.

All 964 model-library checks pass (three existing ignored checks), plus the
importer's nested-transform/reflection test. The registry regression uses a
fresh filtered test process to prove that a distinctive bundled tetrahedron
reaches the actual cached furniture renderer while an unbundled stool retains
the authored fallback. An initial assertion incorrectly expected raw material
colors after the existing face-light bake; the corrected check retains exact
geometry/index verification and expects the established lighting response.
The actual WASM target compiles successfully. CI now checks this WASM feature
combination too, without downloading assets. Fresh optimized WASM and the matching Rust server were built and copied into
an isolated ignored preview directory. Headless Chrome / SwiftShader checks
pass for valid Kenney scenery, a missing (404) bundle and malformed JSON. All
three open the real PlayersHouse2F preview at (3,4); the imported case also
moves upward and opens the production Start menu through the normal bridge.
Screenshots explicitly show 3D (the initial 2D review attempt was rejected).
The page now recognizes both bedroom and New Bark art previews when choosing
its initial 3D presentation. Normal view preferences are unchanged. Imported
and missing-bundle captures were visually reviewed: the TV/cabinet and three
book groups are visible, and the authored furniture is retained on fallback.
This is actual WASM rendering under SwiftShader, not hardware GPU performance
proof or a complete gameplay regression. Review artifacts are under
`target/model-polish/open-bundle-bedroom-{kenney,missing,malformed}.png` and
`open-bundle-browser-review.json`. At that checkpoint Quaternius textured foliage was still unintegrated. The
older solid crown was visibly angular and was rejected as a replacement.


The 94 browser JavaScript regressions also pass after installing the worktree's
pinned dev dependencies with scripts disabled. The initial JS run lacked
`jsdom`; no application assertions failed once dependencies were present.
The disposable review server and headless browser were stopped afterward.
The actual selected furniture bundle was 416,428 bytes; sources, converted
meshes, this bundle, review images and compiled binaries are all ignored.


### Quaternius cutout foliage

The external importer now accepts local static glTF as well as GLB, retaining
UVs, source wrap modes, double-sided leaves and PNG/JPEG albedo. Textures are
bounded and reduced to at most 512 pixels per edge for these scenery imports.
Leaf transparency becomes a crisp depth-writing cutout. An explicit
`--cutout-color 749b79` palette option keeps the alpha silhouettes while using a
muted paper green; bark keeps its source albedo. No microrelief normal map is
imported. The selected source is the CC0 birch from
[Quaternius Ultimate Stylized Nature](https://quaternius.com/packs/ultimatestylizednature.html).
Source downloads, converted meshes, textures and review output remain ignored.

Optional `new_bark/tree.mesh.json` and `johto/tree_lod.mesh.json` override the
Johto trees and the fallback meadow/forest battle trees. Forest conifer plots can use the same external mixed grove; without external
foliage they retain their authored models. Kanto tree catalogs remain unchanged. The current external LOD path uses the same 4,596
triangle tree: this is not a reduced-detail LOD or a completed performance pass.
Geometry uses the existing plots and collision; terrain batches retain their
authentic map ownership. Texture/material reuse precedes spatial batching.
Encounter retention keeps alpha masks and actual image bytes. Fallback battle
foliage uses the authoritative dark/white palette cues after texture sampling,
so a colored albedo cannot defeat a white flash.

`bundle_open_models` now includes the two supported foliage paths, accepts a
root containing only foliage and generates a gzip sidecar. The preview build
removes both stale optional files when no external root is supplied. No open
asset enters the checked-in catalog or the game pack.

All 975 model-library checks pass, with three pre-existing ignored checks,
plus the importer transform/reflection regression. New checks cover texture
alpha/samplers/limits, fitted UV geometry and retained encounter cutout images.
This remains an incremental scenery improvement; the full Pokémon, human,
world-prop and animation polish scope is open.


### Scale and repetition

The selected external grove now has five distinct Quaternius birch geometries,
including narrower, fuller and split-crown shapes. Four optional files under
`johto/tree_variants/tree_{2,3,4,5}.mesh.json` join the main tree in the bundle.
World-coordinate hashes choose the shape, quarter-turn orientation, canopy
width, height, trunk offsets and restrained tint. World trees now stand 9–16
source cells tall, independently of the two-cell source drawing; a regression
checks that mature crowns clear the canonical player-house roof. Recognized
neighboring tree plots authorize overlapping crowns. Interior trunk offsets
reach 30% of plot spacing; isolated trees retain their horizontal footprint.
These choices stay fixed when the camera moves. Grounding, plot ownership and
collision remain unchanged; the key/fill bake rotates with the normals.
Fallback battle groves retain their separate metre scale. Missing optional
variants retain the main tree.

The previous tree test required identical horizontal vertices for different
world coordinates. It now requires changed silhouettes, deterministic rebuilds,
unchanged footing and isolated-tree vertices inside the source plot. A separate
400-placement check requires all five variants and all four orientations while
bounding height and canopy width. This work concerns repeated Johto foliage;
scale and repetition across the remaining props and creatures are still open.

Forest-tileset lawn receives a continuous world-anchored soil/moss field,
scattered paper leaves and thin fallen twigs, rather than the town lawn finish.
The additional faces sit just above the original walking plane. Continuity and
floor-height checks cover the finish. Mixed variants anchor their actual low
trunk geometry in continuous groves rather than using asymmetric crown centres.
Individual textured crowns and solid trunks participate
in the existing whole-object player reveal; fade copies retain the source alpha
cutoff and authentic source-map domain, without inventing another controller.
Frozen encounter materials recover the source mask even when captured mid-fade
and retain it through subsequent opacity/cue updates.


Verified flat lawn/path boundaries now blend through a narrow, world-anchored
irregular band. Only neighboring known lawn/path materials authorize a blend;
water and unknown cells do not. Boundary faces get six subdivisions for smooth
color transitions while all original outer corners, heights, normals and
footing remain exact. Modeled tree lawn underlays participate in these borders.
Continuity, scroll stability and exclusion checks cover the blend; the existing
ground-geometry regression caught float rounding at a source corner, now fixed
by retaining exact outer endpoints.


Fresh current-code Metal captures were reviewed for New Bark, Ilex Forest
(at the verified gate location and a centrally selected walkable tile) and the
fallback battle arena. Individual cutout leaves, split/full crowns, taller
trunks and differing orientations are visible. The central Ilex capture still
now shows mature overlapping crowns, readable player reveal and forest litter.
Earlier, shorter-tree captures exposed unrelated source-art fallback patches at
the distant edge; those source families still need review. The initial attempted Ilex (20,20) was
unwalkable and produced no render. The walking-encounter recording never
entered battle, so it is not retained-battle GPU proof; retained cutout behavior
is covered by the real material/image regression. Browser battle GPU review
and full LOD/performance work remain open.

A fresh optimized WASM build loads the complete optional bundle once as gzip
(23,172,380 JSON bytes / 5,277,475 transferred bytes) in headless Chrome with
SwiftShader. The reviewed 3D New Bark capture shows the mixed tall grove and
softened lawn/path edges. Real bridge input moves CHRIS from (13,6) to (13,7)
and opens the visible production Start menu after movement settles. There are
no page errors. This is rendering/input evidence, not hardware GPU performance
proof. Gzip decompression matches the exact bundle bytes. The prior valid,
missing and malformed furniture-bundle browser checks remain separate evidence.
Review files are ignored under `target/model-polish/`: `rooted-grove/NewBarkTown-2.5d.png`,
`rooted-grove/IlexForest-2.5d.png`, `paper-grove-battle.png`,
`natural-grove-browser.png`, `natural-grove-browser-start.png` and
`natural-grove-browser.json`. No external meshes, textures, sources or build
artifacts are committed.


### Forest floor detail refinement

Forest litter now uses six-sided folded leaves with baked facet normals,
forked fallen sticks and sparse fern fronds with connected stems. Broad
world-coordinate patches control litter density; fern count and orientation
vary per placement. Moss has a stronger olive/soil contrast with smooth field
edges. Every detail stays inside its verified floor cell and below one quarter
of a source-cell height; navigation and authored walking planes remain intact.
The geometry bakes near a local origin before translation, avoiding a subtle
camera-dependent change in fold shading caused by subtracting larger floats.
A 256-cell regression checks containment, height and scroll stability.
Native Metal review of the centrally selected walkable Ilex Forest scene shows
broader leaf silhouettes and keeps the player visible through the existing
crown reveal. The current optimized WASM build also compiles successfully;
this floor refinement has native scene evidence, while the earlier browser
movement/Start review remains separate evidence. Review captures stay ignored
under `target/model-polish/forest-litter-final/`.

## Goldenrod ornamental gardens

A native Metal review of Goldenrod at `(26, 8)` exposed the flat ornamental
patch beside the northern landmark. This is the complete Johto Modern `$66`
drawing, distinct from ordinary animated flower tiles. The new map-scoped
mesher requires all sixteen original source cells and honors existing object
ownership. Partial or edited drawings retain their source art.

The drawing now has two shallow paper-edged beds with green soil, raised coral
and pale flowers, and warm ornamental paving. Flower height and orientation
are seeded in map coordinates. Geometry stays inside the original plot;
collision, actors, scripts, saves, and gameplay state are unaffected. The
existing flower model is reused; no downloaded assets are added.

Verification uses the actual pack and production Bevy renderer on Apple M5
Metal. Matching before/after captures remain ignored under
`target/model-polish/goldenrod-review/GoldenrodCity-north-2.5d.png` and
`target/model-polish/goldenrod-review/gardens-final.png`. The source-identity
regression rejects incomplete drawings, ownership conflicts, and other maps,
and bounds geometry to the source plot. This is local native visual evidence;
public deployment and browser performance are unverified. Building repetition
and broad city paving remain unfinished.

## Goldenrod residential variety

Complete Goldenrod residential shells now receive four coordinated treatments:
ivory, sage, dusty rose, or blue-gray walls with matching trim and muted roofs.
The treatments add slatted paper shutters or flower boxes to the original
outer window bays, vary the upper cornice height, and reposition the roof
service hatch. Civic landmarks retain their authored kit. Existing source
recognition still selects the complete building before this finish applies.

Selection uses the original map coordinate, including the built grid origin.
Door fitting is shared with the shell so decorations follow off-center
entrances. Door/window courses, footprint edges, source ownership, and gameplay
remain unchanged. New details join the building's existing fade range.

The regression checks all four treatments, repeated construction, original
lower-course positions and index order, bounded X/Z geometry, and normalized
normals. Actual production Metal captures at `(12, 26)`, `(20, 18)`, and
`(26, 8)` were reviewed under ignored `target/model-polish/goldenrod-review/`
as `facades-south-final.png`, `facades-center-final.png`, and
`facades-north-final.png`. These verify local native appearance only. Broad
street surfaces, landmark construction, and other cities remain unfinished;
this increment does not establish complete city or full-catalog polish.

## Goldenrod street materials

Goldenrod's recognized brick-road cells now use smaller staggered paper brick
courses with restrained color variation, mortar, and edge shading. Recognized
path cells use larger pale limestone flags. A narrow flat curb course separates
actual path/brick neighbors. Material identities and the original acreage stay
authoritative; this finish does not replace unknown artwork or add navigation.
Eligibility requires Johto Modern source cells inside Goldenrod's canonical
80×72 source-cell bounds. Connected Route 34/35 halo cells retain their existing
ground finish; the boundary regression checks all four edges and other atlases.

Courses and individual stone colors are anchored in map coordinates. Clipping
a partial stone at a source-cell boundary creates no extra joint. All surface
pieces partition the original flat plane at its original height, with upward
normals. Regression checks cover total area, interior overlap/hole probes,
plot bounds, camera translation, and source-material requirements for curbs.

Production native Metal captures at the same south, center, and north Goldenrod
positions were reviewed under ignored `target/model-polish/goldenrod-review/`
as `paving-south.png`, `paving-center.png`, and `paving-north.png`. Building
fade, the garden beds, and the original railway crossing remain visible in
those scenes. Library tests and the WASM compile check pass. These captures
prove local native appearance, not public deployment or browser performance.
Full model/catalog polish, other cities, and broader street composition remain
open.

## Folded modern roof decks

The modern exterior kit's named blue weathering deck now receives narrow raised
paper folds, shallow cross-course joins, and optional low pitched skylights.
Map-coordinate selection varies seam spacing, skylight size/position, and leaves
some roofs without skylights. This applies to recognized complete blue-deck
modern residential and civic shells; red/teal roofs and other kits retain their
existing construction. The original shell vertices/colors, hatch, doors, and
footprint remain intact. All added geometry joins the building's fade range.

Regression checks preserve the shell, bound the additions inside the footprint,
require finite geometry, unit normals, nondegenerate outward-facing triangles,
and deterministic construction with distinct coordinate variants. The existing
cutaway integration regression includes the roof finish. The current worktree's
voxel library suite passes (982 tests, 3 ignored; includes the separate pending
furniture alignment test), and the WASM compile check passes.

Actual native Metal captures at Goldenrod `(26, 8)` and `(20, 18)` were reviewed
as ignored `target/model-polish/goldenrod-review/roofs-north.png` and
`roofs-center.png`. This is local appearance evidence, not public deployment or
browser performance verification. The railway, broader landmark silhouettes,
connected-city continuity, and full character/world catalog remain open.

## Goldenrod railway and crossing

The exact Johto Modern `0x7e` track and `0x7f` crossing drawings now select
three-cell-deep rail bands column by column: source rows `0x36 / 0x46 / 0x36`
for track and `0x36 / 0x06 / 0x36` for the pedestrian crossing. Matching atlas,
metatile, column phase, row phase, tile IDs, and available ownership are all
required. The fourth sidewalk row stays live. Unknown or cropped columns retain
the original source art.

Low rail webs/heads, timber sleepers with small grain variations, sparse buff
ballast chips, and flush cream crossing panels replace the flat yellow pixel
strip. Map-coordinate seeds keep the chips and timber stable as the camera
moves. Existing collision and footing are unchanged; there is no new navigation
or actor dispatcher. Native Metal crossing capture `(20, 18)` was reviewed as
ignored `target/model-polish/goldenrod-review/rail-center.png`. The voxel library
passes 983 tests (3 ignored, including the separate pending furniture test in
this worktree); the WASM compile check passes. This is local native art evidence,
not public deployment, browser performance, or complete catalog polish.

## External pre-rigged people candidates

The user rejected the authored people and requested pre-rigged open-access
replacements. Stop refining that kit as the replacement strategy. Downloaded
Kenney Animated Characters Protagonists and Retro from the official free links:

- https://kenney.nl/assets/animated-characters-protagonists
- https://kenney.nl/assets/animated-characters-retro

Both local archives identify version 1.1 and CC0 1.0 in `License.txt`. Original
archives, models, skins, and licenses remain ignored under the main checkout's
`content-packs/open-models/source/people/kenney-protagonists` and `kenney-retro`.
Protagonists has four skins (two skaters, criminal, cyborg); Retro has two human
and two zombie skins. Their supplied previews were inspected; the Protagonists
style is the preferred first candidate for the papercraft world. These are
publisher previews, not in-game replacements or proof of runtime animation.

Binary FBX inspection verifies that each pack's `Model/characterMedium.fbx`
contains a 58-node limb skeleton, one skin deformer and 45 weighted clusters.
Separate idle/run/jump FBXs have the corresponding same-size skeleton and
named animation stacks. They are real pre-rigged assets, not static meshes.

Quaternius Ultimate Modular Men/Women are additional CC0 candidates with
11/10 outfits, swappable body parts and 24 animations:
https://quaternius.com/packs/ultimatemodularcharacters.html and
https://quaternius.com/packs/ultimatemodularwomen.html . The Men pack's official
license was downloaded, but Casual 2, Casual Hoodie and Suit glTF downloads
returned the publisher's Google Drive quota error. Invalid HTML responses were
removed; no model from that pack is imported. Universal Base Characters is a
newer CC0 humanoid option, with a free Standard edition on the author's itch.io:
https://quaternius.itch.io/universal-base-characters .

The external people path now preserves the real skinning, UVs, embedded skin
textures and authored Idle/Run/Jump clips. Four ignored exports are integrated:
skater male/female and human male/female. Each has 58 skin joints, 1,029 weighted
vertices and 1,604 triangles. The converter corrects the supplied animation FBX
rest pose through its Targeting Pose and Blender action slots; it does not
invent locomotion or partition the mesh into rigid limbs.

`tools/prepare-open-person.py` performs offline Blender conversion, and
`tools/check-open-person.py` independently checks geometry, weights, bind data,
textures and moving clips. `bundle_open_people` validates the same bounded Rust
reader before creating an ignored browser bundle. Native clients load these
models from `CRYSTAL_OPEN_MODEL_ROOT/people/kenney`; missing assets retain the
authored fallback. Downloads, exported GLBs, textures and bundles stay external
and ignored.

The renderer uses Bevy GPU skinning and cached mesh/material/image resources.
Production actor positions, facing and footing remain authoritative; existing
actor motion selects and blends Idle/Run. Jump is retained and validated but has
no gameplay cue yet. Twelve character types use the four source outfits with
child/adult height variation. Specialist costumes still use the original rigs;
this does not complete the 75-person catalog goal.

Validation: 984 renderer library tests passed (five ignored, including pending
furniture verification in this worktree), native and WASM compilation passed,
and separate actual-asset reader/ECS tests passed. Those tests check bind-pose
preservation, animation, resource reuse and cleanup. Goldenrod native movement
and actual browser 3D captures were reviewed. The browser smoke loads the real
bundle, moves through DOM keyboard input, opens production Start and closes it,
with no people fallback warnings or JavaScript errors. Chrome used SwiftShader;
this is browser correctness evidence, not hardware performance evidence.

Goldenrod Underground is now a separate native art review with the new player.
The entrance still exposes flat stair/decor tiles and sparse walls. The main
corridor at (3, 18) shows imported player/NPC rigs, but flat chairs and repeated
planter details still need an environment pass. Captures remain ignored under `target/model-polish` and do
not establish complete tunnel or catalog polish. The full PR polish goal stays
open.
